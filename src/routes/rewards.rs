use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::{reward_fund, rewards};
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(rewards::list))
        .route("/fund", get(reward_fund::fund))
        .route("/suggestions", get(rewards::suggest))
        .route("/create", post(rewards::create))
        .route("/remove", post(rewards::remove))
        .route("/claim", post(rewards::claim))
        .route("/unclaim", post(rewards::unclaim))
        .route("/cap", post(rewards::set_cap))
        .with_state(state)
}
