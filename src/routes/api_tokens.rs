use axum::{Router, routing::{get, post}};

use crate::{handlers::api_tokens, models::AppState};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(api_tokens::list).post(api_tokens::create))
        .route("/{id}/revoke", post(api_tokens::revoke))
        .route("/audit", get(api_tokens::audit))
        .with_state(state)
}
