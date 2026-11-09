use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::{handlers::diet, models::AppState};

/// Mounted at `/api/diet`, behind login.
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(diet::overview))
        .route("/habits", post(diet::create_habit))
        .route("/habits/{id}", patch(diet::update_habit).delete(diet::delete_habit))
        .route("/weeks/{week_start}", put(diet::put_week))
        .route("/marks", get(diet::list_marks).post(diet::create_mark))
        .route("/marks/{id}", delete(diet::delete_mark))
        .route("/settings", put(diet::put_settings))
        .with_state(state)
}
