use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::ingest;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/parse", post(ingest::parse))
        .route("/statement", post(ingest::statement))
        .route("/transactions", get(ingest::list))
        .route("/transactions/create", post(ingest::create))
        .route("/transactions/update", post(ingest::update))
        .route("/transactions/delete", post(ingest::delete))
        .route("/summary", get(ingest::summary))
        .route("/rules", get(ingest::rules))
        .route("/rules/delete", post(ingest::rule_delete))
        .route("/key/status", get(ingest::key_status))
        .route("/key/rotate", post(ingest::key_rotate))
        .with_state(state)
}

/// Authenticated by `X-Ingest-Key`, not a bearer token.
pub fn hook_routes(state: AppState) -> Router {
    Router::new().route("/ingest", post(ingest::hook)).with_state(state)
}
