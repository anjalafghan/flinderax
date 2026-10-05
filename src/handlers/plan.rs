//! Recurring commitments (SIPs, rent, RD instalments...), income sources and planner settings.

use axum::{extract::State, Extension, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    handlers::{
        account::ensure_account_owned,
        common::{bad_request, not_found, validate_day, validate_year_month, AppError},
    },
    models::AppState,
};

type User = Extension<(String, String)>;

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: bool,
}

#[derive(Serialize)]
pub struct IdResponse {
    pub id: String,
    pub status: bool,
}

// ---------- commitments ----------

#[derive(Serialize)]
pub struct CommitmentRow {
    pub commitment_id: String,
    pub label: String,
    pub amount_paise: i64,
    pub day_of_month: i64,
    pub account_id: Option<String>,
    pub category: String,
    pub active: bool,
    pub start_month: Option<String>,
    pub end_month: Option<String>,
}

#[derive(Deserialize)]
pub struct CommitmentPayload {
    /// Required for update, ignored for create.
    pub commitment_id: Option<String>,
    pub label: String,
    pub amount_paise: i64,
    pub day_of_month: i64,
    pub account_id: Option<String>,
    pub category: Option<String>,
    pub active: Option<bool>,
    pub start_month: Option<String>,
    pub end_month: Option<String>,
}

#[derive(Deserialize)]
pub struct IdPayload {
    pub id: String,
}

fn validate_commitment(p: &CommitmentPayload) -> Result<(), AppError> {
    if p.label.trim().is_empty() {
        return Err(bad_request("label is required"));
    }
    if p.amount_paise <= 0 {
        return Err(bad_request("amount_paise must be positive"));
    }
    validate_day(p.day_of_month, "day_of_month")?;
    validate_year_month(&p.start_month, "start_month")?;
    validate_year_month(&p.end_month, "end_month")
}

pub async fn commitment_create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<CommitmentPayload>,
) -> Result<Json<IdResponse>, AppError> {
    validate_commitment(&p)?;
    ensure_account_owned(&state.db, &p.account_id, &user_id).await?;
    let id = nanoid!();
    let category = p.category.unwrap_or_else(|| "other".into());
    let active = p.active.unwrap_or(true);
    sqlx::query!(
        "INSERT INTO commitments (commitment_id, user_id, label, amount_paise, day_of_month, account_id, category, active, start_month, end_month)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        id,
        user_id,
        p.label,
        p.amount_paise,
        p.day_of_month,
        p.account_id,
        category,
        active,
        p.start_month,
        p.end_month
    )
    .execute(&state.db)
    .await?;
    Ok(Json(IdResponse { id, status: true }))
}

pub async fn commitment_update(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<CommitmentPayload>,
) -> Result<Json<StatusResponse>, AppError> {
    validate_commitment(&p)?;
    let id = p.commitment_id.clone().ok_or_else(|| bad_request("commitment_id is required"))?;
    ensure_account_owned(&state.db, &p.account_id, &user_id).await?;
    let category = p.category.unwrap_or_else(|| "other".into());
    let active = p.active.unwrap_or(true);
    let res = sqlx::query!(
        "UPDATE commitments SET label = ?, amount_paise = ?, day_of_month = ?, account_id = ?, category = ?,
                active = ?, start_month = ?, end_month = ?
         WHERE commitment_id = ? AND user_id = ?",
        p.label,
        p.amount_paise,
        p.day_of_month,
        p.account_id,
        category,
        active,
        p.start_month,
        p.end_month,
        id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Commitment not found"));
    }
    Ok(Json(StatusResponse { status: true }))
}

pub async fn commitment_delete(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<IdPayload>,
) -> Result<Json<StatusResponse>, AppError> {
    let res = sqlx::query!(
        "DELETE FROM commitments WHERE commitment_id = ? AND user_id = ?",
        p.id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Commitment not found"));
    }
    Ok(Json(StatusResponse { status: true }))
}

pub async fn commitment_list(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
) -> Result<Json<Vec<CommitmentRow>>, AppError> {
    let rows = sqlx::query_as!(
        CommitmentRow,
        r#"SELECT commitment_id AS "commitment_id!", label, amount_paise, day_of_month, account_id,
                  category, active AS "active: bool", start_month, end_month
           FROM commitments WHERE user_id = ? ORDER BY day_of_month, rowid"#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

// ---------- income ----------

#[derive(Serialize)]
pub struct IncomeRow {
    pub income_id: String,
    pub label: String,
    pub amount_paise: i64,
    pub day_of_month: i64,
    pub account_id: Option<String>,
}

#[derive(Deserialize)]
pub struct IncomePayload {
    /// Required for update, ignored for create.
    pub income_id: Option<String>,
    pub label: String,
    pub amount_paise: i64,
    pub day_of_month: i64,
    pub account_id: Option<String>,
}

fn validate_income(p: &IncomePayload) -> Result<(), AppError> {
    if p.label.trim().is_empty() {
        return Err(bad_request("label is required"));
    }
    if p.amount_paise <= 0 {
        return Err(bad_request("amount_paise must be positive"));
    }
    validate_day(p.day_of_month, "day_of_month")
}

pub async fn income_create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<IncomePayload>,
) -> Result<Json<IdResponse>, AppError> {
    validate_income(&p)?;
    ensure_account_owned(&state.db, &p.account_id, &user_id).await?;
    let id = nanoid!();
    sqlx::query!(
        "INSERT INTO income_sources (income_id, user_id, label, amount_paise, day_of_month, account_id) VALUES (?, ?, ?, ?, ?, ?)",
        id,
        user_id,
        p.label,
        p.amount_paise,
        p.day_of_month,
        p.account_id
    )
    .execute(&state.db)
    .await?;
    Ok(Json(IdResponse { id, status: true }))
}

pub async fn income_update(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<IncomePayload>,
) -> Result<Json<StatusResponse>, AppError> {
    validate_income(&p)?;
    let id = p.income_id.clone().ok_or_else(|| bad_request("income_id is required"))?;
    ensure_account_owned(&state.db, &p.account_id, &user_id).await?;
    let res = sqlx::query!(
        "UPDATE income_sources SET label = ?, amount_paise = ?, day_of_month = ?, account_id = ?
         WHERE income_id = ? AND user_id = ?",
        p.label,
        p.amount_paise,
        p.day_of_month,
        p.account_id,
        id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Income source not found"));
    }
    Ok(Json(StatusResponse { status: true }))
}

pub async fn income_delete(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<IdPayload>,
) -> Result<Json<StatusResponse>, AppError> {
    let res = sqlx::query!(
        "DELETE FROM income_sources WHERE income_id = ? AND user_id = ?",
        p.id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Income source not found"));
    }
    Ok(Json(StatusResponse { status: true }))
}

pub async fn income_list(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
) -> Result<Json<Vec<IncomeRow>>, AppError> {
    let rows = sqlx::query_as!(
        IncomeRow,
        r#"SELECT income_id AS "income_id!", label, amount_paise, day_of_month, account_id
           FROM income_sources WHERE user_id = ? ORDER BY day_of_month, rowid"#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

// ---------- settings ----------

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Settings {
    pub cushion_paise: i64,
    pub next_period_living_paise: i64,
    /// 0 = Sunday .. 6 = Saturday
    pub week_start: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { cushion_paise: 500_000, next_period_living_paise: 1_800_000, week_start: 1 }
    }
}

pub async fn load_settings(db: &sqlx::SqlitePool, user_id: &str) -> Result<Settings, AppError> {
    let row = sqlx::query!(
        "SELECT cushion_paise, next_period_living_paise, week_start FROM user_settings WHERE user_id = ?",
        user_id
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map_or_else(Settings::default, |r| Settings {
        cushion_paise: r.cushion_paise,
        next_period_living_paise: r.next_period_living_paise,
        week_start: r.week_start,
    }))
}

pub async fn settings_get(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
) -> Result<Json<Settings>, AppError> {
    Ok(Json(load_settings(&state.db, &user_id).await?))
}

pub async fn settings_update(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<Settings>,
) -> Result<Json<StatusResponse>, AppError> {
    if p.cushion_paise < 0 || p.next_period_living_paise < 0 {
        return Err(bad_request("amounts cannot be negative"));
    }
    if !(0..=6).contains(&p.week_start) {
        return Err(bad_request("week_start must be between 0 and 6"));
    }
    sqlx::query!(
        "INSERT INTO user_settings (user_id, cushion_paise, next_period_living_paise, week_start) VALUES (?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET cushion_paise = excluded.cushion_paise,
            next_period_living_paise = excluded.next_period_living_paise, week_start = excluded.week_start",
        user_id,
        p.cushion_paise,
        p.next_period_living_paise,
        p.week_start
    )
    .execute(&state.db)
    .await?;
    Ok(Json(StatusResponse { status: true }))
}
