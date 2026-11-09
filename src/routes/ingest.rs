use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use crate::handlers::{ingest, payees};
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/parse", post(ingest::parse))
        .route("/statement", post(ingest::statement))
        // The whole file is held in memory for the import, so the upload is capped at 5 MB.
        .route("/statement/upload", post(ingest::statement_upload).layer(DefaultBodyLimit::max(5 * 1024 * 1024)))
        .route("/statement/preview", post(ingest::statement_preview).layer(DefaultBodyLimit::max(5 * 1024 * 1024)))
        .route("/holder-names", get(ingest::holder_names).post(ingest::set_holder_names))
        .route("/recategorize", post(ingest::recategorize))
        .route("/transactions", get(ingest::list))
        .route("/transactions/create", post(ingest::create))
        .route("/transactions/update", post(ingest::update))
        .route("/transactions/delete", post(ingest::delete))
        .route("/summary", get(ingest::summary))
        .route("/rules", get(ingest::rules))
        .route("/rules/delete", post(ingest::rule_delete))
        .route("/payees", get(payees::list))
        .route("/payees/categorize", post(payees::categorize))
        .route("/key/status", get(ingest::key_status))
        .route("/key/rotate", post(ingest::key_rotate))
        .with_state(state)
}

/// Authenticated by `X-Ingest-Key`, not a bearer token.
pub fn hook_routes(state: AppState) -> Router {
    Router::new().route("/ingest", post(ingest::hook)).with_state(state)
}
