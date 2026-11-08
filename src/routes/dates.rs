//! Dates (Joy) as a JSON API under `/api/dates`. The handlers live in `crate::joy::api`.

use axum::{
    routing::{get, post},
    Router,
};

use crate::joy::api::{decide, envelope, middle, wishlist};
use crate::models::AppState;

/// Everything behind login: mounted at `/api/dates`.
pub fn routes(state: AppState) -> Router {
    Router::new()
        // The jar and the draw.
        .route("/jar", get(decide::page))
        .route("/pick", post(decide::pick))
        .route("/match", post(decide::matching))
        .route("/accept/{id}", post(decide::accept))
        .route("/middle", post(middle::check_in))
        // Slips.
        .route("/slips", get(wishlist::page).post(wishlist::create))
        .route("/slips/new", get(wishlist::new))
        .route("/slips/{id}", get(wishlist::edit).post(wishlist::update))
        .route("/slips/{id}/archive", post(wishlist::archive))
        .route("/slips/{id}/restore", post(wishlist::restore))
        .route("/slips/{id}/delete", post(wishlist::delete))
        .route("/slips/{id}/unveto", post(wishlist::unveto))
        .route("/slips/{id}/plan", post(wishlist::plan))
        // History (Kept), budget, savings, Plan B and jobs.
        .merge(crate::joy::api::history::router())
        // Regulars, nights, month and year pages.
        .merge(crate::joy::api::regular::router())
        .with_state(state)
}

/// No login: mounted at `/api/public/dates`. Each is keyed by an unguessable token.
pub fn public_routes(state: AppState) -> Router {
    Router::new()
        .route("/e/{token}", get(envelope::page))
        // signed-URL routes (invite.ics, photos): the handlers verify `joy::api::verify` themselves.
        .merge(crate::joy::api::history::public_router())
        // Signed-URL routes: a regular night's photo.
        .merge(crate::joy::api::regular::public_router())
        .with_state(state)
}
