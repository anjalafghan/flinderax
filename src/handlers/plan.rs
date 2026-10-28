//! Recurring commitments (SIPs, rent, RD instalments...), income sources and planner settings.

use axum::{extract::State, Extension, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    planner::{default_bucket, Bucket},
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
    /// needs | savings | wants
    pub bucket: String,
    /// monthly | yearly
    pub frequency: String,
    /// 1-12, for yearly bills (None = month not known yet)
    pub due_month: Option<i64>,
    /// Where the money ends up when it is a transfer to savings (an RD, an FD top-up...).
    pub to_account_id: Option<String>,
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
    /// Defaults from the category when omitted.
    pub bucket: Option<String>,
    pub frequency: Option<String>,
    pub due_month: Option<i64>,
    pub to_account_id: Option<String>,
    pub active: Option<bool>,
    pub start_month: Option<String>,
    pub end_month: Option<String>,
}

#[derive(Deserialize)]
pub struct IdPayload {
    pub id: String,
}

/// Explicit bucket, else the default for the category.
fn bucket_for(bucket: &Option<String>, category: &str) -> String {
    match bucket.as_deref().and_then(Bucket::parse) {
        Some(Bucket::Needs) => "needs",
        Some(Bucket::Savings) => "savings",
        Some(Bucket::Wants) => "wants",
        None => match default_bucket(category) {
            Bucket::Needs => "needs",
            Bucket::Savings => "savings",
            Bucket::Wants => "wants",
        },
    }
    .to_string()
}

fn validate_commitment(p: &CommitmentPayload) -> Result<(), AppError> {
    if p.label.trim().is_empty() {
        return Err(bad_request("label is required"));
    }
    if p.amount_paise <= 0 {
        return Err(bad_request("amount_paise must be positive"));
    }
    validate_day(p.day_of_month, "day_of_month")?;
    if p.bucket.as_deref().is_some_and(|b| Bucket::parse(b).is_none()) {
        return Err(bad_request("bucket must be needs, savings or wants"));
    }
    if p.frequency.as_deref().is_some_and(|f| !matches!(f, "monthly" | "yearly")) {
        return Err(bad_request("frequency must be monthly or yearly"));
    }
    if p.due_month.is_some_and(|m| !(1..=12).contains(&m)) {
        return Err(bad_request("due_month must be between 1 and 12"));
    }
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
    ensure_account_owned(&state.db, &p.to_account_id, &user_id).await?;
    let id = nanoid!();
    let frequency = p.frequency.clone().unwrap_or_else(|| "monthly".into());
    let category = p.category.unwrap_or_else(|| "other".into());
    let bucket = bucket_for(&p.bucket, &category);
    let active = p.active.unwrap_or(true);
    sqlx::query!(
        "INSERT INTO commitments (commitment_id, user_id, label, amount_paise, day_of_month, account_id, category, bucket, frequency, due_month, to_account_id, active, start_month, end_month)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        id,
        user_id,
        p.label,
        p.amount_paise,
        p.day_of_month,
        p.account_id,
        category,
        bucket,
        frequency,
        p.due_month,
        p.to_account_id,
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
    ensure_account_owned(&state.db, &p.to_account_id, &user_id).await?;
    let frequency = p.frequency.clone().unwrap_or_else(|| "monthly".into());
    let category = p.category.unwrap_or_else(|| "other".into());
    let bucket = bucket_for(&p.bucket, &category);
    let active = p.active.unwrap_or(true);
    let res = sqlx::query!(
        "UPDATE commitments SET label = ?, amount_paise = ?, day_of_month = ?, account_id = ?, category = ?,
                bucket = ?, frequency = ?, due_month = ?, to_account_id = ?, active = ?, start_month = ?, end_month = ?
         WHERE commitment_id = ? AND user_id = ?",
        p.label,
        p.amount_paise,
        p.day_of_month,
        p.account_id,
        category,
        bucket,
        frequency,
        p.due_month,
        p.to_account_id,
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
                  category, bucket, frequency, due_month, to_account_id, active AS "active: bool", start_month, end_month
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
    /// Monthly spending allowance as an amount. Set this or `spending_bps`, not both.
    pub spending_paise: Option<i64>,
    /// Monthly spending allowance as a share of income in basis points (2500 = 25%).
    /// Neither set = 20% of income.
    pub spending_bps: Option<i64>,
    /// Day of the month the savings and needs money should have reached the reserve account.
    pub allocation_day: i64,
    /// Savings in the reserve account that card bills must not eat. `null` = no floor.
    pub savings_floor_paise: Option<i64>,
    /// Pay periods over which an overspend is paid back.
    pub recovery_months: i64,
    /// 0 = Sunday .. 6 = Saturday
    pub week_start: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { spending_paise: None, spending_bps: None, allocation_day: 10, savings_floor_paise: None, recovery_months: 3, week_start: 1 }
    }
}

pub async fn load_settings(db: &sqlx::SqlitePool, user_id: &str) -> Result<Settings, AppError> {
    let row = sqlx::query!(
        "SELECT spending_paise, spending_bps, allocation_day, savings_floor_paise, recovery_months, week_start
         FROM user_settings WHERE user_id = ?",
        user_id
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map_or_else(Settings::default, |r| Settings {
        spending_paise: r.spending_paise,
        spending_bps: r.spending_bps,
        allocation_day: r.allocation_day,
        savings_floor_paise: r.savings_floor_paise,
        recovery_months: r.recovery_months,
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
    if p.spending_paise.is_some_and(|v| v < 0) || p.savings_floor_paise.is_some_and(|v| v < 0) {
        return Err(bad_request("amounts cannot be negative"));
    }
    if p.spending_paise.is_some() && p.spending_bps.is_some() {
        return Err(bad_request("set either spending_paise or spending_bps, not both"));
    }
    if p.spending_bps.is_some_and(|b| !(0..=10_000).contains(&b)) {
        return Err(bad_request("spending_bps must be between 0 and 10000"));
    }
    validate_day(p.allocation_day, "allocation_day")?;
    if !(1..=24).contains(&p.recovery_months) {
        return Err(bad_request("recovery_months must be between 1 and 24"));
    }
    if !(0..=6).contains(&p.week_start) {
        return Err(bad_request("week_start must be between 0 and 6"));
    }
    sqlx::query!(
        "INSERT INTO user_settings (user_id, spending_paise, spending_bps, allocation_day, savings_floor_paise, recovery_months, week_start)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET spending_paise = excluded.spending_paise, spending_bps = excluded.spending_bps,
            allocation_day = excluded.allocation_day, savings_floor_paise = excluded.savings_floor_paise,
            recovery_months = excluded.recovery_months, week_start = excluded.week_start",
        user_id,
        p.spending_paise,
        p.spending_bps,
        p.allocation_day,
        p.savings_floor_paise,
        p.recovery_months,
        p.week_start
    )
    .execute(&state.db)
    .await?;
    Ok(Json(StatusResponse { status: true }))
}
