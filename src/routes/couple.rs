use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::couple;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(couple::status))
        .route("/create", post(couple::create))
        .route("/invite", post(couple::invite))
        .route("/join", post(couple::join))
        .route("/leave", post(couple::leave))
        .with_state(state)
}
