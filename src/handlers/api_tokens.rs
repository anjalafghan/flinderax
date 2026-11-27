use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    app::AuthContext,
    handlers::common::AppError,
    models::AppState,
};

const ALLOWED_SCOPES: &[&str] = &[
    "money:read",
    "money:write",
    "dates:read",
    "dates:write",
    "inbox:write",
    "analysis:write",
];

#[derive(Deserialize)]
pub struct CreateToken {
    pub name: String,
    pub scopes: Vec<String>,
    /// Optional lifetime in days, from 1 to 365. None means no expiry.
    pub expires_in_days: Option<i64>,
}

#[derive(Serialize)]
pub struct TokenView {
    id: String,
    name: String,
    scopes: Vec<String>,
    created_at: i64,
    last_used_at: Option<i64>,
    expires_at: Option<i64>,
    revoked_at: Option<i64>,
}

#[derive(Serialize)]
pub struct CreatedToken {
    token: String,
    id: String,
    name: String,
    scopes: Vec<String>,
    created_at: i64,
    expires_at: Option<i64>,
}

fn session_only(auth: &AuthContext) -> Result<(), AppError> {
    if auth.token_id.is_some() {
        return Err(AppError(StatusCode::FORBIDDEN, "API tokens cannot manage API tokens.".into()));
    }
    Ok(())
}

pub fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

pub async fn create(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Extension(auth): Extension<AuthContext>,
    Json(body): Json<CreateToken>,
) -> Result<(StatusCode, Json<CreatedToken>), AppError> {
    session_only(&auth)?;
    let name = body.name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(AppError(StatusCode::BAD_REQUEST, "Name must be 1 to 80 characters.".into()));
    }
    if body.scopes.is_empty() || body.scopes.iter().any(|s| !ALLOWED_SCOPES.contains(&s.as_str())) {
        return Err(AppError(StatusCode::BAD_REQUEST, "Choose one or more supported scopes.".into()));
    }
    if body.scopes.iter().collect::<std::collections::HashSet<_>>().len() != body.scopes.len() {
        return Err(AppError(StatusCode::BAD_REQUEST, "Scopes must be unique.".into()));
    }
    if body.expires_in_days.is_some_and(|d| !(1..=365).contains(&d)) {
        return Err(AppError(StatusCode::BAD_REQUEST, "Expiry must be between 1 and 365 days.".into()));
    }

    let id = nanoid!(16);
    let token = format!("joy_{}", nanoid!(48));
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let expires_at = body.expires_in_days.map(|days| now + days * 86_400);
    let scopes_json = serde_json::to_string(&body.scopes).map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query("INSERT INTO api_tokens (id, user_id, name, token_hash, scopes, expires_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&user_id)
        .bind(name)
        .bind(hash_token(&token))
        .bind(scopes_json)
        .bind(expires_at)
        .execute(&state.db)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(CreatedToken {
        token,
        id,
        name: name.to_string(),
        scopes: body.scopes,
        created_at: now,
        expires_at,
    })))
}

pub async fn list(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<TokenView>>, AppError> {
    session_only(&auth)?;
    let rows: Vec<(String, String, String, i64, Option<i64>, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT id, name, scopes, created_at, last_used_at, expires_at, revoked_at
         FROM api_tokens WHERE user_id = ? ORDER BY created_at DESC, id",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    let tokens = rows.into_iter().map(|(id, name, scopes, created_at, last_used_at, expires_at, revoked_at)| {
        TokenView { id, name, scopes: serde_json::from_str(&scopes).unwrap_or_default(), created_at, last_used_at, expires_at, revoked_at }
    }).collect();
    Ok(Json(tokens))
}

pub async fn revoke(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    session_only(&auth)?;
    let done = sqlx::query("UPDATE api_tokens SET revoked_at = unixepoch() WHERE id = ? AND user_id = ? AND revoked_at IS NULL")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    if done.rows_affected() == 0 {
        return Err(AppError(StatusCode::NOT_FOUND, "Token not found.".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct AuditView {
    id: i64,
    token_id: String,
    method: String,
    route: String,
    status: i64,
    created_at: i64,
}

pub async fn audit(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<AuditView>>, AppError> {
    session_only(&auth)?;
    let rows: Vec<(i64, String, String, String, i64, i64)> = sqlx::query_as(
        "SELECT id, token_id, method, route, status, created_at FROM audit_log
         WHERE user_id = ? ORDER BY created_at DESC, id DESC LIMIT 200",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows.into_iter().map(|(id, token_id, method, route, status, created_at)| AuditView { id, token_id, method, route, status, created_at }).collect()))
}

#[derive(Serialize)]
pub struct TokenInfo {
    pub id: String,
    pub scopes: Vec<String>,
}

pub async fn token_info(Extension(auth): Extension<AuthContext>) -> Result<Json<TokenInfo>, AppError> {
    let id = auth.token_id.ok_or_else(|| AppError(StatusCode::FORBIDDEN, "A scoped API token is required.".into()))?;
    Ok(Json(TokenInfo { id, scopes: auth.scopes }))
}
