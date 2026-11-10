//! Dates (Joy) API: a surprise one partner is planning stays sealed on every path that lists or looks
//! up a couple's slips (jar, accept, pick, savings, the slip form), and a partner who leaves cannot
//! hand their unrevealed surprises to whoever takes the empty slot.
#![allow(dead_code)]

use std::{
    sync::{
        Arc, Once,
        atomic::{AtomicUsize, Ordering},
    },
};

use axum::{
    Router,
    body::{Body, Bytes, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode},
};
use joy::{app::build_router, models::AppState};
use rusty_paseto::prelude::{Key, PasetoSymmetricKey};
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower::ServiceExt;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

struct T {
    app: Router,
    db: SqlitePool,
}

struct Res {
    status: StatusCode,
    headers: HeaderMap,
    json: Value,
    text: String,
    bytes: Bytes,
}

async fn setup() -> T {
    // Sign-ups need the code from the SIGNUP_CODE secret; the tests use a fixed one.
    // Safety: every test in this binary sets the same value before any request reads it.
    unsafe { std::env::set_var("SIGNUP_CODE", "test-signup-code") };
    static PHOTOS: Once = Once::new();
    PHOTOS.call_once(|| {
        let dir = std::env::temp_dir().join(format!("joy-sealed-{}", std::process::id()));
        // Safety: set once, before any request in this binary reads it; nothing else writes the environment.
        unsafe { std::env::set_var("JOY_PHOTOS_DIR", dir) };
    });
    let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    let app = build_router(AppState { db: db.clone(), redis: None, paseto_key: Arc::new(key) });
    T { app, db }
}

async fn finish(res: axum::response::Response) -> Res {
    let (status, headers) = (res.status(), res.headers().clone());
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    Res { status, headers, json: serde_json::from_str(&text).unwrap_or(Value::Null), text, bytes }
}

impl T {
    async fn call(&self, method: Method, path: &str, token: Option<&str>, body: Option<Value>) -> Res {
        let mut req = Request::builder().method(method).uri(path).header("fly-client-ip", "10.99.0.1");
        if let Some(t) = token {
            req = req.header("authorization", format!("Bearer {t}"));
        }
        let req = match body {
            Some(b) => req.header("content-type", "application/json").body(Body::from(b.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        finish(self.app.clone().oneshot(req).await.unwrap()).await
    }

    async fn get(&self, path: &str, token: &str) -> Res {
        self.call(Method::GET, path, Some(token), None).await
    }

    async fn post(&self, path: &str, token: &str, body: Value) -> Res {
        self.call(Method::POST, path, Some(token), Some(body)).await
    }

    /// No login: a signed link on the public side.
    async fn public(&self, path: &str) -> Res {
        self.call(Method::GET, path, None, None).await
    }

    /// A multipart upload with one `photo` field.
    async fn upload(&self, path: &str, token: &str, bytes: &[u8]) -> Res {
        let b = "XBOUNDARYX";
        let mut body =
            format!("--{b}\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"p.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n").into_bytes();
        body.extend_from_slice(bytes);
        body.extend_from_slice(format!("\r\n--{b}--\r\n").as_bytes());
        let req = Request::post(path)
            .header("fly-client-ip", "10.99.0.1")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", format!("multipart/form-data; boundary={b}"))
            .body(Body::from(body))
            .unwrap();
        finish(self.app.clone().oneshot(req).await.unwrap()).await
    }

    /// A registered, logged-in user; returns their bearer token.
    async fn user(&self, label: &str) -> String {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let (name, ip) = (format!("{label}{n}"), format!("10.88.{}.{}", n / 250, n % 250));
        let creds = json!({"user_name": name, "user_password": "correct horse", "signup_code": "test-signup-code"});
        for (path, expect) in [("/api/common/register", StatusCode::OK), ("/api/common/login", StatusCode::OK)] {
            let req = Request::post(path)
                .header("content-type", "application/json")
                .header("fly-client-ip", &ip)
                .body(Body::from(creds.to_string()))
                .unwrap();
            let res = self.app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), expect, "{path}");
            if path.ends_with("login") {
                let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
                return serde_json::from_slice::<Value>(&bytes).unwrap()["access_token"].as_str().unwrap().to_string();
            }
        }
        unreachable!()
    }

    /// Two users who are a couple: (rohit's token, neha's token, couple id). Rohit is slot 0.
    async fn couple(&self) -> (String, String, String) {
        let (a, b) = (self.user("rohit").await, self.user("neha").await);
        assert!(self.post("/api/couple/create", &a, json!({})).await.status.is_success());
        let code = self.post("/api/couple/invite", &a, json!({})).await.json["code"].as_str().unwrap().to_string();
        let joined = self.post("/api/couple/join", &b, json!({"code": code})).await;
        assert!(joined.status.is_success(), "{}", joined.text);
        let id = self.text("SELECT couple_id FROM couples ORDER BY created_at DESC, rowid DESC LIMIT 1").await;
        (a, b, id)
    }

    /// Someone in a couple of their own (just themselves).
    async fn loner(&self) -> String {
        let c = self.user("carol").await;
        assert!(self.post("/api/couple/create", &c, json!({})).await.status.is_success());
        c
    }

    async fn exec(&self, sql: &str) {
        sqlx::query(sql).execute(&self.db).await.unwrap();
    }

    async fn scalar(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.db).await.unwrap()
    }

    async fn text(&self, sql: &str) -> String {
        sqlx::query_scalar(sql).fetch_one(&self.db).await.unwrap()
    }

    async fn id_of(&self, couple: &str, title: &str) -> i64 {
        sqlx::query_scalar("SELECT id FROM joy_activities WHERE couple_id = ? AND title = ?")
            .bind(couple)
            .bind(title)
            .fetch_one(&self.db)
            .await
            .unwrap()
    }

    /// Keep a slip as `token`'s user; returns the history id.
    async fn keep(&self, couple: &str, token: &str, title: &str, surprise: bool) -> i64 {
        let id = self.id_of(couple, title).await;
        let r = self.post(&format!("/api/dates/accept/{id}"), token, json!({"surprise": surprise})).await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.text);
        self.scalar(&format!("SELECT max(id) FROM joy_history WHERE activity_id = {id}")).await
    }

    /// The Kept page's item for a history id.
    async fn item(&self, token: &str, hid: i64) -> Value {
        let page = self.get("/api/dates/kept", token).await;
        assert_eq!(page.status, StatusCode::OK, "{}", page.text);
        page.json["months"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|m| m["items"].as_array().unwrap().clone())
            .find(|i| i["id"] == hid)
            .unwrap_or_else(|| panic!("no item {hid} in {}", page.text))
    }
}

const PLAY: &str = "A play at Prithvi Theatre";
const COOK: &str = "Cook together";

const NEEDLE: &str = "Zanzibar";
const SECRET: &str = "Zanzibar sunset picnic";

impl T {
    /// Rohit (slot 0) plans PLAY as a surprise for Neha; returns (rohit, neha, couple, slip id, history id).
    async fn sealed_setup(&self) -> (String, String, String, i64, i64) {
        let (rohit, neha, couple) = self.couple().await;
        let id = self.id_of(&couple, PLAY).await;
        self.exec(&format!("UPDATE joy_activities SET est_cost = 800, title = '{SECRET}', notes = 'quiet rooftop' WHERE id = {id}")).await;
        let hid = self.keep(&couple, &rohit, SECRET, true).await;
        (rohit, neha, couple, id, hid)
    }
}

fn slip_ids(jar: &Value) -> Vec<i64> {
    jar["slips"].as_array().unwrap().iter().map(|s| s["id"].as_i64().unwrap()).collect()
}

#[tokio::test]
async fn the_jar_page_and_match_hide_a_sealed_slip_from_the_partner() {
    let t = setup().await;
    let (rohit, neha, _c, id, _h) = t.sealed_setup().await;
    let theirs = t.get("/api/dates/jar", &neha).await;
    assert_eq!(theirs.status, StatusCode::OK, "{}", theirs.text);
    assert!(!slip_ids(&theirs.json).contains(&id), "{}", theirs.text);
    assert!(!theirs.text.contains(NEEDLE), "{}", theirs.text);
    let mine = t.get("/api/dates/jar", &rohit).await;
    assert!(slip_ids(&mine.json).contains(&id), "the planner still sees their own slip");
    assert_eq!(mine.json["count"].as_i64().unwrap(), theirs.json["count"].as_i64().unwrap() + 1);
    let m = t.post("/api/dates/match", &neha, json!({})).await;
    assert!(!m.json["ids"].as_array().unwrap().iter().any(|i| i == id), "{}", m.text);
}

#[tokio::test]
async fn accepting_a_sealed_slip_is_a_404_alone_or_as_a_pair() {
    let t = setup().await;
    let (_rohit, neha, couple, id, hid) = t.sealed_setup().await;
    let before = t.scalar("SELECT count(*) FROM joy_history").await;
    let r = t.post(&format!("/api/dates/accept/{id}"), &neha, json!({})).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND, "{}", r.text);
    assert!(!r.text.contains(NEEDLE));
    // As the `with` of another slip it can't ride along either.
    let cook = t.id_of(&couple, COOK).await;
    t.exec(&format!("INSERT INTO joy_pairs (a_id, b_id) VALUES ({cook}, {id})")).await;
    let r = t.post(&format!("/api/dates/accept/{cook}"), &neha, json!({"with": id.to_string()})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert!(r.json["paired"].is_null() && !r.text.contains(NEEDLE), "{}", r.text);
    assert_eq!(t.scalar("SELECT count(*) FROM joy_history").await, before + 1, "only Cook was kept");
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_history WHERE activity_id = {id}")).await, 1);
    assert_eq!(t.scalar(&format!("SELECT revealed FROM joy_history WHERE id = {hid}")).await, 0);
}

#[tokio::test]
async fn the_draw_never_hands_the_partner_a_sealed_slip_and_reroll_and_veto_are_404() {
    let t = setup().await;
    let (rohit, neha, _c, id, _h) = t.sealed_setup().await;
    for _ in 0..40 {
        let r = t.post("/api/dates/pick", &neha, json!({})).await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.text);
        assert!(!r.text.contains(NEEDLE), "{}", r.text);
    }
    for body in [json!({"reroll": "1", "current": id.to_string(), "used": "9"}), json!({"veto": "1", "current": id.to_string()})] {
        let r = t.post("/api/dates/pick", &neha, body).await;
        assert_eq!(r.status, StatusCode::NOT_FOUND, "{}", r.text);
        assert!(!r.text.contains(NEEDLE));
    }
    assert_eq!(t.scalar("SELECT count(*) FROM joy_vetoes").await, 0);
    // The planner can still draw from their own jar.
    assert_eq!(t.post("/api/dates/pick", &rohit, json!({})).await.status, StatusCode::OK);
}

#[tokio::test]
async fn savings_choices_and_goal_hide_a_sealed_slip() {
    let t = setup().await;
    let (rohit, neha, _c, id, _h) = t.sealed_setup().await;
    let page = t.get("/api/dates/kept", &neha).await;
    assert!(!page.text.contains(NEEDLE), "{}", page.text);
    let choices = page.json["savings"]["choices"].to_string();
    assert!(!choices.contains(&format!("\"{id}\"")), "{choices}");
    let r = t.post("/api/dates/savings/goal", &neha, json!({"id": id.to_string()})).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND, "{}", r.text);
    assert_eq!(t.scalar("SELECT count(*) FROM joy_settings WHERE key = 'saving_for'").await, 0);
    // The planner can save for it, and the partner still doesn't see it as the goal.
    let ok = t.post("/api/dates/savings/goal", &rohit, json!({"id": id.to_string()})).await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.text);
    assert!(t.get("/api/dates/kept", &rohit).await.text.contains(NEEDLE));
    let page = t.get("/api/dates/kept", &neha).await;
    assert!(page.json["savings"]["goal"].is_null() && !page.text.contains(NEEDLE), "{}", page.text);
    let r = t.post("/api/dates/savings/keep", &neha, json!({})).await;
    assert!(!r.text.contains(NEEDLE));
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_history WHERE activity_id = {id}")).await, 1, "nothing was kept for her");
}

#[tokio::test]
async fn the_slip_form_hides_a_sealed_slip_from_pair_options_and_by_id() {
    let t = setup().await;
    let (rohit, neha, couple, id, _h) = t.sealed_setup().await;
    let cook = t.id_of(&couple, COOK).await;
    for path in ["/api/dates/slips/new".to_string(), format!("/api/dates/slips/{cook}")] {
        let r = t.get(&path, &neha).await;
        assert_eq!(r.status, StatusCode::OK, "{path} {}", r.text);
        assert!(!r.text.contains(NEEDLE), "{path} {}", r.text);
        assert!(t.get(&path, &rohit).await.text.contains(NEEDLE), "the planner sees it as a pair option");
    }
    let r = t.get(&format!("/api/dates/slips/{id}"), &neha).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND, "{}", r.text);
}

#[tokio::test]
async fn the_money_summary_hides_the_title_and_day_of_a_sealed_surprise() {
    let t = setup().await;
    let (rohit, neha, _c, _id, hid) = t.sealed_setup().await;
    t.exec(&format!("UPDATE joy_history SET actual_cost = 500 WHERE id = {hid}")).await;
    let find = |r: &Res| r.json["recent"].as_array().unwrap().iter().find(|e| e["kind"] == "date").cloned().unwrap_or_else(|| panic!("{}", r.text));
    let theirs = t.get("/api/joy/summary", &neha).await;
    let e = find(&theirs);
    assert!(!theirs.text.contains(NEEDLE), "{}", theirs.text);
    assert_eq!(e["title"], "A surprise date");
    assert!(e["day"].as_str().unwrap().ends_with("-01"), "only the month survives: {e}");
    assert!(e["amount_paise"].as_i64().unwrap() > 0, "the amount stays so the budget adds up");
    let mine = t.get("/api/joy/summary", &rohit).await;
    assert!(find(&mine)["title"].as_str().unwrap().contains(NEEDLE));
}

/// Rohit (slot 0) plans a surprise and leaves; Neha invites a new person who lands in slot 0.
#[tokio::test]
async fn a_new_partner_in_the_leavers_slot_does_not_inherit_their_surprise() {
    let t = setup().await;
    let (rohit, neha, couple, id, hid) = t.sealed_setup().await;
    assert!(t.post("/api/couple/leave", &rohit, json!({})).await.status.is_success());
    assert_eq!(t.scalar(&format!("SELECT drawn_by IS NULL FROM joy_history WHERE id = {hid}")).await, 1, "no planner now");
    let code = t.post("/api/couple/invite", &neha, json!({})).await.json["code"].as_str().unwrap().to_string();
    let newbie = t.user("sam").await;
    let joined = t.post("/api/couple/join", &newbie, json!({"code": code})).await;
    assert!(joined.status.is_success(), "{}", joined.text);
    assert_eq!(t.scalar(&format!("SELECT user_0 IS NOT NULL FROM couples WHERE couple_id = '{couple}'")).await, 1);

    for who in [&newbie, &neha] {
        let jar = t.get("/api/dates/jar", who).await;
        assert!(!slip_ids(&jar.json).contains(&id) && !jar.text.contains(NEEDLE), "{}", jar.text);
        assert_eq!(t.post(&format!("/api/dates/accept/{id}"), who, json!({})).await.status, StatusCode::NOT_FOUND);
        let kept = t.get("/api/dates/kept", who).await;
        assert!(!kept.text.contains(NEEDLE), "{}", kept.text);
        assert_eq!(t.item(who, hid).await["sealed"], true);
        let r = t.post(&format!("/api/dates/history/{hid}/reveal"), who, json!({})).await;
        assert_eq!(r.status, StatusCode::FORBIDDEN, "{}", r.text);
        assert_eq!(t.get(&format!("/api/dates/slips/{id}"), who).await.status, StatusCode::NOT_FOUND);
    }
    assert_eq!(t.scalar(&format!("SELECT revealed FROM joy_history WHERE id = {hid}")).await, 0);
}
