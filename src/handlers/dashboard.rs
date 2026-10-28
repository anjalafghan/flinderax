//! `GET /api/dashboard/summary`: loads everything for the user and runs the pure planner.

use axum::{extract::State, Extension, Json};
use time::{macros::format_description, Date};

use crate::{
    handlers::{
        card_cycle::{load_breakdowns, spent_this_period},
        common::{today_local, AppError},
        moves::coverage,
        plan::load_settings,
    },
    models::AppState,
    planner::{
        self, parse_year_month, AccountIn, AllocationState, Bucket, CardIn, CommitmentIn, Freq, IncomeIn,
        PlanInput, Role, SettingsIn, Summary,
    },
};

fn parse_db_date(ts: &str) -> Option<Date> {
    Date::parse(ts.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

/// The ledger key for a card's bill due on `due` (see `handlers::moves`).
fn bill_key(card_id: &str, due: Option<Date>) -> String {
    format!("{}:{}", card_id, due.map_or("none".to_string(), |d| d.to_string()))
}

/// Everything the planner needs for `user_id` as of `today`.
pub async fn load_input(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<PlanInput, AppError> {
    let accounts: Vec<AccountIn> = sqlx::query!(
        r#"SELECT account_id AS "account_id!", name, balance_paise, locked AS "locked: bool", role,
                  updated_at AS "updated_at: String"
           FROM accounts WHERE user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|a| AccountIn {
        id: a.account_id,
        name: a.name,
        balance: a.balance_paise,
        locked: a.locked,
        role: Role::parse(&a.role),
        updated: parse_db_date(&a.updated_at),
    })
    .collect();

    let incomes: Vec<IncomeIn> = sqlx::query!(
        "SELECT label, amount_paise, day_of_month, account_id FROM income_sources WHERE user_id = ?",
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|i| IncomeIn { label: i.label, amount: i.amount_paise, day: i.day_of_month as u8, account_id: i.account_id })
    .collect();

    let commitments: Vec<CommitmentIn> = sqlx::query!(
        r#"SELECT label, amount_paise, day_of_month, account_id, bucket, frequency, due_month, to_account_id,
                  active AS "active: bool", start_month, end_month
           FROM commitments WHERE user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|c| CommitmentIn {
        label: c.label,
        amount: c.amount_paise,
        day: c.day_of_month as u8,
        account_id: c.account_id,
        active: c.active,
        bucket: Bucket::parse(&c.bucket).unwrap_or(Bucket::Needs),
        freq: if c.frequency == "yearly" { Freq::Yearly } else { Freq::Monthly },
        due_month: c.due_month.map(|m| m as u8),
        to_account_id: c.to_account_id,
        start_month: c.start_month.as_deref().and_then(parse_year_month),
        end_month: c.end_month.as_deref().and_then(parse_year_month),
    })
    .collect();

    let period = planner::pay_period(today, &incomes);
    let cov = coverage(db, user_id).await?;
    let spent = spent_this_period(db, user_id, period.start, today).await?;
    let covered = |key: &str| cov.get(key).map_or(0, |c| c.0);

    let cards: Vec<CardIn> = load_breakdowns(db, user_id, None, today)
        .await?
        .into_iter()
        .map(|b| CardIn {
            due_covered: covered(&bill_key(&b.card_id, b.due_date)),
            next_covered: covered(&bill_key(&b.card_id, b.next_due_date)),
            spent_this_period: spent.get(&b.card_id).map(|s| s.0),
            tracked_since: spent.get(&b.card_id).map(|s| s.1),
            card_id: b.card_id,
            name: b.card_name,
            due_now: b.due_now_paise,
            due_date: b.due_date,
            next_bill_est: b.next_bill_est_paise,
            next_due_date: b.next_due_date,
            last_updated: b.last_updated_date,
            pay_from: b.pay_from_account_id,
        })
        .collect();

    let s = load_settings(db, user_id).await?;
    let (moved, skipped) = cov.get(&period.start.to_string()).copied().unwrap_or((0, 0));
    Ok(PlanInput {
        today,
        accounts,
        incomes,
        commitments,
        cards,
        settings: SettingsIn {
            spending: s.spending_paise,
            spending_bps: s.spending_bps,
            allocation_day: s.allocation_day as u8,
            savings_floor: s.savings_floor_paise,
            recovery_months: s.recovery_months,
        },
        allocation: AllocationState { moved, skipped },
    })
}

pub async fn summary(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<Summary>, AppError> {
    let input = load_input(&state.db, &user_id, today_local()).await?;
    Ok(Json(planner::compute(&input)))
}
