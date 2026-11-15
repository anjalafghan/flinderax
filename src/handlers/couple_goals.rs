//! `/api/couple-goals`: goals a couple agrees on and both pay into. Either partner can add or
//! archive one; each payment is attributed to whoever made it, and only its maker can take it back.

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    handlers::{
        common::{bad_request, not_found, today_local, AppError},
        couple::{couple_of, require_couple},
    },
    models::AppState,
    planner::couple_goals::{monthly_needed, percent, Share},
    validate,
};

type User = Extension<(String, String)>;

fn parse_date(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

#[derive(Serialize)]
pub struct ContributionOut {
    pub id: i64,
    pub name: String,
    pub amount_paise: i64,
    pub on_day: String,
    pub note: String,
    /// Yours: only you can take it back.
    pub mine: bool,
}

#[derive(Serialize)]
pub struct GoalOut {
    pub goal_id: String,
    pub name: String,
    pub target_paise: i64,
    pub target_date: Option<String>,
    pub saved_paise: i64,
    pub percent: i64,
    /// What to put aside each month to make the date; none without one.
    pub monthly_needed_paise: Option<i64>,
    pub shares: Vec<Share>,
    pub contributions: Vec<ContributionOut>,
}

#[derive(Serialize)]
pub struct GoalsOut {
    pub in_couple: bool,
    pub goals: Vec<GoalOut>,
}

/// `GET /api/couple-goals`
pub async fn list(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<GoalsOut>, AppError> {
    let Some((couple, _)) = couple_of(&state.db, &user_id).await? else {
        return Ok(Json(GoalsOut { in_couple: false, goals: vec![] }));
    };
    let today = today_local();
    let rows: Vec<(String, String, i64, Option<String>)> = sqlx::query_as(
        "SELECT goal_id, name, target_paise, target_date FROM couple_goals WHERE couple_id = ? AND archived = 0 ORDER BY created_at, goal_id",
    )
    .bind(&couple)
    .fetch_all(&state.db)
    .await?;
    let mut goals = Vec::new();
    for (goal_id, name, target, date) in rows {
        let pays: Vec<(i64, Option<String>, String, i64, String, String)> = sqlx::query_as(
            "SELECT id, user_id, name, amount_paise, on_day, note FROM couple_goal_contributions WHERE goal_id = ? ORDER BY on_day DESC, id DESC",
        )
        .bind(&goal_id)
        .fetch_all(&state.db)
        .await?;
        let saved: i64 = pays.iter().map(|p| p.3).sum();
        let mut shares: Vec<Share> = Vec::new();
        for (_, _, who, amount, _, _) in &pays {
            match shares.iter_mut().find(|s| &s.name == who) {
                Some(s) => s.paise += amount,
                None => shares.push(Share { name: who.clone(), paise: *amount }),
            }
        }
        goals.push(GoalOut {
            monthly_needed_paise: monthly_needed(target, saved, today, date.as_deref().and_then(parse_date)),
            percent: percent(target, saved),
            saved_paise: saved,
            shares,
            contributions: pays
                .into_iter()
                .map(|(id, uid, name, amount_paise, on_day, note)| ContributionOut { id, mine: uid.as_deref() == Some(user_id.as_str()), name, amount_paise, on_day, note })
                .collect(),
            goal_id,
            name,
            target_paise: target,
            target_date: date,
        });
    }
    Ok(Json(GoalsOut { in_couple: true, goals }))
}

#[derive(Deserialize)]
pub struct NewGoal {
    pub name: String,
    pub target_paise: i64,
    pub target_date: Option<String>,
}

#[derive(Serialize)]
pub struct GoalId {
    pub goal_id: String,
    pub status: bool,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

/// `POST /api/couple-goals`
pub async fn create(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<NewGoal>) -> Result<Json<GoalId>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    let name = p.name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(bad_request("Give the goal a name of up to 80 characters"));
    }
    validate::positive_amount(p.target_paise).map_err(|e| bad_request(format!("The target: {e}")))?;
    let date = p.target_date.filter(|d| !d.is_empty());
    if date.as_deref().is_some_and(|d| parse_date(d).is_none()) {
        return Err(bad_request("target_date must look like 2027-04-01"));
    }
    let goal_id = nanoid!();
    sqlx::query("INSERT INTO couple_goals (goal_id, couple_id, name, target_paise, target_date, created_by) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&goal_id)
        .bind(&couple)
        .bind(name)
        .bind(p.target_paise)
        .bind(date)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(GoalId { goal_id, status: true }))
}

/// A goal of this couple, or a 404.
async fn mine(db: &sqlx::SqlitePool, couple: &str, goal_id: &str) -> Result<(), AppError> {
    let found: Option<String> = sqlx::query_scalar("SELECT goal_id FROM couple_goals WHERE goal_id = ? AND couple_id = ? AND archived = 0")
        .bind(goal_id)
        .bind(couple)
        .fetch_optional(db)
        .await?;
    found.map(|_| ()).ok_or_else(|| not_found("Goal not found"))
}

#[derive(Deserialize)]
pub struct Contribute {
    pub amount_paise: i64,
    #[serde(default)]
    pub note: String,
}

/// `POST /api/couple-goals/{id}/contribute`: put money towards it, in your own name.
pub async fn contribute(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(goal_id): Path<String>,
    Json(p): Json<Contribute>,
) -> Result<Json<Done>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    validate::positive_amount(p.amount_paise).map_err(|e| bad_request(format!("The amount: {e}")))?;
    let note = p.note.trim();
    if note.chars().count() > 200 {
        return Err(bad_request("Keep the note under 200 characters"));
    }
    let name: String = sqlx::query_scalar("SELECT user_name FROM users WHERE user_id = ?").bind(&user_id).fetch_one(&state.db).await?;
    sqlx::query("INSERT INTO couple_goal_contributions (goal_id, user_id, name, amount_paise, on_day, note) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&goal_id)
        .bind(&user_id)
        .bind(name)
        .bind(p.amount_paise)
        .bind(today_local().to_string())
        .bind(note)
        .execute(&state.db)
        .await?;
    Ok(Json(Done { status: true }))
}

/// `POST /api/couple-goals/{id}/contributions/{cid}/delete`: take back one of your own payments.
pub async fn delete_contribution(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path((goal_id, cid)): Path<(String, i64)>,
) -> Result<Json<Done>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    let done = sqlx::query("DELETE FROM couple_goal_contributions WHERE id = ? AND goal_id = ? AND user_id = ?")
        .bind(cid)
        .bind(&goal_id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    if done.rows_affected() == 0 {
        return Err(not_found("Payment not found"));
    }
    Ok(Json(Done { status: true }))
}

/// `POST /api/couple-goals/{id}/archive`: either of you can put it away.
pub async fn archive(State(state): State<AppState>, Extension((user_id, _)): User, Path(goal_id): Path<String>) -> Result<Json<Done>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    sqlx::query("UPDATE couple_goals SET archived = 1 WHERE goal_id = ? AND couple_id = ?").bind(&goal_id).bind(&couple).execute(&state.db).await?;
    Ok(Json(Done { status: true }))
}
