//! `GET /api/dashboard/streaks`: the runs worth cheering for. Days in a row with a transaction
//! logged, months in a row you spent no more than you earned, and (for a couple) months in a row
//! with a date.

use std::collections::{BTreeMap, BTreeSet};

use axum::{extract::State, Extension, Json};
use serde::Serialize;
use time::{macros::format_description, Date, Month};

use crate::{
    alerts::is_spending,
    handlers::{
        common::{today_local, AppError},
        couple::couple_of,
    },
    models::AppState,
    planner::{
        cycle::add_months,
        month_index,
        streaks::{dates_anchor, day_streak, month_streak, saved_months, Streak},
    },
};

/// How far back to look for logged days.
const DAY_WINDOW: i64 = 400;
/// How many completed months to judge.
const MONTH_WINDOW: i32 = 24;

#[derive(Serialize)]
pub struct StreaksOut {
    /// Days in a row with at least one transaction.
    pub logging: Streak,
    /// Months in a row with income, and spending no more than it.
    pub saved: Streak,
    /// Months in a row with a date. Null when you are not in a couple.
    pub dates: Option<Streak>,
}

fn parse(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

/// `"2026-10"` as a month index.
fn month_of(s: &str) -> Option<i32> {
    let (y, m) = s.split_once('-')?;
    let (y, m): (i32, u8) = (y.parse().ok()?, m.get(..2)?.parse().ok()?);
    Some(month_index(Date::from_calendar_date(y, Month::try_from(m).ok()?, 1).ok()?))
}

pub async fn get(State(state): State<AppState>, Extension((user_id, _)): Extension<(String, String)>) -> Result<Json<StreaksOut>, AppError> {
    let db = &state.db;
    let today = today_local();

    let since = (today - time::Duration::days(DAY_WINDOW)).to_string();
    let day_rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT occurred_on FROM transactions WHERE user_id = ? AND occurred_on >= ? AND occurred_on <= ?",
    )
    .bind(&user_id)
    .bind(&since)
    .bind(today.to_string())
    .fetch_all(db)
    .await?;
    let days: BTreeSet<Date> = day_rows.iter().filter_map(|(d,)| parse(d)).collect();

    // Completed months only: the current one is still being earned and spent.
    let (fy, fm) = add_months(today.year(), today.month(), -MONTH_WINDOW);
    let from = Date::from_calendar_date(fy, fm, 1).map_err(|e| AppError(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let this_first = Date::from_calendar_date(today.year(), today.month(), 1)
        .map_err(|e| AppError(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let month_rows: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT substr(occurred_on, 1, 7), direction, category, SUM(amount_paise) FROM transactions
         WHERE user_id = ? AND occurred_on >= ? AND occurred_on < ? GROUP BY 1, 2, 3",
    )
    .bind(&user_id)
    .bind(from.to_string())
    .bind(this_first.to_string())
    .fetch_all(db)
    .await?;
    let mut totals: BTreeMap<i32, (i64, i64)> = BTreeMap::new();
    for (month, direction, category, total) in month_rows {
        let Some(m) = month_of(&month) else { continue };
        let e = totals.entry(m).or_default();
        if direction == "credit" && category == "income" {
            e.0 = e.0.saturating_add(total);
        } else if direction == "debit" && is_spending(&category) {
            e.1 = e.1.saturating_add(total);
        }
    }
    let saved = month_streak(&saved_months(&totals), month_index(today) - 1);

    let dates = match couple_of(db, &user_id).await? {
        None => None,
        Some((couple, who)) => {
            // A surprise still sealed from you does not count: that would give it away.
            let rows: Vec<(String,)> = sqlx::query_as(
                "SELECT DISTINCT strftime('%Y-%m', COALESCE(planned_at, picked_at), 'unixepoch', 'localtime') FROM joy_history
                 WHERE couple_id = ?1 AND outcome = 'did_it' AND COALESCE(planned_at, picked_at) <= unixepoch()
                   AND NOT (surprise = 1 AND revealed = 0 AND COALESCE(drawn_by, -1) <> ?2)",
            )
            .bind(&couple)
            .bind(who as i64)
            .fetch_all(db)
            .await?;
            let good: BTreeSet<i32> = rows.iter().filter_map(|(m,)| month_of(m)).collect();
            Some(month_streak(&good, dates_anchor(&good, today)))
        }
    };

    Ok(Json(StreaksOut { logging: day_streak(&days, today), saved, dates }))
}
