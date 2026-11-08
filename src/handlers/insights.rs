//! `GET /api/dashboard/insights`: the user's last 400 days of transactions and liquid balances,
//! run through the pure insights in `planner::insights`.

use axum::{extract::State, Extension, Json};
use time::{macros::format_description, Date, Duration};

use crate::{
    handlers::common::{today_local, AppError},
    models::AppState,
    planner::{
        insights::{self, InsightsOut, Txn},
        pay_period, IncomeIn,
    },
};

const HISTORY_DAYS: i64 = 400;

pub async fn get(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<InsightsOut>, AppError> {
    Ok(Json(compute(&state.db, &user_id, today_local()).await?))
}

/// Loads the user's data as of `today` and computes every insight.
pub async fn compute(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<InsightsOut, AppError> {
    let since = (today - Duration::days(HISTORY_DAYS)).to_string();
    type Row = (String, String, String, i64, Option<String>, String, Option<String>);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT txn_id, occurred_on, direction, amount_paise, merchant, category, raw_text
         FROM transactions WHERE user_id = ? AND occurred_on >= ? AND occurred_on <= ?",
    )
    .bind(user_id)
    .bind(since)
    .bind(today.to_string())
    .fetch_all(db)
    .await?;
    let txns: Vec<Txn> = rows
        .into_iter()
        .filter_map(|(txn_id, on, direction, amount, merchant, category, raw_text)| {
            let on = Date::parse(&on, format_description!("[year]-[month]-[day]")).ok()?;
            Some(Txn { txn_id, on, debit: direction == "debit", amount, merchant, category, raw_text })
        })
        .collect();

    let liquid: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(balance_paise), 0) FROM accounts WHERE user_id = ? AND locked = 0",
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;

    let incomes: Vec<IncomeIn> = sqlx::query_as::<_, (String, i64, i64, Option<String>)>(
        "SELECT label, amount_paise, day_of_month, account_id FROM income_sources WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|(label, amount, day, account_id)| IncomeIn { label, amount, day: day as u8, account_id })
    .collect();
    let next_payday = (!incomes.is_empty()).then(|| pay_period(today, &incomes).payday1);

    Ok(insights::build(&txns, liquid, next_payday, today))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::insights::Rail;
    use time::Month;

    #[tokio::test]
    async fn failed_upi_debit_and_a_fee_show_up() {
        use sqlx::sqlite::SqlitePoolOptions;
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise) VALUES ('acctA', 'u', 'HDFC', 'bank', 300000)",
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key)
             VALUES ('t1', 'u', '2026-11-01', 'debit', 145000, 'rahul.k@okaxis', 'uncategorized', 'UPI payment to rahul.k@okaxis failed', 'sms', 'fp:t1'),
                    ('t2', 'u', '2026-10-20', 'debit', 50000, 'HDFC Bank', 'uncategorized', 'Annual fee debited for card', 'sms', 'fp:t2'),
                    ('t3', 'u', '2026-10-20', 'debit', 9000, 'HDFC Bank', 'uncategorized', 'GST on annual fee', 'sms', 'fp:t3')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let today = Date::from_calendar_date(2026, Month::November, 9).unwrap();
        let out = compute(&db, "u", today).await.ok().expect("insights compute");

        assert_eq!(out.refunds.len(), 1);
        let r = &out.refunds[0];
        assert_eq!(
            (r.txn_id.as_str(), r.rail, r.due_by.as_str(), r.days_overdue, r.compensation_paise),
            ("t1", Rail::P2p, "2026-11-02", 7, 70_000)
        );
        assert_eq!(
            out.charges.items.iter().map(|i| (i.label.as_str(), i.amount_paise, i.count)).collect::<Vec<_>>(),
            vec![("Annual fee", 59_000, 2)]
        );
    }
}
