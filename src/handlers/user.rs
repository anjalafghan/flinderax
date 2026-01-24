use crate::models::{GetUserResponse, GetUsers};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::SqlitePool;
use tracing::error;

pub struct AppError(StatusCode, String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    user_name: String,
) -> Result<Json<bool>, AppError> {
    sqlx::query!("DELETE FROM users WHERE user_name  = ?", user_name)
        .execute(&pool)
        .await
        .map_err(|e| {
            error!("Failed to insert user {}", e);
            AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    Ok(Json(true))
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
