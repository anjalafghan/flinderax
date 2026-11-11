//! `GET /api/dashboard/summary`: loads everything for the user and runs the pure planner.
//! `GET /api/dashboard/history`: net worth over time and the other tracker panels.

use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    handlers::{
        autopost,
        card_cycle::{load_breakdowns, spent_this_period},
        common::{today_local, AppError},
        moves::coverage,
        plan::load_settings,
    },
    models::AppState,
    planner::{
        self,
        history::{monthly_net_worth, MonthPoint, Point}, parse_year_month, AccountIn, AllocationState, Bucket, CardIn, CommitmentIn, Freq, IncomeIn,
        PlanInput, PlannedDateIn, Role, SettingsIn, Summary,
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

    let planned_dates = load_planned_dates(db, user_id).await?;
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
        planned_dates,
    })
}

/// Dates planned for later whose cost is not yet paid: the estimate, less what the jar covers, in
/// paise. A surprise still sealed from this user keeps its amount but not its name.
async fn load_planned_dates(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<PlannedDateIn>, AppError> {
    let Some((couple, who)) = crate::handlers::couple::couple_of(db, user_id).await? else { return Ok(Vec::new()) };
    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT date(h.planned_at, 'unixepoch', 'localtime'),
                CASE WHEN h.surprise = 1 AND h.revealed = 0 AND COALESCE(h.drawn_by, -1) <> ?2
                     THEN 'A surprise date' ELSE COALESCE(a.title, 'A date') END,
                MAX(h.est_cost - COALESCE(h.from_savings, 0), 0)
           FROM joy_history h LEFT JOIN joy_activities a ON a.id = h.activity_id
          WHERE h.couple_id = ?1 AND h.outcome = 'did_it' AND h.actual_cost IS NULL
            AND h.planned_at IS NOT NULL AND h.planned_at > unixepoch() AND h.est_cost > 0",
    )
    .bind(&couple)
    .bind(who as i64)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(day, label, rupees)| {
            let date = time::Date::parse(&day, time::macros::format_description!("[year]-[month]-[day]")).ok()?;
            Some(PlannedDateIn { date, label, amount: rupees.saturating_mul(100) })
        })
        .collect())
}

pub async fn summary(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<Summary>, AppError> {
    let today = today_local();
    crate::backup::maybe_backup(&state.db, today).await;
    autopost::apply(&state.db, &user_id, today).await?;
    let input = load_input(&state.db, &user_id, today).await?;
    Ok(Json(planner::compute(&input)))
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub months: Option<u32>,
}

#[derive(Serialize)]
pub struct CardUse {
    pub card_id: String,
    pub name: String,
    pub outstanding_paise: i64,
    pub limit_paise: Option<i64>,
    /// outstanding / limit in basis points; `null` without a limit.
    pub utilization_bps: Option<i64>,
}

#[derive(Serialize)]
pub struct Maturity {
    pub account_id: String,
    pub name: String,
    pub kind: String,
    pub balance_paise: i64,
    pub maturity_date: String,
    pub days_left: i64,
}

#[derive(Serialize)]
pub struct AutoPostingRow {
    pub occurs_on: String,
    pub label: String,
    pub account_name: String,
    pub amount_paise: i64,
}

#[derive(Serialize)]
pub struct History {
    pub months: Vec<MonthPoint>,
    pub cards: Vec<CardUse>,
    pub maturities: Vec<Maturity>,
    pub auto_postings: Vec<AutoPostingRow>,
    pub auto_post: bool,
}

/// `GET /api/dashboard/history?months=12`: net worth by month, card usage, FD/RD maturities and
/// recent automatic postings.
pub async fn history(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<History>, AppError> {
    let db = &state.db;
    let today = today_local();
    autopost::apply(db, &user_id, today).await?;
    let months = q.months.unwrap_or(12).clamp(1, 60);

    let accounts = sqlx::query!(
        r#"SELECT account_id AS "account_id!", name, kind, balance_paise, locked AS "locked: bool", maturity_date
           FROM accounts WHERE user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?;
    let locked: HashSet<String> = accounts.iter().filter(|a| a.locked).map(|a| a.account_id.clone()).collect();

    let balances: Vec<Point> = sqlx::query!(
        r#"SELECT h.account_id AS "account_id!", h.balance_paise, h.recorded_at AS "recorded_at: String"
           FROM balance_history h JOIN accounts a ON a.account_id = h.account_id
           WHERE a.user_id = ? ORDER BY h.recorded_at, h.history_id"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .filter_map(|r| Some(Point { at: parse_db_date(&r.recorded_at)?, id: r.account_id, value: r.balance_paise }))
    .collect();

    let snapshots = sqlx::query!(
        r#"SELECT s.card_id AS "card_id!", s.outstanding_paise, s.created_at AS "created_at: String"
           FROM card_snapshots s JOIN cards c ON c.card_id = s.card_id
           WHERE c.user_id = ? ORDER BY s.created_at, s.rowid"#,
        user_id
    )
    .fetch_all(db)
    .await?;
    let debts: Vec<Point> = snapshots
        .iter()
        .filter_map(|r| Some(Point { at: parse_db_date(&r.created_at)?, id: r.card_id.clone(), value: r.outstanding_paise }))
        .collect();

    let series = monthly_net_worth(&balances, |id| locked.contains(id), &debts, today, months);

    let latest: HashMap<&str, i64> = snapshots.iter().map(|s| (s.card_id.as_str(), s.outstanding_paise)).collect();
    let cards = sqlx::query!(
        r#"SELECT card_id AS "card_id!", card_name, credit_limit_paise FROM cards WHERE user_id = ? ORDER BY rowid"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|c| {
        let outstanding = latest.get(c.card_id.as_str()).copied().unwrap_or(0).max(0);
        let limit = c.credit_limit_paise.filter(|l| *l > 0);
        CardUse {
            utilization_bps: limit.map(|l| outstanding * 10_000 / l),
            card_id: c.card_id,
            name: c.card_name,
            outstanding_paise: outstanding,
            limit_paise: limit,
        }
    })
    .collect();

    let mut maturities: Vec<Maturity> = accounts
        .iter()
        .filter_map(|a| {
            let raw = a.maturity_date.as_ref()?;
            let days_left = (parse_db_date(raw)? - today).whole_days();
            (days_left >= 0).then(|| Maturity {
                account_id: a.account_id.clone(),
                name: a.name.clone(),
                kind: a.kind.clone(),
                balance_paise: a.balance_paise,
                maturity_date: raw.clone(),
                days_left,
            })
        })
        .collect();
    maturities.sort_by_key(|m| m.days_left);

    let auto_postings = sqlx::query_as!(
        AutoPostingRow,
        r#"SELECT p.occurs_on, p.label, a.name AS account_name, p.amount_paise
           FROM auto_postings p JOIN accounts a ON a.account_id = p.account_id
           WHERE p.user_id = ? ORDER BY p.occurs_on DESC, p.created_at DESC LIMIT 10"#,
        user_id
    )
    .fetch_all(db)
    .await?;

    Ok(Json(History {
        months: series,
        cards,
        maturities,
        auto_postings,
        auto_post: load_settings(db, &user_id).await?.auto_post,
    }))
}
