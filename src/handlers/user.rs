use dotenvy;

use crate::models::{
    CreateUserPayload, CreateUserResponse, GetUserResponse, GetUsers, LoginPayload, LoginResponse,
    Users,
};
use base64::{engine::general_purpose, Engine};

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
use sqlx::SqlitePool;
use tracing::error;

pub struct AppError(StatusCode, String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(login_payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    dotenvy::dotenv().map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = sqlx::query!(
        "SELECT user_id, user_name, user_password FROM users WHERE user_name = ? ",
        login_payload.user_name
    )
    .fetch_optional(&pool)
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

    let _ = verify_user(user.user_password, login_payload.user_password);

    let token = get_paseto_token()?;

    Ok(Json(LoginResponse {
        access_token: token,
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

fn get_paseto_token() -> Result<String, AppError> {
    let key_str = std::env::var("PASETO_KEY")
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let key_bytes = general_purpose::STANDARD
        .decode(key_str)
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(key_bytes.as_slice()));

    let token = PasetoBuilder::<V4, Local>::default()
        .build(&key)
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(token)
}

pub async fn delete(State(pool): State<SqlitePool>) -> Result<Json<Vec<Users>>, AppError> {
    let users = sqlx::query_as::<_, Users>("SELECT user_id FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(users))
}

pub async fn create(
    State(pool): State<SqlitePool>,
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
    .execute(&pool)
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
pub async fn get_user(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<GetUserResponse>>, AppError> {
    let db_users: Vec<GetUsers> = sqlx::query_as::<_, GetUsers>(
        "SELECT user_id, user_name, user_password, user_role FROM users",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Error fetching users {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let users: Vec<GetUserResponse> = db_users
        .into_iter()
        .map(|v| GetUserResponse {
            user_id: v.user_id,
            user_name: v.user_name,
            user_password: v.user_password,
            user_role: v.user_role,
        })
        .collect();

    Ok(Json(users))
}
