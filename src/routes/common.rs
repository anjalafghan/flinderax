use axum::{routing::post, Router};
use sqlx::SqlitePool;

use crate::handlers::common;

pub fn routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/login", post(common::login))
        .route("/register", post(common::register))
        .with_state(pool)
}
