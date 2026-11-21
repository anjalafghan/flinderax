//! Characterization test for the month and year pages (`GET /api/dates/year/{y}` and `/{y}/{m}`).
//! A fixed couple, kept dates pinned to 2025 timestamps, regulars' nights, other spending and budgets are
//! seeded; every month page, the year page (both partners) and the edge cases are pinned. A refactor must
//! leave `tests/golden/dates_pages.json` byte-for-byte alone. Regenerate on purpose with
//! `UPDATE_GOLDEN=1 cargo test --test dates_pages_golden`. Signed links embed an expiry, so their query
//! strings are masked.

use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use joy::{app::build_router, models::AppState};
use rusty_paseto::prelude::{Key, PasetoSymmetricKey};
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower::ServiceExt;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/dates_pages.json");

struct T {
    app: Router,
    db: SqlitePool,
}

impl T {
    async fn call(&self, method: Method, path: &str, token: &str, body: Option<Value>) -> (StatusCode, Value) {
        let req = Request::builder().method(method).uri(path).header("fly-client-ip", "10.99.0.1").header("authorization", format!("Bearer {token}"));
        let req = match body {
            Some(b) => req.header("content-type", "application/json").body(Body::from(b.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    async fn exec(&self, sql: &str) {
        sqlx::query(sql).execute(&self.db).await.unwrap();
    }

    async fn user(&self, name: &str, ip: &str) -> String {
        let creds = json!({"user_name": name, "user_password": "correct horse", "signup_code": "test-signup-code"});
        for path in ["/api/common/register", "/api/common/login"] {
            let req = Request::post(path).header("content-type", "application/json").header("fly-client-ip", ip).body(Body::from(creds.to_string())).unwrap();
            let res = self.app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK, "{path}");
            if path.ends_with("login") {
                let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
                return serde_json::from_slice::<Value>(&bytes).unwrap()["access_token"].as_str().unwrap().to_string();
            }
        }
        unreachable!()
    }
}

async fn setup() -> T {
    unsafe { std::env::set_var("SIGNUP_CODE", "test-signup-code") };
    let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    T { app: build_router(AppState { db: db.clone(), paseto_key: Arc::new(key) }), db }
}

/// Strip what changes between runs: the query string of a signed link.
fn mask(v: &mut Value) {
    match v {
        Value::String(s) if s.contains("sig=") || s.contains("exp=") => *s = format!("{}?<signed>", s.split('?').next().unwrap_or("")),
        Value::Array(a) => a.iter_mut().for_each(mask),
        Value::Object(o) => o.values_mut().for_each(mask),
        _ => {}
    }
}

async fn seed(t: &T) -> (String, String) {
    let (rohit, neha) = (t.user("rohit", "10.0.0.1").await, t.user("neha", "10.0.0.2").await);
    assert!(t.call(Method::POST, "/api/couple/create", &rohit, Some(json!({}))).await.0.is_success());
    let code = t.call(Method::POST, "/api/couple/invite", &rohit, Some(json!({}))).await.1["code"].as_str().unwrap().to_string();
    assert!(t.call(Method::POST, "/api/couple/join", &neha, Some(json!({"code": code}))).await.0.is_success());
    let couple: String = sqlx::query_scalar("SELECT couple_id FROM couples LIMIT 1").fetch_one(&t.db).await.unwrap();
    let c = &couple;
    t.exec(&format!("INSERT INTO joy_settings (couple_id, key, value) VALUES ('{c}', 'PERSON_A', 'Rohit'), ('{c}', 'PERSON_B', 'Neha'), ('{c}', 'HOME_A_AREA', 'Bandra'), ('{c}', 'monthly_budget', '600000') ON CONFLICT(couple_id, key) DO UPDATE SET value = excluded.value")).await;

    let ids: Vec<i64> = sqlx::query_scalar(&format!("SELECT id FROM joy_activities WHERE couple_id = '{c}' ORDER BY id LIMIT 8")).fetch_all(&t.db).await.unwrap();
    t.exec(&format!("UPDATE joy_activities SET venue = 'Rohit''s Kitchen in Bandra', tags = 'foodie, Music' WHERE id = {}", ids[0])).await;
    t.exec(&format!("UPDATE joy_activities SET venue = '', tags = '' WHERE id = {}", ids[1])).await;
    // (activity, day at midday UTC, outcome extras)
    let kept = [
        (0, "2025-02-14", "rating, note, est_cost, actual_cost", "NULL, '', NULL, NULL"),
        (0, "2025-03-02", "rating, note, est_cost, actual_cost, cab_fare_0, cab_fare_1", "5, 'Lovely evening\nsecond line', 50000, 90000, 20000, 15000"),
        (1, "2025-03-16", "rating, note, est_cost, actual_cost", "3, 'Fine', 100000, 60000"),
        (2, "2025-03-30", "rating, note, est_cost, actual_cost", "5, 'Also lovely', 30000, 30000"),
        (0, "2025-04-06", "rating, note, est_cost, actual_cost", "4, 'Again', 50000, 50000"),
        (3, "2025-11-09", "rating, note, est_cost", "2, 'Meh', 80000"),
    ];
    for (a, day, cols, vals) in kept {
        t.exec(&format!("INSERT INTO joy_history (couple_id, activity_id, outcome, picked_at, {cols}) VALUES ('{c}', {}, 'did_it', unixepoch('{day} 12:00:00'), {vals})", ids[a])).await;
    }
    // Planned (booked) in the past, a sealed surprise only Rohit drew, a revealed one, one passed over, one in the future.
    t.exec(&format!("INSERT INTO joy_history (couple_id, activity_id, outcome, picked_at, planned_at, rating) VALUES ('{c}', {}, 'did_it', unixepoch('2025-04-01 12:00:00'), unixepoch('2025-05-20 12:00:00'), 4)", ids[4])).await;
    t.exec(&format!("INSERT INTO joy_history (couple_id, activity_id, outcome, picked_at, surprise, revealed, drawn_by, rating) VALUES ('{c}', {}, 'did_it', unixepoch('2025-12-12 12:00:00'), 1, 0, 0, 5)", ids[5])).await;
    t.exec(&format!("INSERT INTO joy_history (couple_id, activity_id, outcome, picked_at, surprise, revealed, drawn_by, rating) VALUES ('{c}', {}, 'did_it', unixepoch('2025-12-20 12:00:00'), 1, 1, 1, 4)", ids[6])).await;
    t.exec(&format!("INSERT INTO joy_history (couple_id, activity_id, outcome, picked_at, planned_at) VALUES ('{c}', {}, 'did_it', unixepoch('2025-06-01 12:00:00'), unixepoch('2099-01-05 12:00:00'))", ids[7])).await;
    t.exec(&format!("INSERT INTO joy_history (couple_id, activity_id, outcome, picked_at) VALUES ('{c}', {}, 'passed', unixepoch('2025-07-01 12:00:00'))", ids[7])).await;
    // Photos on the March date (the second is its hero), and the flag.
    let march: i64 = sqlx::query_scalar("SELECT id FROM joy_history WHERE rating = 5 AND note LIKE 'Lovely%'").fetch_one(&t.db).await.unwrap();
    t.exec(&format!("INSERT INTO joy_history_photos (history_id, file) VALUES ({march}, 'a.jpg'), ({march}, 'b.jpg')")).await;
    t.exec(&format!("UPDATE joy_history SET has_photo = 1, hero_photo_id = (SELECT max(id) FROM joy_history_photos) WHERE id = {march}")).await;

    // Spending that wasn't a slip, and a month with its own budget.
    t.exec(&format!("INSERT INTO joy_other_spending (couple_id, amount, note, spent_on) VALUES ('{c}', 12000, 'Ice cream', '2025-03-10'), ('{c}', 5000, 'Chai', '2025-03-11'), ('{c}', 90000, 'Gift', '2025-09-01')")).await;
    t.exec(&format!("INSERT INTO joy_month_budget (couple_id, month, base, carry) VALUES ('{c}', '2025-03', 400000, 50000), ('{c}', '2025-04', 120000, 0)")).await;

    // Two regulars; nights went in March and April (one with a photo), one skipped, one paid more.
    for (title, place) in [("Blue Tokai", "Goregaon East"), ("Sunday Chai", "Bandra")] {
        let r = t.call(Method::POST, "/api/dates/regulars", &rohit, Some(json!({"title": title, "place": place, "days": [2, 4], "cost": "300"}))).await;
        assert_eq!(r.0, StatusCode::OK, "{}", r.1);
    }
    for (rid, day, amount) in [(1, "2025-03-04", 450), (1, "2025-03-18", 300), (2, "2025-03-06", 200), (1, "2025-04-03", 350)] {
        let r = t.call(Method::POST, &format!("/api/dates/regulars/{rid}/went"), &rohit, Some(json!({"day": day, "amount": amount}))).await;
        assert_eq!(r.0, StatusCode::OK, "{}", r.1);
    }
    t.exec("UPDATE joy_regular_visits SET has_photo = 1 WHERE day = '2025-03-04'").await;
    // Older nights were never kept dates: they stand on their own on the month page.
    t.exec("UPDATE joy_regular_visits SET history_id = NULL, has_photo = 1 WHERE day IN ('2025-03-18', '2025-03-06', '2025-04-03')").await;
    (rohit, neha)
}

async fn snapshot() -> String {
    let t = setup().await;
    let (rohit, neha) = seed(&t).await;
    let mut pages = Vec::new();
    let mut paths: Vec<(String, &str)> = (1..=12).map(|m| (format!("/api/dates/year/2025/{m}"), "rohit")).collect();
    paths.extend([
        ("/api/dates/year/2025".into(), "rohit"),
        ("/api/dates/year/2025".into(), "neha"),
        ("/api/dates/year/2025/3".into(), "neha"),
        ("/api/dates/year/2024".into(), "rohit"),
        ("/api/dates/year/2024/6".into(), "rohit"),
        ("/api/dates/year/2099".into(), "rohit"),
        ("/api/dates/year/1999".into(), "rohit"),
        ("/api/dates/year/2025/13".into(), "rohit"),
        ("/api/dates/year/2025/abc".into(), "rohit"),
        ("/api/dates/year/2025/0".into(), "rohit"),
    ]);
    for (path, who) in paths {
        let token = if who == "rohit" { &rohit } else { &neha };
        let (status, mut body) = t.call(Method::GET, &path, token, None).await;
        mask(&mut body);
        pages.push(json!({ "path": path, "as": who, "status": status.as_u16(), "body": body }));
    }
    serde_json::to_string_pretty(&pages).unwrap() + "\n"
}

#[tokio::test]
async fn month_and_year_pages_match_the_pinned_output() {
    let now = snapshot().await;
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::create_dir_all(std::path::Path::new(FILE).parent().unwrap()).unwrap();
        std::fs::write(FILE, &now).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
    assert!(now == pinned, "month/year pages changed; if on purpose, rerun with UPDATE_GOLDEN=1");
}
