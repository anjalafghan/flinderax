//! Bills: what's due in the next weeks (fixed payments, yearly bills, card bills), how far each
//! yearly bill's set-aside has got, and repeating payments spotted in transactions that aren't
//! fixed payments yet ("Add Netflix ₹649 monthly?").

use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date, Duration};

use crate::{
    alerts::normalize,
    handlers::{
        autopost,
        common::{bad_request, today_local, AppError},
        dashboard::load_input,
    },
    models::AppState,
    planner::{
        bills::{calendar, sinking_funds, Bill, SinkingFund},
        recurring::{detect, Recurring, Seen},
    },
};

type User = Extension<(String, String)>;

/// How far back to look for repeating payments: a bit over a year, to catch yearly ones.
const LOOKBACK_DAYS: i64 = 400;

#[derive(Deserialize)]
pub struct DaysQuery {
    pub days: Option<i64>,
}

#[derive(Serialize)]
pub struct BillsOut {
    pub bills: Vec<Bill>,
    pub total_paise: i64,
    pub sinking_funds: Vec<SinkingFund>,
}

/// `GET /api/bills?days=30`
pub async fn upcoming(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Query(q): Query<DaysQuery>,
) -> Result<Json<BillsOut>, AppError> {
    let today = today_local();
    autopost::apply(&state.db, &user_id, today).await?;
    let input = load_input(&state.db, &user_id, today).await?;
    let days = q.days.unwrap_or(30).clamp(1, 120);
    let bills = calendar(&input, today, today + Duration::days(days));
    Ok(Json(BillsOut {
        total_paise: bills.iter().map(|b| b.amount_paise - b.covered_paise).sum(),
        sinking_funds: sinking_funds(&input),
        bills,
    }))
}

fn parse_date(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

/// `GET /api/bills/suggestions`: repeating payments that aren't fixed payments yet.
pub async fn suggestions(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Vec<Recurring>>, AppError> {
    let since = (today_local() - Duration::days(LOOKBACK_DAYS)).to_string();
    let debits: Vec<Seen> = sqlx::query!(
        r#"SELECT merchant AS "merchant!", occurred_on, amount_paise FROM transactions
           WHERE user_id = ? AND direction = 'debit' AND merchant IS NOT NULL AND occurred_on >= ?
             AND category NOT IN ('transfer', 'income')"#,
        user_id,
        since
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .filter_map(|t| Some(Seen { merchant: normalize(&t.merchant), on: parse_date(&t.occurred_on)?, amount: t.amount_paise, label: t.merchant }))
    .collect();

    let mut known: Vec<String> = sqlx::query_scalar!("SELECT label FROM commitments WHERE user_id = ?", user_id)
        .fetch_all(&state.db)
        .await?
        .iter()
        .map(|l| normalize(l))
        .collect();
    known.extend(sqlx::query_scalar!("SELECT merchant FROM dismissed_recurring WHERE user_id = ?", user_id).fetch_all(&state.db).await?);
    Ok(Json(detect(&debits, &known)))
}

#[derive(Deserialize)]
pub struct DismissPayload {
    pub merchant: String,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

/// `POST /api/bills/suggestions/dismiss`: "not a subscription", never offered again.
pub async fn dismiss(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<DismissPayload>,
) -> Result<Json<Done>, AppError> {
    let m = normalize(&p.merchant);
    if m.is_empty() {
        return Err(bad_request("merchant is required"));
    }
    sqlx::query!("INSERT OR IGNORE INTO dismissed_recurring (user_id, merchant) VALUES (?, ?)", user_id, m)
        .execute(&state.db)
        .await?;
    Ok(Json(Done { status: true }))
}

#[derive(Serialize)]
pub struct Restored {
    pub status: bool,
    pub restored: u64,
}

/// `POST /api/bills/suggestions/restore`: forgets every "not a subscription" you said, so those
/// repeating payments are offered again.
pub async fn restore(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Restored>, AppError> {
    let restored = sqlx::query("DELETE FROM dismissed_recurring WHERE user_id = ?")
        .bind(&user_id)
        .execute(&state.db)
        .await?
        .rows_affected();
    Ok(Json(Restored { status: true, restored }))
}
