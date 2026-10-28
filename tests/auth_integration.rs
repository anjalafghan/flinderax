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
