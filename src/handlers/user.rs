use crate::models::{AppState, GetUserResponse, GetUsers};
use crate::handlers::common::AppError;
use axum::{extract::State, http::StatusCode, Extension, Json};
use tracing::error;

/// Admin delete: the same full clean-up as deleting your own account (see `lifecycle::delete_user`).
/// You can't delete yourself here, and the last admin can't be deleted at all.
pub async fn delete(
    State(state): State<AppState>,
    Extension((caller, _)): Extension<(String, String)>,
    user_name: String,
) -> Result<Json<bool>, AppError> {
    let id: Option<String> = sqlx::query_scalar("SELECT user_id FROM users WHERE user_name = ?")
        .bind(user_name.trim())
        .fetch_optional(&state.db)
        .await?;
    let Some(id) = id else { return Ok(Json(true)) };
    if id == caller {
        return Err(AppError(StatusCode::CONFLICT, "Use delete account in your own settings to remove yourself.".into()));
    }
    crate::handlers::lifecycle::delete_user(&state.db, &id).await?;
    Ok(Json(true))
}

pub async fn get_user(
    State(state): State<AppState>,
) -> Result<Json<Vec<GetUserResponse>>, AppError> {
    let db_users: Vec<GetUsers> = sqlx::query_as::<_, GetUsers>(
        "SELECT user_id, user_name, '' AS user_password, user_role FROM users",
    )
    .fetch_all(&state.db)
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
            user_role: v.user_role,
        })
        .collect();

    Ok(Json(users))
}
