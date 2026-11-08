use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::joy;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/summary", get(joy::summary))
        .route("/connect", post(joy::connect))
        .route("/partner", post(joy::set_partner))
        .route("/disconnect", post(joy::disconnect))
        .route("/settle", get(joy::settle).post(joy::settle_record))
        .with_state(state)
}
