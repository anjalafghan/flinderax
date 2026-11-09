//! `GET /api/dashboard/analytics`: the user's last 400 days of transactions and their statement
//! coverage, run through the pure analytics in `planner::analytics`.

use axum::{extract::State, Extension, Json};
use time::{macros::format_description, Date, Duration};

use crate::{
    handlers::common::{today_local, AppError},
    models::AppState,
    planner::analytics::{self, AnalyticsOut, CoverageIn, Txn},
};

const HISTORY_DAYS: i64 = 400;

pub async fn get(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<AnalyticsOut>, AppError> {
    Ok(Json(compute(&state.db, &user_id, today_local()).await?))
}

/// Loads the user's data as of `today` and computes the analytics.
pub async fn compute(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<AnalyticsOut, AppError> {
    let since = (today - Duration::days(HISTORY_DAYS)).to_string();
    type Row = (String, String, i64, Option<String>, String, Option<String>);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT occurred_on, direction, amount_paise, merchant, category, raw_text
         FROM transactions WHERE user_id = ? AND occurred_on >= ? AND occurred_on <= ?",
    )
    .bind(user_id)
    .bind(since)
    .bind(today.to_string())
    .fetch_all(db)
    .await?;
    let txns: Vec<Txn> = rows
        .into_iter()
        .filter_map(|(on, direction, amount, merchant, category, raw_text)| {
            let on = Date::parse(&on, format_description!("[year]-[month]-[day]")).ok()?;
            Some(Txn { on, debit: direction == "debit", amount, merchant, category, raw_text })
        })
        .collect();

    let coverage = statement_coverage(db, user_id).await;
    Ok(analytics::build(&txns, coverage, today))
}

/// Span of the user's statement imports. A missing table, a query error or no imports all mean
/// there is no coverage, so the panel shows its empty state rather than failing.
async fn statement_coverage(db: &sqlx::SqlitePool, user_id: &str) -> Option<CoverageIn> {
    let (first, last, imports): (Option<String>, Option<String>, i64) = sqlx::query_as(
        "SELECT MIN(first_date), MAX(last_date), COUNT(*) FROM statement_imports WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .ok()?;
    let fmt = format_description!("[year]-[month]-[day]");
    Some(CoverageIn {
        first: Date::parse(&first?, fmt).ok()?,
        last: Date::parse(&last?, fmt).ok()?,
        imports,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    #[tokio::test]
    async fn analytics_reads_only_this_users_transactions_inside_the_window() {
        use sqlx::sqlite::SqlitePoolOptions;
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('other', 'other', 'x', 'user')",
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key)
             VALUES ('t1', 'u', '2026-10-05', 'debit', 2000, 'Corner Tea', 'food', 'UPI payment', 'manual', 'k1'),
                    ('t2', 'u', '2026-10-06', 'debit', 3000, 'Corner Tea', 'food', 'UPI payment', 'manual', 'k2'),
                    ('t3', 'other', '2026-10-06', 'debit', 999999, 'Other Shop', 'food', 'UPI payment', 'manual', 'k3'),
                    ('t4', 'u', '2020-01-01', 'debit', 777777, 'Old Shop', 'food', 'UPI payment', 'manual', 'k4')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let today = Date::from_calendar_date(2026, Month::November, 9).unwrap();
        let out = compute(&db, "u", today).await.ok().expect("analytics compute");

        let oct = out.months.iter().find(|m| m.month == "2026-10").expect("october");
        assert_eq!(oct.spend_paise, 5000);
        assert_eq!(out.months.iter().map(|m| m.spend_paise).sum::<i64>(), 5000);
        assert_eq!(out.categories.len(), 1);
        assert_eq!(out.top_payees.first().map(|p| p.name.as_str()), Some("CORNER TEA"));
        assert!(out.coverage.is_none());
    }
}
