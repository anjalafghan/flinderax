use axum::{routing::get, routing::post, Router};
use sqlx::SqlitePool;

use crate::handlers::user;

pub fn routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/create", post(user::create))
        .route("/login", post(user::login))
        .route("/delete", post(user::delete))
        .route("/get", get(user::get_user))
        .with_state(pool)
}
