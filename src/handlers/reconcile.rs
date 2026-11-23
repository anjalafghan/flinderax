//! `GET /api/ingest/reconcile`: between observed bank balances, does what joy recorded add up?
//! The rule is pure in `planner::reconcile`; this module loads the user's rows.

use axum::{extract::State, Extension, Json};
use sqlx::SqlitePool;
use time::{macros::format_description, Date, Duration};

use crate::{
    handlers::common::{today_local, utc_offset_minutes, AppError},
    models::AppState,
    planner::reconcile::{
        self, AccountIn, AdjustIn, AdjustKind, ReconcileIn, Report, SnapSource, SnapshotIn, TxnIn,
    },
};

/// Rows older than this (before the earliest snapshot, or today) are not loaded.
const LOOKBACK_DAYS: i64 = 200;

fn parse_day(s: &str) -> Option<Date> {
    Date::parse(s, format_description!("[year]-[month]-[day]")).ok()
}

pub async fn report(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<Report>, AppError> {
    Ok(Json(load_report(&state.db, &user_id, today_local()).await?))
}

pub async fn load_report(db: &SqlitePool, user_id: &str, today: Date) -> Result<Report, AppError> {
    let fmt = format_description!("[year]-[month]-[day]");
    let floor = (today - Duration::days(LOOKBACK_DAYS)).format(&fmt).unwrap_or_default();

    let accounts: Vec<(String, String, String, i64)> =
        sqlx::query_as("SELECT account_id, name, kind, locked FROM accounts WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(db)
            .await?;
    let accounts: Vec<AccountIn> = accounts
        .into_iter()
        .map(|(account_id, name, kind, locked)| AccountIn { account_id, name, kind, locked: locked != 0 })
        .collect();

    let snaps: Vec<(String, String, i64, String, i64)> = sqlx::query_as(
        "SELECT account_id, as_of, balance_paise, source, rowid FROM balance_snapshots
         WHERE user_id = ? AND as_of >= ? ORDER BY as_of, rowid",
    )
    .bind(user_id)
    .bind(&floor)
    .fetch_all(db)
    .await?;
    let snapshots: Vec<SnapshotIn> = snaps
        .into_iter()
        .filter_map(|(account_id, as_of, balance_paise, source, seq)| {
            let source = match source.as_str() {
                "alert" => SnapSource::Alert,
                "statement" => SnapSource::Statement,
                _ => SnapSource::Typed,
            };
            Some(SnapshotIn { account_id, as_of: parse_day(&as_of)?, balance_paise, source, seq })
        })
        .collect();

    // Flows from the earliest snapshot's day on (a little before, for the match window).
    let from = snapshots
        .iter()
        .map(|s| s.as_of)
        .min()
        .map(|d| d - Duration::days(3))
        .unwrap_or(today)
        .format(&fmt)
        .unwrap_or_default();

    let txn_rows: Vec<(String, String, i64, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT occurred_on, direction, amount_paise, account_id, card_id,
                (source != 'manual' AND dedupe_key NOT LIKE 'stmt:%')
         FROM transactions WHERE user_id = ? AND occurred_on >= ?",
    )
    .bind(user_id)
    .bind(&from)
    .fetch_all(db)
    .await?;
    let txns: Vec<TxnIn> = txn_rows
        .into_iter()
        .filter_map(|(on, direction, amount, account_id, card_id, from_alert)| {
            let signed = if direction == "credit" { amount } else { -amount };
            Some(TxnIn { on: parse_day(&on)?, amount_paise: signed, account_id, card_id, from_alert: from_alert != 0 })
        })
        .collect();

    let mut adjustments = Vec::new();
    let autos: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT account_id, occurs_on, amount_paise FROM auto_postings WHERE user_id = ? AND occurs_on >= ?",
    )
    .bind(user_id)
    .bind(&from)
    .fetch_all(db)
    .await?;
    for (account_id, on, amount_paise) in autos {
        if let Some(on) = parse_day(&on) {
            adjustments.push(AdjustIn { account_id, on, amount_paise, kind: AdjustKind::AutoPost });
        }
    }

    // `created_at` is UTC; the bank's day is local.
    let offset = format!("{:+} minutes", utc_offset_minutes());
    let moves: Vec<(Option<String>, Option<String>, i64, String)> = sqlx::query_as(
        "SELECT from_account_id, to_account_id, amount_paise, date(created_at, ?) FROM moves
         WHERE user_id = ? AND mode = 'moved' AND date(created_at, ?) >= ?",
    )
    .bind(&offset)
    .bind(user_id)
    .bind(&offset)
    .bind(&from)
    .fetch_all(db)
    .await?;
    for (from_id, to_id, amount, on) in moves {
        let Some(on) = parse_day(&on) else { continue };
        if let Some(account_id) = from_id {
            adjustments.push(AdjustIn { account_id, on, amount_paise: -amount, kind: AdjustKind::Move });
        }
        if let Some(account_id) = to_id {
            adjustments.push(AdjustIn { account_id, on, amount_paise: amount, kind: AdjustKind::Move });
        }
    }

    Ok(reconcile::reconcile(&ReconcileIn { today, accounts, snapshots, txns, adjustments }))
}
