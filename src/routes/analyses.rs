use axum::{
    routing::{delete, get},
    Router,
};

use crate::handlers::analyses as h;
use crate::models::AppState;

/// Mounted at `/api/analyses`, behind login.
pub fn routes(state: AppState) -> Router {
    Router::new().route("/", get(h::list).post(h::create)).route("/{id}", delete(h::delete)).with_state(state)
}
