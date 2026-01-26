use axum::{routing::post, Router};
use crate::models::AppState;
use crate::handlers::common;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/login", post(common::login))
        .route("/register", post(common::register))
        .with_state(state)
}
