use axum::{
    routing::{get, post, put},
    Router,
};

use crate::handlers::work as h;
use crate::models::AppState;

/// Mounted at `/api/work`, behind login.
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(h::list).post(h::create))
        .route("/clear", post(h::clear))
        .route("/{id}", put(h::replace).delete(h::delete))
        .with_state(state)
}
