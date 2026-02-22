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
