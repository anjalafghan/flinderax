//! Applies due salary / fixed-payment postings to balances (see `planner::autopost`).
//! Runs lazily whenever balances are read, so it needs no scheduler and works on a machine that
//! sleeps between requests. Idempotent: `auto_postings` remembers what has been applied.

use std::collections::HashMap;

use time::{macros::format_description, Date};

use crate::{
    handlers::{common::AppError, plan::load_settings},
    planner::{
        autopost::due_postings, parse_year_month, Bucket, CommitmentIn, Freq, IncomeIn,
    },
};

fn parse_date(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

/// Posts everything due for `user_id` as of `today`, unless the user turned auto-posting off.
pub async fn apply(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<(), AppError> {
    if !load_settings(db, user_id).await?.auto_post {
        return Ok(());
    }

    let baselines: HashMap<String, Date> = sqlx::query!(
        r#"SELECT account_id AS "account_id!", balance_set_on FROM accounts WHERE user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .filter_map(|a| Some((a.account_id, parse_date(a.balance_set_on.as_deref()?)?)))
    .collect();
    if baselines.is_empty() {
        return Ok(());
    }

    let incomes: Vec<(String, IncomeIn)> = sqlx::query!(
        r#"SELECT income_id AS "income_id!", label, amount_paise, day_of_month, account_id
           FROM income_sources WHERE user_id = ? AND account_id IS NOT NULL"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|i| {
        (
            i.income_id,
            IncomeIn { label: i.label, amount: i.amount_paise, day: i.day_of_month as u8, account_id: i.account_id },
        )
    })
    .collect();

    let commitments: Vec<(String, CommitmentIn)> = sqlx::query!(
        r#"SELECT commitment_id AS "commitment_id!", label, amount_paise, day_of_month, account_id, bucket,
                  frequency, due_month, to_account_id, active AS "active: bool", start_month, end_month
           FROM commitments
           WHERE user_id = ? AND active = 1 AND (account_id IS NOT NULL OR to_account_id IS NOT NULL)"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|c| {
        (
            c.commitment_id,
            CommitmentIn {
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
            },
        )
    })
    .collect();

    let due = due_postings(&incomes, &commitments, &baselines, today);
    if due.is_empty() {
        return Ok(());
    }

    let mut tx = db.begin().await?;
    for p in due {
        let (kind, on) = (p.kind.as_str(), p.on.to_string());
        let inserted = sqlx::query!(
            "INSERT OR IGNORE INTO auto_postings (user_id, source_kind, source_id, account_id, occurs_on, label, amount_paise)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            user_id,
            kind,
            p.source_id,
            p.account_id,
            on,
            p.label,
            p.amount
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if inserted == 1 {
            // updated_at is left alone: the balance is an estimate, so "update it" reminders stay.
            sqlx::query!(
                "UPDATE accounts SET balance_paise = balance_paise + ? WHERE account_id = ? AND user_id = ?",
                p.amount,
                p.account_id,
                user_id
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}
