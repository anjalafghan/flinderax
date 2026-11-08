//! Goals and sinking funds. `GET /api/goals` plans this month: the savings left after the SIP
//! (from the same budget Safe to Spend uses) shared out by priority, with fixes for any goal
//! that's behind, plus how many months of needs your savings cover.

use axum::{extract::State, Extension, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    validate,
    handlers::{
        account::ensure_account_owned,
        autopost,
        common::{bad_request, not_found, today_local, AppError},
        dashboard::load_input,
    },
    models::AppState,
    planner::{
        self,
        goals::{months_covered_tenths, plan, GoalIn, GoalPlan},
        format_inr, Role,
    },
};

type User = Extension<(String, String)>;

/// The emergency fund suggested when you don't have one: this many months of needs.
const EMERGENCY_MONTHS: i64 = 6;

fn parse_date(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

/// A goal's plan plus what the edit form needs.
#[derive(Serialize)]
pub struct GoalOut {
    #[serde(flatten)]
    pub plan: GoalPlan,
    pub kind: String,
    pub target_date: Option<String>,
    pub priority: i64,
    pub account_id: Option<String>,
}

#[derive(Serialize)]
pub struct GoalsOut {
    pub goals: Vec<GoalOut>,
    /// Savings after the SIP this month: what the goals share.
    pub available_paise: i64,
    pub unallocated_paise: i64,
    /// Reserve + locked savings, in months of needs (tenths: 42 = 4.2).
    pub months_covered_tenths: Option<i64>,
    /// Shown when there's no emergency goal yet: 6 months of needs.
    pub emergency_suggested_paise: Option<i64>,
}

pub async fn list(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<GoalsOut>, AppError> {
    let db = &state.db;
    let today = today_local();
    autopost::apply(db, &user_id, today).await?;
    let input = load_input(db, &user_id, today).await?;
    let summary = planner::compute(&input);
    let available = summary.budget.rest_savings_paise;
    let needs = summary.budget.needs_paise;

    let rows = sqlx::query!(
        r#"SELECT g.goal_id AS "goal_id!", g.name, g.kind, g.target_paise, g.target_date, g.priority, g.account_id,
                  COALESCE((SELECT SUM(amount_paise) FROM goal_contributions c WHERE c.goal_id = g.goal_id), 0) AS "saved!: i64"
           FROM goals g WHERE g.user_id = ? AND g.archived = 0"#,
        user_id
    )
    .fetch_all(db)
    .await?;
    let has_emergency = rows.iter().any(|r| r.kind == "emergency");
    let meta: std::collections::HashMap<String, (String, Option<String>, i64, Option<String>)> = rows
        .iter()
        .map(|r| (r.goal_id.clone(), (r.kind.clone(), r.target_date.clone(), r.priority, r.account_id.clone())))
        .collect();
    let goals: Vec<GoalIn> = rows
        .into_iter()
        .map(|r| GoalIn {
            by: r.target_date.as_deref().and_then(parse_date),
            id: r.goal_id,
            name: r.name,
            target: r.target_paise,
            saved: r.saved,
            priority: r.priority,
        })
        .collect();
    let planned = plan(&goals, available, today, format_inr);
    let used: i64 = planned.iter().map(|p| p.this_month_paise).sum();

    let cushion: i64 = input
        .accounts
        .iter()
        .filter(|a| a.locked || a.role == Role::Reserve)
        .map(|a| a.balance)
        .sum();
    let goals = planned
        .into_iter()
        .map(|plan| {
            let (kind, target_date, priority, account_id) = meta.get(&plan.id).cloned().unwrap_or_default();
            GoalOut { plan, kind, target_date, priority, account_id }
        })
        .collect();
    Ok(Json(GoalsOut {
        goals,
        available_paise: available,
        unallocated_paise: (available - used).max(0),
        months_covered_tenths: months_covered_tenths(cushion, needs),
        emergency_suggested_paise: (!has_emergency && needs > 0).then(|| needs * EMERGENCY_MONTHS),
    }))
}

#[derive(Deserialize)]
pub struct GoalPayload {
    /// Required for update.
    pub goal_id: Option<String>,
    pub name: String,
    /// emergency | custom (default)
    pub kind: Option<String>,
    pub target_paise: i64,
    pub target_date: Option<String>,
    pub priority: Option<i64>,
    pub account_id: Option<String>,
}

#[derive(Serialize)]
pub struct GoalId {
    pub goal_id: String,
    pub status: bool,
}

fn validate(p: &GoalPayload) -> Result<(), AppError> {
    if p.name.trim().is_empty() {
        return Err(bad_request("Give the goal a name"));
    }
    validate::positive_amount(p.target_paise).map_err(|e| bad_request(format!("The target: {e}")))?;
    if p.kind.as_deref().is_some_and(|k| !matches!(k, "emergency" | "custom")) {
        return Err(bad_request("kind must be emergency or custom"));
    }
    if let Some(d) = p.target_date.as_deref().filter(|d| !d.is_empty()) {
        parse_date(d).ok_or_else(|| bad_request("target_date must look like 2027-04-01"))?;
    }
    Ok(())
}

pub async fn create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<GoalPayload>,
) -> Result<Json<GoalId>, AppError> {
    validate(&p)?;
    ensure_account_owned(&state.db, &p.account_id, &user_id).await?;
    let goal_id = nanoid!();
    let kind = p.kind.unwrap_or_else(|| "custom".into());
    let date = p.target_date.filter(|d| !d.is_empty());
    let priority = p.priority.unwrap_or(1);
    let name = p.name.trim().to_string();
    sqlx::query!(
        "INSERT INTO goals (goal_id, user_id, name, kind, target_paise, target_date, priority, account_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        goal_id,
        user_id,
        name,
        kind,
        p.target_paise,
        date,
        priority,
        p.account_id
    )
    .execute(&state.db)
    .await?;
    Ok(Json(GoalId { goal_id, status: true }))
}

pub async fn update(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<GoalPayload>,
) -> Result<Json<GoalId>, AppError> {
    validate(&p)?;
    ensure_account_owned(&state.db, &p.account_id, &user_id).await?;
    let goal_id = p.goal_id.clone().ok_or_else(|| bad_request("goal_id is required"))?;
    let kind = p.kind.unwrap_or_else(|| "custom".into());
    let date = p.target_date.filter(|d| !d.is_empty());
    let priority = p.priority.unwrap_or(1);
    let name = p.name.trim().to_string();
    let res = sqlx::query!(
        "UPDATE goals SET name = ?, kind = ?, target_paise = ?, target_date = ?, priority = ?, account_id = ?
         WHERE goal_id = ? AND user_id = ?",
        name,
        kind,
        p.target_paise,
        date,
        priority,
        p.account_id,
        goal_id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Goal not found"));
    }
    Ok(Json(GoalId { goal_id, status: true }))
}

#[derive(Deserialize)]
pub struct GoalRef {
    pub goal_id: String,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

/// Archives a goal (its history stays); `archive` again with the same id is a no-op.
pub async fn archive(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<GoalRef>,
) -> Result<Json<Done>, AppError> {
    let res = sqlx::query!("UPDATE goals SET archived = 1 WHERE goal_id = ? AND user_id = ?", p.goal_id, user_id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Goal not found"));
    }
    Ok(Json(Done { status: true }))
}

#[derive(Deserialize)]
pub struct ContributePayload {
    pub goal_id: String,
    /// Negative takes money back out of the goal.
    pub amount_paise: i64,
    pub note: Option<String>,
}

/// Logs money put towards (or taken from) a goal. Moving the money in the bank is up to you;
/// this only records it.
pub async fn contribute(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<ContributePayload>,
) -> Result<Json<Done>, AppError> {
    if p.amount_paise == 0 {
        return Err(bad_request("The amount can't be zero"));
    }
    validate::bounded_amount(p.amount_paise).map_err(|e| bad_request(format!("The amount: {e}")))?;
    let owned = sqlx::query_scalar!("SELECT goal_id FROM goals WHERE goal_id = ? AND user_id = ?", p.goal_id, user_id)
        .fetch_optional(&state.db)
        .await?;
    if owned.is_none() {
        return Err(not_found("Goal not found"));
    }
    let today = today_local().to_string();
    let note = p.note.unwrap_or_default();
    sqlx::query!(
        "INSERT INTO goal_contributions (goal_id, amount_paise, on_day, note) VALUES (?, ?, ?, ?)",
        p.goal_id,
        p.amount_paise,
        today,
        note
    )
    .execute(&state.db)
    .await?;
    Ok(Json(Done { status: true }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(target: i64) -> GoalPayload {
        GoalPayload {
            goal_id: None,
            name: "Trip".into(),
            kind: None,
            target_paise: target,
            target_date: Some("2027-04-01".into()),
            priority: None,
            account_id: None,
        }
    }

    #[test]
    fn a_normal_target_passes() {
        assert!(validate(&payload(6_000_000)).is_ok());
    }

    #[test]
    fn an_absurd_target_is_rejected() {
        assert!(validate(&payload(i64::MAX)).is_err());
        assert!(validate(&payload(0)).is_err());
        assert!(validate(&payload(-5)).is_err());
    }
}
