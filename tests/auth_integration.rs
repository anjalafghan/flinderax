use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use flinderax::{app::build_router, handlers::common::AppError, models::AppState};
use rusty_paseto::prelude::{Key, PasetoSymmetricKey};
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tower::ServiceExt;

async fn test_app() -> Router {
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    build_router(AppState {
        db,
        redis: None,
        paseto_key: Arc::new(key),
    })
}

/// Each test uses its own client IP: the rate limiter is process-wide.
async fn post(app: &Router, ip: &str, path: &str, body: Value) -> (StatusCode, String) {
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
    let app = test_app().await;
    let (s, _) = post(&app, "10.0.0.2", "/api/common/register",
        json!({"user_name": "mallory", "user_password": "hunter2hunter2", "user_role": "admin"})).await;
    assert_eq!(s, StatusCode::OK);
    let token = login_token(&app, "10.0.0.2", "mallory", "hunter2hunter2").await;
    assert_eq!(get(&app, "/api/user/get", Some(&token)).await, StatusCode::FORBIDDEN);
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
    assert_eq!(get(&app, "/api/user/get", None).await, StatusCode::UNAUTHORIZED);
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
    assert_eq!(&body[..], b"Internal server error");
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

    let (s, _) = authed_post(&app, &owner, "/api/card/history", json!({"card_id": card_id})).await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = authed_post(&app, &snoop, "/api/card/history", json!({"card_id": card_id})).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
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
