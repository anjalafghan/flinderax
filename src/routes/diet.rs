use axum::Router;
use crate::models::AppState;

/// Mounted at `/api/diet`, behind login. TODO(diet builder): routes.
pub fn routes(state: AppState) -> Router {
    Router::new().with_state(state)
}
