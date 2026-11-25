//! Recurring commitments (SIPs, rent, RD instalments...), income sources and planner settings.

use axum::{extract::State, Extension, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    planner::{self, default_bucket, Bucket},
    handlers::{
        account::ensure_account_owned,
        common::{bad_request, not_found, today_local, validate_day, validate_year_month, AppError},
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
    /// 1 for a plain monthly bill; 2-11 for one due every that many months (monthly frequency only).
    pub every_months: i64,
    /// `2026-12`: the first month a bill every few months is due. None when every_months is 1.
    pub first_month: Option<String>,
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
    /// 1-11, with a monthly frequency: due every that many months. Left out on update, the stored
    /// value stays.
    pub every_months: Option<i64>,
    /// `2026-12`. With every_months above 1 it defaults to the stored value, else this month.
    pub first_month: Option<String>,
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
    if p.every_months.is_some_and(|n| !(1..=11).contains(&n)) {
        return Err(bad_request("every_months must be between 1 and 11 (use yearly for every 12)"));
    }
    validate_year_month(&p.first_month, "first_month")?;
    validate_year_month(&p.start_month, "start_month")?;
    validate_year_month(&p.end_month, "end_month")
}

/// The cadence to store: `(every_months, first_month)`. `current` is what is stored now (update).
/// A bill every few months must be monthly-frequency; a plain monthly or a yearly one keeps 1 and
/// no first month, so a stale value cannot come back if the cadence changes again later.
fn resolve_cadence(
    p: &CommitmentPayload,
    frequency: &str,
    current: Option<(i64, Option<String>)>,
    this_month: &str,
) -> Result<(i64, Option<String>), AppError> {
    let every = p.every_months.or(current.as_ref().map(|c| c.0)).unwrap_or(1);
    if frequency == "yearly" {
        if p.every_months.is_some_and(|n| n > 1) {
            return Err(bad_request("every_months only applies to a monthly bill; a yearly bill is every 12 months"));
        }
        return Ok((1, None));
    }
    if every <= 1 {
        return Ok((1, None));
    }
    let first = p.first_month.clone().or(current.and_then(|c| c.1)).unwrap_or_else(|| this_month.to_string());
    // Due only from its first month on, so an end before that month would never fall due.
    let end = p.end_month.as_deref().and_then(crate::planner::parse_year_month);
    if let (Some(f), Some(e)) = (crate::planner::parse_year_month(&first), end)
        && f > e
    {
        return Err(bad_request("first_month is after end_month, so it would never fall due"));
    }
    Ok((every, Some(first)))
}

fn year_month(d: time::Date) -> String {
    format!("{:04}-{:02}", d.year(), d.month() as u8)
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
    let (every_months, first_month) = resolve_cadence(&p, &frequency, None, &year_month(today_local()))?;
    let category = p.category.unwrap_or_else(|| "other".into());
    let bucket = bucket_for(&p.bucket, &category);
    let active = p.active.unwrap_or(true);
    sqlx::query!(
        "INSERT INTO commitments (commitment_id, user_id, label, amount_paise, day_of_month, account_id, category, bucket, frequency, due_month, every_months, first_month, to_account_id, active, start_month, end_month)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        every_months,
        first_month,
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
    // A partial update keeps what is stored for the fields it leaves out, rather than resetting them
    // (which would, say, resume a paused yearly bill as a monthly one).
    // (A runtime query, not `query!`: the committed `.sqlx` data would need regenerating.)
    let (cur_frequency, cur_category, cur_active, cur_every, cur_first): (String, String, i64, i64, Option<String>) = sqlx::query_as(
        "SELECT frequency, category, active, every_months, first_month FROM commitments WHERE commitment_id = ? AND user_id = ?",
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| not_found("Commitment not found"))?;
    let frequency = p.frequency.clone().unwrap_or(cur_frequency);
    let (every_months, first_month) =
        resolve_cadence(&p, &frequency, Some((cur_every, cur_first)), &year_month(today_local()))?;
    let category = p.category.unwrap_or(cur_category);
    let bucket = bucket_for(&p.bucket, &category);
    let active = p.active.unwrap_or(cur_active != 0);
    let res = sqlx::query!(
        "UPDATE commitments SET label = ?, amount_paise = ?, day_of_month = ?, account_id = ?, category = ?,
                bucket = ?, frequency = ?, due_month = ?, every_months = ?, first_month = ?, to_account_id = ?, active = ?, start_month = ?, end_month = ?,
                waiting_from = CASE WHEN first_month IS ? THEN waiting_from ELSE NULL END
         WHERE commitment_id = ? AND user_id = ?",
        p.label,
        p.amount_paise,
        p.day_of_month,
        p.account_id,
        category,
        bucket,
        frequency,
        p.due_month,
        every_months,
        first_month,
        p.to_account_id,
        active,
        p.start_month,
        p.end_month,
        // A new first month is a new schedule: it is no longer waiting on one put off.
        first_month,
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

#[derive(Serialize)]
pub struct PutOffResponse {
    /// `2027-01`: the month it now falls due.
    pub first_month: String,
    /// `2026-12`: the month it was first due, while it is put off. None after the last undo.
    pub waiting_from: Option<String>,
}

async fn load_commitment(state: &AppState, user_id: &str, id: &str) -> Result<crate::planner::CommitmentIn, AppError> {
    crate::handlers::dashboard::load_commitments(&state.db, user_id)
        .await?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| not_found("Commitment not found"))
}

async fn store_cadence(state: &AppState, user_id: &str, id: &str, first: i32, waiting: Option<i32>) -> Result<PutOffResponse, AppError> {
    let out = PutOffResponse { first_month: planner::year_month_of(first), waiting_from: waiting.map(planner::year_month_of) };
    sqlx::query!(
        "UPDATE commitments SET first_month = ?, waiting_from = ? WHERE commitment_id = ? AND user_id = ?",
        out.first_month,
        out.waiting_from,
        id,
        user_id
    )
    .execute(&state.db)
    .await?;
    Ok(out)
}

/// `POST /api/plan/commitment/put-off`: "Not this month" for a payment every few months. The next
/// one moves a month on and the schedule counts on from there; nothing more is set aside meanwhile.
pub async fn commitment_put_off(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<IdPayload>,
) -> Result<Json<PutOffResponse>, AppError> {
    let c = load_commitment(&state, &user_id, &p.id).await?;
    let (first, waiting) = planner::put_off(&c, today_local()).map_err(|r| {
        bad_request(match r {
            planner::PutOffRefusal::NotEveryFewMonths => "Only a payment every few months can be put off",
            planner::PutOffRefusal::NothingDue => "Nothing is due to put off",
            planner::PutOffRefusal::PastEnd => "This is its last one: a month later is after its end month",
        })
    })?;
    Ok(Json(store_cadence(&state, &user_id, &p.id, first, Some(waiting)).await?))
}

/// `POST /api/plan/commitment/put-off/undo`: brings a put-off payment back a month.
pub async fn commitment_put_off_undo(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<IdPayload>,
) -> Result<Json<PutOffResponse>, AppError> {
    let c = load_commitment(&state, &user_id, &p.id).await?;
    let (first, waiting) = planner::undo_put_off(&c).ok_or_else(|| bad_request("This payment isn't put off"))?;
    Ok(Json(store_cadence(&state, &user_id, &p.id, first, waiting).await?))
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
                  category, bucket, frequency, due_month, every_months, first_month, to_account_id, active AS "active: bool", start_month, end_month
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
    /// Post salary and fixed payments to balances on their dates (see `handlers::autopost`).
    #[serde(default = "default_auto_post")]
    pub auto_post: bool,
}

fn default_auto_post() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self { spending_paise: None, spending_bps: None, allocation_day: 10, savings_floor_paise: None, recovery_months: 3, week_start: 1, auto_post: true }
    }
}

pub async fn load_settings(db: &sqlx::SqlitePool, user_id: &str) -> Result<Settings, AppError> {
    let row = sqlx::query!(
        r#"SELECT spending_paise, spending_bps, allocation_day, savings_floor_paise, recovery_months, week_start,
                  auto_post AS "auto_post: bool"
           FROM user_settings WHERE user_id = ?"#,
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
        auto_post: r.auto_post,
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
        "INSERT INTO user_settings (user_id, spending_paise, spending_bps, allocation_day, savings_floor_paise, recovery_months, week_start, auto_post)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET spending_paise = excluded.spending_paise, spending_bps = excluded.spending_bps,
            allocation_day = excluded.allocation_day, savings_floor_paise = excluded.savings_floor_paise,
            recovery_months = excluded.recovery_months, week_start = excluded.week_start, auto_post = excluded.auto_post",
        user_id,
        p.spending_paise,
        p.spending_bps,
        p.allocation_day,
        p.savings_floor_paise,
        p.recovery_months,
        p.week_start,
        p.auto_post
    )
    .execute(&state.db)
    .await?;
    Ok(Json(StatusResponse { status: true }))
}

/// Puts the budget settings back to their defaults (spending allowance by the usual share of
/// income, top-up on the 10th, no savings floor, 3 months to pay back a shortfall, week from
/// Monday, automatic postings on). Other things kept in the same row, such as holder names and the
/// eating-out cap, are left alone.
pub async fn settings_reset(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<StatusResponse>, AppError> {
    let d = Settings::default();
    sqlx::query(
        "UPDATE user_settings SET spending_paise = NULL, spending_bps = NULL, allocation_day = ?, savings_floor_paise = NULL,
                recovery_months = ?, week_start = ?, auto_post = ? WHERE user_id = ?",
    )
    .bind(d.allocation_day)
    .bind(d.recovery_months)
    .bind(d.week_start)
    .bind(d.auto_post)
    .bind(&user_id)
    .execute(&state.db)
    .await?;
    Ok(Json(StatusResponse { status: true }))
}

// ---------- what-if scenarios ----------

const MAX_SCENARIOS: usize = 50;
const MAX_SCENARIO_BYTES: usize = 64 * 1024;

#[derive(Serialize, Deserialize)]
pub struct Scenarios {
    pub drafts: serde_json::Value,
}

/// `GET /api/plan/scenarios`: the what-if drafts this person has saved.
pub async fn scenarios_get(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Scenarios>, AppError> {
    let raw: Option<String> = sqlx::query_scalar("SELECT drafts_json FROM whatif_scenarios WHERE user_id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await?;
    let drafts = raw.and_then(|r| serde_json::from_str(&r).ok()).unwrap_or_else(|| serde_json::json!([]));
    Ok(Json(Scenarios { drafts }))
}

/// `POST /api/plan/scenarios`: replaces the saved drafts. The page owns their shape; the server only
/// keeps a list, within limits, for this person.
pub async fn scenarios_put(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<Scenarios>,
) -> Result<Json<StatusResponse>, AppError> {
    let Some(list) = p.drafts.as_array() else {
        return Err(AppError(axum::http::StatusCode::UNPROCESSABLE_ENTITY, "drafts must be a list".into()));
    };
    if list.len() > MAX_SCENARIOS {
        return Err(bad_request(format!("at most {MAX_SCENARIOS} scenarios")));
    }
    let json = p.drafts.to_string();
    if json.len() > MAX_SCENARIO_BYTES {
        return Err(bad_request("those scenarios are too large to save"));
    }
    sqlx::query(
        "INSERT INTO whatif_scenarios (user_id, drafts_json) VALUES (?, ?)
         ON CONFLICT(user_id) DO UPDATE SET drafts_json = excluded.drafts_json, updated_at = CURRENT_TIMESTAMP",
    )
    .bind(&user_id)
    .bind(json)
    .execute(&state.db)
    .await?;
    Ok(Json(StatusResponse { status: true }))
}
