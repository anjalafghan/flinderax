use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::models::Users;
use sqlx::{SqlitePool, pool};


pub struct AppError(StatusCode, String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}


pub async fn login(State(pool): State<SqlitePool>) -> Result<Json<Vec<Users>>, AppError> {
    let users = sqlx::query_as::<_,Users>("SELECT user_id FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
Ok(Json(users))
    
}


pub async fn delete(State(pool): State<SqlitePool>) -> Result<Json<Vec<Users>>, AppError> {
    let users = sqlx::query_as::<_,Users>("SELECT user_id FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
Ok(Json(users))
    
}
