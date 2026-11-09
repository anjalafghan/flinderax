use axum::{routing::{get, post}, Router};
use crate::handlers::checkin;
use crate::models::AppState;

/// Mounted at `/api/checkin`, behind login.
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(checkin::status))
        .route("/report", get(checkin::report))
        .route("/complete", post(checkin::complete))
        .route("/history", get(checkin::history))
        .with_state(state)
}
