//! `POST /api/plan/guidance`: a month plan from the figures in the body. Stateless; the rules are
//! in `planner::guidance`.

use axum::Json;

use crate::{
    handlers::common::{bad_request, AppError},
    planner::guidance::{plan, validate, Guidance, Input},
};

pub async fn post(Json(body): Json<Input>) -> Result<Json<Guidance>, AppError> {
    validate(&body).map_err(bad_request)?;
    Ok(Json(plan(&body)))
}
