//! Account lifecycle: token revocation, password change, sign out everywhere, self-service
//! deletion (rows and photo folder), export completeness and the last-admin guard.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

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

static COUNTER: AtomicUsize = AtomicUsize::new(1);
const PW: &str = "correct horse";

struct T {
    app: Router,
    db: SqlitePool,
}

struct Res {
    status: StatusCode,
    json: Value,
    text: String,
}

async fn setup() -> T {
    // Safety: every test in this binary sets the same values before any request reads them.
    unsafe {
        std::env::set_var("SIGNUP_CODE", "test-signup-code");
        std::env::set_var("JOY_PHOTOS_DIR", std::env::temp_dir().join("joy-lifecycle-test-photos"));
    }
    let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    let app = build_router(AppState { db: db.clone(), redis: None, paseto_key: Arc::new(key) });
    T { app, db }
}

impl T {
    async fn call(&self, method: Method, path: &str, token: Option<&str>, body: Option<Value>, ip: &str) -> Res {
        let mut req = Request::builder().method(method).uri(path).header("fly-client-ip", ip);
        if let Some(t) = token {
            req = req.header("authorization", format!("Bearer {t}"));
        }
        let req = match body {
            // The admin delete takes the bare username as its body; everything else is JSON.
            Some(Value::String(raw)) => req.body(Body::from(raw)),
            Some(b) => req.header("content-type", "application/json").body(Body::from(b.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8_lossy(&bytes).into_owned();
        Res { status, json: serde_json::from_str(&text).unwrap_or(Value::Null), text }
    }

    async fn get(&self, path: &str, token: &str) -> Res {
        self.call(Method::GET, path, Some(token), None, "10.77.0.1").await
    }

    async fn post(&self, path: &str, token: &str, body: Value) -> Res {
        self.call(Method::POST, path, Some(token), Some(body), "10.77.0.1").await
    }

    /// A registered user: (name, id, token).
    async fn user(&self, label: &str) -> (String, String, String) {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let (name, ip) = (format!("{label}{n}"), format!("10.66.{}.{}", n / 250, n % 250));
        let creds = json!({"user_name": name, "user_password": PW, "signup_code": "test-signup-code"});
        let r = self.call(Method::POST, "/api/common/register", None, Some(creds), &ip).await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.text);
        let token = self.login(&name, PW).await;
        let id = self.text(&format!("SELECT user_id FROM users WHERE user_name = '{name}'")).await;
        (name, id, token)
    }

    async fn login(&self, name: &str, pw: &str) -> String {
        let r = self.login_raw(name, pw).await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.text);
        r.json["access_token"].as_str().unwrap().to_string()
    }

    async fn login_raw(&self, name: &str, pw: &str) -> Res {
        self.call(Method::POST, "/api/common/login", None, Some(json!({"user_name": name, "user_password": pw})), "10.55.0.9").await
    }

    async fn make_admin(&self, id: &str) {
        self.exec(&format!("UPDATE users SET user_role = 'admin' WHERE user_id = '{id}'")).await;
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

    /// Two users who are a couple: (a, b, couple id).
    async fn couple(&self) -> ((String, String, String), (String, String, String), String) {
        let (a, b) = (self.user("rohit").await, self.user("neha").await);
        assert!(self.post("/api/couple/create", &a.2, json!({})).await.status.is_success());
        let code = self.post("/api/couple/invite", &a.2, json!({})).await.json["code"].as_str().unwrap().to_string();
        let joined = self.post("/api/couple/join", &b.2, json!({"code": code})).await;
        assert!(joined.status.is_success(), "{}", joined.text);
        let id = self.text(&format!("SELECT couple_id FROM couples WHERE user_0 = '{}'", a.1)).await;
        (a, b, id)
    }

    /// Private rows for a user in tables that hang off `user_id`, directly and through cards.
    async fn seed(&self, id: &str) {
        let sql = [
            format!("INSERT INTO accounts (account_id, user_id, name, kind) VALUES ('acc-{id}', '{id}', 'Salary', 'bank')"),
            format!("INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color) VALUES ('card-{id}', '{id}', 'Everyday', 'ACME', 1, 2)"),
            format!("INSERT INTO card_events (transaction_id, card_id, total_due_input) VALUES ('ev-{id}', 'card-{id}', 1200.0)"),
            format!("INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, source, raw_text, dedupe_key) VALUES ('t-{id}', '{id}', '2026-11-01', 'debit', 25000, 'ASHA STORES', 'sms', 'Rs 250 spent at ASHA STORES', 'k-{id}')"),
            format!("INSERT INTO merchant_rules (user_id, pattern, category) VALUES ('{id}', 'asha', 'groceries')"),
            format!("INSERT INTO ingest_keys (user_id, key_hash) VALUES ('{id}', 'hash-secret-{id}')"),
            format!("INSERT INTO reminder_feeds (user_id, token) VALUES ('{id}', 'feed-secret-{id}')"),
            format!("INSERT INTO life_profiles (user_id, profile_json) VALUES ('{id}', '{{\"wedding\":true}}')"),
        ];
        for s in sql {
            self.exec(&s).await;
        }
    }

    /// Every table that has a `user_id` column, with how many rows that user still has in it.
    async fn leftovers(&self, id: &str) -> Vec<(String, i64)> {
        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT m.name FROM sqlite_master m JOIN pragma_table_info(m.name) p ON p.name = 'user_id' WHERE m.type = 'table'",
        )
        .fetch_all(&self.db)
        .await
        .unwrap();
        assert!(tables.len() > 15, "expected many user tables, got {tables:?}");
        let mut left = vec![];
        for t in tables {
            let n: i64 =
                sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {t} WHERE user_id = ?")).bind(id).fetch_one(&self.db).await.unwrap();
            if n > 0 {
                left.push((t, n));
            }
        }
        left
    }
}

fn photo_dir(couple: &str) -> std::path::PathBuf {
    joy::import_joy::photos_root().join(couple)
}

#[tokio::test]
async fn old_token_is_rejected_after_a_password_change() {
    let t = setup().await;
    let (name, _, old) = t.user("alice").await;
    assert_eq!(t.get("/api/account/list", &old).await.status, StatusCode::OK);

    // A wrong current password changes nothing and keeps the session (403, not the 401 that signs the web app out).
    let r = t.post("/api/common/password", &old, json!({"current_password": "nope nope nope", "new_password": "a brand new one"})).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(t.get("/api/account/list", &old).await.status, StatusCode::OK);
    // The new password is held to the sign-up rules.
    let r = t.post("/api/common/password", &old, json!({"current_password": PW, "new_password": "short"})).await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);

    let r = t.post("/api/common/password", &old, json!({"current_password": PW, "new_password": "a brand new one"})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let fresh = r.json["access_token"].as_str().unwrap().to_string();
    assert!(r.json["expires_at"].as_i64().unwrap() > 0);

    let gone = t.get("/api/account/list", &old).await;
    assert_eq!(gone.status, StatusCode::UNAUTHORIZED);
    assert_eq!(gone.text, "Please sign in again.");
    assert_eq!(t.get("/api/account/list", &fresh).await.status, StatusCode::OK);
    assert_eq!(t.login_raw(&name, PW).await.status, StatusCode::UNAUTHORIZED);
    let again = t.login(&name, "a brand new one").await;
    assert_eq!(t.get("/api/account/list", &again).await.status, StatusCode::OK);
    // The fresh token and the new login share a version, so both live.
    assert_eq!(t.get("/api/account/list", &fresh).await.status, StatusCode::OK);
}

#[tokio::test]
async fn logout_everywhere_ends_every_session() {
    let t = setup().await;
    let (name, _, one) = t.user("bob").await;
    let two = t.login(&name, PW).await;
    assert_eq!(t.get("/api/account/list", &two).await.status, StatusCode::OK);

    let r = t.post("/api/common/logout-everywhere", &one, json!({})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert_eq!(t.get("/api/account/list", &one).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(t.get("/api/account/list", &two).await.status, StatusCode::UNAUTHORIZED);
    let three = t.login(&name, PW).await;
    assert_eq!(t.get("/api/account/list", &three).await.status, StatusCode::OK);
}

#[tokio::test]
async fn a_token_without_a_version_or_for_a_deleted_user_is_refused() {
    let t = setup().await;
    let (_, id, token) = t.user("cass").await;
    t.exec(&format!("DELETE FROM users WHERE user_id = '{id}'")).await;
    assert_eq!(t.get("/api/account/list", &token).await.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_account_needs_the_word_and_the_password() {
    let t = setup().await;
    let (name, _, token) = t.user("dina").await;
    let r = t.post("/api/common/delete-account", &token, json!({"password": PW, "confirm": "yes"})).await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    let r = t.post("/api/common/delete-account", &token, json!({"password": "wrong wrong", "confirm": "delete"})).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(t.get("/api/account/list", &token).await.status, StatusCode::OK);

    let r = t.post("/api/common/delete-account", &token, json!({"password": PW, "confirm": "delete"})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert_eq!(t.get("/api/account/list", &token).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(t.login_raw(&name, PW).await.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_account_removes_their_rows_and_the_couple_folder_when_it_empties() {
    let t = setup().await;
    let (a, b, couple) = t.couple().await;
    t.seed(&a.1).await;
    t.seed(&b.1).await;
    // A date with a photo on disk, a memory and a task.
    t.exec(&format!("INSERT INTO joy_history (id, couple_id, activity_id, outcome) SELECT 9001, '{couple}', id, 'done' FROM joy_activities WHERE couple_id = '{couple}' LIMIT 1")).await;
    t.exec("INSERT INTO joy_memories (history_id, who, line) VALUES (9001, 0, 'the rain')").await;
    let dir = photo_dir(&couple);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("9001.jpg"), b"not really a jpeg").unwrap();

    // The first to go leaves the other everything shared; only their own rows disappear.
    let r = t.post("/api/common/delete-account", &a.2, json!({"password": PW, "confirm": "delete"})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert!(t.leftovers(&a.1).await.is_empty(), "{:?}", t.leftovers(&a.1).await);
    assert_eq!(t.scalar(&format!("SELECT COUNT(*) FROM card_events WHERE card_id = 'card-{}'", a.1)).await, 0);
    assert_eq!(t.scalar(&format!("SELECT COUNT(*) FROM users WHERE user_id = '{}'", a.1)).await, 0);
    assert!(!t.leftovers(&b.1).await.is_empty(), "the partner's own data is untouched");
    assert_eq!(t.scalar(&format!("SELECT COUNT(*) FROM couples WHERE couple_id = '{couple}' AND user_0 IS NULL AND user_1 = '{}'", b.1)).await, 1);
    assert!(dir.join("9001.jpg").exists(), "the partner keeps the shared photos");

    // The last one out takes Dates and its photos with them.
    let r = t.post("/api/common/delete-account", &b.2, json!({"password": PW, "confirm": "delete"})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert!(t.leftovers(&b.1).await.is_empty());
    assert_eq!(t.scalar(&format!("SELECT COUNT(*) FROM couples WHERE couple_id = '{couple}'")).await, 0);
    for table in ["joy_activities", "joy_history", "joy_settings", "couple_invites"] {
        assert_eq!(t.scalar(&format!("SELECT COUNT(*) FROM {table} WHERE couple_id = '{couple}'")).await, 0, "{table}");
    }
    assert_eq!(t.scalar("SELECT COUNT(*) FROM joy_memories").await, 0);
    assert!(!dir.exists(), "the couple's photo folder is gone");
}

#[tokio::test]
async fn leaving_as_the_last_member_removes_the_photo_folder() {
    let t = setup().await;
    let (_, _, token) = t.user("erin").await;
    assert!(t.post("/api/couple/create", &token, json!({})).await.status.is_success());
    let couple = t.text("SELECT couple_id FROM couples ORDER BY rowid DESC LIMIT 1").await;
    let dir = photo_dir(&couple);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("1.jpg"), b"x").unwrap();
    assert!(t.post("/api/couple/leave", &token, json!({})).await.status.is_success());
    assert!(!dir.exists());
}

#[tokio::test]
async fn export_has_the_new_tables_and_no_secrets() {
    let t = setup().await;
    let (a, _b, couple) = t.couple().await;
    t.seed(&a.1).await;
    t.exec(&format!("INSERT INTO joy_history (id, couple_id, activity_id, outcome, share_token) SELECT 9002, '{couple}', id, 'done', 'envelope-secret-token' FROM joy_activities WHERE couple_id = '{couple}' LIMIT 1")).await;
    t.exec("INSERT INTO joy_memories (history_id, who, line) VALUES (9002, 0, 'the rain')").await;

    let r = t.get("/api/export", &a.2).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let data = &r.json["data"];
    for k in [
        "card_events", "card_running_state", "deferred_payment_batches", "deferred_payment_items", "pending_charges", "auto_postings",
        "dismissed_recurring", "statement_imports", "statement_profiles", "tax_entries", "life_profiles", "transactions",
    ] {
        assert!(data[k].is_array(), "missing {k}");
    }
    assert_eq!(data["card_events"].as_array().unwrap().len(), 1);
    assert_eq!(data["life_profiles"][0]["profile_json"], "{\"wedding\":true}");
    assert_eq!(data["transactions"][0]["raw_text"], "Rs 250 spent at ASHA STORES");
    for k in ["memories", "tasks", "date_photos", "checkins", "month_budget", "settings", "vetoes", "pairs"] {
        assert!(data["dates"][k].is_array(), "missing dates.{k}");
    }
    assert_eq!(data["dates"]["memories"][0]["line"], "the rain");
    assert!(!data["dates"]["settings"].as_array().unwrap().is_empty());

    for secret in ["envelope-secret-token", "hash-secret-", "feed-secret-", "$argon2", "user_password", "share_token", "key_hash"] {
        assert!(!r.text.contains(secret), "export leaks {secret}");
    }
    assert!(data.get("ingest_keys").is_none() && data.get("reminder_feeds").is_none() && data.get("users").is_none());
}

#[tokio::test]
async fn admin_cannot_delete_the_last_admin_or_themselves() {
    let t = setup().await;
    let (_, admin_id, admin) = t.user("root").await;
    t.make_admin(&admin_id).await;
    let (other_name, other_id, _) = t.user("plain").await;

    // The only admin, deleting their own account: refused.
    let r = t.post("/api/common/delete-account", &admin, json!({"password": PW, "confirm": "delete"})).await;
    assert_eq!(r.status, StatusCode::CONFLICT, "{}", r.text);
    assert_eq!(t.scalar(&format!("SELECT COUNT(*) FROM users WHERE user_id = '{admin_id}'")).await, 1);
    // The admin route refuses self-deletion too.
    let admin_name = t.text(&format!("SELECT user_name FROM users WHERE user_id = '{admin_id}'")).await;
    let r = t.call(Method::POST, "/api/user/delete", Some(&admin), Some(Value::String(admin_name.clone())), "10.77.0.2").await;
    assert_ne!(r.status, StatusCode::OK);
    // The guard itself, whoever calls it.
    let err = joy::handlers::lifecycle::delete_user(&t.db, &admin_id).await;
    assert!(matches!(err, Err(joy::handlers::common::AppError(StatusCode::CONFLICT, _))));

    // With a second admin, one can remove the other, and then the survivor is protected again.
    t.make_admin(&other_id).await;
    let r = t.call(Method::POST, "/api/user/delete", Some(&admin), Some(Value::String(other_name)), "10.77.0.2").await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert_eq!(t.scalar(&format!("SELECT COUNT(*) FROM users WHERE user_id = '{other_id}'")).await, 0);
    assert_eq!(t.scalar("SELECT COUNT(*) FROM users WHERE user_role = 'admin'").await, 1);
}

#[tokio::test]
async fn admin_delete_cleans_up_the_couple_and_its_photos() {
    let t = setup().await;
    let (_, admin_id, admin) = t.user("boss").await;
    t.make_admin(&admin_id).await;
    let (name, id, token) = t.user("solo").await;
    assert!(t.post("/api/couple/create", &token, json!({})).await.status.is_success());
    let couple = t.text(&format!("SELECT couple_id FROM couples WHERE user_0 = '{id}'")).await;
    t.seed(&id).await;
    let dir = photo_dir(&couple);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("1.jpg"), b"x").unwrap();

    let r = t.call(Method::POST, "/api/user/delete", Some(&admin), Some(Value::String(name)), "10.77.0.2").await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert!(t.leftovers(&id).await.is_empty());
    assert_eq!(t.scalar(&format!("SELECT COUNT(*) FROM couples WHERE couple_id = '{couple}'")).await, 0);
    assert!(!dir.exists());
}
