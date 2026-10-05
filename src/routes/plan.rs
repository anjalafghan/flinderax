use axum::{routing::{get, post}, Router};
use crate::handlers::plan;
use crate::models::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/commitment/create", post(plan::commitment_create))
        .route("/commitment/update", post(plan::commitment_update))
        .route("/commitment/delete", post(plan::commitment_delete))
        .route("/commitment/list", get(plan::commitment_list))
        .route("/income/create", post(plan::income_create))
        .route("/income/update", post(plan::income_update))
        .route("/income/delete", post(plan::income_delete))
        .route("/income/list", get(plan::income_list))
        .route("/settings/get", get(plan::settings_get))
        .route("/settings/update", post(plan::settings_update))
        .with_state(state)
}
