use crate::ratelimit;
use crate::models::{AppState, CreateUserPayload, CreateUserResponse, LoginPayload, LoginResponse};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
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
        // Internal details (SQL errors etc.) stay in the logs, never in the response body.
        if self.0.is_server_error() {
            error!("{}: {}", self.0, self.1);
            return (self.0, "Internal server error".to_string()).into_response();
        }
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

/// Failed sign-ins per username per minute. Only failures count, so nobody can lock a user out by
/// hammering their name, and a correct password always gets through while under the limit.
const LOGIN_ATTEMPTS_PER_MINUTE: u32 = 10;
/// Longest username we ever store (see `validate_credentials`); anything longer can't exist.
const MAX_USERNAME: usize = 32;
/// Longest password we accept; longer ones are refused before Argon2 runs.
const MAX_PASSWORD: usize = 128;

fn invalid_login() -> AppError {
    AppError(StatusCode::UNAUTHORIZED, "Invalid username or password".into())
}

/// A real Argon2 hash of a random password, checked when the username doesn't exist so that a
/// wrong name takes as long as a wrong password and the timing doesn't say which names are real.
fn dummy_hash() -> &'static str {
    static H: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    H.get_or_init(|| get_password_hash(nanoid!(32)).unwrap_or_default())
}

/// Equal without stopping at the first difference.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
const REGISTRATIONS_PER_MINUTE: u32 = 5;

fn too_many() -> AppError {
    AppError(StatusCode::TOO_MANY_REQUESTS, "Too many attempts, try again in a minute".into())
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(login_payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    let ip_key = format!("login-ip:{}", ratelimit::client_key(&headers));
    if !ratelimit::allow(&ip_key, LOGIN_ATTEMPTS_PER_MINUTE * 3) {
        return Err(too_many());
    }
    // Nothing longer than a real username or password reaches the limiter's map or Argon2.
    if login_payload.user_name.chars().count() > MAX_USERNAME || login_payload.user_password.chars().count() > MAX_PASSWORD {
        return Err(invalid_login());
    }
    let name_key = format!("login-user:{}", login_payload.user_name.to_lowercase());
    if !ratelimit::under(&name_key, LOGIN_ATTEMPTS_PER_MINUTE) {
        return Err(too_many());
    }

    let user = sqlx::query!(
        "SELECT user_id, user_name, user_password, user_role FROM users WHERE user_name = ? ",
        login_payload.user_name
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let Some(user) = user else {
        let _ = verify_user(dummy_hash().to_string(), login_payload.user_password);
        ratelimit::hit(&name_key);
        return Err(invalid_login());
    };

    if let Err(e) = verify_user(user.user_password, login_payload.user_password) {
        ratelimit::hit(&name_key);
        return Err(e);
    }

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

pub fn validate_credentials(user_name: &str, user_password: &str) -> Result<(), AppError> {
    let name_len = user_name.chars().count();
    if !(3..=32).contains(&name_len)
        || !user_name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err(bad_request(
            "Username must be 3-32 characters: letters, digits, '_', '-' or '.'",
        ));
    }
    if !(8..=128).contains(&user_password.chars().count()) {
        return Err(bad_request("Password must be 8-128 characters"));
    }
    Ok(())
}

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(create_user): Json<CreateUserPayload>,
) -> Result<Json<CreateUserResponse>, AppError> {
    if !ratelimit::allow(
        &format!("register-ip:{}", ratelimit::client_key(&headers)),
        REGISTRATIONS_PER_MINUTE,
    ) {
        return Err(too_many());
    }
    let user_id = nanoid!();

    // The client-supplied role is ignored: admins are promoted in the database, never self-assigned.
    let CreateUserPayload {
        user_name,
        user_password,
        signup_code,
        ..
    } = create_user;
    // joy is for a couple, not the internet: a new account needs the code from the `SIGNUP_CODE`
    // secret, and with no secret set nobody can sign up.
    let expected = std::env::var("SIGNUP_CODE").unwrap_or_default();
    if expected.trim().is_empty() {
        return Err(AppError(StatusCode::FORBIDDEN, "Sign-ups are closed. Ask the owner for a sign-up code.".into()));
    }
    if !same(signup_code.trim().as_bytes(), expected.trim().as_bytes()) {
        return Err(AppError(StatusCode::FORBIDDEN, "That sign-up code isn't right.".into()));
    }
    let user_role = "user";
    validate_credentials(&user_name, &user_password)?;

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
    .map_err(|e| match e {
        sqlx::Error::Database(ref d) if d.is_unique_violation() => {
            AppError(StatusCode::CONFLICT, "Username is already taken".into())
        }
        e => {
            error!("Failed to insert user {}", e);
            AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
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
