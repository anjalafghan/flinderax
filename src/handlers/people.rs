//! `/api/people`: the circle, important dates, catch-ups, gifts and festival seasons. Phase 5.
//! TODO(people builder): endpoints, `habits_input` and `report`.

use time::Date;

use crate::{
    handlers::common::AppError,
    planner::{
        checkin::Period,
        people::{PeopleIn, PeopleReport},
    },
};

/// The habit rules' input for one user.
pub async fn habits_input(_db: &sqlx::SqlitePool, _user_id: &str, _today: Date) -> Result<PeopleIn, AppError> {
    Ok(PeopleIn::default())
}

/// The people section for a check-in period, or None when the user has no people yet.
pub async fn report(_db: &sqlx::SqlitePool, _user_id: &str, _period: Period, _today: Date) -> Result<Option<PeopleReport>, AppError> {
    Ok(None)
}
