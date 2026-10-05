use axum::{routing::{get, post}, Router};
use crate::handlers::account;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/create", post(account::create))
        .route("/update", post(account::update))
        .route("/update_balance", post(account::update_balance))
        .route("/list", get(account::list))
        .route("/delete", post(account::delete))
        .with_state(state)
}
