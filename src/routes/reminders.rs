use axum::{routing::{get, post}, Router};
use crate::handlers::reminders;
use crate::models::AppState;

/// Mounted at `/api/reminders`, behind login.
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/link", get(reminders::link))
        .route("/link/reset", post(reminders::reset_link))
        .with_state(state)
}

/// Mounted at `/api/public/reminders`: the calendar feed, keyed by its token. No login.
pub fn public_routes(state: AppState) -> Router {
    Router::new().route("/{file}", get(reminders::feed)).with_state(state)
}
