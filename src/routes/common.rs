use axum::{middleware, routing::post, Router};
use crate::app::token_validator_middleware;
use crate::handlers::lifecycle;
use crate::models::AppState;
use crate::handlers::common;

pub fn routes(state: AppState) -> Router {
    // Sign-in and sign-up are open; these need a live session.
    let signed_in = Router::new()
        .route("/password", post(lifecycle::change_password))
        .route("/logout-everywhere", post(lifecycle::logout_everywhere))
        .route("/delete-account", post(lifecycle::delete_account))
        .layer(middleware::from_fn_with_state(state.clone(), token_validator_middleware));
    Router::new()
        .route("/login", post(common::login))
        .route("/register", post(common::register))
        .merge(signed_in)
        .with_state(state)
}
