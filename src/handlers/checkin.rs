//! Phase 2 check-in API, behind login: `GET /api/checkin` (the status), `GET /api/checkin/report`,
//! `POST /api/checkin/complete` and `GET /api/checkin/history`. The rules are pure in
//! `planner::checkin`. This module loads the user's rows and saves a finished check-in.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Extension, Json,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    handlers::{
        calibration,
        common::{bad_request, today_local, AppError},
        diet, habits, people,
    },
    models::AppState,
    planner::checkin::{
        self, AccountIn, CardIn, CheckinStatus, LastCheckin, Period, QuarterReport, Span,
        StatusIn, Totals, TxnIn,
    },
};

/// Longest report window, in days from start to end.
const MAX_REPORT_DAYS: i64 = 400;
/// Most finished check-ins in the history.
const HISTORY_LIMIT: i64 = 12;

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub start: Option<String>,
    pub end: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HistoryOut {
    pub checkins: Vec<HistoryRow>,
}

#[derive(Debug, Serialize)]
pub struct HistoryRow {
    pub done_on: String,
    pub period: Span,
    pub totals: Totals,
}

/// The part of a saved report the history reads back.
#[derive(Deserialize)]
struct StoredSummary {
    totals: Totals,
}

pub async fn status(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<CheckinStatus>, AppError> {
    Ok(Json(compute_status(&state, &user_id, today_local()).await?))
}

pub async fn report(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<QuarterReport>, AppError> {
    Ok(Json(report_on(&state, &user_id, today_local(), query).await?))
}

pub async fn complete(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<CheckinStatus>, AppError> {
    Ok(Json(complete_on(&state, &user_id, today_local()).await?))
}

pub async fn history(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<HistoryOut>, AppError> {
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT done_on, period_start, period_end, summary_json FROM checkins
         WHERE user_id = ? ORDER BY done_on DESC, created_at DESC LIMIT ?",
    )
    .bind(&user_id)
    .bind(HISTORY_LIMIT)
    .fetch_all(&state.db)
    .await?;
    let checkins = rows
        .into_iter()
        .map(|(done_on, start, end, summary)| -> Result<HistoryRow, AppError> {
            let stored: StoredSummary = serde_json::from_str(&summary).map_err(stored_error)?;
            Ok(HistoryRow {
                done_on,
                period: Span { start, end },
                totals: stored.totals,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    Ok(Json(HistoryOut { checkins }))
}

/// The status for a user as of `today`.
pub async fn compute_status(
    state: &AppState,
    user_id: &str,
    today: Date,
) -> Result<CheckinStatus, AppError> {
    let input = load_status_input(&state.db, user_id, today).await?;
    Ok(checkin::status(&input))
}

/// The quarter report for a query as of `today`. Missing dates default to the status period.
/// A default start is never after the end, so a check-in finished today still gives a report.
pub async fn report_on(
    state: &AppState,
    user_id: &str,
    today: Date,
    query: ReportQuery,
) -> Result<QuarterReport, AppError> {
    let input = load_status_input(&state.db, user_id, today).await?;
    let default = checkin::period_for(today, input.last.as_ref());
    let end = match query.end {
        Some(s) => parse_day(&s)?,
        None => default.end,
    };
    let start = match query.start {
        Some(s) => parse_day(&s)?,
        None => default.start.min(end),
    };
    if start > end {
        return Err(bad_request("start must be on or before end"));
    }
    if (end - start).whole_days() > MAX_REPORT_DAYS {
        return Err(bad_request("a report covers at most 400 days"));
    }
    let period = Period { start, end };
    let habits = habits::compute(state, user_id, today).await?;
    let mut report = checkin::quarter_report(&input.txns, period, habits);
    report.calibration = calibration::latest(state, user_id, today).await?;
    report.diet = diet::report(&state.db, user_id, period, today).await?;
    report.people = people::report(&state.db, user_id, period, today).await?;
    Ok(report)
}

/// Saves today's check-in with the report for its period, replacing any saved today, then returns
/// the new status. The period starts after the last check-in done before today.
pub async fn complete_on(
    state: &AppState,
    user_id: &str,
    today: Date,
) -> Result<CheckinStatus, AppError> {
    let db = &state.db;
    let prior_end: Option<String> = sqlx::query_scalar(
        "SELECT period_end FROM checkins WHERE user_id = ? AND done_on < ?
         ORDER BY done_on DESC LIMIT 1",
    )
    .bind(user_id)
    .bind(today.to_string())
    .fetch_optional(db)
    .await?;
    let prior_end = prior_end.as_deref().and_then(stored_day);
    let period = checkin::period_after(today, prior_end);

    let txns = load_txns(db, user_id).await?;
    let habits = habits::compute(state, user_id, today).await?;
    let mut report = checkin::quarter_report(&txns, period, habits);
    report.calibration = calibration::latest(state, user_id, today).await?;
    report.diet = diet::report(db, user_id, period, today).await?;
    report.people = people::report(db, user_id, period, today).await?;
    let summary = serde_json::to_string(&report).map_err(stored_error)?;
    // The forecast as it stands now, for the next check-in to compare with.
    let snap = calibration::snapshot(state, user_id, today).await?;
    let forecast_json = serde_json::to_string(&snap).map_err(stored_error)?;

    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM checkins WHERE user_id = ? AND done_on = ?")
        .bind(user_id)
        .bind(today.to_string())
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO checkins (checkin_id, user_id, done_on, period_start, period_end, summary_json, forecast_json)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new_id())
    .bind(user_id)
    .bind(today.to_string())
    .bind(period.start.to_string())
    .bind(period.end.to_string())
    .bind(summary)
    .bind(forecast_json)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    compute_status(state, user_id, today).await
}

/// Everything the status reads for one user.
async fn load_status_input(
    db: &sqlx::SqlitePool,
    user_id: &str,
    today: Date,
) -> Result<StatusIn, AppError> {
    let last: Option<(String, String, String)> = sqlx::query_as(
        "SELECT done_on, period_start, period_end FROM checkins WHERE user_id = ?
         ORDER BY done_on DESC, created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;
    let last = last.and_then(|(done, start, end)| {
        Some(LastCheckin {
            done_on: stored_day(&done)?,
            period_start: stored_day(&start)?,
            period_end: stored_day(&end)?,
        })
    });

    type AccountRow = (
        String,
        String,
        String,
        i64,
        i64,
        Option<String>,
        Option<String>,
    );
    let rows: Vec<AccountRow> = sqlx::query_as(
        "SELECT a.account_id, a.name, a.kind, a.locked, a.balance_paise, a.balance_set_on,
                (SELECT MAX(s.last_date) FROM statement_imports s
                 WHERE s.account_id = a.account_id AND s.user_id = a.user_id)
         FROM accounts a WHERE a.user_id = ? ORDER BY a.name, a.account_id",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    let accounts = rows
        .into_iter()
        .map(|(account_id, name, kind, locked, balance, set_on, covered)| AccountIn {
            account_id,
            name,
            kind,
            locked: locked != 0,
            balance_paise: balance,
            balance_set_on: set_on.as_deref().and_then(stored_day),
            covered_to: covered.as_deref().and_then(stored_day),
        })
        .collect();

    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT c.card_id, c.card_name,
                (SELECT MAX(s.last_date) FROM statement_imports s
                 WHERE s.card_id = c.card_id AND s.user_id = c.user_id)
         FROM cards c WHERE c.user_id = ? ORDER BY c.card_name, c.card_id",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    let cards = rows
        .into_iter()
        .map(|(card_id, name, covered)| CardIn {
            card_id,
            name,
            covered_to: covered.as_deref().and_then(stored_day),
        })
        .collect();

    let weighed: Vec<String> = sqlx::query_scalar("SELECT measured_on FROM health_marks WHERE user_id = ? AND kind = 'weight'")
        .bind(user_id)
        .fetch_all(db)
        .await?;
    Ok(StatusIn {
        today,
        last,
        accounts,
        cards,
        txns: load_txns(db, user_id).await?,
        weighed_on: weighed.iter().filter_map(|d| stored_day(d)).collect(),
    })
}

/// Every transaction the user has, as the check-in reads them.
pub(crate) async fn load_txns(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<TxnIn>, AppError> {
    /// date, direction, paise, merchant, category, account
    type TxnRow = (String, String, i64, Option<String>, String, Option<String>);
    let rows: Vec<TxnRow> = sqlx::query_as(
        "SELECT occurred_on, direction, amount_paise, merchant, category, account_id
         FROM transactions WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(on, direction, amount, merchant, category, account_id)| {
            Some(TxnIn {
                on: stored_day(&on)?,
                debit: direction == "debit",
                amount_paise: amount,
                merchant,
                category,
                account_id,
            })
        })
        .collect())
}

/// A date from a query string, 'YYYY-MM-DD'.
fn parse_day(s: &str) -> Result<Date, AppError> {
    stored_day(s).ok_or_else(|| bad_request("dates must look like 2026-11-09"))
}

/// A date stored as 'YYYY-MM-DD' (the first ten characters of a longer timestamp are read too).
fn stored_day(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

/// A saved row that no longer reads back is a server fault, logged and hidden from the client.
fn stored_error(e: serde_json::Error) -> AppError {
    AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

/// A random 128-bit id as lowercase hex.
fn new_id() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::checkin::DueState;
    use sqlx::sqlite::SqlitePoolOptions;
    use time::Month;

    /// The in-memory DB pattern from `handlers::ingest`.
    async fn test_state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('other', 'other', 'x', 'user')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32][..]),
        );
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    async fn seed(state: &AppState, statements: &[&str]) {
        for q in statements {
            sqlx::query(q).execute(&state.db).await.unwrap_or_else(|e| panic!("{q}: {e}"));
        }
    }

    fn day(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), d).unwrap()
    }

    /// `AppError` has no Debug, so unwrap by hand with its message.
    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("handler failed: {} {}", e.0, e.1),
        }
    }

    /// The history as the real handler returns it, for one user.
    async fn history_of(state: &AppState, user_id: &str) -> Vec<HistoryRow> {
        let Json(out) = ok(history(
            State(state.clone()),
            Extension((user_id.to_string(), String::new())),
        )
        .await);
        out.checkins
    }

    /// Accounts, cards and statements for `u`, and one set of rows for `other`, which must not leak.
    ///
    /// For `u`: HDFC (bank, balance 100,000 set 1 Oct, last statement 1 Aug), Kotak (bank, never
    /// uploaded, never set), a locked FD, and the card Millennia (statement to 20 Oct).
    /// Transactions on HDFC: salary 900,000 on 15 Aug, rent 300,000 on 20 Jul, tea 60,000 on
    /// 20 Sep, an uncategorised 5,000 on 20 Aug. All amounts in rupees here, stored as paise.
    async fn seed_user_data(state: &AppState) {
        seed(
            state,
            &[
                "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise, balance_set_on) VALUES ('acctA', 'u', 'HDFC', 'bank', 10000000, '2026-10-01')",
                "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise, balance_set_on) VALUES ('acctB', 'u', 'Kotak', 'bank', 0, NULL)",
                "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise, locked) VALUES ('acctL', 'u', 'Locked FD', 'fd', 0, 1)",
                "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise, balance_set_on) VALUES ('acctO', 'other', 'Other Bank', 'bank', 99999900, '2026-10-01')",
                "INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color) VALUES ('cardA', 'u', 'Millennia', 'HDFC', 0, 0)",
                "INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color) VALUES ('cardO', 'other', 'Their Card', 'HDFC', 0, 0)",
                "INSERT INTO statement_imports (import_id, user_id, account_id, first_date, last_date, rows) VALUES ('imp1', 'u', 'acctA', '2026-07-01', '2026-08-01', 40)",
                "INSERT INTO statement_imports (import_id, user_id, card_id, first_date, last_date, rows) VALUES ('imp2', 'u', 'cardA', '2026-10-01', '2026-10-20', 10)",
                "INSERT INTO statement_imports (import_id, user_id, account_id, first_date, last_date, rows) VALUES ('imp3', 'other', 'acctO', '2026-01-01', '2026-11-01', 40)",
                "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, account_id, raw_text, source, dedupe_key)
                 VALUES ('t1', 'u', '2026-08-15', 'credit', 90000000, 'Acme', 'income', 'acctA', 'x', 'manual', 'k1'),
                        ('t2', 'u', '2026-07-20', 'debit', 30000000, 'Asha Rent', 'rent', 'acctA', 'x', 'manual', 'k2'),
                        ('t3', 'u', '2026-09-20', 'debit', 6000000, 'Corner Tea', 'food', 'acctA', 'x', 'manual', 'k3'),
                        ('t4', 'u', '2026-08-20', 'debit', 500000, 'Shop', 'uncategorized', 'acctA', 'x', 'manual', 'k4')",
                "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, account_id, raw_text, source, dedupe_key)
                 VALUES ('o1', 'other', '2026-08-10', 'debit', 7777700, 'Their Shop', 'food', 'acctO', 'x', 'manual', 'ko1')",
            ],
        )
        .await;
    }

    #[tokio::test]
    async fn with_no_data_the_check_in_is_due_today_and_the_steps_are_in_order() {
        let state = test_state().await;
        let s = ok(compute_status(&state, "u", day(2026, 11, 9)).await);
        assert_eq!(s.last_done_on, None);
        assert_eq!(s.next_due_on, "2026-11-09");
        assert_eq!(s.status, DueState::DueSoon);
        assert_eq!(s.period, Span { start: "2026-08-11".into(), end: "2026-11-09".into() });
        let keys: Vec<&str> = s.steps.iter().map(|x| x.key).collect();
        assert_eq!(keys, vec!["statements", "balances", "payees", "body", "report"]);
        assert_eq!(s.steps.iter().map(|x| x.done).collect::<Vec<_>>(), vec![true, true, true, false, false]);
        assert!(s.uploads.is_empty() && s.estimates.is_empty());
    }

    #[tokio::test]
    async fn uploads_and_the_first_due_date_come_from_this_users_statements() {
        let state = test_state().await;
        seed_user_data(&state).await;
        let s = ok(compute_status(&state, "u", day(2026, 11, 9)).await);

        // HDFC covers to 1 Aug (needed), Kotak never uploaded (needed), the FD is not a bank and is
        // locked, the card covers to 20 Oct (not needed), and the other user's rows never appear.
        let rows: Vec<(&str, bool)> = s.uploads.iter().map(|u| (u.id.as_str(), u.needed)).collect();
        assert_eq!(rows, vec![("acctA", true), ("acctB", true), ("cardA", false)]);
        let st = s.steps.iter().find(|x| x.key == "statements").unwrap();
        assert_eq!((st.done, st.count), (false, 2));
        assert_eq!(st.detail, "2 of 3 need a newer statement.");

        // No check-in yet: the earliest bank statement ends 1 Aug, so 90 days on is 30 Oct, overdue now.
        assert_eq!(s.next_due_on, "2026-10-30");
        assert_eq!(s.status, DueState::Overdue);
    }

    #[tokio::test]
    async fn estimates_read_only_this_users_accounts_with_a_week_and_history() {
        let state = test_state().await;
        seed_user_data(&state).await;
        let s = ok(compute_status(&state, "u", day(2026, 11, 9)).await);

        // HDFC: 100,000 set on 1 Oct, 39 days ago. Every transaction falls in the 90 days to 1 Oct:
        // net 900,000 - 300,000 - 60,000 - 5,000 = 535,000 rupees, i.e. 53,500,000 paise.
        // Estimate: 10,000,000 + 53,500,000 * 39 / 90 = 33,183,333 paise. Monthly net: 17,833,333 paise.
        assert_eq!(s.estimates.len(), 1);
        let e = &s.estimates[0];
        assert_eq!(e.name, "HDFC");
        assert_eq!(e.known_paise, 10_000_000);
        assert_eq!(e.known_on, "2026-10-01");
        assert_eq!(e.days_since, 39);
        assert_eq!(e.estimate_paise, 33_183_333);
        assert_eq!(e.basis, "Usual net ₹1,78,333 a month in, over 39 days since 1 Oct.");
    }

    #[tokio::test]
    async fn a_check_in_moves_the_due_date_and_the_period() {
        let state = test_state().await;
        seed(
            &state,
            &["INSERT INTO checkins (checkin_id, user_id, done_on, period_start, period_end, summary_json)
               VALUES ('c1', 'u', '2026-08-10', '2026-05-12', '2026-08-10', '{}')"],
        )
        .await;
        let s = ok(compute_status(&state, "u", day(2026, 9, 1)).await);
        assert_eq!(s.last_done_on.as_deref(), Some("2026-08-10"));
        assert_eq!(s.next_due_on, "2026-11-09");
        assert_eq!(s.period, Span { start: "2026-08-11".into(), end: "2026-09-01".into() });
        assert_eq!(s.status, DueState::Ok);
    }

    #[tokio::test]
    async fn complete_saves_the_report_then_history_shows_it() {
        let state = test_state().await;
        seed_user_data(&state).await;
        let after = ok(complete_on(&state, "u", day(2026, 11, 9)).await);
        assert_eq!(after.last_done_on.as_deref(), Some("2026-11-09"));
        assert_eq!(after.next_due_on, "2027-02-08");
        assert_eq!(after.status, DueState::Ok);

        let rows = history_of(&state, "u").await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].done_on, "2026-11-09");
        assert_eq!(rows[0].period, Span { start: "2026-08-11".into(), end: "2026-11-09".into() });
        // In the period: salary 90,000,000 paise in. Spend: tea 6,000,000 and uncategorised 500,000.
        // The rent of 20 Jul is before the period.
        assert_eq!(rows[0].totals.income_paise, 90_000_000);
        assert_eq!(rows[0].totals.spend_paise, 6_000_000 + 500_000);
    }

    /// A plan for `u` with take-home and the emergency fund's saved amount, in rupees.
    async fn save_plan(state: &AppState, emergency_saved_rupees: i64) {
        let json = format!(r#"{{"my_take_home_paise": 10000000, "monthly_expenses_paise": 5000000, "emergency_saved_paise": {}}}"#, emergency_saved_rupees * 100);
        sqlx::query("INSERT INTO life_profiles (user_id, profile_json) VALUES ('u', ?) ON CONFLICT(user_id) DO UPDATE SET profile_json = excluded.profile_json")
            .bind(json)
            .execute(&state.db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn the_next_check_in_sets_the_kept_forecast_against_what_happened() {
        let state = test_state().await;
        save_plan(&state, 10_000).await;
        // First check-in: keeps the forecast as it stands.
        ok(complete_on(&state, "u", day(2026, 8, 10)).await);
        let kept: Option<String> = sqlx::query_scalar("SELECT forecast_json FROM checkins WHERE user_id = 'u'").fetch_one(&state.db).await.unwrap();
        let snap: crate::planner::calibration::Snapshot = serde_json::from_str(&kept.unwrap()).unwrap();
        assert_eq!(snap.made_on, "2026-08-10");
        let funded = snap.goals.iter().find(|g| g.key == "emergency").unwrap().funded_monthly_paise;
        assert!(funded > 0, "{snap:?}");
        // Nothing to compare with on the same day.
        assert!(ok(report_on(&state, "u", day(2026, 8, 10), ReportQuery { start: None, end: None }).await).calibration.is_none());

        // A quarter on, the emergency fund gained only ₹1,000.
        save_plan(&state, 11_000).await;
        let today = day(2026, 11, 9);
        let r = ok(report_on(&state, "u", today, ReportQuery { start: None, end: None }).await);
        let c = r.calibration.expect("a calibration");
        assert_eq!(c.since, "2026-08-10");
        let e = c.goals.iter().find(|g| g.key == "emergency").unwrap();
        assert_eq!(e.actual_paise, 100_000);
        assert!(!e.on_pace && e.expected_paise > 100_000, "{e:?}");
        let ft = c.next_follow_through_bps.unwrap();
        assert!(ft < 10_000, "{ft}");

        // The forecast now assumes only that share reaches the goals, and says so.
        let f = ok(crate::handlers::forecast::compute(&state, "u", today).await);
        assert_eq!(f.basis.follow_through_bps, Some(ft));
        assert!(f.basis.calibration_note.as_deref().unwrap().starts_with("Since your check-in on 10 Aug 2026, about "), "{:?}", f.basis.calibration_note);

        // Finishing this check-in keeps the comparison in the saved report, and a forecast that
        // remembers the share it assumed.
        ok(complete_on(&state, "u", today).await);
        let (summary, kept): (String, String) =
            sqlx::query_as("SELECT summary_json, forecast_json FROM checkins WHERE done_on = '2026-11-09'").fetch_one(&state.db).await.unwrap();
        assert!(summary.contains("\"calibration\":{"), "{summary}");
        let snap: crate::planner::calibration::Snapshot = serde_json::from_str(&kept).unwrap();
        assert_eq!(snap.applied_follow_through_bps, ft);
    }

    #[tokio::test]
    async fn another_users_kept_forecast_is_never_read() {
        let state = test_state().await;
        save_plan(&state, 10_000).await;
        seed(
            &state,
            &[r#"INSERT INTO checkins (checkin_id, user_id, done_on, period_start, period_end, summary_json, forecast_json)
                 VALUES ('x', 'other', '2026-08-10', '2026-05-12', '2026-08-10', '{}', '{"made_on":"2026-08-10","source":"profile","surplus_paise":1,"goals":[{"key":"emergency","label":"Emergency fund","saved_paise":0,"funded_monthly_paise":5000000,"likelihood_bps":0}]}')"#],
        )
        .await;
        let r = ok(report_on(&state, "u", day(2026, 11, 9), ReportQuery { start: None, end: None }).await);
        assert!(r.calibration.is_none());
    }

    #[tokio::test]
    async fn a_second_completion_today_replaces_todays_row_with_the_same_period() {
        let state = test_state().await;
        seed_user_data(&state).await;
        let today = day(2026, 11, 9);
        ok(complete_on(&state, "u", today).await);
        let first = history_of(&state, "u").await;
        ok(complete_on(&state, "u", today).await);
        let second = history_of(&state, "u").await;
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].period, first[0].period);
        assert_eq!(second[0].period, Span { start: "2026-08-11".into(), end: "2026-11-09".into() });
    }

    #[tokio::test]
    async fn the_next_check_in_starts_the_day_after_the_last_period_ends() {
        let state = test_state().await;
        seed_user_data(&state).await;
        ok(complete_on(&state, "u", day(2026, 11, 9)).await);
        ok(complete_on(&state, "u", day(2026, 11, 20)).await);
        let rows = history_of(&state, "u").await;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].done_on, "2026-11-20", "newest first");
        assert_eq!(rows[0].period, Span { start: "2026-11-10".into(), end: "2026-11-20".into() });
    }

    #[tokio::test]
    async fn history_is_newest_first_and_at_most_twelve() {
        let state = test_state().await;
        for i in 0..13u8 {
            let done = format!("2026-01-{:02}", i + 1);
            sqlx::query(
                "INSERT INTO checkins (checkin_id, user_id, done_on, period_start, period_end, summary_json)
                 VALUES (?, 'u', ?, '2026-01-01', '2026-01-02', ?)",
            )
            .bind(format!("c{i}"))
            .bind(&done)
            .bind(r#"{"totals":{"income_paise":5,"spend_paise":2,"invested_paise":1,"savings_rate_bps":6000}}"#)
            .execute(&state.db)
            .await
            .unwrap();
        }
        let rows = history_of(&state, "u").await;
        assert_eq!(rows.len(), 12);
        assert_eq!(rows[0].done_on, "2026-01-13", "the 13th day is newest");
        assert_eq!(rows[11].done_on, "2026-01-02", "the first day is cut off");
        assert!(rows.windows(2).all(|w| w[0].done_on > w[1].done_on));
        assert_eq!(rows[0].totals.savings_rate_bps, Some(6000));
    }

    #[tokio::test]
    async fn other_users_check_ins_and_statements_never_leak() {
        let state = test_state().await;
        seed_user_data(&state).await;
        seed(
            &state,
            &["INSERT INTO checkins (checkin_id, user_id, done_on, period_start, period_end, summary_json)
               VALUES ('theirs', 'other', '2026-11-01', '2026-08-01', '2026-11-01', '{\"totals\":{\"income_paise\":1,\"spend_paise\":1,\"invested_paise\":0,\"savings_rate_bps\":null}}')"],
        )
        .await;
        assert!(history_of(&state, "u").await.is_empty());
        assert!(history_of(&state, "other").await.len() == 1);

        let s = ok(compute_status(&state, "u", day(2026, 11, 9)).await);
        assert_eq!(s.last_done_on, None, "the other user's check-in does not count");
        assert!(s.uploads.iter().all(|u| u.id != "acctO" && u.id != "cardO"));

        let query = ReportQuery {
            start: Some("2026-08-01".into()),
            end: Some("2026-11-09".into()),
        };
        let r = ok(report_on(&state, "u", day(2026, 11, 9), query).await);
        // Their 77,777 on 10 Aug is not in the report: only tea and the uncategorised 5,000.
        assert_eq!(r.totals.spend_paise, 6_000_000 + 500_000);
    }

    #[tokio::test]
    async fn report_defaults_to_the_status_period_and_checks_the_dates() {
        let state = test_state().await;
        seed_user_data(&state).await;
        let today = day(2026, 11, 9);

        let no_query = ReportQuery { start: None, end: None };
        let r = ok(report_on(&state, "u", today, no_query).await);
        assert_eq!(r.period, Span { start: "2026-08-11".into(), end: "2026-11-09".into() });
        assert_eq!(r.previous, Span { start: "2026-05-12".into(), end: "2026-08-10".into() });

        let bad = |start: Option<&str>, end: Option<&str>| ReportQuery {
            start: start.map(str::to_string),
            end: end.map(str::to_string),
        };
        for q in [
            bad(Some("2026-13-01"), None),
            bad(Some("11/09/2026"), Some("2026-11-09")),
            bad(Some("2026-11-10"), Some("2026-11-09")),
            bad(Some("2025-01-01"), Some("2026-11-09")),
        ] {
            let Err(err) = report_on(&state, "u", today, q).await else {
                panic!("a bad query is refused");
            };
            assert_eq!(err.0, StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn on_the_day_a_check_in_is_finished_status_and_report_show_its_quarter() {
        let state = test_state().await;
        seed_user_data(&state).await;
        let today = day(2026, 11, 9);
        ok(complete_on(&state, "u", today).await);
        let no_query = ReportQuery { start: None, end: None };
        // The report shows the quarter that check-in covered.
        let r = ok(report_on(&state, "u", today, no_query).await);
        assert_eq!(r.period.start, "2026-08-11");
        assert_eq!(r.period.end, "2026-11-09");
        let s = ok(compute_status(&state, "u", today).await);
        assert!(s.done_today);
        assert_eq!(s.period, r.period);
    }
}

