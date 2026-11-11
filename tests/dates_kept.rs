//! Dates (Joy) JSON API, part 2: Kept, ratings, costs, lines, photos, invites, budget and savings.
//! Surprises stay sealed for the partner, only the planner reveals, signed links hold up, and
//! one couple never reaches another's dates.

use std::{
    io::Cursor,
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
use joy::{app::build_router, joy::api::sign, models::AppState};
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
        let dir = std::env::temp_dir().join(format!("joy-kept-{}", std::process::id()));
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

/// A 4x4 JPEG.
fn tiny_jpeg() -> Vec<u8> {
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 4, image::Rgb([200, 80, 40])))
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Jpeg)
        .unwrap();
    out
}

/// A signed link as the API hands it out, aimed at the public mount.
fn public_url(signed: &str) -> String {
    signed.replacen("/api/dates/", "/api/public/dates/", 1)
}

#[tokio::test]
async fn kept_lists_a_stamped_date() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let hid = t.keep(&couple, &neha, COOK, false).await;
    for token in [&rohit, &neha] {
        let page = t.get("/api/dates/kept", token).await;
        assert_eq!(page.status, StatusCode::OK, "{}", page.text);
        assert_eq!(page.json["kept"], 1);
        let month = &page.json["months"][0];
        assert!(month["href"].as_str().unwrap().starts_with("/joy/year/"), "{month}");
        let item = &month["items"][0];
        assert_eq!((item["id"].as_i64(), item["title"].as_str(), item["sealed"].as_bool()), (Some(hid), Some(COOK), Some(false)));
        assert_eq!(item["from_name"].as_str().map(|n| n.starts_with("neha")), Some(true));
        assert_eq!(item["pair_ids"], hid.to_string());
        assert_eq!(item["ics_url"], "", "no time yet");
        assert!(item["photo_url"].is_null());
        assert_eq!(item["lines"]["can_write"], true);
    }
    assert_eq!(t.get("/api/dates/kept", "garbage").await.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rating_cost_and_a_line_work() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let hid = t.keep(&couple, &rohit, COOK, false).await;

    let r = t.post(&format!("/api/dates/history/{hid}"), &neha, json!({"rating": 4, "note": " lovely "})).await;
    assert_eq!((r.status, r.json["to"].as_str()), (StatusCode::OK, Some(format!("/joy/kept#k{hid}").as_str())), "{}", r.text);
    let item = t.item(&rohit, hid).await;
    assert_eq!((item["rating"].as_i64(), item["note"].as_str()), (Some(4), Some("lovely")));

    // Real costs: strings or numbers, with the currency and commas allowed.
    let est = t.scalar(&format!("SELECT COALESCE(est_cost, 0) FROM joy_history WHERE id = {hid}")).await;
    let r = t.post(&format!("/api/dates/history/{hid}/cost"), &rohit, json!({"actual": format!("₹{}", est + 500), "cab_0": 120, "cab_1": ""})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let item = t.item(&neha, hid).await;
    assert_eq!(item["cost_actual_raw"], (est + 500).to_string());
    assert!(item["cost_diff"].as_str().unwrap().ends_with("more"), "{item}");
    assert_eq!(item["cost_over"], true);
    assert_eq!(item["cabs"][0]["fare"], "120");
    assert_eq!(item["cabs"][1]["fare"], "");
    let bad = t.post(&format!("/api/dates/history/{hid}/cost"), &rohit, json!({"actual": "lots"})).await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    t.post(&format!("/api/dates/history/{hid}/cost"), &rohit, json!({"actual": ""})).await;
    assert_eq!(t.item(&rohit, hid).await["cost_actual"], "");

    // A line is capped at 280 characters.
    let long = "x".repeat(400);
    t.post(&format!("/api/dates/history/{hid}/line"), &rohit, json!({"line": long})).await;
    let len = t.scalar(&format!("SELECT length(line) FROM joy_memories WHERE history_id = {hid} AND who = 0")).await;
    assert_eq!(len, 280);

    let r = t.post(&format!("/api/dates/history/{hid}/delete"), &neha, json!({})).await;
    assert_eq!((r.status, r.json["to"].as_str()), (StatusCode::OK, Some("/joy/kept")));
    assert_eq!(t.get("/api/dates/kept", &neha).await.json["kept"], 0);
}

#[tokio::test]
async fn a_memory_line_stays_sealed_until_both_write() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let hid = t.keep(&couple, &rohit, COOK, false).await;
    let line = |token: &str, text: &str| {
        let (t, token, text) = (&t, token.to_string(), text.to_string());
        async move { t.post(&format!("/api/dates/history/{hid}/line"), &token, json!({"line": text})).await }
    };
    assert_eq!(line(&rohit, "the rain, the samosas").await.status, StatusCode::OK);

    // Neha hasn't written hers: she sees that Rohit has, not what.
    let l = &t.item(&neha, hid).await["lines"];
    assert_eq!((l["theirs_waiting"].as_bool(), l["theirs"].as_str(), l["mine"].as_str()), (Some(true), Some(""), Some("")));
    let page = t.get("/api/dates/kept", &neha).await;
    assert!(!page.text.contains("samosas"), "{}", page.text);
    // Rohit sees only his own.
    let l = &t.item(&rohit, hid).await["lines"];
    assert_eq!((l["mine"].as_str(), l["theirs"].as_str(), l["theirs_waiting"].as_bool()), (Some("the rain, the samosas"), Some(""), Some(false)));

    assert_eq!(line(&neha, "you laughed at the soup").await.status, StatusCode::OK);
    for (token, mine, theirs) in [(&rohit, "the rain, the samosas", "you laughed at the soup"), (&neha, "you laughed at the soup", "the rain, the samosas")] {
        let l = &t.item(token, hid).await["lines"];
        assert_eq!((l["mine"].as_str(), l["theirs"].as_str()), (Some(mine), Some(theirs)));
    }
}

#[tokio::test]
async fn a_surprise_is_hidden_from_the_partner_who_cannot_reveal_or_touch_it() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let hid = t.keep(&couple, &rohit, PLAY, true).await;
    t.exec(&format!("UPDATE joy_history SET planned_at = unixepoch('now', '+1 days') WHERE id = {hid}")).await;
    t.exec(&format!("UPDATE joy_history SET has_photo = 1 WHERE id = {hid}")).await;
    t.exec(&format!("INSERT INTO joy_history_photos (history_id, file) VALUES ({hid}, '{hid}.jpg')")).await;
    let pid = t.scalar("SELECT max(id) FROM joy_history_photos").await;

    let page = t.get("/api/dates/kept", &neha).await;
    assert!(!page.text.contains("Prithvi") && !page.text.contains("Theatre"), "{}", page.text);
    let item = t.item(&neha, hid).await;
    assert_eq!(item["sealed"], true);
    assert_eq!(item["title"], "");
    assert!(item["photo_url"].is_null(), "no photo link for a sealed date");
    assert!(item["photos"].as_array().unwrap().is_empty(), "no photos for a sealed date");
    assert!(item["hints"].as_array().unwrap().len() >= 3);
    assert_eq!(item["can_reveal"], false);

    let mine = t.item(&rohit, hid).await;
    assert_eq!((mine["sealed"].as_bool(), mine["can_reveal"].as_bool()), (Some(false), Some(true)));
    assert!(mine["title"].as_str().unwrap().contains("Prithvi"));
    assert!(mine["surprise_for"].as_str().unwrap().starts_with("neha"));

    // Everything that acts on it is a 403 for Neha; the data stays as it was.
    for (path, body) in [
        (format!("/api/dates/history/{hid}"), json!({"rating": 1})),
        (format!("/api/dates/history/{hid}/cost"), json!({"actual": "5"})),
        (format!("/api/dates/history/{hid}/line"), json!({"line": "peek"})),
        (format!("/api/dates/history/{hid}/delete"), json!({})),
        (format!("/api/dates/history/{hid}/photos/{pid}/delete"), json!({})),
        (format!("/api/dates/history/{hid}/plan-b"), json!({})),
        ("/api/dates/history/when".to_string(), json!({"ids": hid, "at": "2030-01-02T19:00"})),
        ("/api/dates/budget/spend".to_string(), json!({"amount": 100, "history_id": hid})),
        ("/api/dates/savings/add".to_string(), json!({"amount": 100, "history_id": hid.to_string()})),
    ] {
        let r = t.post(&path, &neha, body).await;
        assert_eq!(r.status, StatusCode::FORBIDDEN, "{path}: {}", r.text);
    }
    let up = t.upload(&format!("/api/dates/history/{hid}/photo"), &neha, &tiny_jpeg()).await;
    assert_eq!(up.status, StatusCode::FORBIDDEN);
    assert_eq!(t.scalar("SELECT count(*) FROM joy_other_spending").await + t.scalar("SELECT count(*) FROM joy_savings").await, 0);
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_history WHERE id = {hid} AND rating IS NULL")).await, 1);

    // Reveal: the partner gets 403 and it stays sealed; the planner reveals.
    let r = t.post(&format!("/api/dates/history/{hid}/reveal"), &neha, json!({})).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN, "{}", r.text);
    assert_eq!(t.scalar(&format!("SELECT revealed FROM joy_history WHERE id = {hid}")).await, 0);
    assert_eq!(t.item(&neha, hid).await["sealed"], true);
    let r = t.post(&format!("/api/dates/history/{hid}/reveal"), &rohit, json!({})).await;
    assert_eq!((r.status, r.json["to"].as_str()), (StatusCode::OK, Some(format!("/joy/kept#k{hid}").as_str())), "{}", r.text);
    let open = t.item(&neha, hid).await;
    assert_eq!((open["sealed"].as_bool(), open["title"].as_str().map(|s| s.contains("Prithvi"))), (Some(false), Some(true)));
    assert_eq!(t.post(&format!("/api/dates/history/{hid}"), &neha, json!({"rating": 5})).await.status, StatusCode::OK);
}

#[tokio::test]
async fn the_budget_can_be_set_and_last_months_leftover_rolled_over() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let view = t.get("/api/dates/kept", &rohit).await.json;
    assert_eq!((view["budget"]["budget"].as_str(), view["budget"]["rollover"].is_null()), (Some(""), true));

    let r = t.post("/api/dates/budget", &rohit, json!({"amount": "6,000", "back": "/joy"})).await;
    assert_eq!((r.status, r.json["to"].as_str()), (StatusCode::OK, Some("/joy")), "{}", r.text);
    let r = t.post("/api/dates/budget", &rohit, json!({"amount": 6000, "back": "//evil.example"})).await;
    assert_eq!(r.json["to"], "/joy/kept", "no open redirects");
    let view = t.get("/api/dates/kept", &neha).await.json;
    assert_eq!((view["budget"]["budget_raw"].as_str(), view["currency"].as_str()), (Some("6000"), Some("₹")));

    // Last month cost ₹1,000 against the ₹6,000 budget: ₹5,000 is on offer.
    t.exec(&format!(
        "INSERT INTO joy_other_spending (couple_id, amount, note, spent_on) VALUES ('{couple}', 1000, 'old', date('now', 'localtime', 'start of month', '-1 day'))"
    ))
    .await;
    let offer = &t.get("/api/dates/kept", &rohit).await.json["budget"]["rollover"];
    assert_eq!(offer["left"], "₹5,000");
    let r = t.post("/api/dates/budget/rollover", &neha, json!({"choice": "roll"})).await;
    assert_eq!((r.status, r.json["to"].as_str()), (StatusCode::OK, Some("/joy/kept")), "{}", r.text);
    let b = &t.get("/api/dates/kept", &rohit).await.json["budget"];
    assert_eq!((b["budget"].as_str(), b["rollover"].is_null()), (Some("₹11,000"), true), "{b}");
    assert!(b["carry_note"].as_str().unwrap().contains("rolled over"));
    assert!(t.get("/api/dates/kept", &rohit).await.json["past"].as_array().is_some());

    // The other choices: put it in the jar, or start fresh.
    for (choice, saved) in [("save", 5000), ("fresh", 0)] {
        let t2 = setup().await;
        let (a, _, c) = t2.couple().await;
        t2.post("/api/dates/budget", &a, json!({"amount": 6000})).await;
        t2.exec(&format!(
            "INSERT INTO joy_other_spending (couple_id, amount, note, spent_on) VALUES ('{c}', 1000, 'old', date('now', 'localtime', 'start of month', '-1 day'))"
        ))
        .await;
        t2.post("/api/dates/budget/rollover", &a, json!({"choice": choice})).await;
        let v = t2.get("/api/dates/kept", &a).await.json;
        assert_eq!(v["savings"]["saved_raw"], saved, "{choice}");
        assert_eq!(v["budget"]["budget"], "₹6,000", "{choice}");
    }
}

#[tokio::test]
async fn other_spending_and_savings_work() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let r = t.post("/api/dates/budget/spend", &rohit, json!({"amount": "850", "note": "chai and a cab"})).await;
    assert_eq!((r.status, r.json["to"].as_str()), (StatusCode::OK, Some("/joy/kept")), "{}", r.text);
    t.post("/api/dates/budget/spend", &neha, json!({"amount": 0, "note": "nothing"})).await;
    let view = t.get("/api/dates/kept", &neha).await.json;
    let spend = view["other_spend"].as_array().unwrap();
    assert_eq!(spend.len(), 1);
    assert_eq!((spend[0]["amount"].as_str(), spend[0]["note"].as_str()), (Some("₹850"), Some("chai and a cab")));
    assert_eq!(view["budget"]["spent"], "₹850");
    let sid = spend[0]["id"].as_i64().unwrap();
    let r = t.post(&format!("/api/dates/budget/spend/{sid}/delete"), &neha, json!({})).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(t.get("/api/dates/kept", &rohit).await.json["other_spend"].as_array().unwrap().len(), 0);

    // The savings jar: put money in, take a hand-made deposit back, save for a slip, keep it.
    let r = t.post("/api/dates/savings/add", &rohit, json!({"amount": 2000, "note": "diwali money"})).await;
    assert_eq!((r.status, r.json["to"].as_str()), (StatusCode::OK, Some("/joy/kept#k-savings")), "{}", r.text);
    t.post("/api/dates/savings/add", &neha, json!({"amount": "300"})).await;
    let jar = &t.get("/api/dates/kept", &rohit).await.json["savings"];
    assert_eq!(jar["saved"], "₹2,300");
    let oops = jar["recent"][0]["id"].as_i64().unwrap();
    assert_eq!(jar["recent"][0]["removable"], true);
    t.post(&format!("/api/dates/savings/{oops}/delete"), &neha, json!({})).await;
    assert_eq!(t.get("/api/dates/kept", &rohit).await.json["savings"]["saved_raw"], 2000);

    let play = t.id_of(&couple, PLAY).await;
    t.post("/api/dates/savings/goal", &rohit, json!({"id": play})).await;
    let jar = t.get("/api/dates/kept", &rohit).await.json["savings"].clone();
    assert_eq!(jar["goal"]["title"], PLAY);
    assert!(jar["choices"].as_array().unwrap().iter().any(|c| c["selected"] == true));
    // Not enough yet: nothing is kept.
    let cost = t.scalar(&format!("SELECT est_cost FROM joy_activities WHERE id = {play}")).await;
    t.exec(&format!("DELETE FROM joy_savings WHERE couple_id = '{couple}'")).await;
    let r = t.post("/api/dates/savings/keep", &rohit, json!({})).await;
    assert_eq!(r.json["to"], "/joy/kept#k-savings");
    assert_eq!(t.scalar("SELECT count(*) FROM joy_history").await, 0);
    t.post("/api/dates/savings/add", &rohit, json!({"amount": cost * 4 + 1000})).await;
    let r = t.post("/api/dates/savings/keep", &neha, json!({})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let hid = t.scalar("SELECT max(id) FROM joy_history").await;
    assert_eq!(r.json["to"], format!("/joy/kept#k{hid}"));
    assert_eq!(t.scalar(&format!("SELECT drawn_by FROM joy_history WHERE id = {hid}")).await, 1);
    let left = t.get("/api/dates/kept", &rohit).await.json["savings"]["saved_raw"].as_i64().unwrap();
    assert!(left > 0 && left < cost * 4 + 1000, "the jar paid for it: {left}");
    // A double tap keeps it once, and a goal that isn't ours means nothing.
    t.post("/api/dates/savings/keep", &neha, json!({})).await;
    assert_eq!(t.scalar("SELECT count(*) FROM joy_history").await, 1);
    t.post("/api/dates/savings/goal", &rohit, json!({"id": 999999})).await;
    assert!(t.get("/api/dates/kept", &rohit).await.json["savings"]["goal"].is_null());
}

#[tokio::test]
async fn a_date_takes_several_photos_each_served_through_its_own_signed_link() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let hid = t.keep(&couple, &rohit, COOK, false).await;

    // Junk is refused with a reason; so is an empty form.
    let bad = t.upload(&format!("/api/dates/history/{hid}/photo"), &rohit, b"definitely not a photo").await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", bad.text);
    assert!(bad.text.contains("photo"), "{}", bad.text);
    assert_eq!(t.scalar(&format!("SELECT has_photo FROM joy_history WHERE id = {hid}")).await, 0);

    // Each upload adds one more; the first stays the cover.
    for who in [&neha, &rohit] {
        let up = t.upload(&format!("/api/dates/history/{hid}/photo"), who, &tiny_jpeg()).await;
        assert_eq!((up.status, up.json["to"].as_str()), (StatusCode::OK, Some(format!("/joy/kept#k{hid}").as_str())), "{}", up.text);
    }
    assert_eq!(t.scalar(&format!("SELECT has_photo FROM joy_history WHERE id = {hid}")).await, 1);
    let item = t.item(&rohit, hid).await;
    let pics = item["photos"].as_array().unwrap().clone();
    assert_eq!(pics.len(), 2);
    assert_eq!(item["can_add_photo"], true);
    let (first, second) = (pics[0]["id"].as_i64().unwrap(), pics[1]["id"].as_i64().unwrap());
    assert!(first < second);
    assert_eq!(item["photo_url"], pics[0]["url"], "the cover is the first photo");

    let signed = pics[0]["url"].as_str().unwrap().to_string();
    assert!(signed.starts_with(&format!("/api/public/dates/history/{hid}/photos/{first}?exp=")) && signed.contains("&sig="), "{signed}");
    let url = public_url(&signed);
    let url2 = public_url(pics[1]["url"].as_str().unwrap());
    for u in [&url, &url2] {
        let ok = t.public(u).await;
        assert_eq!(ok.status, StatusCode::OK, "{}", ok.text);
        assert_eq!(ok.headers["content-type"], "image/jpeg");
        let decoded = image::load_from_memory(&ok.bytes).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (4, 4));
    }
    // The link needs no login; the protected prefix has no such route.
    assert_ne!(t.call(Method::GET, &signed.replacen("/api/public", "/api", 1), None, None).await.status, StatusCode::OK);

    // Tampering: a flipped signature, another date, another photo, a longer life, or no signature.
    let flipped = if url.ends_with('0') { format!("{}1", &url[..url.len() - 1]) } else { format!("{}0", &url[..url.len() - 1]) };
    assert_eq!(t.public(&flipped).await.status, StatusCode::FORBIDDEN);
    assert_eq!(t.public(&url.replace(&format!("history/{hid}/"), &format!("history/{}/", hid + 1))).await.status, StatusCode::FORBIDDEN);
    assert_eq!(t.public(&url.replace(&format!("photos/{first}?"), &format!("photos/{second}?"))).await.status, StatusCode::FORBIDDEN);
    let exp = url.split("exp=").nth(1).unwrap().split('&').next().unwrap();
    assert_eq!(t.public(&url.replace(exp, &(exp.parse::<i64>().unwrap() + 100).to_string())).await.status, StatusCode::FORBIDDEN);
    assert_eq!(t.public(url.split("?exp=").next().unwrap()).await.status, StatusCode::FORBIDDEN);
    // Expired: correctly signed, but already past.
    let expired = public_url(&sign(&format!("/api/dates/history/{hid}/photos/{first}"), -5));
    assert_eq!(t.public(&expired).await.status, StatusCode::FORBIDDEN);
    // A good signature for a photo of another date is still a 404 here.
    let wrong_date = public_url(&sign(&format!("/api/dates/history/{}/photos/{first}", hid + 1), 60));
    assert_eq!(t.public(&wrong_date).await.status, StatusCode::NOT_FOUND);

    // Removing one leaves the other, which becomes the cover.
    let r = t.post(&format!("/api/dates/history/{hid}/photos/{first}/delete"), &rohit, json!({})).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(t.public(&url).await.status, StatusCode::NOT_FOUND);
    assert_eq!(t.public(&url2).await.status, StatusCode::OK);
    let item = t.item(&rohit, hid).await;
    assert_eq!(item["photos"].as_array().unwrap().len(), 1);
    assert!(item["photo_url"].as_str().unwrap().contains(&format!("/photos/{second}?")));
    assert_eq!(t.scalar(&format!("SELECT has_photo FROM joy_history WHERE id = {hid}")).await, 1);

    // Gone once the last is removed: the page stops offering any.
    t.post(&format!("/api/dates/history/{hid}/photos/{second}/delete"), &neha, json!({})).await;
    assert_eq!(t.public(&url2).await.status, StatusCode::NOT_FOUND);
    let item = t.item(&rohit, hid).await;
    assert!(item["photo_url"].is_null() && item["photos"].as_array().unwrap().is_empty());
    assert_eq!(t.scalar(&format!("SELECT has_photo FROM joy_history WHERE id = {hid}")).await, 0);
}

#[tokio::test]
async fn a_sealed_surprise_photo_is_a_404_even_with_a_good_signature() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let hid = t.keep(&couple, &rohit, PLAY, true).await;
    assert_eq!(t.upload(&format!("/api/dates/history/{hid}/photo"), &rohit, &tiny_jpeg()).await.status, StatusCode::OK);
    let pid = t.scalar("SELECT max(id) FROM joy_history_photos").await;

    // The planner's own link names them; it works for them and for nobody else.
    let mine = t.item(&rohit, hid).await["photo_url"].as_str().unwrap().to_string();
    assert!(mine.contains("?who=0&"), "{mine}");
    assert_eq!(t.public(&public_url(&mine)).await.status, StatusCode::OK);
    let as_neha = public_url(&sign(&format!("/api/dates/history/{hid}/photos/{pid}?who=1"), 60));
    assert_eq!(t.public(&as_neha).await.status, StatusCode::NOT_FOUND);
    let nobody = public_url(&sign(&format!("/api/dates/history/{hid}/photos/{pid}"), 60));
    assert_eq!(t.public(&nobody).await.status, StatusCode::NOT_FOUND);
    // Swapping `who` by hand breaks the signature.
    assert_eq!(t.public(&public_url(&mine).replace("who=0", "who=1")).await.status, StatusCode::FORBIDDEN);
    assert!(t.item(&neha, hid).await["photo_url"].is_null());
    // Revealed, a plain link works for anyone.
    t.post(&format!("/api/dates/history/{hid}/reveal"), &rohit, json!({})).await;
    assert_eq!(t.public(&nobody).await.status, StatusCode::OK);
}

#[tokio::test]
async fn the_ics_comes_back_and_is_sealed_for_the_partner_of_a_surprise() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let open = t.keep(&couple, &neha, COOK, false).await;
    let secret = t.keep(&couple, &rohit, PLAY, true).await;

    // Set the time: a bad one is refused; a good one gives signed links for both of us.
    let bad = t.post("/api/dates/history/when", &rohit, json!({"ids": open, "at": "tomorrow"})).await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = t.post("/api/dates/history/when", &rohit, json!({"ids": open.to_string(), "at": "2030-03-02T19:00"})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert_eq!((r.json["label"].as_str(), r.json["input"].as_str()), (Some("Sat 2 Mar, 7:00 pm"), Some("2030-03-02T19:00")));
    assert!(r.json["ics_url"].as_str().unwrap().starts_with("/api/public/dates/history/"));
    assert!(r.json["send_url"].as_str().unwrap().contains("invite.ics?who=1&exp="));
    assert!(r.json["partner"].as_str().unwrap().starts_with("neha"));

    let ics = t.item(&neha, open).await["ics_url"].as_str().unwrap().to_string();
    assert!(ics.starts_with(&format!("/api/public/dates/history/{open}/invite.ics?who=1&exp=")), "{ics}");
    let res = t.public(&public_url(&ics)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    assert_eq!(res.headers["content-type"], "text/calendar; charset=utf-8");
    assert!(res.text.starts_with("BEGIN:VCALENDAR") && res.text.contains(COOK), "{}", res.text);
    assert!(res.text.contains("DTSTART:2030030"), "{}", res.text);
    // Tampered, missing or someone else's signature: no calendar.
    assert_eq!(t.public(&public_url(&ics).replace("who=1", "who=0")).await.status, StatusCode::FORBIDDEN);
    assert_eq!(t.public(&format!("/api/public/dates/history/{open}/invite.ics")).await.status, StatusCode::FORBIDDEN);
    assert_eq!(t.public(&public_url(&sign(&format!("/api/dates/history/{open}/invite.ics?who=1"), -1))).await.status, StatusCode::FORBIDDEN);
    assert_ne!(t.call(Method::GET, &ics.replacen("/api/public", "/api", 1), None, None).await.status, StatusCode::OK);

    // A surprise: the planner's invite spells it out, the partner's keeps it sealed.
    t.post("/api/dates/history/when", &rohit, json!({"ids": [secret], "at": "2030-03-09T18:30"})).await;
    let theirs = t.post("/api/dates/history/when", &rohit, json!({"ids": secret, "at": "2030-03-09T18:30"})).await;
    let send = theirs.json["send_url"].as_str().unwrap().to_string();
    let sealed = t.public(&public_url(&send)).await;
    assert_eq!(sealed.status, StatusCode::OK, "{}", sealed.text);
    assert!(sealed.text.contains("A surprise from rohit"), "{}", sealed.text);
    assert!(!sealed.text.contains("Prithvi") && !sealed.text.contains("Theatre"), "{}", sealed.text);
    assert!(t.item(&neha, secret).await["ics_url"].as_str().unwrap().contains("who=1"));
    let planner = t.public(&public_url(theirs.json["ics_url"].as_str().unwrap())).await;
    assert!(planner.text.contains("Prithvi"), "{}", planner.text);
    // The partner's link can't be passed off as the planner's.
    assert_eq!(t.public(&public_url(&send).replace("who=1", "who=0")).await.status, StatusCode::FORBIDDEN);
    // And with no viewer named at all, a surprise fails closed.
    let anon = t.public(&public_url(&sign(&format!("/api/dates/history/{secret}/invite.ics"), 60))).await;
    assert!(anon.text.contains("A surprise") && !anon.text.contains("Prithvi"), "{}", anon.text);

    // Clearing the time clears the links.
    let r = t.post("/api/dates/history/when", &neha, json!({"ids": open, "at": ""})).await;
    assert_eq!((r.json["label"].as_str(), r.json["ics_url"].as_str()), (Some(""), Some("")));
    assert_eq!(t.item(&rohit, open).await["ics_url"], "");
}

#[tokio::test]
async fn jobs_are_ticked_by_their_assignee_and_plan_b_swaps() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let hid = t.keep(&couple, &rohit, COOK, false).await;
    t.exec(&format!("INSERT INTO joy_tasks (history_id, who, task) VALUES ({hid}, 0, 'Book Saltt')")).await;
    let task = t.scalar("SELECT max(id) FROM joy_tasks").await;
    let r = t.post(&format!("/api/dates/tasks/{task}/done"), &neha, json!({"done": "1"})).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN, "{}", r.text);
    assert_eq!(t.scalar(&format!("SELECT done_at IS NULL FROM joy_tasks WHERE id = {task}")).await, 1);
    for (body, done) in [(json!({"done": "1"}), 0), (json!({"done": true}), 0), (json!({"done": "0"}), 1), (json!({}), 1)] {
        let r = t.post(&format!("/api/dates/tasks/{task}/done"), &rohit, body).await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.text);
        assert_eq!(t.scalar(&format!("SELECT done_at IS NULL FROM joy_tasks WHERE id = {task}")).await, done);
    }
    assert_eq!(t.post("/api/dates/tasks/999999/done", &rohit, json!({})).await.status, StatusCode::NOT_FOUND);

    // Plan B: nothing to swap to is fine; a real one swaps and swaps back.
    let a = t.id_of(&couple, COOK).await;
    let b = t.id_of(&couple, "Zoo visit").await;
    assert_eq!(t.post(&format!("/api/dates/history/{hid}/plan-b"), &neha, json!({})).await.status, StatusCode::OK);
    t.exec(&format!("UPDATE joy_history SET plan_b = {b} WHERE id = {hid}")).await;
    t.post(&format!("/api/dates/history/{hid}/plan-b"), &neha, json!({})).await;
    assert_eq!(t.scalar(&format!("SELECT activity_id FROM joy_history WHERE id = {hid}")).await, b);
    t.post(&format!("/api/dates/history/{hid}/plan-b"), &rohit, json!({})).await;
    assert_eq!(t.scalar(&format!("SELECT activity_id FROM joy_history WHERE id = {hid}")).await, a);
}

#[tokio::test]
async fn another_couple_gets_404_for_everything() {
    let t = setup().await;
    let (rohit, _neha, couple) = t.couple().await;
    let hid = t.keep(&couple, &rohit, COOK, false).await;
    t.exec(&format!("UPDATE joy_history SET planned_at = unixepoch('now', '+1 days') WHERE id = {hid}")).await;
    t.exec(&format!("INSERT INTO joy_tasks (history_id, who, task) VALUES ({hid}, 0, 'Book')")).await;
    let task = t.scalar("SELECT max(id) FROM joy_tasks").await;
    t.exec(&format!("INSERT INTO joy_other_spending (couple_id, amount, note) VALUES ('{couple}', 77, 'mine')")).await;
    let spend = t.scalar("SELECT max(id) FROM joy_other_spending").await;
    let carol = t.loner().await;

    for (path, body) in [
        (format!("/api/dates/history/{hid}"), json!({"rating": 1})),
        (format!("/api/dates/history/{hid}/reveal"), json!({})),
        (format!("/api/dates/history/{hid}/delete"), json!({})),
        (format!("/api/dates/history/{hid}/cost"), json!({"actual": "1"})),
        (format!("/api/dates/history/{hid}/line"), json!({"line": "hi"})),
        (format!("/api/dates/history/{hid}/photos/1/delete"), json!({})),
        (format!("/api/dates/history/{hid}/plan-b"), json!({})),
        (format!("/api/dates/tasks/{task}/done"), json!({"done": "1"})),
        ("/api/dates/history/when".to_string(), json!({"ids": hid, "at": "2030-01-02T19:00"})),
        ("/api/dates/budget/spend".to_string(), json!({"amount": 5, "history_id": hid})),
        ("/api/dates/savings/add".to_string(), json!({"amount": 5, "history_id": hid})),
    ] {
        let r = t.post(&path, &carol, body).await;
        assert_eq!(r.status, StatusCode::NOT_FOUND, "{path}: {}", r.text);
    }
    assert_eq!(t.upload(&format!("/api/dates/history/{hid}/photo"), &carol, &tiny_jpeg()).await.status, StatusCode::NOT_FOUND);
    // Money she can't see isn't hers to delete either (a no-op, not a leak).
    t.post(&format!("/api/dates/budget/spend/{spend}/delete"), &carol, json!({})).await;
    assert_eq!(t.scalar("SELECT count(*) FROM joy_other_spending").await, 1);
    // Her Kept page is her own.
    let page = t.get("/api/dates/kept", &carol).await;
    assert_eq!((page.status, page.json["kept"].as_i64()), (StatusCode::OK, Some(0)));
    assert!(!page.text.contains("Cook together"));
    // Untouched.
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_history WHERE id = {hid} AND rating IS NULL AND revealed = 0")).await, 1);
    assert_eq!(t.scalar(&format!("SELECT done_at IS NULL FROM joy_tasks WHERE id = {task}")).await, 1);
    // No couple, no Kept.
    let loose = t.user("dave").await;
    assert_eq!(t.get("/api/dates/kept", &loose).await.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_date_we_had_without_a_slip_goes_straight_into_kept_and_takes_photos() {
    let t = setup().await;
    let (rohit, neha, _) = t.couple().await;
    for (body, why) in [
        (json!({"title": "", "on": "2026-03-14"}), "a name"),
        (json!({"title": "Picnic", "on": "2999-01-01"}), "already happened"),
        (json!({"title": "Picnic", "on": "14/03/2026"}), "didn't look right"),
        (json!({"title": "Picnic", "setting": "moon"}), "isn't one we know"),
        (json!({"title": "Picnic", "cost": "-5"}), ""),
    ] {
        let r = t.post("/api/dates/history/custom", &rohit, body).await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.text);
        assert!(r.text.contains(why), "{}", r.text);
    }
    let r = t.post("/api/dates/history/custom", &rohit, json!({"title": "Picnic at the lake", "place": "Powai", "on": "2026-03-14", "cost": "1,200"})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let hid: i64 = r.json["to"].as_str().unwrap().rsplit("#k").next().unwrap().parse().unwrap();
    // Both of us see it in Kept, on its day, with what it cost.
    for who in [&rohit, &neha] {
        let item = t.item(who, hid).await;
        assert_eq!((item["title"].as_str(), item["day"].as_str(), item["place"].as_str()), (Some("Picnic at the lake"), Some("2026-03-14"), Some("Powai")));
        assert_eq!(item["cost_actual_raw"], "1200");
        assert_eq!(item["can_add_photo"], true);
    }
    // Its slip never reaches the jar.
    assert_eq!(t.scalar(&format!("SELECT archived FROM joy_activities WHERE id = (SELECT activity_id FROM joy_history WHERE id = {hid})")).await, 1);
    // It takes photos like any kept date.
    assert_eq!(t.upload(&format!("/api/dates/history/{hid}/photo"), &neha, &tiny_jpeg()).await.status, StatusCode::OK);
    assert_eq!(t.item(&rohit, hid).await["photos"].as_array().unwrap().len(), 1);
}
