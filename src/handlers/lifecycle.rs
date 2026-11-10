//! The life of an account: change the password, end every session, delete everything.

use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::{
    handlers::{
        common::{bad_request, get_password_hash, get_paseto_token, validate_password, verify_user, AppError},
        couple::{leave_in_tx, remove_couple_photos},
    },
    models::{AppState, LoginResponse},
    ratelimit,
};

type User = Extension<(String, String)>;

/// Wrong-password tries per account per minute on the routes below.
const TRIES_PER_MINUTE: u32 = 10;

#[derive(Deserialize)]
pub struct PasswordPayload {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct DeletePayload {
    pub password: String,
    pub confirm: String,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

/// A wrong password here is 403, not 401: the web app treats a 401 as "your session ended".
fn wrong_password() -> AppError {
    AppError(StatusCode::FORBIDDEN, "That isn't your current password.".into())
}

/// Checks `password` against the stored hash, with a per-account limit on wrong tries.
async fn check_password(state: &AppState, user_id: &str, password: String) -> Result<(), AppError> {
    let key = format!("pw-check:{user_id}");
    if !ratelimit::under(&key, TRIES_PER_MINUTE) {
        return Err(AppError(StatusCode::TOO_MANY_REQUESTS, "Too many attempts, try again in a minute".into()));
    }
    // Nothing longer than a real password reaches Argon2.
    if password.chars().count() > 128 {
        ratelimit::hit(&key);
        return Err(wrong_password());
    }
    let hash: String = sqlx::query_scalar("SELECT user_password FROM users WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError(StatusCode::UNAUTHORIZED, "Please sign in again.".into()))?;
    let ok = tokio::task::spawn_blocking(move || verify_user(hash, password).is_ok())
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if ok {
        Ok(())
    } else {
        ratelimit::hit(&key);
        Err(wrong_password())
    }
}

/// `POST /api/common/password`: new password, every other session ends, and this one gets a fresh token.
pub async fn change_password(
    State(state): State<AppState>,
    Extension((user_id, role)): User,
    Json(p): Json<PasswordPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    check_password(&state, &user_id, p.current_password).await?;
    validate_password(&p.new_password)?;
    let new_hash = get_password_hash(p.new_password)?;
    let version: i64 = sqlx::query_scalar(
        "UPDATE users SET user_password = ?, token_version = token_version + 1 WHERE user_id = ? RETURNING token_version",
    )
    .bind(new_hash)
    .bind(&user_id)
    .fetch_one(&state.db)
    .await?;
    fresh_token(&state, &user_id, role, version)
}

/// `POST /api/common/logout-everywhere`: every token issued so far stops working, this one included.
pub async fn logout_everywhere(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Done>, AppError> {
    sqlx::query("UPDATE users SET token_version = token_version + 1 WHERE user_id = ?")
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(Done { status: true }))
}

fn fresh_token(state: &AppState, user_id: &str, role: String, version: i64) -> Result<Json<LoginResponse>, AppError> {
    let expiration = OffsetDateTime::now_utc() + Duration::hours(24);
    let token = get_paseto_token(user_id, role, version, &state.paseto_key, expiration)?;
    Ok(Json(LoginResponse { access_token: token, expires_at: expiration.unix_timestamp() }))
}

/// `POST /api/common/delete-account`: the caller's own account, after typing "delete" and their password.
pub async fn delete_account(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<DeletePayload>,
) -> Result<Json<Done>, AppError> {
    if p.confirm.trim() != "delete" {
        return Err(bad_request("Type delete to confirm."));
    }
    check_password(&state, &user_id, p.password).await?;
    delete_user(&state.db, &user_id).await?;
    Ok(Json(Done { status: true }))
}

/// Deletes a user and everything that is theirs. Per-user tables go through `ON DELETE CASCADE`.
/// A couple they are in is left first (the partner keeps the shared history); when the couple is
/// then empty its Dates data goes with it and so does its photo folder on disk. The last admin
/// can't be deleted. A user who doesn't exist is not an error.
pub async fn delete_user(db: &sqlx::SqlitePool, user_id: &str) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    let role: Option<String> =
        sqlx::query_scalar("SELECT user_role FROM users WHERE user_id = ?").bind(user_id).fetch_optional(&mut *tx).await?;
    let Some(role) = role else { return Ok(()) };
    if role == "admin" {
        let admins: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE user_role = 'admin'").fetch_one(&mut *tx).await?;
        if admins <= 1 {
            return Err(AppError(StatusCode::CONFLICT, "That is the last admin account, so it can't be deleted.".into()));
        }
    }
    let emptied = leave_in_tx(&mut tx, user_id).await?.flatten();
    sqlx::query("DELETE FROM users WHERE user_id = ?").bind(user_id).execute(&mut *tx).await?;
    tx.commit().await?;
    if let Some(couple) = emptied {
        remove_couple_photos(&couple).await;
    }
    Ok(())
}
