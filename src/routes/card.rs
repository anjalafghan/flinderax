use axum::{routing::get, routing::post, Router};
use sqlx::SqlitePool;

use crate::handlers::card;

pub fn routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/create", post(card::create))
        .route("/update", post(card::update))
        .route("/delete", post(card::delete))
        .route("/get", get(card::get_card))
        .with_state(pool)
}
