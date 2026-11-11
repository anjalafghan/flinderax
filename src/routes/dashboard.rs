use axum::{routing::get, Router};
use crate::handlers::{analytics, dashboard, habits, insights, streaks};
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/summary", get(dashboard::summary))
        .route("/history", get(dashboard::history))
        .route("/insights", get(insights::get))
        .route("/analytics", get(analytics::get))
        .route("/habits", get(habits::get))
        .route("/streaks", get(streaks::get))
        .with_state(state)
}
