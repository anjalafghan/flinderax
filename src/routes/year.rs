use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::year;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(year::view))
        .route("/outflows", get(year::list).post(year::create))
        .route("/outflows/{id}/delete", post(year::delete))
        .with_state(state)
}
