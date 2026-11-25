//! `GET /api/activity`: what you have put into the app lately, from the `activity_log` ledger.
//! `note` is how the write paths (statement import, bank texts, manual entries, check-ins) add to it.
//! The rules are pure in `planner::activity`.

use std::collections::BTreeSet;

use axum::{extract::State, Extension, Json};
use serde::Serialize;
use time::{Date};

use crate::{
    handlers::common::{today_local, AppError},
    models::AppState,
    planner::{
        activity::{active_streak, grant, points, Kind},
        streaks::Streak,
    },
};

/// How far back the streak looks.
const WINDOW_DAYS: i64 = 400;

#[derive(Serialize)]
pub struct ActivityOut {
    /// Days in a row with at least one action.
    pub streak: Streak,
    /// Points earned this month.
    pub month_points: i64,
    /// Days this month with at least one action.
    pub month_days: i64,
    pub today_done: bool,
}

/// Records one action. `new_rows` is what it really added (0 earns nothing); `ref_id` names the
/// thing so a retry is counted once. Best effort: a failure here is logged and never fails the
/// action that called it.
pub async fn note(db: &sqlx::SqlitePool, user_id: &str, kind: Kind, ref_id: &str, new_rows: usize) {
    if let Err(e) = record(db, user_id, kind, ref_id, new_rows, today_local()).await {
        tracing::warn!(error = %e.1, "activity not recorded");
    }
}

/// Returns the points granted.
pub async fn record(db: &sqlx::SqlitePool, user_id: &str, kind: Kind, ref_id: &str, new_rows: usize, today: Date) -> Result<i64, AppError> {
    let wanted = points(kind, new_rows);
    if wanted == 0 {
        return Ok(0);
    }
    let day = today.to_string();
    let already: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(points), 0) FROM activity_log WHERE user_id = ? AND kind = ? AND day = ?")
        .bind(user_id)
        .bind(kind.as_str())
        .bind(&day)
        .fetch_one(db)
        .await?;
    // A day that is over the cap still counts as an active day: the row stays, with 0 points.
    let pts = grant(kind, wanted, already);
    let done = sqlx::query("INSERT INTO activity_log (user_id, kind, day, ref_id, points) VALUES (?, ?, ?, ?, ?) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(kind.as_str())
        .bind(&day)
        .bind(ref_id)
        .bind(pts)
        .execute(db)
        .await?
        .rows_affected();
    Ok(if done == 1 { pts } else { 0 })
}

fn parse(s: &str) -> Option<Date> {
    crate::planner::dates::parse_ymd_prefix(s)
}

pub async fn compute(state: &AppState, user_id: &str, today: Date) -> Result<ActivityOut, AppError> {
    let since = (today - time::Duration::days(WINDOW_DAYS)).to_string();
    let rows: Vec<(String,)> = sqlx::query_as("SELECT DISTINCT day FROM activity_log WHERE user_id = ? AND day >= ? AND day <= ?")
        .bind(user_id)
        .bind(&since)
        .bind(today.to_string())
        .fetch_all(&state.db)
        .await?;
    let days: BTreeSet<Date> = rows.iter().filter_map(|(d,)| parse(d)).collect();
    let month = &today.to_string()[..7];
    let (month_points, month_days): (i64, i64) = sqlx::query_as(
        "SELECT COALESCE(SUM(points), 0), COUNT(DISTINCT day) FROM activity_log WHERE user_id = ? AND substr(day, 1, 7) = ? AND day <= ?",
    )
    .bind(user_id)
    .bind(month)
    .bind(today.to_string())
    .fetch_one(&state.db)
    .await?;
    Ok(ActivityOut { streak: active_streak(&days, today), month_points, month_days, today_done: days.contains(&today) })
}

pub async fn get(State(state): State<AppState>, Extension((user_id, _)): Extension<(String, String)>) -> Result<Json<ActivityOut>, AppError> {
    Ok(Json(compute(&state, &user_id, today_local()).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use time::Month;

    async fn db() -> sqlx::SqlitePool {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')").execute(&db).await.unwrap();
        db
    }

    fn ok<T>(r: Result<T, AppError>) -> T {
        r.unwrap_or_else(|e| panic!("{}", e.1))
    }

    fn day(d: u8) -> Date {
        Date::from_calendar_date(2026, Month::November, d).unwrap()
    }

    #[tokio::test]
    async fn an_import_is_counted_once_and_nothing_new_earns_nothing() {
        let db = db().await;
        assert_eq!(ok(record(&db, "u", Kind::Statement, "imp1", 25, day(10)).await), 7);
        assert_eq!(ok(record(&db, "u", Kind::Statement, "imp1", 25, day(10)).await), 0); // retry
        assert_eq!(ok(record(&db, "u", Kind::Statement, "imp2", 0, day(10)).await), 0); // re-upload, all duplicates
    }

    #[tokio::test]
    async fn the_daily_cap_holds_but_the_day_still_counts() {
        let db = db().await;
        for i in 0..5 {
            ok(record(&db, "u", Kind::Txn, &format!("t{i}"), 1, day(10)).await);
        }
        let total: i64 = sqlx::query_scalar("SELECT SUM(points) FROM activity_log WHERE user_id = 'u'").fetch_one(&db).await.unwrap();
        assert_eq!(total, 3);
    }
}
