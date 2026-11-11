//! Dates (Joy) JSON API, part 1: the jar, the draw, slips and envelopes. Ported from Joy's route
//! tests: surprises stay sealed, peeks are capped on the server, double taps don't double-record,
//! the jar skips what it should, and one couple never sees another's slips.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    body::{Body, to_bytes},
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
}

async fn setup() -> T {
    // Sign-ups need the code from the SIGNUP_CODE secret; the tests use a fixed one.
    // Safety: every test in this binary sets the same value before any request reads it.
    unsafe { std::env::set_var("SIGNUP_CODE", "test-signup-code") };
    let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    let app = build_router(AppState { db: db.clone(), redis: None, paseto_key: Arc::new(key) });
    T { app, db }
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
        let res = self.app.clone().oneshot(req).await.unwrap();
        let (status, headers) = (res.status(), res.headers().clone());
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8_lossy(&bytes).into_owned();
        Res { status, headers, json: serde_json::from_str(&text).unwrap_or(Value::Null), text }
    }

    async fn get(&self, path: &str, token: &str) -> Res {
        self.call(Method::GET, path, Some(token), None).await
    }

    async fn post(&self, path: &str, token: &str, body: Value) -> Res {
        self.call(Method::POST, path, Some(token), Some(body)).await
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
    async fn loner(&self) -> (String, String) {
        let c = self.user("carol").await;
        assert!(self.post("/api/couple/create", &c, json!({})).await.status.is_success());
        let id = self.text(&"SELECT couple_id FROM couples WHERE user_0 = (SELECT user_id FROM users ORDER BY rowid DESC LIMIT 1)".to_string())
        .await;
        (c, id)
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
}

const PLAY: &str = "A play at Prithvi Theatre";

fn ids(v: &Value) -> Vec<i64> {
    v["ids"].as_array().unwrap().iter().map(|i| i.as_i64().unwrap()).collect()
}

fn drawn(card: &Value) -> i64 {
    card["activity"]["id"].as_i64().unwrap_or_else(|| panic!("no slip drawn: {card}"))
}

/// A page's JSON without its list of every slip (slip titles are in the jar by design).
fn without_slips(mut v: Value) -> String {
    v.as_object_mut().unwrap().remove("slips");
    v.to_string()
}

// ── the draw ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_draw_returns_a_slip_and_respects_budget_and_time() {
    let t = setup().await;
    let (rohit, _, couple) = t.couple().await;

    let card = t.post("/api/dates/pick", &rohit, json!({})).await;
    assert_eq!(card.status, StatusCode::OK, "{}", card.text);
    let slip = drawn(&card.json);
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_activities WHERE id = {slip} AND couple_id = '{couple}'")).await, 1);
    assert!(card.json["activity"]["title"].as_str().unwrap().len() > 2);
    assert_eq!(card.json["max_rerolls"], 3);
    assert_eq!(card.json["state"]["current"], slip.to_string(), "the card carries where the draw has got to");

    // Free and short only. Numbers work where Joy's form had text.
    for _ in 0..20 {
        let c = t.post("/api/dates/pick", &rohit, json!({"budget": 0, "max_duration": "short"})).await.json;
        let id = drawn(&c);
        let (cost, duration): (i64, String) =
            sqlx::query_as(&format!("SELECT est_cost, duration FROM joy_activities WHERE id = {id}")).fetch_one(&t.db).await.unwrap();
        assert_eq!((cost, duration.as_str()), (0, "short"), "{c}");
        assert_eq!(c["activity"]["cost"], "Free");
    }
    let free = ids(&t.post("/api/dates/match", &rohit, json!({"budget": "0"})).await.json);
    let some = ids(&t.post("/api/dates/match", &rohit, json!({"budget": "1000"})).await.json);
    let all = t.post("/api/dates/match", &rohit, json!({})).await.json;
    assert!(!free.is_empty() && free.len() < some.len() && some.len() < all["count"].as_u64().unwrap() as usize);
    assert!(all["count"].as_u64() <= all["total"].as_u64(), "closed days and seasons can still rule slips out");
    for id in &free {
        assert_eq!(t.scalar(&format!("SELECT est_cost FROM joy_activities WHERE id = {id}")).await, 0);
    }
    // An empty body is a draw too.
    assert_eq!(t.call(Method::POST, "/api/dates/match", Some(&rohit), None).await.status, StatusCode::OK);
}

#[tokio::test]
async fn the_jar_page_carries_its_view_model() {
    let t = setup().await;
    let (rohit, neha, _) = t.couple().await;
    let jar = t.get("/api/dates/jar", &rohit).await;
    assert_eq!(jar.status, StatusCode::OK, "{}", jar.text);
    let j = &jar.json;
    assert!(j["me_name"].as_str().unwrap().starts_with("rohit"));
    assert_eq!(j["count"].as_u64().unwrap() as usize, j["slips"].as_array().unwrap().len());
    assert_eq!(j["budget"]["spent"], "₹0");
    assert_eq!(j["middle"]["both_in"], false);
    assert_eq!(j["regulars"]["show"], false);
    assert_eq!(j["my_turn"], false);
    // Whoever didn't keep the last date draws next, with a bonus peek.
    let id = t.scalar("SELECT min(id) FROM joy_activities").await;
    assert_eq!(t.post(&format!("/api/dates/accept/{id}"), &rohit, json!({})).await.status, StatusCode::OK);
    assert_eq!(t.get("/api/dates/jar", &neha).await.json["my_turn"], true);
    assert_eq!(t.get("/api/dates/jar", &rohit).await.json["my_turn"], false);
}

#[tokio::test]
async fn nobody_gets_in_without_a_token_or_a_couple() {
    let t = setup().await;
    let lonely = t.user("dave").await;
    for path in ["/api/dates/jar", "/api/dates/slips", "/api/dates/slips/new"] {
        assert_eq!(t.call(Method::GET, path, None, None).await.status, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(t.get(path, &lonely).await.status, StatusCode::FORBIDDEN, "{path} needs a couple");
    }
    assert_eq!(t.post("/api/dates/pick", &lonely, json!({})).await.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn peeks_stop_at_three_and_the_server_keeps_count() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    // Generated slips go back for free; keep this test about the peek cap only.
    t.exec("UPDATE joy_activities SET generated = 0").await;
    let current = t.id_of(&couple, "Play Pokémon Emerald together").await;

    // Used 3 of 3: the card shows the same slip again and does not record another put-back.
    let before = t.scalar("SELECT count(*) FROM joy_history WHERE outcome = 'skipped'").await;
    let r = t.post("/api/dates/pick", &rohit, json!({"used": 3, "current": current, "reroll": true, "excluded": []})).await;
    assert_eq!(drawn(&r.json), current, "out of peeks, so the same slip stays");
    assert_eq!((r.json["rerolls_left"].as_u64(), r.json["max_rerolls"].as_u64()), (Some(0), Some(3)));
    assert_eq!(t.scalar("SELECT count(*) FROM joy_history WHERE outcome = 'skipped'").await, before);

    // Used 2: one more peek, and it costs the last one.
    let r = t.post("/api/dates/pick", &rohit, json!({"used": "2", "current": current.to_string(), "reroll": "1"})).await;
    assert_ne!(drawn(&r.json), current);
    assert_eq!(r.json["state"]["used"], "3");
    assert_eq!(r.json["rerolls_left"], 0);
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_history WHERE activity_id = {current} AND outcome = 'skipped'")).await, 1);

    // Rohit kept the last date, so it's Neha's turn: she gets a fourth peek, he doesn't.
    let cook = t.id_of(&couple, "Cook together").await;
    t.post(&format!("/api/dates/accept/{cook}"), &rohit, json!({})).await;
    let form = json!({"used": 3, "current": current, "reroll": 1, "excluded": [cook]});
    let r = t.post("/api/dates/pick", &neha, form.clone()).await;
    assert_ne!(drawn(&r.json), current, "Neha's 4th peek draws a new slip");
    assert_eq!((r.json["bonus_peek"].as_bool(), r.json["max_rerolls"].as_u64()), (Some(true), Some(4)));
    assert_eq!(r.json["rerolls_left"], 0, "and that was her last one");
    let r = t.post("/api/dates/pick", &rohit, form).await;
    assert_eq!(drawn(&r.json), current, "Rohit keeps the same slip");
    assert_eq!((r.json["bonus_peek"].as_bool(), r.json["rerolls_left"].as_u64()), (Some(false), Some(0)));
}

#[tokio::test]
async fn nothing_fits_says_why_and_offers_the_smallest_loosening() {
    let t = setup().await;
    let (rohit, _, couple) = t.couple().await;
    t.exec("UPDATE joy_activities SET archived = 1").await;
    let id = t.id_of(&couple, "Cook together").await;
    t.exec(&format!(
        "UPDATE joy_activities SET archived = 0, est_cost = 1500, closed_days = '', season_from = NULL, season_to = NULL WHERE id = {id}"
    ))
    .await;

    let r = t.post("/api/dates/pick", &rohit, json!({"budget": 300})).await;
    assert!(r.json["activity"].is_null(), "{}", r.text);
    let msg = r.json["message"].as_str().unwrap();
    assert!(msg.contains("Nothing in the jar fits under ₹300") && msg.contains("Cook together") && msg.contains("₹1,500"), "{msg}");
    assert_eq!(r.json["relax_label"], "Stretch to ₹1,500");
    assert_eq!(r.json["relax"]["budget"], "1500");
    let hidden = r.json["relax_hidden"].as_array().unwrap();
    assert!(hidden.iter().any(|h| h["name"] == "budget" && h["value"] == "1500"), "Joy's hidden field is kept too");

    // Taking the offer draws it.
    let again = t.post("/api/dates/pick", &rohit, r.json["relax"].clone()).await;
    assert_eq!(drawn(&again.json), id);

    // Free is its own message, and a time that is too short names the slip's real length.
    let free = t.post("/api/dates/pick", &rohit, json!({"budget": 0})).await;
    assert!(free.json["message"].as_str().unwrap().starts_with("Nothing in the jar is free."), "{}", free.text);

    // Closed every day: the jar says so, with nothing to loosen.
    t.exec(&format!("UPDATE joy_activities SET est_cost = 0, closed_days = 'sun,mon,tue,wed,thu,fri,sat' WHERE id = {id}")).await;
    let closed = t.post("/api/dates/pick", &rohit, json!({})).await;
    assert!(closed.json["message"].as_str().unwrap().contains("closed on"), "{}", closed.text);
    assert_eq!(closed.json["relax_label"], "");
    assert!(closed.json["relax"].is_null());
    assert!(!ids(&t.post("/api/dates/match", &rohit, json!({})).await.json).contains(&id));

    // An empty jar.
    t.exec("UPDATE joy_activities SET archived = 1").await;
    let empty = t.post("/api/dates/pick", &rohit, json!({})).await;
    assert_eq!(empty.json["message"], "The jar is empty. Write the first slip.");
}

#[tokio::test]
async fn not_for_me_takes_a_slip_out_of_every_draw_until_brought_back() {
    let t = setup().await;
    let (_, neha, couple) = t.couple().await;
    let drive = t.id_of(&couple, "Long drive to Lonavala").await;
    assert!(ids(&t.post("/api/dates/match", &neha, json!({})).await.json).contains(&drive));

    let r = t.post("/api/dates/pick", &neha, json!({"veto": true, "current": drive, "used": 1})).await;
    assert_eq!(t.scalar(&format!("SELECT person FROM joy_vetoes WHERE activity_id = {drive}")).await, 1);
    assert_ne!(r.json["activity"]["id"], drive);
    assert_eq!(r.json["state"]["used"], "1", "a veto doesn't cost a peek");
    assert!(!ids(&t.post("/api/dates/match", &neha, json!({})).await.json).contains(&drive), "/match skips it");
    for _ in 0..40 {
        assert_ne!(t.post("/api/dates/pick", &neha, json!({})).await.json["activity"]["id"], drive, "/pick skips it");
    }

    let page = t.get("/api/dates/slips", &neha).await.json;
    let item = page["groups"].as_array().unwrap().iter().flat_map(|g| g["items"].as_array().unwrap()).find(|i| i["id"] == drive).unwrap();
    assert!(item["vetoed_by"].as_str().unwrap().starts_with("neha"), "{item}");
    assert_eq!(t.post(&format!("/api/dates/slips/{drive}/unveto"), &neha, json!({})).await.json, json!({"ok": true, "to": "/joy/slips"}));
    assert!(ids(&t.post("/api/dates/match", &neha, json!({})).await.json).contains(&drive), "brought back");
}

// ── keeping a date ────────────────────────────────────────────────────────

#[tokio::test]
async fn accepting_stamps_a_date_once() {
    let t = setup().await;
    let (_, neha, couple) = t.couple().await;
    let id = t.id_of(&couple, "Cook together").await;
    let r = t.post(&format!("/api/dates/accept/{id}"), &neha, json!({})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert_eq!(r.json["done"], true);
    let stamp = r.json["stamp_date"].as_str().unwrap();
    assert_eq!(stamp.matches(" · ").count(), 2, "{stamp}");
    assert_eq!(r.json["activity"]["id"], id);
    assert!(r.json["partner"].as_str().unwrap().starts_with("rohit"));
    let hid = t.scalar(&format!("SELECT max(id) FROM joy_history WHERE activity_id = {id}")).await;
    assert_eq!(r.json["history_ids"], hid.to_string());
    assert_eq!(t.scalar(&format!("SELECT drawn_by FROM joy_history WHERE id = {hid}")).await, 1, "Neha is slot 1");
    assert_eq!(t.scalar(&format!("SELECT times_done FROM joy_activities WHERE id = {id}")).await, 1);

    // A double tap records one date. Options also work from the query string.
    t.post(&format!("/api/dates/accept/{id}"), &neha, json!({})).await;
    t.call(Method::POST, &format!("/api/dates/accept/{id}?surprise=1"), Some(&neha), None).await;
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_history WHERE activity_id = {id} AND outcome = 'did_it'")).await, 1);

    // The share text is an envelope link, not the date spelled out.
    let share = r.json["share_text"].as_str().unwrap();
    assert!(share.starts_with("It's a date! Open the envelope:\n") && share.contains("/joy/e/"), "{share}");
    assert!(r.json["wa_url"].as_str().unwrap().contains("%2Fjoy%2Fe%2F"));
    assert!(!share.contains("Cook"));

    // A pair rides along only if it really pairs with the slip.
    let (a, b) = (t.id_of(&couple, "Dinner at Saltt").await, t.id_of(&couple, "Zoo visit").await);
    t.post(&format!("/api/dates/accept/{a}"), &neha, json!({"with": b})).await;
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_history WHERE activity_id = {b}")).await, 0, "not a partner of that slip");
}

#[tokio::test]
async fn a_surprise_is_sealed_for_the_partner_but_not_the_planner() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let play = t.id_of(&couple, PLAY).await;
    let r = t.post(&format!("/api/dates/accept/{play}"), &rohit, json!({"surprise": true})).await;
    assert_eq!(r.json["surprise"], true);
    let hid = t.scalar(&format!("SELECT max(id) FROM joy_history WHERE activity_id = {play}")).await;
    t.exec(&format!("UPDATE joy_history SET planned_at = unixepoch('now', '+1 days') WHERE id = {hid}")).await;

    let neha_jar = without_slips(t.get("/api/dates/jar", &neha).await.json);
    assert!(!neha_jar.contains("Prithvi"), "the jar page (nudges, memories) leaks it: {neha_jar}");
    assert!(neha_jar.contains("a surprise from rohit"), "{neha_jar}");
    let rohit_jar = without_slips(t.get("/api/dates/jar", &rohit).await.json);
    assert!(rohit_jar.contains("Prithvi"), "the planner sees their own surprise");

    // The envelope for Neha holds hints only, until it's revealed.
    let token = t.text(&format!("SELECT share_token FROM joy_history WHERE id = {hid}")).await;
    assert_eq!(token.len(), 24);
    let sealed = t.call(Method::GET, &format!("/api/public/dates/e/{token}"), None, None).await;
    assert_eq!(sealed.status, StatusCode::OK);
    assert_eq!(sealed.json["sealed"], true);
    assert_eq!(sealed.json["kicker"], "A surprise");
    assert_eq!(sealed.json["title"], "Something's planned");
    assert_eq!(sealed.json["to"].as_str().map(|n| n.starts_with("neha")), Some(true));
    assert!(!sealed.text.contains("Prithvi"), "{}", sealed.text);
    assert_eq!(sealed.json["place"], "");
    assert!(sealed.json["maps_url"].is_null());
    t.exec(&format!("UPDATE joy_history SET revealed = 1 WHERE id = {hid}")).await;
    let open = t.call(Method::GET, &format!("/api/public/dates/e/{token}"), None, None).await;
    assert_eq!((open.json["sealed"].as_bool(), open.json["kicker"].as_str()), (Some(false), Some("It's a date")));
    assert!(open.text.contains("Prithvi"));
    assert!(without_slips(t.get("/api/dates/jar", &neha).await.json).contains("Prithvi"), "revealed to everyone");
}

#[tokio::test]
async fn an_envelope_opens_without_logging_in_and_a_bad_token_gets_nothing() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let play = t.id_of(&couple, PLAY).await;

    // A kept date: the link opens for anyone holding it.
    let r = t.post(&format!("/api/dates/accept/{play}"), &neha, json!({})).await;
    assert!(r.json["share_text"].as_str().unwrap().contains("/joy/e/"));
    let hid = t.scalar(&format!("SELECT max(id) FROM joy_history WHERE activity_id = {play}")).await;
    let token = t.text(&format!("SELECT share_token FROM joy_history WHERE id = {hid}")).await;
    assert_eq!(token.len(), 24);
    assert!(r.json["share_text"].as_str().unwrap().ends_with(&token));
    let page = t.call(Method::GET, &format!("/api/public/dates/e/{token}"), None, None).await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.text);
    assert_eq!(page.json["kicker"], "It's a date");
    assert!(page.json["title"].as_str().unwrap().contains("Prithvi"));
    assert!(page.json["from"].as_str().unwrap().starts_with("neha") && page.json["to"].as_str().unwrap().starts_with("rohit"));
    assert_eq!(page.json["sealed"], false);
    assert_eq!(page.headers.get("x-robots-tag").unwrap(), "noindex");
    assert_eq!(page.headers.get("referrer-policy").unwrap(), "no-referrer");
    assert!(page.json["meta"].as_str().unwrap().contains(" · "));

    // A slip drawn from the jar goes out as an envelope too.
    let card = t.post("/api/dates/pick", &rohit, json!({})).await;
    let slip = drawn(&card.json);
    let slip_token = t.text(&format!("SELECT share_token FROM joy_activities WHERE id = {slip}")).await;
    assert_eq!(slip_token.len(), 24);
    assert!(card.json["share_text"].as_str().unwrap().ends_with(&format!("/joy/e/{slip_token}")), "{}", card.json["share_text"]);
    let env = t.call(Method::GET, &format!("/api/public/dates/e/{slip_token}"), None, None).await;
    assert_eq!(env.json["kicker"], "What about this?");
    assert_eq!(env.json["sealed"], false);

    // Guessing gets nothing.
    for bad in ["AAAAAAAAAAAAAAAAAAAAAAAA", "short", "%27%3B%20DROP%20TABLE%20users%3B--", "aaaaaaaaaaaaaaaaaaaaaaaaa"] {
        assert_eq!(t.call(Method::GET, &format!("/api/public/dates/e/{bad}"), None, None).await.status, StatusCode::NOT_FOUND, "{bad}");
    }
    // The same token every time it's shared.
    t.post(&format!("/api/dates/accept/{play}"), &neha, json!({})).await;
    assert_eq!(t.text(&format!("SELECT share_token FROM joy_history WHERE id = {hid}")).await, token);
}

// ── meet in the middle ────────────────────────────────────────────────────

#[tokio::test]
async fn meet_in_the_middle_draws_from_the_overlap_and_never_shows_the_others_answers() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let (carol, other) = t.loner().await;

    // Not both in yet: no middle draw, just a note.
    let early = t.post("/api/dates/pick", &rohit, json!({"middle": true})).await;
    assert!(early.json["message"].as_str().unwrap().starts_with("Once you've both checked in"), "{}", early.text);
    assert!(early.json["activity"].is_null());

    let r = t
        .post("/api/dates/middle", &rohit, json!({"energy": "high", "budget": 3000, "max_duration": "full_day", "reach": "far", "vibe": "adventurous"}))
        .await;
    assert_eq!(r.json, json!({"ok": true, "to": "/joy#middle"}));
    t.post("/api/dates/middle", &carol, json!({"energy": "low", "budget": "100", "reach": "home"})).await;

    // Neha sees only that Rohit is in; her own form starts from the defaults, not his answers.
    let neha_jar = t.get("/api/dates/jar", &neha).await.json;
    let m = &neha_jar["middle"];
    assert_eq!((m["partner_in"].as_bool(), m["both_in"].as_bool(), m["mine"].is_null()), (Some(true), Some(false), true));
    let picked = |opts: &Value| -> Vec<String> {
        opts.as_array().unwrap().iter().filter(|o| o["selected"] == true).map(|o| o["value"].as_str().unwrap().to_string()).collect()
    };
    assert_eq!(picked(&m["energies"]), ["med"]);
    assert_eq!(picked(&m["budgets"]), ["1500"]);
    assert_eq!(picked(&m["times"]), ["short"]);
    assert_eq!(picked(&m["reaches"]), ["apart"]);
    assert!(picked(&m["vibes"]).is_empty() && m["no_vibe"] == true);
    assert!(!m.to_string().contains("adventurous\",\"selected\":true"));

    // Staying in still means one of us crossing town: half a day, door to door.
    t.post("/api/dates/middle", &neha, json!({"energy": "low", "budget": "500", "max_duration": "half_day", "reach": "home", "vibe": "cozy"})).await;
    let rohit_jar = t.get("/api/dates/jar", &rohit).await.json;
    assert_eq!(rohit_jar["middle"]["both_in"], true);
    assert_eq!(rohit_jar["middle"]["mine"]["energy"], "high", "his own answers are his");
    assert!(!rohit_jar["middle"].to_string().contains("\"cozy\",\"selected\":true"), "not hers");

    let mut current = String::new();
    let mut seen = 0;
    for i in 0..12 {
        let form = if i == 0 { json!({"middle": true}) } else { json!({"middle": true, "reroll": true, "current": current, "used": 0}) };
        let card = t.post("/api/dates/pick", &rohit, form).await.json;
        assert_eq!(card["middle"], true, "{card}");
        let state = card["state"].as_object().cloned().unwrap_or_default();
        for secret in ["budget", "energy", "settings", "max_km", "tag", "max_duration"] {
            assert!(!state.contains_key(secret), "the card carries the merged answers: {card}");
        }
        assert!(!card.to_string().contains("Stretch to"), "a relax hint gives an answer away");
        assert_eq!(card["relax_label"], "");
        let Some(id) = card["activity"]["id"].as_i64() else { break };
        seen += 1;
        current = id.to_string();
        let (setting, energy, cost): (String, String, i64) =
            sqlx::query_as(&format!("SELECT setting, energy, est_cost FROM joy_activities WHERE id = {id}")).fetch_one(&t.db).await.unwrap();
        assert_eq!(setting, "home", "Neha wants to stay in");
        assert_eq!(energy, "low", "Neha is tired");
        assert!(cost <= 500, "Neha's budget");
    }
    assert!(seen > 0, "the middle draw found something cozy at home");

    // Nothing cozy left: the message doesn't say which answer to change.
    t.exec(&format!("UPDATE joy_activities SET archived = 1 WHERE couple_id = '{couple}'")).await;
    let none = t.post("/api/dates/pick", &rohit, json!({"middle": true})).await.json;
    assert!(none["message"].as_str().unwrap().starts_with("The jar is empty") || none["message"].as_str().unwrap().contains("check-in"), "{none}");
    assert!(!none.to_string().contains("500"));

    // Taking a check-in back ends the middle draw, for this couple only.
    t.post("/api/dates/middle", &neha, json!({"energy": "low", "clear": "1"})).await;
    assert_eq!(t.get("/api/dates/jar", &rohit).await.json["middle"]["both_in"], false);
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_checkins WHERE couple_id = '{couple}'")).await, 1);
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_checkins WHERE couple_id = '{other}'")).await, 1, "another couple's check-in stays");
}

// ── slips ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn slips_are_listed_edited_archived_and_deleted() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let page = t.get("/api/dates/slips", &rohit).await.json;
    let total = page["total"].as_u64().unwrap();
    assert!(total > 50, "the starter jar");
    assert_eq!(page["groups"].as_array().unwrap().iter().map(|g| g["items"].as_array().unwrap().len() as u64).sum::<u64>(), total);
    assert!(page["groups"][0]["items"][0]["closed_label"].is_string() && page["groups"][0]["key"].is_string());

    let blank = t.get("/api/dates/slips/new", &neha).await.json;
    assert_eq!((blank["id"].is_null(), blank["setting"].is_null()), (true, true));
    assert!(blank["added_by"].is_null() && blank["people"].as_array().unwrap().iter().any(|p| p["selected"] == true && p["label"].as_str().unwrap().starts_with("neha")));
    assert_eq!(blank["settings"].as_array().unwrap().iter().find(|o| o["selected"] == true).unwrap()["value"], "local");

    // Bad hours are caught, with the form handed back.
    let mut body = json!({"title": "Sunset at Carter Road", "est_cost": 300, "setting": "local", "duration": "short", "energy": "low",
                          "location": "19.0700, 72.8230", "hours": "whenever"});
    let bad = t.post("/api/dates/slips", &rohit, body.clone()).await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(bad.json["error"].as_str().unwrap().starts_with("Hours should look like"), "{}", bad.text);
    assert_eq!((bad.json["details_open"].as_bool(), bad.json["title"].as_str()), (Some(true), Some("Sunset at Carter Road")));
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_activities WHERE couple_id = '{couple}' AND title = 'Sunset at Carter Road'")).await, 0);
    for (field, value, msg) in [("title", json!("  "), "Give it a name."), ("location", json!("nowhere"), "Location should look like"), ("setting", json!("moon"), "Pick a setting"), ("est_cost", json!("lots"), "whole number")] {
        let mut b = body.clone();
        b["hours"] = json!("");
        b[field] = value;
        let r = t.post("/api/dates/slips", &rohit, b).await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{field}");
        assert!(r.json["error"].as_str().unwrap().contains(msg), "{field}: {}", r.text);
    }

    body["hours"] = json!("mon-sun 17:00-23:00");
    body["closed"] = json!(["mon"]);
    body["tags"] = json!("romantic, outdoors");
    let ok = t.post("/api/dates/slips", &rohit, body.clone()).await;
    assert_eq!(ok.json, json!({"ok": true, "to": "/joy/slips"}), "{}", ok.text);
    let id = t.id_of(&couple, "Sunset at Carter Road").await;
    assert_eq!(t.text(&format!("SELECT hours FROM joy_activities WHERE id = {id}")).await, "mon-sun 17:00-23:00");

    let edit = t.get(&format!("/api/dates/slips/{id}"), &neha).await.json;
    assert_eq!((edit["id"].as_i64(), edit["est_cost"].as_str(), edit["location"].as_str()), (Some(id), Some("300"), Some("19.07, 72.823")));
    assert!(edit["closed_opts"].as_array().unwrap().iter().any(|o| o["value"] == "mon" && o["selected"] == true));
    assert!(edit["hours_today"].is_string());

    body["title"] = json!("Sunset at Bandstand");
    body["pairs"] = json!([t.id_of(&couple, "Cook together").await]);
    assert_eq!(t.post(&format!("/api/dates/slips/{id}"), &neha, body).await.status, StatusCode::OK);
    assert_eq!(t.text(&format!("SELECT title FROM joy_activities WHERE id = {id}")).await, "Sunset at Bandstand");
    let after = t.get(&format!("/api/dates/slips/{id}"), &rohit).await.json;
    assert!(after["pair_opts"].as_array().unwrap()[0]["selected"] == true, "current pairs come first");

    let archived = t.post(&format!("/api/dates/slips/{id}/archive"), &rohit, json!({})).await;
    assert_eq!(archived.json, json!({"ok": true, "to": "/joy/slips"}));
    let page = t.get("/api/dates/slips", &rohit).await.json;
    assert_eq!(page["archived"].as_array().unwrap().len(), 1);
    assert_eq!(page["total"].as_u64().unwrap(), total, "one added, one archived");
    t.post(&format!("/api/dates/slips/{id}/restore"), &rohit, json!({})).await;
    assert_eq!(t.get("/api/dates/slips", &rohit).await.json["archived"].as_array().unwrap().len(), 0);
    t.post(&format!("/api/dates/slips/{id}/delete"), &rohit, json!({})).await;
    assert_eq!(t.get(&format!("/api/dates/slips/{id}"), &rohit).await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_slip_can_be_planned_for_a_day_and_sent() {
    let t = setup().await;
    let (_, neha, couple) = t.couple().await;
    let id = t.id_of(&couple, "Do a physical jigsaw puzzle together").await;

    let bad = t.post(&format!("/api/dates/slips/{id}/plan"), &neha, json!({"at": ""})).await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY, "a time is needed");
    assert_eq!(t.call(Method::POST, &format!("/api/dates/slips/{id}/plan"), Some(&neha), None).await.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_history WHERE activity_id = {id}")).await, 0);

    let r = t.post(&format!("/api/dates/slips/{id}/plan"), &neha, json!({"at": "2026-11-02T19:00"})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert_eq!(r.json["label"], "Mon 2 Nov, 7:00 pm");
    assert!(r.json["partner"].as_str().unwrap().starts_with("rohit"));
    let hid = r.json["history_id"].as_i64().unwrap();
    assert_eq!(r.json["to"], format!("/joy/kept#k{hid}"));
    assert_eq!(t.text(&format!("SELECT strftime('%Y-%m-%d %H:%M', planned_at, 'unixepoch', 'localtime') FROM joy_history WHERE id = {hid}")).await, "2026-11-02 19:00");
    // What goes to WhatsApp is an envelope link, not the date spelled out.
    let token = t.text(&format!("SELECT share_token FROM joy_history WHERE id = {hid}")).await;
    let share = r.json["share_text"].as_str().unwrap();
    assert!(share.starts_with("It's a date, Mon 2 Nov, 7:00 pm! Open the envelope:") && share.ends_with(&format!("/joy/e/{token}")), "{share}");
    assert!(!share.contains("jigsaw"));

    // The calendar link is signed for this viewer, and only for them.
    let ics = r.json["ics_url"].as_str().unwrap();
    let (public_path, rest) = ics.split_once("&exp=").unwrap();
    let (exp, sig) = rest.split_once("&sig=").unwrap();
    assert_eq!(public_path, format!("/api/public/dates/history/{hid}/invite.ics?who=1"));
    // The signature covers the protected-style path; the browser fetches the public one.
    let path = &public_path.replace("/api/public/dates/", "/api/dates/");
    let exp: i64 = exp.parse().unwrap();
    assert!(joy::joy::api::verify(path, exp, sig));
    assert!(!joy::joy::api::verify(&path.replace("who=1", "who=0"), exp, sig), "can't pose as the planner");
    assert!(!joy::joy::api::verify(path, exp + 1, sig));
}

#[tokio::test]
async fn keeping_a_scheduled_date_plans_the_day() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    for (k, v) in [
        ("DATE_DAYS", "mon,tue,wed,thu,fri,sat,sun"),
        ("OFFICE", "19.1741, 72.8580"),
        ("OFFICE_NAME", "the office"),
        ("LATEST_A", "22:00"),
        ("LATEST_A_AT", "19.1646, 72.8493"),
        ("LATEST_A_PLACE", "Goregaon station"),
        ("LATEST_B", "23:00"),
    ] {
        sqlx::query("INSERT INTO joy_settings (couple_id, key, value) VALUES (?, ?, ?)").bind(&couple).bind(k).bind(v).execute(&t.db).await.unwrap();
    }
    t.exec(&format!(
        "INSERT INTO joy_activities (couple_id, title, est_cost, setting, venue, area, lat, lng, duration, energy, hours, minutes, booking, book_ahead_days, cover, bring, station)
         VALUES ('{couple}', 'Coffee by the creek', 800, 'local', 'Creek Cafe', 'Versova', 19.135, 72.815, 'short', 'low', 'mon-sun 00:00-23:59', 90, 'always', 1, 'indoor', 'cash, a book', 'Versova')"
    ))
    .await;
    let cafe = t.id_of(&couple, "Coffee by the creek").await;

    let jar = t.get("/api/dates/jar", &neha).await.json;
    let whens = jar["whens"].as_array().unwrap();
    assert!(whens.len() >= 3, "a few dates to choose from: {whens:?}");
    assert_eq!(whens.iter().filter(|w| w["selected"] == true).count(), 1);
    assert_eq!(jar["froms"].as_array().unwrap().last().unwrap()["value"], "office");
    let when = whens.iter().find(|w| w["label"].as_str().unwrap().ends_with("evening")).expect("a weeknight")["value"].as_str().unwrap().to_string();

    let ok = ids(&t.post("/api/dates/match", &neha, json!({"when": when})).await.json);
    assert!(ok.contains(&cafe), "a café near the office fits an evening");
    let any = ids(&t.post("/api/dates/match", &neha, json!({"when": "any"})).await.json);
    assert!(any.len() >= ok.len());

    t.exec(&format!("UPDATE joy_activities SET archived = 1 WHERE couple_id = '{couple}' AND id != {cafe}")).await;
    let card = t.post("/api/dates/pick", &neha, json!({"when": when})).await.json;
    assert_eq!(drawn(&card), cafe);
    assert_eq!(card["when"], when);
    assert!(card["when_label"].as_str().unwrap().ends_with("evening"));
    assert!(!card["plan_line"].as_str().unwrap().is_empty() && !card["route"].as_array().unwrap().is_empty(), "{card}");

    let kept = t.post(&format!("/api/dates/accept/{cafe}"), &neha, json!({"when": when, "start": "office"})).await.json;
    assert_eq!(kept["done"], true);
    let plan = &kept["plan"];
    assert!(plan.is_object(), "a dated plan comes back: {kept}");
    assert_eq!(plan["sealed"], false);
    assert!(plan["ics_url"].as_str().unwrap().contains("invite.ics?who=1&exp="));
    assert!(!plan["steps"].as_array().unwrap().is_empty());
    let hid = kept["history_ids"].as_str().unwrap();
    assert_eq!(t.scalar(&format!("SELECT planned_at IS NOT NULL FROM joy_history WHERE id = {hid}")).await, 1);
    assert_eq!(t.text(&format!("SELECT day_kind FROM joy_history WHERE id = {hid}")).await, "weeknight");
    let tasks = plan["tasks"].as_array().unwrap();
    assert!(tasks.iter().any(|x| x["task"].as_str().unwrap().starts_with("Book ")), "{plan}");
    assert!(tasks.iter().any(|x| x["task"] == "Bring cash"));
    let mine = tasks.iter().filter(|x| x["mine"] == true && x["done"] == false).count();
    assert_eq!(plan["my_open_tasks"].as_u64().unwrap() as usize, mine);

    // Rohit's jar shows the next plan too, with his own side of the jobs.
    let r = t.get("/api/dates/jar", &rohit).await.json;
    assert_eq!(r["next_plan"]["history_id"].as_str().map(str::to_owned).unwrap_or_else(|| r["next_plan"]["history_id"].to_string()), hid);
    assert!(r["nudge"].as_str().unwrap().contains(": Coffee by the creek"), "{}", r["nudge"]);

    // A stale date on an old card plans nothing.
    let id2 = t.id_of(&couple, "Cook together").await;
    t.exec("UPDATE joy_activities SET archived = 0").await;
    t.post(&format!("/api/dates/accept/{id2}"), &neha, json!({"when": "2020-01-01"})).await;
    assert_eq!(t.scalar(&format!("SELECT planned_at IS NULL FROM joy_history WHERE activity_id = {id2}")).await, 1);
}

// ── other couples ─────────────────────────────────────────────────────────

#[tokio::test]
async fn another_couple_cannot_see_or_touch_these_slips() {
    let t = setup().await;
    let (rohit, _, couple) = t.couple().await;
    let (carol, other) = t.loner().await;
    let mine = t.id_of(&couple, "Cook together").await;
    let snapshot = |sql: &'static str| async { t.scalar(sql).await };
    let before = (
        snapshot("SELECT count(*) FROM joy_activities").await,
        snapshot("SELECT count(*) FROM joy_history").await,
        snapshot("SELECT count(*) FROM joy_vetoes").await,
    );

    // Their own lists never show ours.
    let page = t.get("/api/dates/slips", &carol).await.json;
    let seen: Vec<i64> = page["groups"].as_array().unwrap().iter().flat_map(|g| g["items"].as_array().unwrap()).map(|i| i["id"].as_i64().unwrap()).collect();
    assert!(!seen.is_empty() && !seen.contains(&mine));
    assert!(!ids(&t.post("/api/dates/match", &carol, json!({})).await.json).contains(&mine));

    // Reading or changing one of ours: not found.
    let body = json!({"title": "Hijacked", "setting": "local", "duration": "short", "energy": "low"});
    assert_eq!(t.get(&format!("/api/dates/slips/{mine}"), &carol).await.status, StatusCode::NOT_FOUND);
    assert_eq!(t.post(&format!("/api/dates/slips/{mine}"), &carol, body).await.status, StatusCode::NOT_FOUND);
    for action in ["archive", "restore", "delete", "unveto"] {
        assert_eq!(t.post(&format!("/api/dates/slips/{mine}/{action}"), &carol, json!({})).await.status, StatusCode::NOT_FOUND, "{action}");
    }
    assert_eq!(t.post(&format!("/api/dates/slips/{mine}/plan"), &carol, json!({"at": "2026-11-02T19:00"})).await.status, StatusCode::NOT_FOUND);
    assert_eq!(t.post(&format!("/api/dates/accept/{mine}"), &carol, json!({})).await.status, StatusCode::NOT_FOUND);

    // A card that names one of our slips as "current" can't put it back, pass on it or pair it.
    for form in [json!({"current": mine, "reroll": true, "used": 0}), json!({"current": mine, "veto": true})] {
        let r = t.post("/api/dates/pick", &carol, form).await;
        assert_eq!(r.status, StatusCode::OK);
        assert_ne!(r.json["activity"]["id"], mine);
    }
    let theirs = t.id_of(&other, "Dinner at Saltt").await;
    t.post(&format!("/api/dates/accept/{theirs}"), &carol, json!({"with": mine})).await;

    let after = (
        snapshot("SELECT count(*) FROM joy_activities").await,
        snapshot("SELECT count(*) FROM joy_history").await,
        snapshot("SELECT count(*) FROM joy_vetoes").await,
    );
    assert_eq!(after.0, before.0, "nothing deleted or added");
    assert_eq!(after.1, before.1 + 1, "only their own accept");
    assert_eq!(after.2, before.2, "no veto reached our slip");
    assert_eq!(t.text(&format!("SELECT title FROM joy_activities WHERE id = {mine}")).await, "Cook together");
    assert_eq!(t.scalar(&format!("SELECT archived FROM joy_activities WHERE id = {mine}")).await, 0);
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_history WHERE activity_id = {mine}")).await, 0);

    // And they still work for us.
    assert_eq!(t.get(&format!("/api/dates/slips/{mine}"), &rohit).await.status, StatusCode::OK);
}
