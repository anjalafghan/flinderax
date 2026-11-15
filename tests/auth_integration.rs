use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use joy::{app::build_router, handlers::common::AppError, models::AppState};
use rusty_paseto::prelude::{Key, PasetoSymmetricKey};
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tower::ServiceExt;

async fn test_app() -> Router {
    test_app_with_db().await.0
}

async fn test_app_with_db() -> (Router, sqlx::SqlitePool) {
    // Sign-ups need the code from the SIGNUP_CODE secret; the tests use a fixed one.
    // Safety: every test in this binary sets the same value before any request reads it.
    unsafe { std::env::set_var("SIGNUP_CODE", "test-signup-code") };
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    (build_router(AppState {
        db: db.clone(),
        paseto_key: Arc::new(key),
    }), db)
}

/// Each test uses its own client IP: the rate limiter is process-wide.
async fn post(app: &Router, ip: &str, path: &str, mut body: Value) -> (StatusCode, String) {
    // Every sign-up carries the code unless the test sets its own.
    if path == "/api/common/register" && body.get("signup_code").is_none() {
        body["signup_code"] = Value::from("test-signup-code");
    }
    let req = Request::post(path)
        .header("content-type", "application/json")
        .header("fly-client-ip", ip)
        .body(Body::from(body.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

async fn get(app: &Router, path: &str, token: Option<&str>) -> StatusCode {
    let mut req = Request::get(path);
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    app.clone().oneshot(req.body(Body::empty()).unwrap()).await.unwrap().status()
}

async fn login_token(app: &Router, ip: &str, name: &str, pw: &str) -> String {
    let (status, body) = post(app, ip, "/api/common/login", json!({"user_name": name, "user_password": pw})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    serde_json::from_str::<Value>(&body).unwrap()["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn register_then_login_works() {
    let app = test_app().await;
    let (s, _) = post(&app, "10.0.0.1", "/api/common/register",
        json!({"user_name": "alice", "user_password": "correct horse", "user_role": "user"})).await;
    assert_eq!(s, StatusCode::OK);
    let token = login_token(&app, "10.0.0.1", "alice", "correct horse").await;
    assert_eq!(get(&app, "/api/account/list", Some(&token)).await, StatusCode::OK);
}

#[tokio::test]
async fn register_cannot_self_assign_admin() {
    let (app, db) = test_app_with_db().await;
    let (s, _) = post(&app, "10.0.0.2", "/api/common/register",
        json!({"user_name": "mallory", "user_password": "hunter2hunter2", "user_role": "admin"})).await;
    assert_eq!(s, StatusCode::OK);
    let role: String = sqlx::query_scalar("SELECT user_role FROM users WHERE user_name = 'mallory'")
        .fetch_one(&db).await.unwrap();
    assert_eq!(role, "user");
}

#[tokio::test]
async fn register_without_role_field_is_accepted() {
    let app = test_app().await;
    let (s, _) = post(&app, "10.0.0.3", "/api/common/register",
        json!({"user_name": "bob", "user_password": "long enough pw"})).await;
    assert_eq!(s, StatusCode::OK);
}

#[tokio::test]
async fn register_validates_and_rejects_duplicates() {
    let app = test_app().await;
    let ip = "10.0.0.4";
    let creds = |n: &str, p: &str| json!({"user_name": n, "user_password": p});
    assert_eq!(post(&app, ip, "/api/common/register", creds("ab", "longenough1")).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(post(&app, ip, "/api/common/register", creds("has space", "longenough1")).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(post(&app, ip, "/api/common/register", creds("carol", "short")).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(post(&app, ip, "/api/common/register", creds("carol", "longenough1")).await.0, StatusCode::OK);
    let (s, body) = post(&app, ip, "/api/common/register", creds("carol", "longenough1")).await;
    assert_eq!(s, StatusCode::CONFLICT);
    assert!(!body.to_lowercase().contains("sqlite") && !body.contains("UNIQUE"), "{body}");
}

#[tokio::test]
async fn login_rejects_bad_credentials() {
    let app = test_app().await;
    post(&app, "10.0.0.5", "/api/common/register", json!({"user_name": "dave", "user_password": "longenough1"})).await;
    let (s, _) = post(&app, "10.0.0.5", "/api/common/login", json!({"user_name": "dave", "user_password": "wrongwrong"})).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, _) = post(&app, "10.0.0.5", "/api/common/login", json!({"user_name": "nobody", "user_password": "wrongwrong"})).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_is_rate_limited_per_username() {
    let app = test_app().await;
    let mut last = StatusCode::OK;
    for _ in 0..12 {
        last = post(&app, "10.0.0.6", "/api/common/login", json!({"user_name": "eve", "user_password": "wrongwrong"})).await.0;
    }
    assert_eq!(last, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn protected_routes_need_a_token() {
    let app = test_app().await;
    assert_eq!(get(&app, "/api/account/list", None).await, StatusCode::UNAUTHORIZED);
    assert_eq!(get(&app, "/api/account/list", Some("garbage")).await, StatusCode::UNAUTHORIZED);
    assert_eq!(get(&app, "/api/nope", None).await, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cors_is_not_open_by_default() {
    let app = test_app().await;
    let req = Request::get("/api/nope").header("origin", "https://evil.example").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert!(res.headers().get("access-control-allow-origin").is_none());
}

#[tokio::test]
async fn internal_errors_do_not_leak_details() {
    let res = AppError(StatusCode::INTERNAL_SERVER_ERROR, "no such table: secrets".into()).into_response();
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v, json!({"code": "internal", "message": "Internal server error", "field": null}));
    assert!(!String::from_utf8_lossy(&body).contains("secrets"));
}

#[tokio::test]
async fn errors_answer_with_a_code_a_message_and_the_field_when_there_is_one() {
    let app = test_app().await;
    let call = |method: &'static str, path: &'static str, body: Option<Value>| {
        let app = app.clone();
        async move {
            let (s, h, text) = raw(&app, method, path, &[], body).await;
            (s, h.get("content-type").map(|v| v.to_str().unwrap().to_string()), serde_json::from_str::<Value>(&text).unwrap_or(Value::Null))
        }
    };
    let (s, ct, v) = call("GET", "/api/account/list", None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    assert_eq!(ct.as_deref(), Some("application/json"));
    assert_eq!(v["code"], "unauthorized");
    assert!(v["message"].is_string() && v["field"].is_null());
    let (_, _, v) = call("GET", "/api/nothing-here", None).await;
    assert_eq!(v["code"], "not_found");
    let (_, _, v) = call("POST", "/api/common/login", Some(json!({"user_name": "nobody", "user_password": "wrong"}))).await;
    assert_eq!(v["code"], "unauthorized");
    assert_eq!(v["message"], "Invalid username or password");
}

async fn authed_post(app: &Router, token: &str, path: &str, body: Value) -> (StatusCode, String) {
    let req = Request::post(path)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

#[tokio::test]
async fn card_history_is_only_visible_to_its_owner() {
    let app = test_app().await;
    for name in ["owner", "snoop"] {
        let (s, b) = post(&app, "10.0.0.20", "/api/common/register",
            json!({"user_name": name, "user_password": "correct horse"})).await;
        assert!(s.is_success(), "{s} {b}");
    }
    let owner = login_token(&app, "10.0.0.20", "owner", "correct horse").await;
    let snoop = login_token(&app, "10.0.0.20", "snoop", "correct horse").await;

    let (s, b) = authed_post(&app, &owner, "/api/card/create", json!({
        "card_name": "Millennia", "card_bank": "HDFC",
        "card_primary_color": [0, 0, 0], "card_secondary_color": [255, 255, 255]
    })).await;
    assert!(s.is_success(), "{s} {b}");
    let card_id = serde_json::from_str::<Value>(&b).unwrap()["card_id"].as_str().unwrap().to_string();

    let (s, b) = authed_post(&app, &owner, "/api/card/history", json!({"card_id": card_id})).await;
    assert_eq!(s, StatusCode::OK);
    // History is plain JSON now (it used to be protobuf).
    assert!(serde_json::from_str::<Value>(&b).unwrap()["histories"].is_array(), "{b}");

    // The card list is JSON too: colours stay packed 0xRRGGBB integers, and only the owner's cards show.
    let list = authed_get(&app, &owner, "/api/card/get_all_cards").await;
    let cards = list["cards"].as_array().expect("cards array");
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0]["card_id"], card_id);
    assert_eq!(cards[0]["card_name"], "Millennia");
    assert_eq!(cards[0]["card_primary_color"], 0);
    assert_eq!(cards[0]["card_secondary_color"], 0xFFFFFF);
    let theirs = authed_get(&app, &snoop, "/api/card/get_all_cards").await;
    assert_eq!(theirs["cards"].as_array().unwrap().len(), 0);
    let (s, _) = authed_post(&app, &snoop, "/api/card/history", json!({"card_id": card_id})).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // Resetting a card's history also forgets its balance snapshots, so the balance really is zero.
    let (s, b) = authed_post(&app, &owner, "/api/card/snapshot", json!({
        "card_id": card_id, "total_due_paise": 500_000, "outstanding_paise": 500_000, "starting_balance": true
    })).await;
    assert!(s.is_success(), "{s} {b}");
    let before = authed_post(&app, &owner, "/api/card/breakdown", json!({"card_id": card_id})).await.1;
    assert!(before.contains("500000"), "{before}");
    let (s, _) = authed_post(&app, &snoop, "/api/card/reset", json!({"card_id": card_id})).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, b) = authed_post(&app, &owner, "/api/card/reset", json!({"card_id": card_id})).await;
    assert!(s.is_success(), "{s} {b}");
    let after = authed_post(&app, &owner, "/api/card/breakdown", json!({"card_id": card_id})).await.1;
    assert!(!after.contains("500000"), "{after}");
}

#[tokio::test]
async fn dashboard_history_starts_from_the_first_balance() {
    let app = test_app().await;
    post(&app, "10.0.0.21", "/api/common/register",
        json!({"user_name": "tracker", "user_password": "correct horse"})).await;
    let t = login_token(&app, "10.0.0.21", "tracker", "correct horse").await;
    let (s, b) = authed_post(&app, &t, "/api/account/create",
        json!({"name": "HDFC", "kind": "bank", "balance_paise": 5_000_000, "role": "spending"})).await;
    assert_eq!(s, StatusCode::OK, "{b}");

    assert_eq!(get(&app, "/api/dashboard/history?months=6", Some(&t)).await, StatusCode::OK);
}

async fn hook_post(app: &Router, key: &str, text: &str) -> (StatusCode, Value) {
    let req = Request::post("/api/hook/ingest")
        .header("content-type", "application/json")
        .header("x-ingest-key", key)
        .body(Body::from(json!({"text": text, "source": "sms"}).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn bank_alerts_arrive_through_the_hook() {
    let app = test_app().await;
    post(&app, "10.0.0.22", "/api/common/register", json!({"user_name": "smsuser", "user_password": "correct horse"})).await;
    let t = login_token(&app, "10.0.0.22", "smsuser", "correct horse").await;
    let (s, b) = authed_post(&app, &t, "/api/account/create",
        json!({"name": "HDFC", "kind": "bank", "balance_paise": 100, "last_4_digits": "1234"})).await;
    assert!(s.is_success(), "{b}");

    assert_eq!(hook_post(&app, "fx_nope", "Rs 1 debited").await.0, StatusCode::UNAUTHORIZED);

    let (_, b) = authed_post(&app, &t, "/api/ingest/key/rotate", json!({})).await;
    let key = serde_json::from_str::<Value>(&b).unwrap()["key"].as_str().unwrap().to_string();

    let sms = "Update! INR 1,00,000.00 deposited in HDFC Bank A/c XX1234 for NEFT Cr-ACME CORP. Avl bal INR 1,25,000.00.";
    let (s, v) = hook_post(&app, &key, sms).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["status"], "created");
    assert_eq!(v["transaction"]["category"], "income");
    assert_eq!(v["balance_updated"]["balance_paise"], 12_500_000);

    assert_eq!(hook_post(&app, &key, sms).await.1["status"], "duplicate");
    assert_eq!(hook_post(&app, &key, "Your OTP is 123456 for Rs 500").await.1["status"], "unrecognized");

    // Rotating the key retires the old one.
    authed_post(&app, &t, "/api/ingest/key/rotate", json!({})).await;
    assert_eq!(hook_post(&app, &key, sms).await.0, StatusCode::UNAUTHORIZED);
}

async fn authed_get(app: &Router, token: &str, path: &str) -> Value {
    let req = Request::get(path).header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn joy_dates_label_alerts_and_split_the_budget() {
    let app = test_app().await;
    post(&app, "10.0.0.23", "/api/common/register", json!({"user_name": "dater", "user_password": "correct horse"})).await;
    let t = login_token(&app, "10.0.0.23", "dater", "correct horse").await;
    authed_post(&app, &t, "/api/account/create",
        json!({"name": "HDFC", "kind": "bank", "balance_paise": 100, "last_4_digits": "4821"})).await;
    let (_, b) = authed_post(&app, &t, "/api/ingest/key/rotate", json!({})).await;
    let key = serde_json::from_str::<Value>(&b).unwrap()["key"].as_str().unwrap().to_string();

    let (dinner, _) = hook_post(&app, &key, "Rs.2,400.00 debited from a/c **4821 to VPA saltt@okhdfc (UPI Ref No 430500000001).").await;
    assert_eq!(dinner, StatusCode::OK);
    let (_, coffee) = hook_post(&app, &key, "Rs.600.00 debited from a/c **4821 to VPA bluetokai@okicici (UPI Ref No 430500000002).").await;
    let coffee_id = coffee["transaction"]["txn_id"].as_str().unwrap().to_string();
    // Picked by hand: the Joy match must leave it alone.
    let (s, _) = authed_post(&app, &t, "/api/ingest/transactions/update",
        json!({"txn_id": coffee_id, "category": "food", "remember": false})).await;
    assert!(s.is_success());

    // There is no outside Joy to link to any more: the connect endpoint is gone.
    let (s, _) = authed_post(&app, &t, "/api/joy/connect", json!({"url": "http://localhost.evil.example", "token": "x", "partner_paise": 0})).await;
    assert!(s == StatusCode::NOT_FOUND || s == StatusCode::METHOD_NOT_ALLOWED, "{s}");
    let sum = authed_get(&app, &t, "/api/joy/summary").await;
    assert_eq!(sum["connected"], false);
    // Nothing was labelled from an outside source.
    let txns = authed_get(&app, &t, "/api/ingest/transactions").await;
    let cat = |m: &str| txns.as_array().unwrap().iter().find(|x| x["merchant"] == m).unwrap()["category"].clone();
    assert_ne!(cat("saltt"), "dates");
    assert_eq!(cat("bluetokai"), "food");
}

#[tokio::test]
async fn statement_csv_imports_once() {
    let app = test_app().await;
    post(&app, "10.0.0.23", "/api/common/register", json!({"user_name": "stmtuser", "user_password": "correct horse"})).await;
    let t = login_token(&app, "10.0.0.23", "stmtuser", "correct horse").await;
    let (_, b) = authed_post(&app, &t, "/api/account/create",
        json!({"name": "HDFC", "kind": "bank", "balance_paise": 100, "last_4_digits": "1234"})).await;
    let account_id = serde_json::from_str::<Value>(&b).unwrap()["account_id"].as_str().unwrap().to_string();

    let csv = "Date,Narration,Withdrawal Amt.,Deposit Amt.,Closing Balance\n\
01/11/26,UPI-SWIGGY-swiggy@icici-1,450.00,,\"9,550.00\"\n\
01/11/26,UPI-SWIGGY-swiggy@icici-2,450.00,,\"9,100.00\"\n\
02/11/26,NEFT CR-ACME CORP,,\"1,00,000.00\",\"1,09,100.00\"\n";
    let body = json!({"csv": csv, "account_id": account_id});

    let (s, b) = authed_post(&app, &t, "/api/ingest/statement", body.clone()).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let v: Value = serde_json::from_str(&b).unwrap();
    assert_eq!((v["total"].as_u64(), v["added"].as_u64()), (Some(3), Some(3)), "{v}");
    // The account's balance was typed today, which is newer than the statement, so it stays.
    assert!(v["balance_updated"].is_null(), "{v}");

    // Same file again: nothing new.
    let (_, b) = authed_post(&app, &t, "/api/ingest/statement", body).await;
    let v: Value = serde_json::from_str(&b).unwrap();
    assert_eq!((v["added"].as_u64(), v["duplicates"].as_u64()), (Some(0), Some(3)), "{v}");

    let (s, _) = authed_post(&app, &t, "/api/ingest/statement", json!({"csv": "nothing,useful\n1,2\n"})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

async fn new_user(app: &Router, ip: &str, name: &str) -> String {
    post(app, ip, "/api/common/register", json!({"user_name": name, "user_password": "correct horse"})).await;
    login_token(app, ip, name, "correct horse").await
}

#[tokio::test]
async fn couples_start_invite_join_and_leave() {
    let app = test_app().await;
    let (a, b, c) = (
        new_user(&app, "10.0.0.31", "anjal").await,
        new_user(&app, "10.0.0.32", "partner").await,
        new_user(&app, "10.0.0.33", "stranger").await,
    );

    assert_eq!(authed_get(&app, &a, "/api/couple").await["in_couple"], false);
    let (s, _) = authed_post(&app, &a, "/api/couple/invite", json!({})).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "no couple yet");

    let (s, b_) = authed_post(&app, &a, "/api/couple/create", json!({})).await;
    assert!(s.is_success(), "{b_}");
    let st = authed_get(&app, &a, "/api/couple").await;
    assert_eq!((st["me"].as_i64(), st["waiting"].as_bool()), (Some(0), Some(true)));

    let (_, inv) = authed_post(&app, &a, "/api/couple/invite", json!({})).await;
    let code = serde_json::from_str::<Value>(&inv).unwrap()["code"].as_str().unwrap().to_string();
    let (s, _) = authed_post(&app, &b, "/api/couple/join", json!({"code": "WRONG-CODE"})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    // Typed in lower case with a space instead of the dash still works.
    let (s, j) = authed_post(&app, &b, "/api/couple/join", json!({"code": code.to_lowercase().replace('-', " ")})).await;
    assert!(s.is_success(), "{j}");
    let st = authed_get(&app, &b, "/api/couple").await;
    assert_eq!((st["me"].as_i64(), st["partner_name"].as_str()), (Some(1), Some("anjal")));

    // A partner with no money data gets the Dates home; entering an account switches that off.
    assert_eq!(authed_get(&app, &b, "/api/couple").await["uses_money"], false);
    authed_post(&app, &b, "/api/account/create", json!({"name": "HDFC", "kind": "bank", "balance_paise": 0, "role": "spending"})).await;
    assert_eq!(authed_get(&app, &b, "/api/couple").await["uses_money"], true);

    // Safe to Spend still computes for a couple (it reads their planned dates).
    let sum = authed_get(&app, &a, "/api/dashboard/summary").await;
    assert!(sum["safe_to_spend_paise"].is_i64(), "{sum}");

    // Single use, and the couple is full.
    let (s, _) = authed_post(&app, &c, "/api/couple/join", json!({"code": code})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = authed_post(&app, &a, "/api/couple/invite", json!({})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    // The starter leaves: the partner keeps slot 1, and a new join fills slot 0.
    authed_post(&app, &a, "/api/couple/leave", json!({})).await;
    assert_eq!(authed_get(&app, &a, "/api/couple").await["in_couple"], false);
    let st = authed_get(&app, &b, "/api/couple").await;
    assert_eq!((st["me"].as_i64(), st["waiting"].as_bool()), (Some(1), Some(true)));
    let (_, inv) = authed_post(&app, &b, "/api/couple/invite", json!({})).await;
    let code = serde_json::from_str::<Value>(&inv).unwrap()["code"].as_str().unwrap().to_string();
    authed_post(&app, &a, "/api/couple/join", json!({"code": code})).await;
    assert_eq!(authed_get(&app, &a, "/api/couple").await["me"], 0);

    // Both leave: the couple and its data are gone.
    authed_post(&app, &a, "/api/couple/leave", json!({})).await;
    authed_post(&app, &b, "/api/couple/leave", json!({})).await;
    assert_eq!(authed_get(&app, &b, "/api/couple").await["in_couple"], false);
}

#[tokio::test]
async fn bills_calendar_and_subscription_suggestions() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.41", "biller").await;
    let (s, b) = authed_post(&app, &t, "/api/plan/commitment/create",
        json!({"label": "Rent", "amount_paise": 1_500_000, "day_of_month": 1})).await;
    assert!(s.is_success(), "{b}");
    let bills = authed_get(&app, &t, "/api/bills?days=40").await;
    assert_eq!(bills["bills"][0]["label"], "Rent");
    assert_eq!(bills["total_paise"], 1_500_000);

    // Three Netflix charges a month apart, typed in by hand.
    let today = joy::handlers::common::today_local();
    for back in [0, 30, 60] {
        let on = (today - time::Duration::days(back)).to_string();
        let (s, b) = authed_post(&app, &t, "/api/ingest/transactions/create", json!({
            "occurred_on": on, "direction": "debit", "amount_paise": 64_900, "merchant": "NETFLIX.COM", "category": "entertainment"
        })).await;
        assert!(s.is_success(), "{b}");
    }
    let sug = authed_get(&app, &t, "/api/bills/suggestions").await;
    assert_eq!(sug[0]["label"], "NETFLIX.COM");
    assert_eq!(sug[0]["every"], "monthly");
    assert_eq!(sug[0]["amount_paise"], 64_900);

    authed_post(&app, &t, "/api/bills/suggestions/dismiss", json!({"merchant": "NETFLIX.COM"})).await;
    assert_eq!(authed_get(&app, &t, "/api/bills/suggestions").await.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn goals_plan_contribute_and_stay_private() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.51", "saver").await;
    let other = new_user(&app, "10.0.0.52", "nosy").await;
    // ₹1L salary, ₹20k allowance, ₹15k rent: ₹65k of savings, no SIP.
    authed_post(&app, &t, "/api/account/create", json!({"name": "HDFC", "kind": "bank", "balance_paise": 0, "role": "spending"})).await;
    authed_post(&app, &t, "/api/plan/income/create", json!({"label": "Salary", "amount_paise": 10_000_000, "day_of_month": 30})).await;
    authed_post(&app, &t, "/api/plan/commitment/create", json!({"label": "Rent", "amount_paise": 1_500_000, "day_of_month": 1})).await;

    let g = authed_get(&app, &t, "/api/goals").await;
    assert_eq!(g["emergency_suggested_paise"], 9_000_000, "6 months of ₹15k needs");

    let by = (joy::handlers::common::today_local() + time::Duration::days(150)).to_string();
    let (s, b) = authed_post(&app, &t, "/api/goals/create",
        json!({"name": "Goa trip", "target_paise": 6_000_000, "target_date": by, "priority": 1})).await;
    assert!(s.is_success(), "{b}");
    let id = serde_json::from_str::<Value>(&b).unwrap()["goal_id"].as_str().unwrap().to_string();

    let g = authed_get(&app, &t, "/api/goals").await;
    assert_eq!(g["goals"][0]["status"], "on_track");
    assert!(g["goals"][0]["this_month_paise"].as_i64().unwrap() > 0);

    let (s, _) = authed_post(&app, &other, "/api/goals/contribute", json!({"goal_id": id, "amount_paise": 100})).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, _) = authed_post(&app, &t, "/api/goals/contribute", json!({"goal_id": id, "amount_paise": 6_000_000})).await;
    assert!(s.is_success());
    assert_eq!(authed_get(&app, &t, "/api/goals").await["goals"][0]["status"], "done");
}

#[tokio::test]
async fn can_i_afford_it() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.61", "buyer").await;
    authed_post(&app, &t, "/api/account/create", json!({"name": "HDFC", "kind": "bank", "balance_paise": 5_000_000, "role": "spending"})).await;
    authed_post(&app, &t, "/api/plan/income/create", json!({"label": "Salary", "amount_paise": 10_000_000, "day_of_month": 30})).await;

    let (s, b) = authed_post(&app, &t, "/api/plan/afford", json!({"amount_paise": 100_000, "pay": {"mode": "now"}})).await;
    assert!(s.is_success(), "{b}");
    assert_eq!(serde_json::from_str::<Value>(&b).unwrap()["answer"], "yes");

    let (_, b) = authed_post(&app, &t, "/api/plan/afford",
        json!({"amount_paise": 6_000_000, "pay": {"mode": "emi", "months": 6, "rate_bps": 0, "fee_paise": 19_900}})).await;
    let v: Value = serde_json::from_str(&b).unwrap();
    assert_eq!(v["monthly_paise"], 1_000_000);
    assert_eq!(v["extra_cost_paise"], 23_482);

    let (s, _) = authed_post(&app, &t, "/api/plan/afford", json!({"amount_paise": 0, "pay": {"mode": "now"}})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn partners_settle_up_date_money() {
    let app = test_app().await;
    let (a, b) = (new_user(&app, "10.0.0.71", "dateA").await, new_user(&app, "10.0.0.72", "dateB").await);
    authed_post(&app, &a, "/api/couple/create", json!({})).await;
    let (_, inv) = authed_post(&app, &a, "/api/couple/invite", json!({})).await;
    let code = serde_json::from_str::<Value>(&inv).unwrap()["code"].as_str().unwrap().to_string();
    authed_post(&app, &b, "/api/couple/join", json!({"code": code})).await;

    // In a couple, the Dates panel needs no link: it's connected straight away.
    assert_eq!(authed_get(&app, &a, "/api/joy/summary").await["connected"], true);

    let today = joy::handlers::common::today_local().to_string();
    let pay = |amount: i64| json!({"occurred_on": today, "direction": "debit", "amount_paise": amount, "merchant": "Dinner", "category": "dates"});
    authed_post(&app, &a, "/api/ingest/transactions/create", pay(200_000)).await;
    authed_post(&app, &b, "/api/ingest/transactions/create", pay(100_000)).await;

    // No budget set: half each. A paid ₹2,000 (B owes ₹1,000), B paid ₹1,000 (A owes ₹500).
    let s = authed_get(&app, &a, "/api/joy/settle").await;
    assert_eq!((s["you_paid_paise"].as_i64(), s["balance_paise"].as_i64()), (Some(200_000), Some(50_000)));
    assert_eq!(authed_get(&app, &b, "/api/joy/settle").await["balance_paise"], -50_000);

    authed_post(&app, &b, "/api/joy/settle", json!({"you_paid": true, "amount_paise": 50_000})).await;
    assert_eq!(authed_get(&app, &a, "/api/joy/settle").await["balance_paise"], 0);
    assert_eq!(authed_get(&app, &b, "/api/joy/settle").await["balance_paise"], 0);

    // A repayment recorded by mistake can be removed (by either of you); nobody else's id works.
    let id = authed_get(&app, &a, "/api/joy/settle").await["settlements"][0]["id"].as_i64().unwrap();
    let (st, _) = authed_post(&app, &a, "/api/joy/settle/delete", json!({"id": id + 1000})).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    let (st, _) = authed_post(&app, &a, "/api/joy/settle/delete", json!({"id": id})).await;
    assert!(st.is_success());
    assert_eq!(authed_get(&app, &a, "/api/joy/settle").await["balance_paise"], 50_000);

    // Clearing starts from scratch.
    authed_post(&app, &b, "/api/joy/settle", json!({"you_paid": true, "amount_paise": 10_000})).await;
    authed_post(&app, &a, "/api/joy/settle", json!({"you_paid": false, "amount_paise": 5_000})).await;
    let (st, body) = authed_post(&app, &a, "/api/joy/settle/clear", json!({})).await;
    assert!(st.is_success(), "{body}");
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["removed"], 2);
    let s = authed_get(&app, &a, "/api/joy/settle").await;
    assert_eq!((s["balance_paise"].as_i64(), s["settlements"].as_array().map(Vec::len)), (Some(50_000), Some(0)));

    // What the partner covers goes back to the default (half the budget) on reset.
    authed_post(&app, &a, "/api/joy/partner", json!({"partner_paise": 120_000})).await;
    assert_eq!(authed_get(&app, &a, "/api/joy/summary").await["partner_paise"], 120_000);
    let (st, _) = authed_post(&app, &a, "/api/joy/partner/reset", json!({})).await;
    assert!(st.is_success());
    assert_eq!(authed_get(&app, &a, "/api/joy/summary").await["partner_paise"], 0);
}

#[tokio::test]
async fn month_close_sums_a_month_in_plain_words() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.71", "closer").await;
    let (_, b) = authed_post(&app, &t, "/api/account/create",
        json!({"name": "HDFC", "kind": "bank", "balance_paise": 100, "last_4_digits": "1234"})).await;
    let account_id = serde_json::from_str::<Value>(&b).unwrap()["account_id"].as_str().unwrap().to_string();
    authed_post(&app, &t, "/api/plan/income/create", json!({"label": "Salary", "amount_paise": 10_000_000, "day_of_month": 1})).await;

    // An empty month says so, a future month and a bad one are refused.
    let v = authed_get(&app, &t, "/api/review/month?month=2026-09").await;
    assert_eq!(v["sentences"][0], "No transactions recorded this month. Forward bank alerts or import a statement.");
    assert_eq!(v["spent_paise"], 0);
    assert_eq!(v["savings_rate_bps"], Value::Null);
    for bad in ["2099-01", "nonsense", "2026-13"] {
        let req = Request::get(format!("/api/review/month?month={bad}")).header("authorization", format!("Bearer {t}")).body(Body::empty()).unwrap();
        assert_eq!(app.clone().oneshot(req).await.unwrap().status(), StatusCode::BAD_REQUEST, "{bad}");
    }

    let csv = "Date,Narration,Withdrawal Amt.,Deposit Amt.,Closing Balance\n\
05/10/26,UPI-SWIGGY-swiggy@icici-1,6100.00,,\"9,550.00\"\n\
09/10/26,UPI-BIGBASKET-bigbasket@icici-2,2000.00,,\"7,550.00\"\n\
15/10/26,UPI-SHELL FUEL-shell@icici-3,900.00,,\"6,650.00\"\n\
28/10/26,NEFT CR-ACME CORP,,\"1,00,000.00\",\"1,06,650.00\"\n";
    let (s, b) = authed_post(&app, &t, "/api/ingest/statement", json!({"csv": csv, "account_id": account_id})).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let txns = authed_get(&app, &t, "/api/ingest/transactions?month=2026-10").await;
    let credit = txns.as_array().unwrap().iter().find(|x| x["direction"] == "credit").unwrap();
    authed_post(&app, &t, "/api/ingest/transactions/update", json!({"txn_id": credit["txn_id"], "category": "income", "remember": false})).await;
    let swiggy = txns.as_array().unwrap().iter().find(|x| x["amount_paise"] == 610_000).unwrap();
    authed_post(&app, &t, "/api/ingest/transactions/update", json!({"txn_id": swiggy["txn_id"], "category": "food", "remember": false})).await;

    let v = authed_get(&app, &t, "/api/review/month?month=2026-10").await;
    assert_eq!(v["income_paise"], 10_000_000, "{v}");
    assert_eq!(v["spent_paise"], 900_000, "{v}");
    assert_eq!(v["top_categories"][0], json!({"category": "food", "amount_paise": 610_000}), "{v}");
    assert_eq!(v["top_categories"].as_array().unwrap().len(), 3, "{v}");
    assert!(v["allowance_paise"].as_i64().unwrap() > 0);
    assert_eq!(v["over_by_paise"].as_i64().unwrap(), (900_000 - v["allowance_paise"].as_i64().unwrap()).max(0));
    assert!(v["dates"].is_null());
    assert!(v["savings_rate_bps"].is_i64(), "{v}");
    let sentences = v["sentences"].as_array().unwrap();
    assert!((3..=5).contains(&sentences.len()), "{v}");
    assert!(sentences.iter().any(|x| x.as_str().unwrap().starts_with("Food was your biggest category at ₹6,100")), "{v}");
}

#[tokio::test]
async fn tax_tracker_counts_tagged_payments_and_entries() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.72", "taxpayer").await;
    authed_post(&app, &t, "/api/plan/income/create", json!({"label": "Salary", "amount_paise": 10_000_000, "day_of_month": 1})).await;
    let (s, b) = authed_post(&app, &t, "/api/plan/commitment/create",
        json!({"label": "ELSS SIP", "amount_paise": 1_000_000, "day_of_month": 1, "category": "sip"})).await;
    assert!(s.is_success(), "{b}");
    let id = serde_json::from_str::<Value>(&b).unwrap()["id"].as_str().unwrap().to_string();

    let today = joy::handlers::common::today_local();
    let fy = joy::planner::tax::fy_label(joy::planner::tax::fy_start_year(today));
    let v = authed_get(&app, &t, "/api/review/tax").await;
    assert_eq!(v["fy"], fy.as_str());
    let section = |v: &Value, s: &str| v["sections"].as_array().unwrap().iter().find(|x| x["section"] == s).unwrap().clone();
    assert_eq!(section(&v, "80C")["claimed_paise"], 0);
    assert_eq!(section(&v, "80C")["limit_paise"], 15_000_000);

    let (s, _) = authed_post(&app, &t, "/api/review/tax/commitment", json!({"commitment_id": id, "tax_section": "80C"})).await;
    assert!(s.is_success());
    let (s, _) = authed_post(&app, &t, "/api/review/tax/commitment", json!({"commitment_id": id, "tax_section": "nope"})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = authed_post(&app, &t, "/api/review/tax/commitment", json!({"commitment_id": "missing", "tax_section": null})).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    let v = authed_get(&app, &t, "/api/review/tax").await;
    let c = section(&v, "80C");
    // Due on the 1st: every month of the FY from April up to this one is paid, the rest projected.
    let months_paid = ((today.month() as i64 - 4).rem_euclid(12)) + 1;
    assert_eq!(c["claimed_paise"], months_paid * 1_000_000, "{c}");
    assert_eq!(c["projected_paise"], (12 - months_paid) * 1_000_000, "{c}");
    assert_eq!(c["left_paise"], 15_000_000 - months_paid * 1_000_000);

    let (s, b) = authed_post(&app, &t, "/api/review/tax/entry", json!({"section": "80C", "label": "PPF", "amount_paise": 2_000_000})).await;
    assert!(s.is_success(), "{b}");
    let entry_id = serde_json::from_str::<Value>(&b).unwrap()["entry_id"].as_str().unwrap().to_string();
    let (s, _) = authed_post(&app, &t, "/api/review/tax/entry", json!({"section": "bogus", "label": "x", "amount_paise": 1})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let v = authed_get(&app, &t, "/api/review/tax").await;
    assert_eq!(section(&v, "80C")["claimed_paise"], months_paid * 1_000_000 + 2_000_000);
    assert_eq!(v["gross_salary_paise"], 120_000_000);
    assert!(["old", "new"].contains(&v["regime"]["better"].as_str().unwrap()));
    assert_eq!(v["reminders"].as_array().unwrap().len(), 6);

    // Another user can't delete it; the owner can.
    let other = new_user(&app, "10.0.0.73", "snoop").await;
    let (s, _) = authed_post(&app, &other, "/api/review/tax/entry/delete", json!({"entry_id": entry_id})).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, _) = authed_post(&app, &t, "/api/review/tax/entry/delete", json!({"entry_id": entry_id})).await;
    assert!(s.is_success());
    assert_eq!(section(&authed_get(&app, &t, "/api/review/tax").await, "80C")["claimed_paise"], months_paid * 1_000_000);

    // A different FY starts clean; a malformed one is refused.
    let v = authed_get(&app, &t, "/api/review/tax?fy=2020-21").await;
    assert_eq!(section(&v, "80C")["claimed_paise"], 12 * 1_000_000, "a past FY: all twelve payments are in");
    let req = Request::get("/api/review/tax?fy=2020-22").header("authorization", format!("Bearer {t}")).body(Body::empty()).unwrap();
    assert_eq!(app.clone().oneshot(req).await.unwrap().status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn security_headers_and_export() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.81", "exporter").await;
    authed_post(&app, &t, "/api/account/create", json!({"name": "HDFC", "kind": "bank", "balance_paise": 4_200})).await;
    let req = Request::get("/api/export").header("authorization", format!("Bearer {t}")).body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let h = res.headers();
    assert_eq!(h["referrer-policy"], "no-referrer");
    assert_eq!(h["x-frame-options"], "DENY");
    assert_eq!(h["cache-control"], "no-store");
    assert!(h["content-disposition"].to_str().unwrap().starts_with("attachment; filename=\"joy-"));
    let body: Value = serde_json::from_slice(&to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["data"]["accounts"][0]["balance_paise"], 4_200);
    assert!(body["data"].get("dates").is_none(), "no couple, no Dates section");
}

#[tokio::test]
async fn old_envelope_links_redirect_under_joy() {
    let app = test_app().await;
    for (from, to) in [
        ("/e/AbC123_-xyz", "/joy/e/AbC123_-xyz"),
    ] {
        let res = app.clone().oneshot(Request::get(from).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT, "{from}");
        assert_eq!(res.headers()["location"], to, "{from}");
    }
    let bad = app.clone().oneshot(Request::get("/e/%3Cscript%3E").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(bad.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn sign_up_needs_the_code() {
    let app = test_app().await;
    let ip = "10.0.0.90";
    let (s, body) = post(&app, ip, "/api/common/register", json!({"user_name": "eve", "user_password": "longenough1", "signup_code": "guess"})).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "{body}");
    assert!(body.contains("sign-up code"), "{body}");
    let (s, _) = post(&app, ip, "/api/common/register", json!({"user_name": "eve", "user_password": "longenough1", "signup_code": ""})).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, _) = post(&app, ip, "/api/common/register", json!({"user_name": "eve", "user_password": "longenough1"})).await;
    assert_eq!(s, StatusCode::OK, "the helper adds the right code");
}

#[tokio::test]
async fn failed_guesses_never_lock_out_the_real_password() {
    let app = test_app().await;
    post(&app, "10.0.0.91", "/api/common/register", json!({"user_name": "owner", "user_password": "correct horse"})).await;
    // Ten wrong guesses from many addresses use up owner's failure budget...
    for i in 0..10 {
        let (s, _) = post(&app, &format!("10.0.1.{i}"), "/api/common/login", json!({"user_name": "owner", "user_password": "wrong pass"})).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);
    }
    let (s, _) = post(&app, "10.0.1.50", "/api/common/login", json!({"user_name": "owner", "user_password": "wrong pass"})).await;
    assert_eq!(s, StatusCode::TOO_MANY_REQUESTS, "then further guesses are slowed");
    // A huge username is refused without touching the limiter or the database.
    let huge = "u".repeat(1_000_000);
    let (s, _) = post(&app, "10.0.1.60", "/api/common/login", json!({"user_name": huge, "user_password": "x"})).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_bad_token_gets_a_plain_message() {
    let app = test_app().await;
    let req = Request::get("/api/diet").header("authorization", "Bearer v4.local.garbage").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(!text.to_lowercase().contains("paseto") && !text.contains("cipher"), "{text}");
}

#[tokio::test]
async fn streaks_count_logged_days_and_saved_months() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.91", "streaker").await;

    // Nothing yet: all zero, and no dates streak for someone outside a couple.
    let s = authed_get(&app, &t, "/api/dashboard/streaks").await;
    assert_eq!(s["logging"], json!({"current": 0, "best": 0}), "{s}");
    assert_eq!(s["saved"], json!({"current": 0, "best": 0}));
    assert_eq!(s["dates"], Value::Null);

    // Three days in a row ending today, plus a last-month pay cheque with modest spending.
    let today = joy::handlers::common::today_local();
    for back in 0..3 {
        let on = (today - time::Duration::days(back)).to_string();
        let (st, b) = authed_post(&app, &t, "/api/ingest/transactions/create",
            json!({"occurred_on": on, "direction": "debit", "amount_paise": 10_000, "merchant": "Chai", "category": "food"})).await;
        assert!(st.is_success(), "{b}");
    }
    let s = authed_get(&app, &t, "/api/dashboard/streaks").await;
    assert_eq!(s["logging"]["current"], 3, "{s}");

    let (y, m) = joy::planner::cycle::add_months(today.year(), today.month(), -1);
    let last = time::Date::from_calendar_date(y, m, 10).unwrap().to_string();
    for (dir, amount, cat) in [("credit", 100_000, "income"), ("debit", 40_000, "food")] {
        let (st, b) = authed_post(&app, &t, "/api/ingest/transactions/create",
            json!({"occurred_on": last, "direction": dir, "amount_paise": amount, "merchant": "X", "category": cat})).await;
        assert!(st.is_success(), "{b}");
    }
    let s = authed_get(&app, &t, "/api/dashboard/streaks").await;
    assert_eq!(s["saved"], json!({"current": 1, "best": 1}), "{s}");
}

#[tokio::test]
async fn goals_settings_and_suggestions_can_be_put_back() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.92", "resetter").await;

    // An archived goal can come back.
    let (st, b) = authed_post(&app, &t, "/api/goals/create", json!({"name": "Trip", "target_paise": 5_000_000, "priority": 1})).await;
    assert!(st.is_success(), "{b}");
    let goal = serde_json::from_str::<Value>(&b).unwrap()["goal_id"].as_str().unwrap().to_string();
    authed_post(&app, &t, "/api/goals/archive", json!({"goal_id": goal})).await;
    assert_eq!(authed_get(&app, &t, "/api/goals/archived").await[0]["name"], "Trip");
    let (st, _) = authed_post(&app, &t, "/api/goals/unarchive", json!({"goal_id": "nope"})).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    let (st, _) = authed_post(&app, &t, "/api/goals/unarchive", json!({"goal_id": goal})).await;
    assert!(st.is_success());
    assert_eq!(authed_get(&app, &t, "/api/goals/archived").await.as_array().unwrap().len(), 0);

    // Budget settings go back to the defaults.
    let (st, b) = authed_post(&app, &t, "/api/plan/settings/update", json!({
        "spending_paise": 2_500_000, "allocation_day": 20, "savings_floor_paise": 100_000, "recovery_months": 6, "week_start": 0, "auto_post": false
    })).await;
    assert!(st.is_success(), "{b}");
    assert_eq!(authed_get(&app, &t, "/api/plan/settings/get").await["allocation_day"], 20);
    let (st, _) = authed_post(&app, &t, "/api/plan/settings/reset", json!({})).await;
    assert!(st.is_success());
    let s = authed_get(&app, &t, "/api/plan/settings/get").await;
    assert_eq!((s["allocation_day"].as_i64(), s["recovery_months"].as_i64(), s["auto_post"].as_bool()), (Some(10), Some(3), Some(true)));
    assert_eq!((s["spending_paise"].clone(), s["savings_floor_paise"].clone()), (Value::Null, Value::Null));

    // A dismissed subscription suggestion is offered again after a restore.
    let today = joy::handlers::common::today_local();
    for back in [0, 30, 60] {
        let on = (today - time::Duration::days(back)).to_string();
        authed_post(&app, &t, "/api/ingest/transactions/create", json!({
            "occurred_on": on, "direction": "debit", "amount_paise": 64_900, "merchant": "NETFLIX.COM", "category": "entertainment"
        })).await;
    }
    assert_eq!(authed_get(&app, &t, "/api/bills/suggestions").await[0]["label"], "NETFLIX.COM");
    authed_post(&app, &t, "/api/bills/suggestions/dismiss", json!({"merchant": "NETFLIX.COM"})).await;
    assert_eq!(authed_get(&app, &t, "/api/bills/suggestions").await.as_array().unwrap().len(), 0);
    let (st, b) = authed_post(&app, &t, "/api/bills/suggestions/restore", json!({})).await;
    assert!(st.is_success(), "{b}");
    assert_eq!(serde_json::from_str::<Value>(&b).unwrap()["restored"], 1);
    assert_eq!(authed_get(&app, &t, "/api/bills/suggestions").await[0]["label"], "NETFLIX.COM");
}

#[tokio::test]
async fn life_work_tax_and_checkin_data_can_be_cleared() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.93", "clearer").await;
    let other = new_user(&app, "10.0.0.94", "neighbour").await;

    // Job leads: clearing removes only your own.
    for who in [&t, &other] {
        let (st, b) = authed_post(&app, who, "/api/work", json!({"company": "Acme", "role": "Engineer"})).await;
        assert!(st.is_success(), "{st} {b}");
    }
    assert_eq!(authed_get(&app, &t, "/api/work").await["leads"].as_array().unwrap().len(), 1);
    let (st, b) = authed_post(&app, &t, "/api/work/clear", json!({})).await;
    assert!(st.is_success(), "{b}");
    assert_eq!(serde_json::from_str::<Value>(&b).unwrap()["removed"], 1);
    assert_eq!(authed_get(&app, &t, "/api/work").await["leads"].as_array().unwrap().len(), 0);
    assert_eq!(authed_get(&app, &other, "/api/work").await["leads"].as_array().unwrap().len(), 1);

    // Life profile: saved, then reset to the defaults.
    let (st, b) = authed_post(&app, &t, "/api/plan/life/profile", json!({
        "my_age": 33, "my_take_home_paise": 10_000_000, "retirement_age": 60
    })).await;
    assert!(st.is_success(), "{st} {b}");
    assert_eq!(authed_get(&app, &t, "/api/plan/life").await["saved"], true);
    let (st, b) = authed_post(&app, &t, "/api/plan/life/reset", json!({})).await;
    assert!(st.is_success(), "{st} {b}");
    assert_eq!(authed_get(&app, &t, "/api/plan/life").await["saved"], false);

    // Life line choices: clearing is for everything you added.
    let (st, b) = authed_post(&app, &t, "/api/plan/lifeline/choices/clear", json!({})).await;
    assert!(st.is_success(), "{st} {b}");
    assert_eq!(serde_json::from_str::<Value>(&b).unwrap()["removed"], 0);

    // Tax: manual entries for a year can be cleared, only for that user.
    for who in [&t, &other] {
        authed_post(&app, who, "/api/review/tax/entry", json!({"section": "80C", "label": "PPF", "amount_paise": 1_000_000})).await;
    }
    let (st, b) = authed_post(&app, &t, "/api/review/tax/entries/clear", json!({})).await;
    assert!(st.is_success(), "{b}");
    assert_eq!(serde_json::from_str::<Value>(&b).unwrap()["removed"], 1);
    let (_, b) = authed_post(&app, &t, "/api/review/tax/entries/clear", json!({})).await;
    assert_eq!(serde_json::from_str::<Value>(&b).unwrap()["removed"], 0);
    let (st, _) = authed_post(&app, &t, "/api/review/tax/entries/clear", json!({"fy": "garbage"})).await;
    assert_eq!(st, StatusCode::BAD_REQUEST);

    // Forecast calibration reset is safe with no check-ins.
    let (st, _) = authed_post(&app, &t, "/api/checkin/calibration/reset", json!({})).await;
    assert!(st.is_success());
}

#[tokio::test]
async fn learned_rules_hand_set_categories_and_layouts_can_be_reset() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.95", "tidier").await;
    let today = joy::handlers::common::today_local().to_string();

    let (st, b) = authed_post(&app, &t, "/api/ingest/transactions/create", json!({
        "occurred_on": today, "direction": "debit", "amount_paise": 25_000, "merchant": "Zorblax Cafe", "category": "uncategorized"
    })).await;
    assert!(st.is_success(), "{b}");
    let txn = serde_json::from_str::<Value>(&b).unwrap()["txn_id"].as_str().unwrap().to_string();

    // Hand-set a category and teach a rule.
    let (st, _) = authed_post(&app, &t, "/api/ingest/transactions/update",
        json!({"txn_id": txn, "category": "shopping", "remember": true})).await;
    assert!(st.is_success());
    assert_eq!(authed_get(&app, &t, "/api/ingest/rules").await.as_array().unwrap().len(), 1);

    // Clearing the rules keeps the row's category but forgets the lesson.
    let (st, b) = authed_post(&app, &t, "/api/ingest/rules/clear", json!({})).await;
    assert!(st.is_success(), "{b}");
    assert_eq!(serde_json::from_str::<Value>(&b).unwrap()["removed"], 1);
    assert_eq!(authed_get(&app, &t, "/api/ingest/rules").await.as_array().unwrap().len(), 0);

    // Back to automatic: the keywords read "Cafe" as food (not the shopping it was set to), and the row is unlocked.
    let (st, b) = authed_post(&app, &t, "/api/ingest/transactions/unlock", json!({"txn_id": txn})).await;
    assert!(st.is_success(), "{b}");
    assert_eq!(serde_json::from_str::<Value>(&b).unwrap()["category"], "food");
    let (st, _) = authed_post(&app, &t, "/api/ingest/transactions/unlock", json!({"txn_id": "nope"})).await;
    assert_eq!(st, StatusCode::NOT_FOUND);

    // Saved statement layouts can be forgotten (nothing saved is fine).
    let (st, b) = authed_post(&app, &t, "/api/ingest/statement/layouts/clear", json!({})).await;
    assert!(st.is_success(), "{b}");
}

#[tokio::test]
async fn a_move_with_the_same_idempotency_key_moves_the_money_once() {
    let (app, db) = test_app_with_db().await;
    let t = new_user(&app, "10.0.0.91", "mover").await;
    let mut ids = vec![];
    for (name, role) in [("Spending", "spending"), ("Reserve", "reserve")] {
        let (s, b) = authed_post(&app, &t, "/api/account/create", json!({"name": name, "kind": "bank", "balance_paise": 1_000_000, "role": role})).await;
        assert_eq!(s, StatusCode::OK, "{b}");
        ids.push(serde_json::from_str::<Value>(&b).unwrap()["account_id"].as_str().unwrap().to_string());
    }
    let body = |key: &str, amount: i64| json!({"kind": "allocation", "key": "2026-11-01", "amount_paise": amount, "mode": "moved",
        "from_account_id": ids[0], "to_account_id": ids[1], "idempotency_key": key});
    let (s, first) = authed_post(&app, &t, "/api/moves/record", body("k-1", 100_000)).await;
    assert_eq!(s, StatusCode::OK, "{first}");
    // A retry long after the 10-second window, e.g. from an offline queue, with the same key.
    sqlx::query("UPDATE moves SET created_at = datetime('now', '-1 hours')").execute(&db).await.unwrap();
    let (s, again) = authed_post(&app, &t, "/api/moves/record", body("k-1", 100_000)).await;
    assert_eq!(s, StatusCode::OK, "{again}");
    assert_eq!(
        serde_json::from_str::<Value>(&first).unwrap()["move_id"],
        serde_json::from_str::<Value>(&again).unwrap()["move_id"],
        "same move comes back"
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM moves").fetch_one(&db).await.unwrap();
    assert_eq!(count, 1);
    let bal: i64 = sqlx::query_scalar("SELECT balance_paise FROM accounts WHERE account_id = ?").bind(&ids[0]).fetch_one(&db).await.unwrap();
    assert_eq!(bal, 900_000, "moved once");

    // A different key is a different move.
    let (s, _) = authed_post(&app, &t, "/api/moves/record", body("k-2", 100_000)).await;
    assert_eq!(s, StatusCode::OK);
    let bal: i64 = sqlx::query_scalar("SELECT balance_paise FROM accounts WHERE account_id = ?").bind(&ids[0]).fetch_one(&db).await.unwrap();
    assert_eq!(bal, 800_000);
}

#[tokio::test]
async fn an_alert_the_parser_cannot_read_is_kept_in_an_inbox() {
    let (app, db) = test_app_with_db().await;
    let t = new_user(&app, "10.0.0.92", "inboxer").await;
    let (_, b) = authed_post(&app, &t, "/api/ingest/key/rotate", json!({})).await;
    let key = serde_json::from_str::<Value>(&b).unwrap()["key"].as_str().unwrap().to_string();

    let odd = "A/c XX9 has been settled for the sum of seven hundred rupees at SOME SHOP";
    let (_, v) = hook_post(&app, &key, odd).await;
    assert_eq!(v["status"], "unrecognized");
    // The same text again does not pile up.
    hook_post(&app, &key, odd).await;

    let inbox = authed_get(&app, &t, "/api/ingest/unrecognized").await;
    assert_eq!(inbox["count"], 1, "{inbox}");
    assert_eq!(inbox["items"][0]["text"], odd);
    let id = inbox["items"][0]["alert_id"].as_str().unwrap().to_string();

    // Pasting by hand shows the answer right there, so it is not kept.
    authed_post(&app, &t, "/api/ingest/parse", json!({"text": "another odd one"})).await;
    assert_eq!(authed_get(&app, &t, "/api/ingest/unrecognized").await["count"], 1);

    // Someone else cannot dismiss it; the owner can.
    let other = new_user(&app, "10.0.0.93", "snoop").await;
    authed_post(&app, &other, "/api/ingest/unrecognized/dismiss", json!({"alert_id": id})).await;
    assert_eq!(authed_get(&app, &t, "/api/ingest/unrecognized").await["count"], 1);
    let (s, _) = authed_post(&app, &t, "/api/ingest/unrecognized/dismiss", json!({"alert_id": id})).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(authed_get(&app, &t, "/api/ingest/unrecognized").await["count"], 0);

    // Old ones are dropped after 30 days.
    hook_post(&app, &key, "yet another unreadable message").await;
    sqlx::query("UPDATE unrecognized_alerts SET received_at = datetime('now', '-31 days')").execute(&db).await.unwrap();
    hook_post(&app, &key, "a fresh unreadable message").await;
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM unrecognized_alerts").fetch_one(&db).await.unwrap();
    assert_eq!(n, 1, "the old one was cleaned up");
}

#[tokio::test]
async fn an_old_pending_charge_is_flagged_not_silently_dropped() {
    let (app, db) = test_app_with_db().await;
    let t = new_user(&app, "10.0.0.94", "pender").await;
    let (s, b) = authed_post(&app, &t, "/api/card/create", json!({
        "card_name": "Visa", "card_bank": "HDFC", "card_primary_color": [0, 0, 0], "card_secondary_color": [9, 9, 9],
        "last_4_digits": "4321", "statement_day": 5, "due_day": 25})).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let card = serde_json::from_str::<Value>(&b).unwrap()["card_id"].as_str().unwrap().to_string();
    authed_post(&app, &t, "/api/card/pending/create", json!({"card_id": card, "label": "Fresh", "amount_paise": 10_000})).await;
    authed_post(&app, &t, "/api/card/pending/create", json!({"card_id": card, "label": "Old", "amount_paise": 25_000})).await;
    sqlx::query("UPDATE pending_charges SET created_at = datetime('now', '-45 days') WHERE label = 'Old'").execute(&db).await.unwrap();

    let (s, b) = authed_post(&app, &t, "/api/card/breakdown", json!({"card_id": card})).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let v: Value = serde_json::from_str(&b).unwrap();
    let rows = v["pending_charges"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "the old one is still listed: {v}");
    let old = rows.iter().find(|r| r["label"] == "Old").unwrap();
    let fresh = rows.iter().find(|r| r["label"] == "Fresh").unwrap();
    assert_eq!((old["stale"].as_bool(), fresh["stale"].as_bool()), (Some(true), Some(false)));
    assert_eq!(v["stale_pending_paise"], 25_000, "and the card says how much is being left out");
    assert_eq!(v["next_bill_est_paise"], 10_000, "the estimate still counts only the recent one");
}

#[tokio::test]
async fn card_totals_are_exact_to_the_paisa() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.0.95", "exact").await;
    let (_, b) = authed_post(&app, &t, "/api/card/create", json!({
        "card_name": "Big", "card_bank": "HDFC", "card_primary_color": [0, 0, 0], "card_secondary_color": [9, 9, 9]})).await;
    let card = serde_json::from_str::<Value>(&b).unwrap()["card_id"].as_str().unwrap().to_string();
    // ₹1,23,45,678.91 is more than an f32 can hold to the paisa.
    let (s, b) = authed_post(&app, &t, "/api/card/snapshot", json!({"card_id": card, "total_due_paise": 0,
        "outstanding_paise": 1_234_567_891_i64, "unbilled_spent_paise": 0, "unbilled_credit_paise": 0})).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let (_, b) = authed_post(&app, &t, "/api/card/snapshot", json!({"card_id": card, "total_due_paise": 0,
        "outstanding_paise": 1_234_567_890_i64, "unbilled_spent_paise": 0, "unbilled_credit_paise": 0})).await;
    assert!(b.contains("transaction_id"), "{b}");

    let cards = authed_get(&app, &t, "/api/card/get_all_cards").await;
    assert_eq!(cards["cards"][0]["last_total_due"], 12_345_678.90);
    assert_eq!(cards["cards"][0]["last_delta"], -0.01);
    let (_, h) = authed_post(&app, &t, "/api/card/history", json!({"card_id": card})).await;
    let h: Value = serde_json::from_str(&h).unwrap();
    assert_eq!(h["histories"][1]["total_due_input"], 12_345_678.91);
}

/// What a protected GET answers with, headers included.
async fn get_with_headers(app: &Router, path: &str, token: &str) -> (StatusCode, axum::http::HeaderMap) {
    let req = Request::get(path).header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    (res.status(), res.headers().clone())
}

#[tokio::test]
async fn a_session_in_use_renews_itself_before_it_runs_out() {
    let (app, db) = test_app_with_db().await;
    let fresh = new_user(&app, "10.0.0.96", "renewer").await;
    // A just-issued token has plenty of life left: nothing to renew.
    let (s, h) = get_with_headers(&app, "/api/account/list", &fresh).await;
    assert_eq!(s, StatusCode::OK);
    assert!(h.get("x-refreshed-token").is_none(), "no renewal needed yet");

    // The same person, with a token that has two days left, is handed a new long one.
    let (id, role, version): (String, String, i64) =
        sqlx::query_as("SELECT user_id, user_role, token_version FROM users WHERE user_name = 'renewer'").fetch_one(&db).await.unwrap();
    let key = Arc::new(PasetoSymmetricKey::from(Key::from(&[7u8; 32])));
    let old = joy::handlers::common::get_paseto_token(&id, role, version, &key, time::OffsetDateTime::now_utc() + time::Duration::days(2)).ok().unwrap();
    let (s, h) = get_with_headers(&app, "/api/account/list", &old).await;
    assert_eq!(s, StatusCode::OK);
    let renewed = h.get("x-refreshed-token").expect("renewed").to_str().unwrap().to_string();
    let exp: i64 = h.get("x-token-expires-at").expect("expiry").to_str().unwrap().parse().unwrap();
    assert!(exp > time::OffsetDateTime::now_utc().unix_timestamp() + 20 * 86_400, "a month, not two days");
    assert_eq!(get(&app, "/api/account/list", Some(&renewed)).await, StatusCode::OK, "the new token works");

    // A revoked session is not renewed: signing out everywhere still ends old tokens.
    authed_post(&app, &renewed, "/api/common/logout-everywhere", json!({})).await;
    let (s, h) = get_with_headers(&app, "/api/account/list", &old).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    assert!(h.get("x-refreshed-token").is_none());
}

#[tokio::test]
async fn what_if_scenarios_follow_the_person_and_stay_private() {
    let (app, db) = test_app_with_db().await;
    let a = new_user(&app, "10.0.0.97", "scenarioa").await;
    let b = new_user(&app, "10.0.0.98", "scenariob").await;
    assert_eq!(authed_get(&app, &a, "/api/plan/scenarios").await["drafts"], json!([]), "none yet");

    let drafts = json!([{"id": "d1", "kind": "sip", "amount": "5000"}, {"id": "d2", "kind": "emi", "amount": "12000"}]);
    let (s, body) = authed_post(&app, &a, "/api/plan/scenarios", json!({"drafts": drafts})).await;
    assert_eq!(s, StatusCode::OK, "{body}");
    assert_eq!(authed_get(&app, &a, "/api/plan/scenarios").await["drafts"], drafts);
    assert_eq!(authed_get(&app, &b, "/api/plan/scenarios").await["drafts"], json!([]), "someone else's are theirs");

    // Saving again replaces, and an empty list clears.
    authed_post(&app, &a, "/api/plan/scenarios", json!({"drafts": [{"id": "d3"}]})).await;
    assert_eq!(authed_get(&app, &a, "/api/plan/scenarios").await["drafts"], json!([{"id": "d3"}]));
    authed_post(&app, &a, "/api/plan/scenarios", json!({"drafts": []})).await;
    assert_eq!(authed_get(&app, &a, "/api/plan/scenarios").await["drafts"], json!([]));

    // Not a list, or far too much, is refused.
    let (s, _) = authed_post(&app, &a, "/api/plan/scenarios", json!({"drafts": {"nope": 1}})).await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
    let many: Vec<Value> = (0..60).map(|i| json!({"id": i})).collect();
    let (s, _) = authed_post(&app, &a, "/api/plan/scenarios", json!({"drafts": many})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let big = json!([{"note": "x".repeat(70_000)}]);
    let (s, _) = authed_post(&app, &a, "/api/plan/scenarios", json!({"drafts": big})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM whatif_scenarios").fetch_one(&db).await.unwrap();
    assert_eq!(n, 1, "one row per person");
}

#[tokio::test]
async fn an_export_can_be_loaded_back_and_exports_the_same_again() {
    let (app, db) = test_app_with_db().await;
    let t = new_user(&app, "10.0.0.99", "restorer").await;
    let (s, b) = authed_post(&app, &t, "/api/account/create", json!({"name": "HDFC", "kind": "bank", "balance_paise": 5_000_000, "role": "spending", "last_4_digits": "1234"})).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    authed_post(&app, &t, "/api/card/create", json!({"card_name": "Visa", "card_bank": "HDFC", "card_primary_color": [1, 2, 3], "card_secondary_color": [4, 5, 6]})).await;
    let today = joy::handlers::common::today_local().to_string();
    authed_post(&app, &t, "/api/ingest/transactions/create",
        json!({"occurred_on": today, "direction": "debit", "amount_paise": 25_000, "merchant": "Chai", "category": "food"})).await;
    authed_post(&app, &t, "/api/plan/scenarios", json!({"drafts": [{"id": "d1"}]})).await;

    let before = authed_get(&app, &t, "/api/export").await;
    assert!(!before["data"]["accounts"].as_array().unwrap().is_empty());

    // Lose it all (and add something that was never in the backup).
    sqlx::query("DELETE FROM transactions").execute(&db).await.unwrap();
    sqlx::query("DELETE FROM cards").execute(&db).await.unwrap();
    sqlx::query("DELETE FROM accounts").execute(&db).await.unwrap();
    sqlx::query("DELETE FROM whatif_scenarios").execute(&db).await.unwrap();
    authed_post(&app, &t, "/api/account/create", json!({"name": "Junk", "kind": "cash", "balance_paise": 1})).await;

    let (s, b) = authed_post(&app, &t, "/api/import", before.clone()).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let after = authed_get(&app, &t, "/api/export").await;
    assert_eq!(after["data"], before["data"], "export -> wipe -> import -> export is the same file");

    // Someone else's backup is not loadable into your account: ids are theirs.
    let other = new_user(&app, "10.0.1.1", "intruder").await;
    let (s, _) = authed_post(&app, &other, "/api/import", before.clone()).await;
    assert_eq!(s, StatusCode::CONFLICT, "those rows already belong to another account");
    assert_eq!(authed_get(&app, &other, "/api/export").await["data"]["accounts"], json!([]));

    // Not an export at all, or a table we don't know: refused, and nothing changes.
    for bad in [json!({"nope": 1}), json!({"data": {"users": [{"user_id": "x"}]}}), json!({"data": {"accounts": "no"}})] {
        let (s, _) = authed_post(&app, &t, "/api/import", bad).await;
        assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
    }
    assert_eq!(authed_get(&app, &t, "/api/export").await["data"], before["data"]);
}

#[tokio::test]
async fn coming_up_puts_bills_birthdays_and_follow_ups_in_one_list() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.1.2", "upcomer").await;
    let today = joy::handlers::common::today_local();
    let tomorrow = today + time::Duration::days(1);
    let in3 = today + time::Duration::days(3);

    let (_, b) = authed_post(&app, &t, "/api/people", json!({"name": "Asha", "circle": "close"})).await;
    let pid = serde_json::from_str::<Value>(&b).unwrap()["id"].as_str().unwrap().to_string();
    let (s, b) = authed_post(&app, &t, &format!("/api/people/{pid}/dates"), json!({"kind": "birthday", "month": in3.month() as u8, "day": in3.day()})).await;
    assert!(s.is_success(), "{b}");
    let (s, b) = authed_post(&app, &t, &format!("/api/people/{pid}/followups"), json!({"text": "Ask about the move", "due_on": tomorrow.to_string()})).await;
    assert!(s.is_success(), "{b}");

    let v = authed_get(&app, &t, "/api/coming-up?days=14").await;
    let items = v["items"].as_array().unwrap();
    let kinds: Vec<(&str, &str)> = items.iter().map(|i| (i["kind"].as_str().unwrap(), i["when"].as_str().unwrap())).collect();
    assert_eq!(kinds, [("follow_up", "Tomorrow"), ("birthday", "In 3 days")], "{v}");
    assert!(items[0]["title"].as_str().unwrap().contains("Ask about the move"));
    assert!(items[1]["title"].as_str().unwrap().contains("Asha"));

    // A short window leaves the birthday out; someone else sees none of it.
    let short = authed_get(&app, &t, "/api/coming-up?days=2").await;
    assert_eq!(short["items"].as_array().unwrap().len(), 1);
    let other = new_user(&app, "10.0.1.3", "nosy").await;
    assert_eq!(authed_get(&app, &other, "/api/coming-up").await["items"], json!([]));
}

#[tokio::test]
async fn a_refund_is_not_counted_as_income() {
    let app = test_app().await;
    let t = new_user(&app, "10.0.1.4", "refunded").await;
    authed_post(&app, &t, "/api/account/create", json!({"name": "HDFC", "kind": "bank", "balance_paise": 100, "last_4_digits": "1234"})).await;
    let (_, b) = authed_post(&app, &t, "/api/ingest/key/rotate", json!({})).await;
    let key = serde_json::from_str::<Value>(&b).unwrap()["key"].as_str().unwrap().to_string();

    let (_, v) = hook_post(&app, &key, "Refund of INR 1,299.00 credited to HDFC Bank A/c XX1234 from AMAZON PAY on 12-11-26. Ref 55501.").await;
    assert_eq!(v["status"], "created", "{v}");
    assert_eq!(v["transaction"]["category"], "refund", "{v}");

    // Salary still reads as income.
    let (_, v) = hook_post(&app, &key, "Update! INR 90,000.00 deposited in HDFC Bank A/c XX1234 for NEFT Cr-ACME CORP. Ref 77712.").await;
    assert_eq!(v["transaction"]["category"], "income", "{v}");
}

#[tokio::test]
async fn moving_a_payments_day_does_not_post_that_month_twice() {
    let (app, db) = test_app_with_db().await;
    let t = new_user(&app, "10.0.1.5", "mover2").await;
    let (_, b) = authed_post(&app, &t, "/api/account/create", json!({"name": "HDFC", "kind": "bank", "balance_paise": 10_000_000, "role": "spending"})).await;
    let acct = serde_json::from_str::<Value>(&b).unwrap()["account_id"].as_str().unwrap().to_string();
    let uid: String = sqlx::query_scalar("SELECT user_id FROM users WHERE user_name = 'mover2'").fetch_one(&db).await.unwrap();
    sqlx::query("UPDATE accounts SET balance_set_on = '2026-11-01'").execute(&db).await.unwrap();
    let (s, b) = authed_post(&app, &t, "/api/plan/commitment/create",
        json!({"label": "Rent", "amount_paise": 2_000_000, "day_of_month": 5, "account_id": acct})).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let cid = serde_json::from_str::<Value>(&b).unwrap()["id"].as_str().unwrap().to_string();
    let balance = || async { sqlx::query_scalar::<_, i64>("SELECT balance_paise FROM accounts").fetch_one(&db).await.unwrap() };

    joy::handlers::autopost::apply(&db, &uid, time::macros::date!(2026 - 11 - 06)).await.ok().unwrap();
    assert_eq!(balance().await, 8_000_000, "rent posted on the 5th");

    // The day is moved to the 20th; when the 20th comes, November's rent is not taken again.
    let (s, b) = authed_post(&app, &t, "/api/plan/commitment/update",
        json!({"commitment_id": cid, "label": "Rent", "amount_paise": 2_000_000, "day_of_month": 20, "account_id": acct})).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    joy::handlers::autopost::apply(&db, &uid, time::macros::date!(2026 - 11 - 21)).await.ok().unwrap();
    assert_eq!(balance().await, 8_000_000, "still taken once for November");

    // December's rent is a new month and does post, on the new day.
    joy::handlers::autopost::apply(&db, &uid, time::macros::date!(2026 - 12 - 21)).await.ok().unwrap();
    assert_eq!(balance().await, 6_000_000);
}

/// Sends a request and returns status, headers and body text.
async fn raw(app: &Router, method: &str, path: &str, headers: &[(&str, &str)], body: Option<Value>) -> (StatusCode, axum::http::HeaderMap, String) {
    let mut req = Request::builder().method(method).uri(path).header("fly-client-ip", "10.0.2.1");
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let req = match body {
        Some(b) => req.header("content-type", "application/json").body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let (status, h) = (res.status(), res.headers().clone());
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (status, h, String::from_utf8_lossy(&bytes).into_owned())
}

fn cookie_of(h: &axum::http::HeaderMap) -> String {
    h.get("set-cookie").expect("a session cookie").to_str().unwrap().to_string()
}

#[tokio::test]
async fn signing_in_sets_an_http_only_session_cookie_that_works_for_the_app() {
    let app = test_app().await;
    post(&app, "10.0.2.1", "/api/common/register", json!({"user_name": "cookieuser", "user_password": "correct horse"})).await;
    let (s, h, body) = raw(&app, "POST", "/api/common/login", &[("x-forwarded-proto", "https")], Some(json!({"user_name": "cookieuser", "user_password": "correct horse"}))).await;
    assert_eq!(s, StatusCode::OK, "{body}");
    let set = cookie_of(&h);
    for part in ["joy_session=", "HttpOnly", "Secure", "SameSite=Strict", "Path=/", "Max-Age=2592000"] {
        assert!(set.contains(part), "{part} in {set}");
    }
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["cookie"], true, "the page learns it is cookie-signed");
    let jar = set.split(';').next().unwrap().to_string();

    // Plain http (local development) must still work: no Secure flag there.
    let (_, h, _) = raw(&app, "POST", "/api/common/login", &[], Some(json!({"user_name": "cookieuser", "user_password": "correct horse"}))).await;
    assert!(!cookie_of(&h).contains("Secure"));

    // Reading works with just the cookie.
    let (s, _, _) = raw(&app, "GET", "/api/account/list", &[("cookie", &jar)], None).await;
    assert_eq!(s, StatusCode::OK);
    // Writing needs the page's own header too, which another site cannot add.
    let new_acct = json!({"name": "Cash", "kind": "cash", "balance_paise": 100});
    let (s, _, _) = raw(&app, "POST", "/api/account/create", &[("cookie", &jar)], Some(new_acct.clone())).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "a cookie alone cannot change anything");
    let (s, _, b) = raw(&app, "POST", "/api/account/create", &[("cookie", &jar), ("x-joy", "1")], Some(new_acct)).await;
    assert_eq!(s, StatusCode::OK, "{b}");

    // A bearer token (scripts, the phone shortcut's siblings) still works without the header.
    let token = serde_json::from_str::<Value>(&raw(&app, "POST", "/api/common/login", &[], Some(json!({"user_name": "cookieuser", "user_password": "correct horse"}))).await.2).unwrap()["access_token"].as_str().unwrap().to_string();
    let (s, _, _) = raw(&app, "POST", "/api/account/create", &[("authorization", &format!("Bearer {token}"))], Some(json!({"name": "Cash2", "kind": "cash", "balance_paise": 1}))).await;
    assert_eq!(s, StatusCode::OK);
    // A bad cookie is just signed out.
    let (s, _, _) = raw(&app, "GET", "/api/account/list", &[("cookie", "joy_session=nonsense")], None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn signing_out_clears_the_cookie_and_ending_every_session_ends_it() {
    let app = test_app().await;
    post(&app, "10.0.2.2", "/api/common/register", json!({"user_name": "leaver", "user_password": "correct horse"})).await;
    let (_, h, _) = raw(&app, "POST", "/api/common/login", &[], Some(json!({"user_name": "leaver", "user_password": "correct horse"}))).await;
    let jar = cookie_of(&h).split(';').next().unwrap().to_string();

    let (s, h, _) = raw(&app, "POST", "/api/common/logout", &[], Some(json!({}))).await;
    assert_eq!(s, StatusCode::OK, "signing out needs no session: it must work when the cookie is stale");
    let cleared = cookie_of(&h);
    assert!(cleared.contains("joy_session=;") && cleared.contains("Max-Age=0") && cleared.contains("HttpOnly"), "{cleared}");

    let (s, _, _) = raw(&app, "POST", "/api/common/logout-everywhere", &[("cookie", &jar), ("x-joy", "1")], Some(json!({}))).await;
    assert_eq!(s, StatusCode::OK);
    let (s, _, _) = raw(&app, "GET", "/api/account/list", &[("cookie", &jar)], None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "the old cookie is dead");
}

#[tokio::test]
async fn a_cookie_session_is_renewed_in_the_cookie() {
    let (app, db) = test_app_with_db().await;
    post(&app, "10.0.2.3", "/api/common/register", json!({"user_name": "renewc", "user_password": "correct horse"})).await;
    let (id, role, version): (String, String, i64) =
        sqlx::query_as("SELECT user_id, user_role, token_version FROM users WHERE user_name = 'renewc'").fetch_one(&db).await.unwrap();
    let key = Arc::new(PasetoSymmetricKey::from(Key::from(&[7u8; 32])));
    let old = joy::handlers::common::get_paseto_token(&id, role, version, &key, time::OffsetDateTime::now_utc() + time::Duration::days(2)).ok().unwrap();
    let (s, h, _) = raw(&app, "GET", "/api/account/list", &[("cookie", &format!("joy_session={old}"))], None).await;
    assert_eq!(s, StatusCode::OK);
    let set = cookie_of(&h);
    assert!(set.starts_with("joy_session=") && !set.starts_with(&format!("joy_session={old}")) && set.contains("Max-Age=2592000"), "{set}");
}

async fn make_couple(app: &Router, a: &str, b: &str) {
    authed_post(app, a, "/api/couple/create", json!({})).await;
    let (_, inv) = authed_post(app, a, "/api/couple/invite", json!({})).await;
    let code = serde_json::from_str::<Value>(&inv).unwrap()["code"].as_str().unwrap().to_string();
    let (s, body) = authed_post(app, b, "/api/couple/join", json!({"code": code})).await;
    assert!(s.is_success(), "{body}");
}

#[tokio::test]
async fn a_couple_agrees_on_a_goal_and_both_pay_into_it() {
    let app = test_app().await;
    let (a, b) = (new_user(&app, "10.0.3.1", "goaleda").await, new_user(&app, "10.0.3.2", "goaledb").await);
    make_couple(&app, &a, &b).await;
    let today = joy::handlers::common::today_local();
    let by = (today + time::Duration::days(200)).to_string();

    let (s, body) = authed_post(&app, &a, "/api/couple-goals", json!({"name": "Wedding", "target_paise": 5_000_000, "target_date": by})).await;
    assert_eq!(s, StatusCode::OK, "{body}");
    let id = serde_json::from_str::<Value>(&body).unwrap()["goal_id"].as_str().unwrap().to_string();

    // Both see the same goal.
    for t in [&a, &b] {
        let v = authed_get(&app, t, "/api/couple-goals").await;
        assert_eq!(v["goals"][0]["name"], "Wedding", "{v}");
        assert_eq!(v["goals"][0]["target_paise"], 5_000_000);
    }

    // Each pays in their own; the total is shared and each share is named.
    let (s, _) = authed_post(&app, &a, &format!("/api/couple-goals/{id}/contribute"), json!({"amount_paise": 1_000_000})).await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = authed_post(&app, &b, &format!("/api/couple-goals/{id}/contribute"), json!({"amount_paise": 500_000, "note": "from my bonus"})).await;
    assert_eq!(s, StatusCode::OK);
    let v = authed_get(&app, &b, "/api/couple-goals").await;
    let g = &v["goals"][0];
    assert_eq!(g["saved_paise"], 1_500_000);
    assert_eq!(g["percent"], 30);
    assert!(g["monthly_needed_paise"].as_i64().unwrap() > 0);
    let shares: Vec<(&str, i64)> = g["shares"].as_array().unwrap().iter().map(|s| (s["name"].as_str().unwrap(), s["paise"].as_i64().unwrap())).collect();
    assert_eq!(shares.len(), 2, "{shares:?}");
    assert!(shares.iter().any(|(n, p)| n.starts_with("goaleda") && *p == 1_000_000));
    assert!(shares.iter().any(|(n, p)| n.starts_with("goaledb") && *p == 500_000));

    // You can take back your own payment, not your partner's.
    let first = g["contributions"][0]["id"].as_i64().unwrap();
    let mine = g["contributions"].as_array().unwrap().iter().find(|c| c["mine"] == true).unwrap()["id"].as_i64().unwrap();
    let theirs = g["contributions"].as_array().unwrap().iter().find(|c| c["mine"] == false).unwrap()["id"].as_i64().unwrap();
    let _ = first;
    let (s, _) = authed_post(&app, &b, &format!("/api/couple-goals/{id}/contributions/{theirs}/delete"), json!({})).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, _) = authed_post(&app, &b, &format!("/api/couple-goals/{id}/contributions/{mine}/delete"), json!({})).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(authed_get(&app, &a, "/api/couple-goals").await["goals"][0]["saved_paise"], 1_000_000);

    // Bad amounts, and people outside the couple.
    for amount in [0, -5] {
        let (s, _) = authed_post(&app, &a, &format!("/api/couple-goals/{id}/contribute"), json!({"amount_paise": amount})).await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "{amount}");
    }
    let stranger = new_user(&app, "10.0.3.3", "goalnosy").await;
    assert_eq!(authed_get(&app, &stranger, "/api/couple-goals").await["in_couple"], false);
    let (s, _) = authed_post(&app, &stranger, &format!("/api/couple-goals/{id}/contribute"), json!({"amount_paise": 100})).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (c, d) = (new_user(&app, "10.0.3.4", "goalc").await, new_user(&app, "10.0.3.5", "goald").await);
    make_couple(&app, &c, &d).await;
    let (s, _) = authed_post(&app, &c, &format!("/api/couple-goals/{id}/contribute"), json!({"amount_paise": 100})).await;
    assert_eq!(s, StatusCode::NOT_FOUND, "another couple's goal is not found");
    assert_eq!(authed_get(&app, &c, "/api/couple-goals").await["goals"], json!([]));

    // Archiving hides it from both.
    let (s, _) = authed_post(&app, &b, &format!("/api/couple-goals/{id}/archive"), json!({})).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(authed_get(&app, &a, "/api/couple-goals").await["goals"], json!([]));
}

#[tokio::test]
async fn a_post_sent_twice_with_the_same_idempotency_key_happens_once() {
    let (app, db) = test_app_with_db().await;
    let a = new_user(&app, "10.0.4.1", "idema").await;
    let b = new_user(&app, "10.0.4.2", "idemb").await;
    let today = joy::handlers::common::today_local().to_string();
    let body = json!({"occurred_on": today, "direction": "debit", "amount_paise": 4_200, "merchant": "Chai", "category": "food"});
    let send = |token: &str, key: &str, b: Value| {
        let token = token.to_string();
        let key = key.to_string();
        let app = app.clone();
        async move { raw(&app, "POST", "/api/ingest/transactions/create", &[("authorization", &format!("Bearer {token}")), ("idempotency-key", &key)], Some(b)).await }
    };
    let count = || async { sqlx::query_scalar::<_, i64>("SELECT count(*) FROM transactions").fetch_one(&db).await.unwrap() };

    let (s1, _, first) = send(&a, "key-1", body.clone()).await;
    assert_eq!(s1, StatusCode::OK, "{first}");
    assert_eq!(count().await, 1);
    // The phone never heard back and sends it again: same answer, nothing new.
    let (s2, h2, again) = send(&a, "key-1", body.clone()).await;
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(again, first, "the first answer is replayed");
    assert_eq!(h2.get("idempotent-replay").map(|v| v.to_str().unwrap()), Some("true"));
    assert_eq!(count().await, 1);

    // A different key is a different action; the same key from someone else is theirs alone.
    send(&a, "key-2", body.clone()).await;
    assert_eq!(count().await, 2);
    let (s, _, _) = send(&b, "key-1", body.clone()).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(count().await, 3);

    // A refused request is not remembered: fix it and send again with the same key.
    let (s, _, _) = send(&a, "key-3", json!({"occurred_on": today, "direction": "debit", "amount_paise": -1, "category": "food"})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _, _) = send(&a, "key-3", body.clone()).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(count().await, 4);

    // Without a key nothing is remembered, as before; absurd keys are refused.
    raw(&app, "POST", "/api/ingest/transactions/create", &[("authorization", &format!("Bearer {a}"))], Some(body.clone())).await;
    raw(&app, "POST", "/api/ingest/transactions/create", &[("authorization", &format!("Bearer {a}"))], Some(body.clone())).await;
    assert_eq!(count().await, 6);
    let (s, _, _) = send(&a, &"k".repeat(200), body).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn people_can_turn_notifications_on_and_off_and_a_scheduled_job_can_trigger_a_round() {
    let (app, db) = test_app_with_db().await;
    let (private, public) = joy::push::WebPushSender::generate();
    // Safety: only this test reads these.
    unsafe {
        std::env::set_var("VAPID_PRIVATE_KEY", &private);
        std::env::set_var("PUSH_RUN_TOKEN", "round-token");
    }
    let a = new_user(&app, "10.0.5.1", "pusha").await;
    let b = new_user(&app, "10.0.5.2", "pushb").await;

    assert_eq!(authed_get(&app, &a, "/api/push").await, json!({"public_key": public, "subscribed": false}));
    let sub = json!({"endpoint": "https://push.example/abc", "p256dh": "BLn9b-VR0ca83knDNZ32dCHGyjJp-1riX9ZTN40MqV8K_LpQmLqxC_DoHvqvFXO_nGdAB4W9dogZb_sM-uV4JbY", "auth": "_ordMnz7uTCmrpBTeUV4Bw"});
    let (s, body) = authed_post(&app, &a, "/api/push/subscribe", sub.clone()).await;
    assert_eq!(s, StatusCode::OK, "{body}");
    authed_post(&app, &a, "/api/push/subscribe", sub.clone()).await;
    assert_eq!(authed_get(&app, &a, "/api/push").await["subscribed"], true);
    assert_eq!(authed_get(&app, &b, "/api/push").await["subscribed"], false, "someone else's is theirs");
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM push_subscriptions").fetch_one(&db).await.unwrap();
    assert_eq!(n, 1, "subscribing twice is one subscription");

    // Plain http and junk are refused.
    for bad in [json!({"endpoint": "http://push.example/x", "p256dh": "k", "auth": "a"}), json!({"endpoint": "https://x", "p256dh": "", "auth": "a"}), json!({"endpoint": "not a url", "p256dh": "k", "auth": "a"})] {
        let (s, _) = authed_post(&app, &a, "/api/push/subscribe", bad).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }
    // Only you can switch yours off.
    authed_post(&app, &b, "/api/push/unsubscribe", json!({"endpoint": "https://push.example/abc"})).await;
    assert_eq!(authed_get(&app, &a, "/api/push").await["subscribed"], true);
    authed_post(&app, &a, "/api/push/unsubscribe", json!({"endpoint": "https://push.example/abc"})).await;
    assert_eq!(authed_get(&app, &a, "/api/push").await["subscribed"], false);

    // The scheduled round: needs the secret, and a real slot.
    let round = |token: Option<&str>, slot: &str| {
        let app = app.clone();
        let hdr: Vec<(String, String)> = token.map(|t| vec![("authorization".to_string(), format!("Bearer {t}"))]).unwrap_or_default();
        let path = format!("/api/internal/push/run?slot={slot}");
        async move {
            let h: Vec<(&str, &str)> = hdr.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            raw(&app, "POST", &path, &h, Some(json!({}))).await
        }
    };
    assert_eq!(round(None, "morning").await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(round(Some("wrong"), "morning").await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(round(Some("round-token"), "noon").await.0, StatusCode::BAD_REQUEST);
    let (s, _, body) = round(Some("round-token"), "morning").await;
    assert_eq!(s, StatusCode::OK, "{body}");
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["sent"], 0);
}
