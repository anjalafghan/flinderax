use axum::{routing::get, Router};
use crate::handlers::dashboard;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/summary", get(dashboard::summary))
        .with_state(state)
}
