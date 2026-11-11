use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::handlers::people as h;
use crate::models::AppState;

/// Mounted at `/api/people`, behind login. Fixed segments (`seasons`, `gifts`, ...) win over `/{id}`.
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(h::list).post(h::create))
        .route("/seasons", get(h::list_seasons).post(h::add_season))
        .route("/seasons/{id}", patch(h::patch_season).delete(h::delete_season))
        .route("/gift-plan", get(h::gift_plan))
        .route("/settings", get(h::get_settings).put(h::put_settings))
        .route("/gifts", post(h::add_gift))
        .route("/gifts/{id}", patch(h::patch_gift).delete(h::delete_gift))
        .route("/dates/{id}", patch(h::patch_date).delete(h::delete_date))
        .route("/touches/{id}", delete(h::delete_touch))
        .route("/{id}", get(h::get).patch(h::patch).delete(h::delete))
        .route("/{id}/touches", post(h::add_touch).delete(h::clear_touches))
        .route("/{id}/dates", post(h::add_date))
        .with_state(state)
}
