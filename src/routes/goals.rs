use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::goals;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(goals::list))
        .route("/create", post(goals::create))
        .route("/update", post(goals::update))
        .route("/archive", post(goals::archive))
        .route("/contribute", post(goals::contribute))
        .with_state(state)
}
