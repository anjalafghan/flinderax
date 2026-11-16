//! `GET /api/decisions`: what needs a decision this week, from data other pages already hold.
//! Read-only. The same counts drive the weekly push (counts only, see `push.rs`).

use axum::{extract::State, Extension, Json};
use serde::Serialize;
use sqlx::SqlitePool;
use time::{Date, Duration};

use crate::{
    handlers::{
        coming,
        common::{today_local, AppError},
        couple::couple_of,
        year::lumpy_for,
    },
    models::AppState,
    planner::{
        coming::Kind as ComingKind,
        couple_goals::percent,
        week_digest::{decisions, Counts, Decision},
        year_ahead::{next_occurrence, Direction},
    },
};

type User = Extension<(String, String)>;

/// How far ahead a large amount or a birthday counts as "soon".
const SOON_DAYS: i64 = 14;
const BIRTHDAY_DAYS: i64 = 7;
/// A shared goal is behind when its date is this close (or past) and it is below this share funded.
const GOAL_WINDOW_DAYS: i64 = 60;
const GOAL_FUNDED_PCT: i64 = 80;
/// Transactions this old are no longer nagged about.
const UNCATEGORIZED_DAYS: i64 = 60;

#[derive(Serialize)]
pub struct DecisionsOut {
    pub items: Vec<Decision>,
}

pub async fn counts(db: &SqlitePool, user_id: &str, today: Date) -> Result<Counts, AppError> {
    let since = (today - Duration::days(UNCATEGORIZED_DAYS)).to_string();
    let uncategorized: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM transactions WHERE user_id = ? AND category = 'uncategorized' AND direction = 'debit' AND occurred_on >= ?",
    )
    .bind(user_id)
    .bind(&since)
    .fetch_one(db)
    .await?;

    let mut goals_behind = 0usize;
    if let Some((couple, _)) = couple_of(db, user_id).await? {
        let rows: Vec<(i64, Option<String>, i64)> = sqlx::query_as(
            "SELECT g.target_paise, g.target_date, COALESCE((SELECT SUM(amount_paise) FROM couple_goal_contributions c WHERE c.goal_id = g.goal_id), 0)
             FROM couple_goals g WHERE g.couple_id = ? AND g.archived = 0",
        )
        .bind(&couple)
        .fetch_all(db)
        .await?;
        let limit = today + Duration::days(GOAL_WINDOW_DAYS);
        for (target, date, saved) in rows {
            let Some(date) = date.as_deref().and_then(|d| d.get(..10)).and_then(|d| Date::parse(d, time::macros::format_description!("[year]-[month]-[day]")).ok()) else {
                continue;
            };
            if date <= limit && percent(target, saved) < GOAL_FUNDED_PCT {
                goals_behind += 1;
            }
        }
    }

    let soon_end = today + Duration::days(SOON_DAYS);
    let large_amounts_soon = lumpy_for(db, user_id)
        .await?
        .iter()
        .filter(|l| l.direction == Direction::Out)
        .filter(|l| next_occurrence(l, today).is_some_and(|d| d <= soon_end))
        .count();

    let coming = coming::gather(db, user_id, today, SOON_DAYS).await?;
    let birthdays_soon = coming.iter().filter(|e| e.item.kind == ComingKind::Birthday && e.days_away <= BIRTHDAY_DAYS).count();
    let followups_overdue = coming.iter().filter(|e| e.item.kind == ComingKind::FollowUp && e.days_away < 0).count();

    Ok(Counts {
        uncategorized: uncategorized as usize,
        goals_behind,
        large_amounts_soon,
        milestones_soon: 0,
        birthdays_soon,
        followups_overdue,
    })
}

pub async fn get(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<DecisionsOut>, AppError> {
    let c = counts(&state.db, &user_id, today_local()).await?;
    Ok(Json(DecisionsOut { items: decisions(&c) }))
}
