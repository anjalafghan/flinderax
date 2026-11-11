use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::review;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/month", get(review::month))
        .route("/tax", get(review::tax))
        .route("/tax/entry", post(review::add_entry))
        .route("/tax/entry/delete", post(review::delete_entry))
        .route("/tax/entries/clear", post(review::clear_entries))
        .route("/tax/commitment", post(review::tag_commitment))
        .with_state(state)
}
