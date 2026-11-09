//! Phase 2 check-in API (see the contract). Stubs until the builder fills them in.

use axum::http::StatusCode;
use crate::handlers::common::AppError;

fn todo() -> AppError {
    AppError(StatusCode::NOT_IMPLEMENTED, "Not built yet".to_string())
}

pub async fn status() -> Result<(), AppError> { Err(todo()) }
pub async fn report() -> Result<(), AppError> { Err(todo()) }
pub async fn complete() -> Result<(), AppError> { Err(todo()) }
pub async fn history() -> Result<(), AppError> { Err(todo()) }
