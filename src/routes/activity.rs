use axum::{routing::get, Router};
use crate::handlers::activity;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new().route("/", get(activity::get)).with_state(state)
}
