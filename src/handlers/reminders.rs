//! Phase 2 reminder calendar feed (see the contract). Stubs until the builder fills them in.

use axum::http::StatusCode;
use crate::handlers::common::AppError;

fn todo() -> AppError {
    AppError(StatusCode::NOT_IMPLEMENTED, "Not built yet".to_string())
}

pub async fn link() -> Result<(), AppError> { Err(todo()) }
pub async fn reset_link() -> Result<(), AppError> { Err(todo()) }
pub async fn feed() -> Result<(), AppError> { Err(todo()) }
