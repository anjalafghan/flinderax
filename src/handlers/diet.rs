//! `/api/diet`: weekly diet habits, food money, body marks. Phase 5.
//! TODO(diet builder): endpoints, `habits_input` and `report`.

use time::Date;

use crate::{
    handlers::common::AppError,
    planner::{
        checkin::Period,
        diet::{DietIn, DietReport},
    },
};

/// The habit rules' input for one user.
pub async fn habits_input(_db: &sqlx::SqlitePool, _user_id: &str, _today: Date) -> Result<DietIn, AppError> {
    Ok(DietIn::default())
}

/// The diet section for a check-in period, or None when there is nothing to say.
pub async fn report(_db: &sqlx::SqlitePool, _user_id: &str, _period: Period, _today: Date) -> Result<Option<DietReport>, AppError> {
    Ok(None)
}
