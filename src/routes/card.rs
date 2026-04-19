use axum::{routing::{get, post}, Router};
use crate::models::AppState;
use crate::handlers::card;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/create", post(card::create_card))
        .route("/update", post(card::update))
        .route("/delete", post(card::delete_card))
        .route("/get_card", post(card::get_card))
        .route("/get_all_cards", get(card::get_all_cards))
        .route("/insert_transaction", post(card::insert_transaction))
        .route("/history", post(card::get_history))
        .route("/reset", post(card::reset_transactions))
        .route("/defer_update", post(card::defer_update))
        .route("/defer_status", get(card::get_deferred_status))
        .route("/defer_settle", post(card::settle_deferred))
        .route("/defer_cancel", post(card::cancel_deferred))
        .with_state(state)
}
