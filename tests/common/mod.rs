//! Shared harness for integration tests: an in-memory migrated app, sign-up/sign-in (bearer token
//! or the web `joy_session` cookie) and JSON request helpers.
//!
//! Use it from a test file with `mod common;`. Existing test files keep their own copies; new ones
//! should use this. Items a given test binary does not use are allowed to be dead.
#![allow(dead_code)]

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

pub const SIGNUP_CODE: &str = "test-signup-code";
pub const PASSWORD: &str = "correct horse";

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// A migrated in-memory database with the router on top of it.
pub struct TestApp {
    pub app: Router,
    pub db: SqlitePool,
}

/// What came back from a request.
pub struct Res {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub json: Value,
    pub text: String,
}

/// A signed-in user: both credentials the API accepts.
pub struct Session {
    pub user_name: String,
    /// Bearer token (for scripts).
    pub token: String,
    /// `joy_session=<token>` as a `Cookie` header value (for the web app).
    pub cookie: String,
}

impl TestApp {
    pub async fn new() -> Self {
        // Sign-ups need the code from the SIGNUP_CODE secret; the tests use a fixed one.
        // Safety: every test in a binary sets the same value before any request reads it.
        unsafe { std::env::set_var("SIGNUP_CODE", SIGNUP_CODE) };
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
        let app = build_router(AppState { db: db.clone(), paseto_key: Arc::new(key) });
        TestApp { app, db }
    }

    /// Send a request; `auth` is a bearer token. Every call gets a fixed client address.
    pub async fn call(&self, method: Method, path: &str, auth: Option<&str>, body: Option<Value>) -> Res {
        let mut req = Request::builder().method(method).uri(path).header("fly-client-ip", "10.99.0.1");
        if let Some(t) = auth {
            req = req.header("authorization", format!("Bearer {t}"));
        }
        self.send(req, body).await
    }

    /// Like `call`, but authenticated the way the web app is: by cookie, with `X-Joy: 1` so unsafe
    /// methods are accepted.
    pub async fn call_cookie(&self, method: Method, path: &str, cookie: &str, body: Option<Value>) -> Res {
        let req = Request::builder()
            .method(method)
            .uri(path)
            .header("fly-client-ip", "10.99.0.1")
            .header("cookie", cookie)
            .header("x-joy", "1");
        self.send(req, body).await
    }

    async fn send(&self, req: axum::http::request::Builder, body: Option<Value>) -> Res {
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

    pub async fn get(&self, path: &str, token: &str) -> Res {
        self.call(Method::GET, path, Some(token), None).await
    }

    pub async fn post(&self, path: &str, token: &str, body: Value) -> Res {
        self.call(Method::POST, path, Some(token), Some(body)).await
    }

    /// Register and sign in a fresh user (each from its own client address: sign-ups are
    /// rate-limited per address).
    pub async fn session(&self, label: &str) -> Session {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let (name, ip) = (format!("{label}{n}"), format!("10.88.{}.{}", n / 250, n % 250));
        let creds = json!({"user_name": name, "user_password": PASSWORD, "signup_code": SIGNUP_CODE});
        let mut token = String::new();
        for path in ["/api/common/register", "/api/common/login"] {
            let req = Request::post(path)
                .header("content-type", "application/json")
                .header("fly-client-ip", &ip)
                .body(Body::from(creds.to_string()))
                .unwrap();
            let res = self.app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK, "{path}");
            if path.ends_with("login") {
                let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
                token = serde_json::from_slice::<Value>(&bytes).unwrap()["access_token"].as_str().unwrap().to_string();
            }
        }
        Session { user_name: name, cookie: format!("joy_session={token}"), token }
    }

    /// Just the bearer token of a fresh user.
    pub async fn user(&self, label: &str) -> String {
        self.session(label).await.token
    }

    pub async fn exec(&self, sql: &str) {
        sqlx::query(sql).execute(&self.db).await.unwrap();
    }

    pub async fn scalar(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.db).await.unwrap()
    }

    pub async fn text(&self, sql: &str) -> String {
        sqlx::query_scalar(sql).fetch_one(&self.db).await.unwrap()
    }
}
