use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use crate::models::AppState;
use crate::app::token_validator_auth_middleware;
use crate::handlers::user;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/delete", post(user::delete))
        .layer(middleware::from_fn(token_validator_auth_middleware))
        .route("/get", get(user::get_user))
        .with_state(state)
}
