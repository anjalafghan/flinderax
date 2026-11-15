//! `Idempotency-Key` on a POST: do it once, answer the same every time. Applies to every
//! authenticated route, so any money-changing request can be retried safely (an offline queue, a
//! double tap, a phone that never heard back). Only successful answers are remembered, so a refused
//! request can be fixed and sent again under the same key. Keys are per person and forgotten after a week.

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{handlers::common::AppError, models::AppState};

const MAX_KEY: usize = 100;
const MAX_STORED_BODY: usize = 256 * 1024;
const KEEP_DAYS: i64 = 7;

pub async fn layer(State(state): State<AppState>, request: Request, next: Next) -> Result<Response, AppError> {
    let key = request.headers().get("idempotency-key").and_then(|v| v.to_str().ok()).map(str::trim).filter(|k| !k.is_empty()).map(String::from);
    let user = request.extensions().get::<(String, String)>().map(|(u, _)| u.clone());
    let (Some(key), Some(user), true) = (key, user, request.method() == Method::POST) else {
        return Ok(next.run(request).await);
    };
    if key.len() > MAX_KEY {
        return Err(AppError(StatusCode::BAD_REQUEST, "Idempotency-Key is too long".into()));
    }

    sqlx::query("DELETE FROM idempotency_keys WHERE created_at < datetime('now', ?)").bind(format!("-{KEEP_DAYS} days")).execute(&state.db).await?;
    let claimed = sqlx::query("INSERT OR IGNORE INTO idempotency_keys (user_id, key) VALUES (?, ?)").bind(&user).bind(&key).execute(&state.db).await?.rows_affected() == 1;
    if !claimed {
        let (status, body): (i64, String) = sqlx::query_as("SELECT status, body FROM idempotency_keys WHERE user_id = ? AND key = ?").bind(&user).bind(&key).fetch_one(&state.db).await?;
        if status == 0 {
            return Err(AppError(StatusCode::CONFLICT, "That request is still being handled. Try again in a moment.".into()));
        }
        let mut res = (StatusCode::from_u16(status as u16).unwrap_or(StatusCode::OK), [(axum::http::header::CONTENT_TYPE, "application/json")], body).into_response();
        res.headers_mut().insert("idempotent-replay", HeaderValue::from_static("true"));
        return Ok(res);
    }

    let response = next.run(request).await;
    if response.status().is_success() {
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, MAX_STORED_BODY).await.map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "response too large to keep".into()))?;
        sqlx::query("UPDATE idempotency_keys SET status = ?, body = ? WHERE user_id = ? AND key = ?")
            .bind(i64::from(parts.status.as_u16()))
            .bind(String::from_utf8_lossy(&bytes).into_owned())
            .bind(&user)
            .bind(&key)
            .execute(&state.db)
            .await?;
        return Ok(Response::from_parts(parts, Body::from(bytes)));
    }
    // Not remembered: let the same key try again once the problem is fixed.
    sqlx::query("DELETE FROM idempotency_keys WHERE user_id = ? AND key = ?").bind(&user).bind(&key).execute(&state.db).await?;
    Ok(response)
}
