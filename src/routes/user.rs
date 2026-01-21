use axum::{Router, routing::post};
use sqlx::SqlitePool;

use crate::handlers::user;

pub fn routes(pool: SqlitePool)-> Router{
    Router::new()
        .route("/login", post(user::login))
        .route("/delete", post(user::delete))
        .with_state(pool)
}
