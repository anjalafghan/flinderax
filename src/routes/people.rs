use axum::Router;
use crate::models::AppState;

/// Mounted at `/api/people`, behind login. TODO(people builder): routes.
pub fn routes(state: AppState) -> Router {
    Router::new().with_state(state)
}
