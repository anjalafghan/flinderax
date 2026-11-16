//! Notifications: turn them on or off for this browser, and the scheduled round that sends them.

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Extension, Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    handlers::common::{bad_request, today_local, AppError},
    models::AppState,
    push::{run, Report, Slot, WebPushSender},
};

type User = Extension<(String, String)>;

#[derive(Serialize)]
pub struct Status {
    /// What the browser needs to subscribe; none when notifications are not set up on the server.
    pub public_key: Option<String>,
    pub subscribed: bool,
}

/// `GET /api/push`
pub async fn status(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Status>, AppError> {
    let subscribed: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM push_subscriptions WHERE user_id = ?)").bind(&user_id).fetch_one(&state.db).await?;
    Ok(Json(Status { public_key: WebPushSender::from_env().map(|s| s.public_key()), subscribed }))
}

#[derive(Deserialize)]
pub struct Subscribe {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

fn key_ok(k: &str) -> bool {
    !k.is_empty() && k.len() <= 200 && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '=')
}

/// `POST /api/push/subscribe`: this browser wants the nudges. Sending again is harmless.
pub async fn subscribe(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<Subscribe>) -> Result<Json<Done>, AppError> {
    if !p.endpoint.starts_with("https://") || p.endpoint.len() > 600 || p.endpoint.contains(char::is_whitespace) {
        return Err(bad_request("That doesn't look like a push address"));
    }
    if !key_ok(&p.p256dh) || !key_ok(&p.auth) {
        return Err(bad_request("The browser's keys look wrong"));
    }
    sqlx::query(
        "INSERT INTO push_subscriptions (user_id, endpoint, p256dh, auth) VALUES (?, ?, ?, ?)
         ON CONFLICT(endpoint) DO UPDATE SET user_id = excluded.user_id, p256dh = excluded.p256dh, auth = excluded.auth",
    )
    .bind(&user_id)
    .bind(&p.endpoint)
    .bind(&p.p256dh)
    .bind(&p.auth)
    .execute(&state.db)
    .await?;
    Ok(Json(Done { status: true }))
}

#[derive(Deserialize)]
pub struct Unsubscribe {
    pub endpoint: String,
}

/// `POST /api/push/unsubscribe`: only your own.
pub async fn unsubscribe(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<Unsubscribe>) -> Result<Json<Done>, AppError> {
    sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ? AND user_id = ?").bind(&p.endpoint).bind(&user_id).execute(&state.db).await?;
    Ok(Json(Done { status: true }))
}

#[derive(Deserialize)]
pub struct RunQuery {
    pub slot: String,
}

fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// `POST /api/internal/push/run?slot=morning|evening`: called by a scheduled job with
/// `Authorization: Bearer $PUSH_RUN_TOKEN`. Waking the machine for it is the point.
pub async fn run_round(State(state): State<AppState>, headers: HeaderMap, Query(q): Query<RunQuery>) -> Result<Json<Report>, AppError> {
    let Some(token) = std::env::var("PUSH_RUN_TOKEN").ok().filter(|t| t.len() >= 8) else {
        return Err(AppError(StatusCode::NOT_FOUND, "Not found".into()));
    };
    let given = headers.get("authorization").and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer ")).unwrap_or("");
    if !same(given.as_bytes(), token.as_bytes()) {
        return Err(AppError(StatusCode::UNAUTHORIZED, "Not allowed".into()));
    }
    let slot = Slot::parse(&q.slot).ok_or_else(|| bad_request("slot must be morning, evening or weekly"))?;
    let Some(sender) = WebPushSender::from_env() else {
        return Err(AppError(StatusCode::SERVICE_UNAVAILABLE, "Notifications are not set up on this server".into()));
    };
    Ok(Json(run(&state.db, &sender, slot, today_local()).await?))
}
