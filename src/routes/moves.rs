use axum::{routing::{get, post}, Router};
use crate::handlers::moves;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/record", post(moves::record))
        .route("/undo", post(moves::undo))
        .route("/list", get(moves::list))
        .with_state(state)
}
