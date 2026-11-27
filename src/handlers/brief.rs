//! A compact, safe summary for an assistant to fetch once a day.

use axum::{Extension, Json, extract::State};
use serde::Serialize;
use time::{Date, Duration};

use crate::{
    alerts::is_spending,
    handlers::{
        common::{AppError, today_local},
        couple::couple_of,
        dashboard::load_input,
        reward_fund,
    },
    models::AppState,
    planner::{self, bills::{BillKind, calendar}},
};

type User = Extension<(String, String)>;

#[derive(Serialize)]
pub struct DueItem {
    pub due_on: Date,
    pub title: String,
    pub amount_paise: i64,
    pub kind: BillKind,
}

#[derive(Serialize)]
pub struct GoalPace {
    pub target_paise: i64,
    pub saved_paise: i64,
    pub monthly_needed_paise: Option<i64>,
}

#[derive(Serialize)]
pub struct KeptDate {
    pub day: String,
    pub title: String,
    pub sealed: bool,
}

#[derive(Serialize)]
pub struct Brief {
    pub as_of: Date,
    pub due_next_7_days: Vec<DueItem>,
    pub safe_to_spend_this_month_paise: i64,
    pub money_allowance_this_month_paise: i64,
    pub money_spent_this_month_paise: i64,
    pub dates_budget_this_month_paise: Option<i64>,
    pub dates_spent_this_month_paise: i64,
    pub reward_fund_score_bps: i64,
    pub wedding_fund: Option<GoalPace>,
    pub emergency_fund: Option<GoalPace>,
    pub unread_bank_text_count: i64,
    pub next_kept_date: Option<KeptDate>,
}

async fn wedding_fund(db: &sqlx::SqlitePool, couple: Option<&str>, today: Date) -> Result<Option<GoalPace>, AppError> {
    let Some(couple) = couple else { return Ok(None) };
    let row: Option<(i64, Option<String>, i64)> = sqlx::query_as(
        "SELECT g.target_paise, g.target_date, COALESCE(SUM(c.amount_paise), 0)
         FROM couple_goals g LEFT JOIN couple_goal_contributions c ON c.goal_id = g.goal_id
         WHERE g.couple_id = ? AND g.archived = 0 AND lower(g.name) LIKE '%wedding%'
         GROUP BY g.goal_id ORDER BY g.created_at LIMIT 1",
    )
    .bind(couple)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(target, date, saved)| GoalPace {
        target_paise: target,
        saved_paise: saved,
        monthly_needed_paise: planner::couple_goals::monthly_needed(target, saved, today, date.as_deref().and_then(crate::handlers::couple_goals::parse_date)),
    }))
}
async fn emergency_fund(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<Option<GoalPace>, AppError> {
    let row: Option<(i64, Option<String>, i64)> = sqlx::query_as(
        "SELECT g.target_paise, g.target_date,
                COALESCE((SELECT SUM(amount_paise) FROM goal_contributions c WHERE c.goal_id = g.goal_id), 0)
         FROM goals g WHERE g.user_id = ? AND g.archived = 0 AND g.kind = 'emergency'
         ORDER BY g.priority, g.created_at LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(target, date, saved)| GoalPace {
        target_paise: target,
        saved_paise: saved,
        monthly_needed_paise: planner::couple_goals::monthly_needed(target, saved, today, date.as_deref().and_then(crate::handlers::couple_goals::parse_date)),
    }))
}

async fn kept_on(db: &sqlx::SqlitePool, couple: &str, who: usize, today: Date) -> Result<Option<KeptDate>, AppError> {
    let row: Option<(String, String, bool)> = sqlx::query_as(
        "SELECT date(COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime'),
                CASE WHEN h.surprise = 1 AND h.revealed = 0 AND COALESCE(h.drawn_by, -1) <> ?2 THEN '' ELSE a.title END,
                (h.surprise = 1 AND h.revealed = 0 AND COALESCE(h.drawn_by, -1) <> ?2)
         FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ?1 AND h.outcome = 'did_it'
           AND date(COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime') >= ?3
         ORDER BY COALESCE(h.planned_at, h.picked_at), h.id LIMIT 1",
    )
    .bind(couple)
    .bind(who as i64)
    .bind(today.to_string())
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(day, title, sealed)| KeptDate { day, title, sealed }))
}

pub async fn get(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Brief>, AppError> {
    let today = today_local();
    let input = load_input(&state.db, &user_id, today).await?;
    let summary = planner::compute(&input);
    let month_end = {
        let (year, month) = planner::cycle::add_months(today.year(), today.month(), 1);
        format!("{year}-{:02}-01", month as u8)
    };
    // Use the same spending categories as the reward planner, not transfers or income.
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT category, amount_paise FROM transactions
         WHERE user_id = ? AND direction = 'debit' AND occurred_on >= ? AND occurred_on < ?",
    )
    .bind(&user_id)
    .bind(format!("{}-{:02}-01", today.year(), today.month() as u8))
    .bind(month_end)
    .fetch_all(&state.db)
    .await?;
    let spent = rows.into_iter().filter(|(category, _)| is_spending(category)).map(|(_, amount)| amount).sum::<i64>();
    let reward = reward_fund::compute(&state, &user_id, today).await?;
    let due = calendar(&input, today, today + Duration::days(8))
        .into_iter()
        .filter(|b| b.kind != BillKind::Date)
        .map(|b| DueItem { due_on: b.date, title: b.label, amount_paise: b.amount_paise - b.covered_paise, kind: b.kind })
        .collect();
    let couple = couple_of(&state.db, &user_id).await?;
    let (dates_budget, dates_spent) = match couple.as_ref() {
        Some((id, _)) => {
            let spent = crate::joy::db::spent_this_month(&state.db, id).await?;
            let month = crate::joy::db::month_offset(&state.db, 0).await?;
            let standing = crate::joy::db::setting(&state.db, id, "monthly_budget").await?.and_then(|v| v.parse::<i64>().ok()).filter(|v| *v > 0);
            let budget = if let Some(base) = standing {
                let carry = crate::joy::db::month_budget(&state.db, id, &month).await?.map(|(_, c)| c).unwrap_or(0);
                Some(base.saturating_add(carry))
            } else { None };
            (budget, spent)
        }
        None => (None, 0),
    };
    let next_kept_date = match couple.as_ref() {
        Some((id, who)) => kept_on(&state.db, id, *who, today).await?,
        None => None,
    };
    let unread: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM unrecognized_alerts WHERE user_id = ? AND received_at >= datetime('now', '-30 days')",
    )
    .bind(&user_id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(Brief {
        as_of: today,
        due_next_7_days: due,
        safe_to_spend_this_month_paise: summary.budget.spending_paise.saturating_sub(spent).max(0),
        money_allowance_this_month_paise: summary.budget.spending_paise,
        money_spent_this_month_paise: spent,
        dates_budget_this_month_paise: dates_budget,
        dates_spent_this_month_paise: dates_spent,
        reward_fund_score_bps: reward.this_month.score.score_bps,
        wedding_fund: wedding_fund(&state.db, couple.as_ref().map(|c| c.0.as_str()), today).await?,
        emergency_fund: emergency_fund(&state.db, &user_id, today).await?,
        unread_bank_text_count: unread,
        next_kept_date,
    }))
}
