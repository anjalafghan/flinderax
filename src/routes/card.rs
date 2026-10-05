use axum::{routing::{get, post}, Router};
use crate::models::AppState;
use crate::handlers::{card, card_cycle};

 pub fn routes(state: AppState) -> Router {
     Router::new()
         .route("/create", post(card::create_card))
         .route("/update", post(card::update))
         .route("/delete", post(card::delete_card))
         .route("/delete_transaction", post(card::delete_transaction))
         .route("/get_card", post(card::get_card))
         .route("/get_all_cards", get(card::get_all_cards))
         .route("/insert_transaction", post(card::insert_transaction))
         .route("/history", post(card::get_history))
         .route("/reset", post(card::reset_transactions))
         .route("/defer_update", post(card::defer_update))
         .route("/defer_status", get(card::get_deferred_status))
         .route("/defer_settle", post(card::settle_deferred))
         .route("/defer_cancel", post(card::cancel_deferred))
         .route("/snapshot", post(card_cycle::snapshot))
         .route("/breakdown", post(card_cycle::breakdown))
         .route("/emi/create", post(card_cycle::emi_create))
         .route("/emi/update", post(card_cycle::emi_update))
         .route("/emi/delete", post(card_cycle::emi_delete))
         .route("/pending/create", post(card_cycle::pending_create))
         .route("/pending/clear", post(card_cycle::pending_clear))
         .with_state(state)
 }
