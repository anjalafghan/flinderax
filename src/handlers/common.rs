use crate::models::{AppState, CreateUserPayload, CreateUserResponse, LoginPayload, LoginResponse};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use nanoid::nanoid;
use rusty_paseto::prelude::*;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use tracing::error;

pub struct AppError(pub StatusCode, pub String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        error!("database error: {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

pub fn bad_request(msg: impl Into<String>) -> AppError {
    AppError(StatusCode::BAD_REQUEST, msg.into())
}

pub fn not_found(msg: impl Into<String>) -> AppError {
    AppError(StatusCode::NOT_FOUND, msg.into())
}

/// Drops the cached protobuf card list for this user (same key the card handlers use).
pub async fn invalidate_cards_cache(state: &AppState, user_id: &str) {
    if let Some(mut redis) = state.redis.clone() {
        let key = format!("user_cards_proto_v2:{}", user_id);
        let _: () = redis::AsyncCommands::del(&mut redis, key).await.unwrap_or_default();
    }
}

/// Today's date for planning. Bank cycles are local, so the offset is configurable
/// (`FLINDERAX_UTC_OFFSET_MINUTES`, default +330 = IST).
pub fn today_local() -> time::Date {
    let minutes: i64 = std::env::var("FLINDERAX_UTC_OFFSET_MINUTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(330);
    (OffsetDateTime::now_utc() + Duration::minutes(minutes)).date()
}

pub fn validate_day(day: i64, what: &str) -> Result<(), AppError> {
    if (1..=31).contains(&day) {
        Ok(())
    } else {
        Err(bad_request(format!("{what} must be between 1 and 31")))
    }
}

pub fn validate_year_month(v: &Option<String>, what: &str) -> Result<(), AppError> {
    match v {
        Some(s) if crate::planner::parse_year_month(s).is_none() => {
            Err(bad_request(format!("{what} must look like 2026-12")))
        }
        _ => Ok(()),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(login_payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    dotenvy::dotenv().ok();

    let user = sqlx::query!(
        "SELECT user_id, user_name, user_password, user_role FROM users WHERE user_name = ? ",
        login_payload.user_name
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = match user {
        Some(u) => u,
        None => {
            return Err(AppError(
                StatusCode::UNAUTHORIZED,
                "INVALID USERNAME or password".into(),
            ))
        }
    };

    verify_user(user.user_password, login_payload.user_password)?;

    let user_id = user
        .user_id
        .as_deref()
        .ok_or(AppError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting user id".to_string(),
        ))?
        .to_string();

    let expiration = OffsetDateTime::now_utc() + Duration::hours(24);
    let token = get_paseto_token(&user_id, user.user_role, &state.paseto_key, expiration)?;

    Ok(Json(LoginResponse {
        access_token: token,
        expires_at: expiration.unix_timestamp(),
    }))
}
fn verify_user(database_password: String, user_input_password: String) -> Result<(), AppError> {
    let parsed_hash = PasswordHash::new(&database_password)
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let argon2 = Argon2::default();

    argon2
        .verify_password(user_input_password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            AppError(
                StatusCode::UNAUTHORIZED,
                "Invalid username or password".to_string(),
            )
        })?;
    Ok(())
}

fn get_paseto_token(
    user_id: &str,
    user_role: String,
    key: &Arc<PasetoSymmetricKey<V4, Local>>,
    expiration: OffsetDateTime,
) -> Result<String, AppError> {
    let role_claim = CustomClaim::try_from(("role", user_role.as_str()))
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let token = PasetoBuilder::<V4, Local>::default()
        .set_claim(SubjectClaim::from(user_id))
        .set_claim(role_claim)
        .set_claim(
            ExpirationClaim::try_from(
                expiration
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            )
            .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        )
        .build(key)
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(token)
}

pub async fn register(
    State(state): State<AppState>,
    Json(create_user): Json<CreateUserPayload>,
) -> Result<Json<CreateUserResponse>, AppError> {
    let user_id = nanoid!();

    let CreateUserPayload {
        user_name,
        user_password,
        user_role,
    } = create_user;

    let password_hash = get_password_hash(user_password)?;

    sqlx::query!(
        "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES (?, ?, ?, ?)",
        user_id,
        user_name,
        password_hash,
        user_role
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to insert user {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(CreateUserResponse { status: true }))
}

fn get_password_hash(user_password: String) -> Result<String, AppError> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(user_password.as_bytes(), &salt)
        .map_err(|e| {
            error!("Error hashing password {}", e);
            AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .to_string();

    Ok(password_hash)
}
