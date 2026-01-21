use crate::models::{CreateUserPayload, CreateUserResponse, GetUserResponse, GetUsers, Users};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use nanoid::nanoid;
use sqlx::{pool, SqlitePool};
use tracing::{error, info, instrument};

pub struct AppError(StatusCode, String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

pub async fn login(State(pool): State<SqlitePool>) -> Result<Json<Vec<Users>>, AppError> {
    let users = sqlx::query_as::<_, Users>("SELECT user_id FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(users))
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
    create_user: Json<CreateUserPayload>,
) -> Result<Json<CreateUserResponse>, AppError> {
    let user_name = create_user.user_name.clone();
    let user_password = create_user.user_password.clone();
    let user_role = create_user.user_role.to_lowercase();
    let user_id = nanoid!();

    sqlx::query(
        "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(user_name)
    .bind(user_password)
    .bind(user_role)
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Failed to insert user {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(CreateUserResponse { status: true }))
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
