//! Card billing cycles: bank-app snapshots, the breakdown, EMIs and pending charges.
//! Money is INTEGER paise on this API (`*_paise`); legacy `card_events` stay in rupees.

use std::collections::HashMap;

use axum::{extract::State, Extension, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    handlers::{
        card::{ensure_card_owned, record_total},
        common::{bad_request, invalidate_cards_cache, not_found, today_local, AppError},
    },
    models::AppState,
    planner::{
        card_amounts, emi_effective, month_index, parse_year_month,
        cycle::{day_in_month_offset, due_date_for, last_statement_date, next_statement_date},
        CardNumbers,
    },
};

type User = Extension<(String, String)>;

#[derive(Serialize)]
pub struct StatusOk {
    pub status: bool,
}

// ---------- snapshot ----------

#[derive(Deserialize)]
pub struct SnapshotPayload {
    pub card_id: String,
    pub total_due_paise: i64,
    pub outstanding_paise: i64,
    pub unbilled_spent_paise: Option<i64>,
    pub unbilled_credit_paise: Option<i64>,
    /// For a card's first snapshot: true = money owed before tracking began (not new spending),
    /// false = new spending this month. Defaults to true for a first snapshot, false afterwards.
    pub starting_balance: Option<bool>,
}

#[derive(Serialize)]
pub struct SnapshotResponse {
    pub snapshot_id: String,
    pub transaction_id: String,
    pub status: bool,
}

/// Stores the four bank-app numbers and mirrors `outstanding` into the legacy event history so
/// the existing history/delta UI keeps working.
pub async fn snapshot(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<SnapshotPayload>,
) -> Result<Json<SnapshotResponse>, AppError> {
    if p.total_due_paise < 0 {
        return Err(bad_request("total_due_paise cannot be negative"));
    }
    ensure_card_owned(&state.db, &p.card_id, &user_id).await?;

    let snapshot_id = nanoid!();
    let mut tx = state.db.begin().await?;
    let existing = sqlx::query!(
        r#"SELECT COUNT(*) AS "n!: i64" FROM card_snapshots WHERE card_id = ?"#,
        p.card_id
    )
    .fetch_one(&mut *tx)
    .await?
    .n;
    // Only a card's first snapshot can be a starting balance.
    let is_starting = existing == 0 && p.starting_balance.unwrap_or(true);
    sqlx::query!(
        "INSERT INTO card_snapshots (snapshot_id, card_id, total_due_paise, outstanding_paise, unbilled_spent_paise, unbilled_credit_paise, is_starting)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        snapshot_id,
        p.card_id,
        p.total_due_paise,
        p.outstanding_paise,
        p.unbilled_spent_paise,
        p.unbilled_credit_paise,
        is_starting
    )
    .execute(&mut *tx)
    .await?;
    let (transaction_id, _) =
        record_total(&mut tx, &p.card_id, (p.outstanding_paise as f64 / 100.0) as f32).await?;
    tx.commit().await?;

    invalidate_cards_cache(&state, &user_id).await;
    Ok(Json(SnapshotResponse { snapshot_id, transaction_id, status: true }))
}

// ---------- breakdown ----------

#[derive(Serialize)]
pub struct EmiRow {
    pub emi_id: String,
    pub label: String,
    pub monthly_paise: i64,
    pub remaining_principal_paise: i64,
    pub months_left: i64,
}

#[derive(Serialize)]
pub struct PendingRow {
    pub charge_id: String,
    pub label: String,
    pub amount_paise: i64,
}

#[derive(Serialize, Clone)]
pub struct CardBreakdown {
    pub card_id: String,
    pub card_name: String,
    pub due_now_paise: i64,
    pub due_date: Option<Date>,
    pub next_bill_est_paise: i64,
    pub next_statement_date: Option<Date>,
    pub next_due_date: Option<Date>,
    pub loan_not_due_paise: i64,
    /// When the bank-app snapshot behind these numbers was entered (UTC, RFC 3339).
    pub last_updated: Option<String>,
    #[serde(skip)]
    pub last_updated_date: Option<Date>,
    /// Account that pays this card's bill (e.g. Kotak).
    pub pay_from_account_id: Option<String>,
    /// Bank-reported unbilled spent - unbilled credit, when the snapshot has it.
    pub unbilled_net_paise: Option<i64>,
}

#[derive(Serialize)]
pub struct BreakdownResponse {
    #[serde(flatten)]
    pub breakdown: CardBreakdown,
    pub emis: Vec<EmiRow>,
    pub pending_charges: Vec<PendingRow>,
}

#[derive(Deserialize)]
pub struct CardIdPayload {
    pub card_id: String,
}

fn parse_db_date(ts: &str) -> Option<Date> {
    Date::parse(ts.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

/// Breakdowns for all of a user's cards, or just `only_card` when given.
pub async fn load_breakdowns(
    db: &sqlx::SqlitePool,
    user_id: &str,
    only_card: Option<&str>,
    today: Date,
) -> Result<Vec<CardBreakdown>, AppError> {
    let cards = sqlx::query!(
        r#"SELECT c.card_id AS "card_id!", c.card_name, c.statement_day, c.due_day, c.pay_from_account_id,
                  s.unbilled_spent_paise AS "unbilled_spent?: i64", s.unbilled_credit_paise AS "unbilled_credit?: i64",
                  s.total_due_paise AS "total_due_paise?: i64", s.outstanding_paise AS "outstanding_paise?: i64",
                  s.created_at AS "snapshot_at?: String"
           FROM cards c
           LEFT JOIN card_snapshots s ON s.snapshot_id = (
               SELECT snapshot_id FROM card_snapshots
               WHERE card_id = c.card_id ORDER BY created_at DESC, rowid DESC LIMIT 1)
           WHERE c.user_id = ? AND (? IS NULL OR c.card_id = ?)
           ORDER BY c.created_at"#,
        user_id,
        only_card,
        only_card
    )
    .fetch_all(db)
    .await?;

    // Each EMI is aged to today's month before summing, so finished EMIs drop out by themselves.
    let mut emi_sums: HashMap<String, (i64, i64)> = HashMap::new();
    for e in sqlx::query!(
        r#"SELECT e.card_id, e.monthly_paise, e.remaining_principal_paise, e.months_left,
                  COALESCE(e.as_of_month, strftime('%Y-%m', e.created_at)) AS "as_of_month!: String"
           FROM card_emis e JOIN cards c ON c.card_id = e.card_id
           WHERE c.user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?
    {
        let (monthly, remaining, _) = emi_effective(
            e.monthly_paise,
            e.remaining_principal_paise,
            e.months_left,
            parse_year_month(&e.as_of_month).unwrap_or_else(|| month_index(today)),
            month_index(today),
        );
        let sum = emi_sums.entry(e.card_id).or_default();
        sum.0 += monthly;
        sum.1 += remaining;
    }

    let pending_sums: HashMap<String, i64> = sqlx::query!(
        r#"SELECT p.card_id, SUM(p.amount_paise) AS "total!: i64"
           FROM pending_charges p JOIN cards c ON c.card_id = p.card_id
           WHERE c.user_id = ? AND p.cleared_at IS NULL
             AND p.created_at >= datetime('now', '-30 days') GROUP BY p.card_id"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| (r.card_id, r.total))
    .collect();

    Ok(cards
        .into_iter()
        .map(|c| {
            let (emi_monthly, emi_remaining) = emi_sums.get(&c.card_id).copied().unwrap_or((0, 0));
            let amounts = card_amounts(CardNumbers {
                total_due: c.total_due_paise.unwrap_or(0),
                outstanding: c.outstanding_paise.unwrap_or(0),
                unbilled_net: c.unbilled_spent.map(|spent| spent - c.unbilled_credit.unwrap_or(0)),
                emi_monthly,
                emi_remaining,
                pending_charges: pending_sums.get(&c.card_id).copied().unwrap_or(0),
            });
            let (due_date, next_statement, next_due) = match (c.statement_day, c.due_day) {
                (Some(sd), Some(dd)) => {
                    let (sd, dd) = (sd as u8, dd as u8);
                    let next = next_statement_date(today, sd);
                    (
                        Some(due_date_for(last_statement_date(today, sd), dd)),
                        Some(next),
                        Some(due_date_for(next, dd)),
                    )
                }
                _ => (None, None, None),
            };
            CardBreakdown {
                card_id: c.card_id,
                card_name: c.card_name,
                due_now_paise: amounts.due_now_paise,
                due_date,
                next_bill_est_paise: amounts.next_bill_est_paise,
                next_statement_date: next_statement,
                next_due_date: next_due,
                loan_not_due_paise: amounts.loan_not_due_paise,
                pay_from_account_id: c.pay_from_account_id,
                unbilled_net_paise: c.unbilled_spent.map(|spent| spent - c.unbilled_credit.unwrap_or(0)),
                last_updated_date: c.snapshot_at.as_deref().and_then(parse_db_date),
                last_updated: c.snapshot_at.map(|t| format!("{}Z", t.replace(' ', "T"))),
            }
        })
        .collect())
}

pub async fn breakdown(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<CardIdPayload>,
) -> Result<Json<BreakdownResponse>, AppError> {
    let breakdown = load_breakdowns(&state.db, &user_id, Some(&p.card_id), today_local())
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| not_found("Card not found"))?;

    let today = today_local();
    let emis = sqlx::query!(
        r#"SELECT emi_id AS "emi_id!", label, monthly_paise, remaining_principal_paise, months_left,
                  COALESCE(as_of_month, strftime('%Y-%m', created_at)) AS "as_of_month!: String"
           FROM card_emis WHERE card_id = ? ORDER BY created_at"#,
        p.card_id
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|e| {
        let (monthly, remaining, left) = emi_effective(
            e.monthly_paise,
            e.remaining_principal_paise,
            e.months_left,
            parse_year_month(&e.as_of_month).unwrap_or_else(|| month_index(today)),
            month_index(today),
        );
        EmiRow {
            emi_id: e.emi_id,
            label: e.label,
            // finished EMIs keep their label (so they can be removed) but show 0 months left
            monthly_paise: if left == 0 { e.monthly_paise } else { monthly },
            remaining_principal_paise: remaining,
            months_left: left,
        }
    })
    .collect();
    let pending_charges = sqlx::query_as!(
        PendingRow,
        r#"SELECT charge_id AS "charge_id!", label, amount_paise
           FROM pending_charges
           WHERE card_id = ? AND cleared_at IS NULL AND created_at >= datetime('now', '-30 days')
           ORDER BY created_at"#,
        p.card_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(BreakdownResponse { breakdown, emis, pending_charges }))
}

// ---------- EMIs ----------

#[derive(Deserialize)]
pub struct EmiCreatePayload {
    pub card_id: String,
    pub label: String,
    pub monthly_paise: i64,
    pub remaining_principal_paise: i64,
    pub months_left: i64,
}

#[derive(Deserialize)]
pub struct EmiUpdatePayload {
    pub emi_id: String,
    pub label: String,
    pub monthly_paise: i64,
    pub remaining_principal_paise: i64,
    pub months_left: i64,
}

#[derive(Deserialize)]
pub struct EmiDeletePayload {
    pub emi_id: String,
}

#[derive(Serialize)]
pub struct IdResponse {
    pub id: String,
    pub status: bool,
}

/// 'YYYY-MM' of today; the numbers a user types are true as of this month.
fn current_month() -> String {
    let t = today_local();
    format!("{:04}-{:02}", t.year(), t.month() as u8)
}

fn validate_emi(label: &str, monthly: i64, remaining: i64, months_left: i64) -> Result<(), AppError> {
    if label.trim().is_empty() {
        return Err(bad_request("label is required"));
    }
    if monthly < 0 || remaining < 0 || months_left < 0 {
        return Err(bad_request("EMI amounts and months_left cannot be negative"));
    }
    Ok(())
}

pub async fn emi_create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<EmiCreatePayload>,
) -> Result<Json<IdResponse>, AppError> {
    validate_emi(&p.label, p.monthly_paise, p.remaining_principal_paise, p.months_left)?;
    ensure_card_owned(&state.db, &p.card_id, &user_id).await?;
    let id = nanoid!();
    let as_of = current_month();
    sqlx::query!(
        "INSERT INTO card_emis (emi_id, card_id, label, monthly_paise, remaining_principal_paise, months_left, as_of_month)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        id,
        p.card_id,
        p.label,
        p.monthly_paise,
        p.remaining_principal_paise,
        p.months_left,
        as_of
    )
    .execute(&state.db)
    .await?;
    invalidate_cards_cache(&state, &user_id).await;
    Ok(Json(IdResponse { id, status: true }))
}

pub async fn emi_update(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<EmiUpdatePayload>,
) -> Result<Json<StatusOk>, AppError> {
    validate_emi(&p.label, p.monthly_paise, p.remaining_principal_paise, p.months_left)?;
    let as_of = current_month();
    let res = sqlx::query!(
        "UPDATE card_emis SET label = ?, monthly_paise = ?, remaining_principal_paise = ?, months_left = ?,
                as_of_month = ?
         WHERE emi_id = ? AND card_id IN (SELECT card_id FROM cards WHERE user_id = ?)",
        p.label,
        p.monthly_paise,
        p.remaining_principal_paise,
        p.months_left,
        as_of,
        p.emi_id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("EMI not found"));
    }
    invalidate_cards_cache(&state, &user_id).await;
    Ok(Json(StatusOk { status: true }))
}

pub async fn emi_delete(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<EmiDeletePayload>,
) -> Result<Json<StatusOk>, AppError> {
    let res = sqlx::query!(
        "DELETE FROM card_emis WHERE emi_id = ? AND card_id IN (SELECT card_id FROM cards WHERE user_id = ?)",
        p.emi_id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("EMI not found"));
    }
    invalidate_cards_cache(&state, &user_id).await;
    Ok(Json(StatusOk { status: true }))
}

// ---------- pending charges ----------

#[derive(Deserialize)]
pub struct PendingCreatePayload {
    pub card_id: String,
    pub label: String,
    pub amount_paise: i64,
}

#[derive(Deserialize)]
pub struct PendingClearPayload {
    pub charge_id: String,
}

pub async fn pending_create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<PendingCreatePayload>,
) -> Result<Json<IdResponse>, AppError> {
    if p.label.trim().is_empty() {
        return Err(bad_request("label is required"));
    }
    if p.amount_paise <= 0 {
        return Err(bad_request("amount_paise must be positive"));
    }
    ensure_card_owned(&state.db, &p.card_id, &user_id).await?;
    let id = nanoid!();
    sqlx::query!(
        "INSERT INTO pending_charges (charge_id, card_id, label, amount_paise) VALUES (?, ?, ?, ?)",
        id,
        p.card_id,
        p.label,
        p.amount_paise
    )
    .execute(&state.db)
    .await?;
    invalidate_cards_cache(&state, &user_id).await;
    Ok(Json(IdResponse { id, status: true }))
}

/// Marks a pending charge as posted (or dropped) so it stops counting toward the next bill.
pub async fn pending_clear(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<PendingClearPayload>,
) -> Result<Json<StatusOk>, AppError> {
    let res = sqlx::query!(
        "UPDATE pending_charges SET cleared_at = CURRENT_TIMESTAMP
         WHERE charge_id = ? AND cleared_at IS NULL
           AND card_id IN (SELECT card_id FROM cards WHERE user_id = ?)",
        p.charge_id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Pending charge not found"));
    }
    invalidate_cards_cache(&state, &user_id).await;
    Ok(Json(StatusOk { status: true }))
}

// ---------- spending inferred from snapshots ----------

struct Snap {
    date: Date,
    total_due: i64,
    /// Everything billed or unbilled on the card, loan principal excluded: the bank's
    /// `total due + unbilled spent - unbilled credit`, or `outstanding - EMI balance` when the
    /// snapshot has no unbilled numbers.
    exposure: i64,
    is_starting: bool,
}

/// Spent on each card since the baseline of the current pay period, and the date of that baseline:
///
/// 1. the latest snapshot on or before the last payday; else
/// 2. the card's starting-balance snapshot (money owed before tracking began); else
/// 3. an implicit zero balance on the day the card was added.
///
/// `exposure now - exposure then + the bill paid in between` (assumed paid in full by its due
/// date). Cards with no snapshot at all are left out, so their spend is "not tracked".
pub async fn spent_this_period(
    db: &sqlx::SqlitePool,
    user_id: &str,
    period_start: Date,
    today: Date,
) -> Result<HashMap<String, (i64, Date)>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT s.card_id, s.total_due_paise, s.outstanding_paise, s.unbilled_spent_paise AS "us?: i64",
                  s.unbilled_credit_paise AS "uc?: i64", s.created_at AS "created_at: String",
                  s.is_starting AS "is_starting: bool", c.statement_day, c.due_day,
                  c.created_at AS "card_created: String"
           FROM card_snapshots s JOIN cards c ON c.card_id = s.card_id
           WHERE c.user_id = ? ORDER BY s.card_id, s.created_at, s.rowid"#,
        user_id
    )
    .fetch_all(db)
    .await?;

    // Remaining EMI principal per card, so `outstanding` can stand in for the bank's unbilled numbers.
    let now_idx = month_index(today);
    let mut emi_rem: HashMap<String, i64> = HashMap::new();
    for e in sqlx::query!(
        r#"SELECT e.card_id, e.monthly_paise, e.remaining_principal_paise, e.months_left,
                  COALESCE(e.as_of_month, strftime('%Y-%m', e.created_at)) AS "as_of_month!: String"
           FROM card_emis e JOIN cards c ON c.card_id = e.card_id WHERE c.user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?
    {
        let (_, remaining, _) = emi_effective(
            e.monthly_paise,
            e.remaining_principal_paise,
            e.months_left,
            parse_year_month(&e.as_of_month).unwrap_or(now_idx),
            now_idx,
        );
        *emi_rem.entry(e.card_id).or_default() += remaining;
    }

    struct PerCard {
        snaps: Vec<Snap>,
        cycle: Option<(u8, u8)>,
        card_created: Option<Date>,
    }
    let mut by_card: HashMap<String, PerCard> = HashMap::new();
    for r in rows {
        let Some(date) = parse_db_date(&r.created_at) else { continue };
        let loan = emi_rem.get(&r.card_id).copied().unwrap_or(0);
        let entry = by_card.entry(r.card_id).or_insert_with(|| PerCard {
            snaps: Vec::new(),
            cycle: r.statement_day.zip(r.due_day).map(|(a, b)| (a as u8, b as u8)),
            card_created: r.card_created.as_deref().and_then(parse_db_date),
        });
        entry.snaps.push(Snap {
            date,
            total_due: r.total_due_paise,
            exposure: match r.us {
                Some(us) => r.total_due_paise + us - r.uc.unwrap_or(0),
                None => (r.outstanding_paise - loan).max(0),
            },
            is_starting: r.is_starting,
        });
    }

    let mut out = HashMap::new();
    for (card_id, c) in by_card {
        if let Some(spent) = spent_for_card(&c.snaps, c.cycle, c.card_created, period_start, today) {
            out.insert(card_id, spent);
        }
    }
    Ok(out)
}

/// `(spent, baseline date)` for one card, or `None` when it has no snapshots. Snapshots must be
/// in time order.
fn spent_for_card(
    snaps: &[Snap],
    cycle: Option<(u8, u8)>,
    card_created: Option<Date>,
    period_start: Date,
    today: Date,
) -> Option<(i64, Date)> {
    let latest = snaps.last()?;
    let last_idx = snaps.len() - 1;
    let base_idx = snaps
        .iter()
        .rposition(|s| s.date <= period_start)
        .or_else(|| snaps.iter().position(|s| s.is_starting));
    let (base_date, base_total_due, base_exposure) = match base_idx {
        Some(i) => (snaps[i].date, snaps[i].total_due, snaps[i].exposure),
        // the card was added at zero: everything on it since is new spending
        None => (card_created.unwrap_or(snaps[0].date), 0, 0),
    };
    if base_idx == Some(last_idx) {
        return Some((0, base_date));
    }
    let paid = bills_paid_since(snaps, base_idx, base_date, base_total_due, cycle, today);
    Some(((latest.exposure - base_exposure + paid).max(0), base_date))
}

/// Bills paid in full between the baseline and `today`, added back so repayments don't read as
/// negative spend.
///
/// With a billing cycle, every statement after the baseline whose due date falls in
/// `(baseline, today]` counts as paid: the baseline's own bill for the first one, then the
/// largest balance shown during that statement's window for each later one.
///
/// Without a cycle there is no due date to go on, so a repayment is any drop in `total_due`
/// between consecutive snapshots since the baseline.
fn bills_paid_since(
    snaps: &[Snap],
    base_idx: Option<usize>,
    base_date: Date,
    base_total_due: i64,
    cycle: Option<(u8, u8)>,
    today: Date,
) -> i64 {
    match cycle {
        Some((sd, dd)) => {
            let first_stmt = last_statement_date(base_date, sd);
            let mut paid = 0;
            for k in 0.. {
                let stmt = day_in_month_offset(first_stmt, k, sd);
                let due = due_date_for(stmt, dd);
                if due > today {
                    break;
                }
                if due > base_date {
                    paid += if k == 0 {
                        base_total_due
                    } else {
                        let next = day_in_month_offset(first_stmt, k + 1, sd);
                        snaps
                            .iter()
                            .filter(|s| s.date >= stmt && s.date < next)
                            .map(|s| s.total_due)
                            .max()
                            .unwrap_or(0)
                    };
                }
            }
            paid
        }
        None => snaps[base_idx.unwrap_or(0)..]
            .windows(2)
            .map(|w| (w[0].total_due - w[1].total_due).max(0))
            .sum(),
    }
}

#[cfg(test)]
mod spend_tests {
    use super::*;
    use time::macros::date;

    fn snap(d: Date, total_due: i64, exposure: i64) -> Snap {
        Snap { date: d, total_due, exposure, is_starting: false }
    }

    #[test]
    fn repayments_across_several_cycles_are_all_added_back() {
        // Statement 10th, due 25th. Dec bill of 1000 paid Dec 25; Jan bill of 800 paid Jan 25;
        // 200 charged in February and paid Feb 25. Total charges since Dec 20: 1000 (800 + 200).
        let snaps = [
            snap(date!(2025 - 12 - 20), 1000, 1000),
            snap(date!(2026 - 01 - 12), 800, 800),
            snap(date!(2026 - 02 - 01), 0, 0),
            snap(date!(2026 - 02 - 20), 200, 200),
            snap(date!(2026 - 03 - 01), 0, 0),
        ];
        let got = spent_for_card(&snaps, Some((10, 25)), None, date!(2025 - 12 - 20), date!(2026 - 03 - 01));
        assert_eq!(got, Some((1000, date!(2025 - 12 - 20))));
    }

    #[test]
    fn card_without_cycle_days_does_not_read_zero_after_a_payoff() {
        // Owed 1000 at the start of the period, paid off, then 300 charged.
        let snaps = [
            snap(date!(2026 - 01 - 01), 1000, 1000),
            snap(date!(2026 - 01 - 15), 0, 0),
            snap(date!(2026 - 02 - 20), 300, 300),
        ];
        let got = spent_for_card(&snaps, None, None, date!(2026 - 01 - 01), date!(2026 - 02 - 20));
        assert_eq!(got, Some((300, date!(2026 - 01 - 01))));
    }

    #[test]
    fn card_without_cycle_days_with_no_payment_counts_the_increase() {
        let snaps = [
            snap(date!(2026 - 01 - 01), 100, 100),
            snap(date!(2026 - 01 - 20), 350, 350),
        ];
        let got = spent_for_card(&snaps, None, None, date!(2026 - 01 - 01), date!(2026 - 01 - 20));
        assert_eq!(got, Some((250, date!(2026 - 01 - 01))));
    }
}
