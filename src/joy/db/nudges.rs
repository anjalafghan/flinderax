//! Gentle nudges: what is coming up, and how long since the last date.

#![allow(unused_imports)]
use super::*;
use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

// ── gentle nudges ─────────────────────────────────────────────────────────

/// Today and tomorrow as local "YYYY-MM-DD", plus today's weekday (0 = Sunday) and month.
pub async fn local_today(db: &SqlitePool) -> sqlx::Result<(String, String, u8, u8)> {
    let (today, tomorrow, wd, m): (String, String, i64, i64) = sqlx::query_as(
        "SELECT date('now', 'localtime'), date('now', 'localtime', '+1 day'),
                CAST(strftime('%w', 'now', 'localtime') AS INTEGER), CAST(strftime('%m', 'now', 'localtime') AS INTEGER)",
    )
    .fetch_one(db)
    .await?;
    Ok((today, tomorrow, wd as u8, m as u8))
}

/// Kept dates planned from now until the end of tomorrow (local), soonest first.
pub async fn upcoming(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<HistoryRow>> {
    sqlx::query_as(&format!(
        "SELECT {HISTORY_COLS} FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ? AND h.outcome = 'did_it' AND h.planned_at >= unixepoch() - 3 * 3600
           AND date(h.planned_at, 'unixepoch', 'localtime') <= date('now', 'localtime', '+1 day')
         ORDER BY h.planned_at, h.id"
    ))
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Is anything planned for later (beyond tomorrow too)?
pub async fn anything_planned(db: &SqlitePool, couple: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM joy_history WHERE couple_id = ? AND outcome = 'did_it' AND planned_at > unixepoch())",
    )
        .bind(couple)
        .fetch_one(db)
        .await
}

/// Whole days since our last date happened (its planned time if set, else when it was kept).
pub async fn days_since_last_date(db: &SqlitePool, couple: &str) -> sqlx::Result<Option<i64>> {
    sqlx::query_scalar(
        "SELECT CAST((unixepoch() - MAX(COALESCE(planned_at, picked_at))) / 86400 AS INTEGER) FROM joy_history
         WHERE couple_id = ? AND outcome = 'did_it' AND COALESCE(planned_at, picked_at) <= unixepoch()",
    )
    .bind(couple)
    .fetch_one(db)
    .await
}

