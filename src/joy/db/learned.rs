//! What the couple's real costs have taught the jar.

#![allow(unused_imports)]
use super::*;
use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

// ── learned from real costs ───────────────────────────────────────────────

/// Fewest samples before we trust what we've learned.
pub const MIN_SAMPLES: i64 = 3;

/// How real plan costs compare to estimates: (sum actual ÷ sum estimate, samples).
pub async fn cost_ratio(db: &SqlitePool, couple: &str) -> sqlx::Result<(f64, i64)> {
    let (actual, est, n): (Option<i64>, Option<i64>, i64) = sqlx::query_as(
        "SELECT SUM(actual_cost), SUM(est_cost), COUNT(*) FROM joy_history
         WHERE couple_id = ? AND outcome = 'did_it' AND actual_cost IS NOT NULL AND est_cost > 0",
    )
    .bind(couple)
    .fetch_one(db)
    .await?;
    Ok(match (actual, est) {
        (Some(a), Some(e)) if e > 0 => (a as f64 / e as f64, n),
        _ => (1.0, 0),
    })
}

/// Real cab fares vs the model's estimates at the time: (ratio, rides).
pub async fn cab_ratio(db: &SqlitePool, couple: &str) -> sqlx::Result<(f64, i64)> {
    let (fare, est, n): (Option<i64>, Option<i64>, i64) = sqlx::query_as(
        "SELECT SUM(fare), SUM(est), COUNT(*) FROM (
           SELECT cab_fare_0 AS fare, cab_est_0 AS est FROM joy_history WHERE couple_id = ?1 AND cab_fare_0 > 0 AND cab_est_0 > 0
           UNION ALL
           SELECT cab_fare_1, cab_est_1 FROM joy_history WHERE couple_id = ?1 AND cab_fare_1 > 0 AND cab_est_1 > 0)",
    )
    .bind(couple)
    .fetch_one(db)
    .await?;
    Ok(match (fare, est) {
        (Some(f), Some(e)) if e > 0 => (f as f64 / e as f64, n),
        _ => (1.0, 0),
    })
}

/// Record a real one-way cab fare for one of us, next to the model's estimate for that trip.
pub async fn set_cab_fare(db: &SqlitePool, couple: &str, id: i64, person: usize, fare: Option<i64>, model_est: Option<i64>) -> sqlx::Result<()> {
    let sql = if person == 0 {
        "UPDATE joy_history SET cab_fare_0 = ?, cab_est_0 = ? WHERE id = ? AND couple_id = ? AND outcome = 'did_it'"
    } else {
        "UPDATE joy_history SET cab_fare_1 = ?, cab_est_1 = ? WHERE id = ? AND couple_id = ? AND outcome = 'did_it'"
    };
    sqlx::query(sql).bind(fare).bind(fare.and(model_est)).bind(id).bind(couple).execute(db).await?;
    Ok(())
}

