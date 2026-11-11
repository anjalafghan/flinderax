use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::bills;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(bills::upcoming))
        .route("/suggestions", get(bills::suggestions))
        .route("/suggestions/dismiss", post(bills::dismiss))
        .route("/suggestions/restore", post(bills::restore))
        .with_state(state)
}
