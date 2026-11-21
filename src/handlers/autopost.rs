//! Applies due salary / fixed-payment postings to balances (see `planner::autopost`).
//! Runs lazily whenever balances are read, so it needs no scheduler and works on a machine that
//! sleeps between requests. Idempotent: `auto_postings` remembers what has been applied.

use std::collections::HashMap;

use time::{macros::format_description, Date, Duration};

use crate::{
    handlers::{common::AppError, plan::load_settings},
    planner::{
        autopost::{due_postings, Posting, Seen, MATCH_WINDOW_DAYS, MAX_LOOKBACK_DAYS},
        parse_year_month, Bucket, CommitmentIn, Freq, IncomeIn,
    },
};

fn parse_date(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

/// Account id -> the date its balance was last typed in. Accounts without one are never touched.
async fn load_baselines(db: &sqlx::SqlitePool, user_id: &str) -> Result<HashMap<String, Date>, AppError> {
    Ok(sqlx::query!(r#"SELECT account_id AS "account_id!", balance_set_on FROM accounts WHERE user_id = ?"#, user_id)
        .fetch_all(db)
        .await?
        .into_iter()
        .filter_map(|a| Some((a.account_id, parse_date(a.balance_set_on.as_deref()?)?)))
        .collect())
}

async fn load_incomes(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<(String, IncomeIn)>, AppError> {
    Ok(sqlx::query!(
        r#"SELECT income_id AS "income_id!", label, amount_paise, day_of_month, account_id
           FROM income_sources WHERE user_id = ? AND account_id IS NOT NULL"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|i| (i.income_id, IncomeIn { label: i.label, amount: i.amount_paise, day: i.day_of_month as u8, account_id: i.account_id }))
    .collect())
}

async fn load_commitments(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<(String, CommitmentIn)>, AppError> {
    Ok(sqlx::query!(
        r#"SELECT commitment_id AS "commitment_id!", label, amount_paise, day_of_month, account_id, bucket,
                  frequency, due_month, every_months, first_month, to_account_id, active AS "active: bool", start_month, end_month
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
                freq: Freq::from_columns(&c.frequency, c.every_months, c.first_month.as_deref()),
                due_month: c.due_month.map(|m| m as u8),
                to_account_id: c.to_account_id,
                start_month: c.start_month.as_deref().and_then(parse_year_month),
                end_month: c.end_month.as_deref().and_then(parse_year_month),
            },
        )
    })
    .collect())
}

/// Recent money movements on the user's accounts, signed: a posting that matches one is not made again.
async fn load_seen(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<Vec<Seen>, AppError> {
    let since = (today - Duration::days(MAX_LOOKBACK_DAYS + MATCH_WINDOW_DAYS)).to_string();
    Ok(sqlx::query!(
        r#"SELECT account_id AS "account_id!", occurred_on, direction, amount_paise FROM transactions
           WHERE user_id = ? AND account_id IS NOT NULL AND occurred_on >= ?"#,
        user_id,
        since
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .filter_map(|t| {
        let sign = if t.direction == "credit" { 1 } else { -1 };
        Some(Seen { account_id: t.account_id, on: parse_date(&t.occurred_on)?, amount: sign * t.amount_paise })
    })
    .collect())
}

/// Records one posting and moves the balance, unless it was already taken.
async fn post(conn: &mut sqlx::SqliteConnection, user_id: &str, p: &Posting) -> Result<(), AppError> {
    let (kind, on) = (p.kind.as_str(), p.on.to_string());
    // One posting per source, account and month. If its day was moved after this month's was
    // taken, the new day must not take it a second time. (A runtime query, so `.sqlx` is unchanged.)
    let already: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM auto_postings WHERE source_id = ? AND account_id = ? AND substr(occurs_on, 1, 7) = ? AND occurs_on <> ?",
    )
    .bind(&p.source_id)
    .bind(&p.account_id)
    .bind(&on[..7])
    .bind(&on)
    .fetch_one(&mut *conn)
    .await?;
    if already > 0 {
        return Ok(());
    }
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
    .execute(&mut *conn)
    .await?
    .rows_affected();
    if inserted == 1 {
        // updated_at is left alone: the balance is an estimate, so "update it" reminders stay.
        sqlx::query!("UPDATE accounts SET balance_paise = balance_paise + ? WHERE account_id = ? AND user_id = ?", p.amount, p.account_id, user_id)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

/// Posts everything due for `user_id` as of `today`, unless the user turned auto-posting off.
pub async fn apply(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<(), AppError> {
    if !load_settings(db, user_id).await?.auto_post {
        return Ok(());
    }
    let baselines = load_baselines(db, user_id).await?;
    if baselines.is_empty() {
        return Ok(());
    }
    let (incomes, commitments) = (load_incomes(db, user_id).await?, load_commitments(db, user_id).await?);
    let seen = load_seen(db, user_id, today).await?;

    let due = due_postings(&incomes, &commitments, &baselines, &seen, today);
    if due.is_empty() {
        return Ok(());
    }
    let mut tx = db.begin().await?;
    for p in &due {
        post(&mut tx, user_id, p).await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Characterization test for [`apply`]: balances and postings after several runs over a seeded database
/// are pinned. A refactor must leave `autopost_golden.json` byte-for-byte alone. Regenerate on purpose
/// with `UPDATE_GOLDEN=1 cargo test autopost`.
#[cfg(test)]
mod golden {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use time::macros::date;

    const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/handlers/autopost_golden.json");

    async fn seeded() -> sqlx::SqlitePool {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        let stmts = [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u1','u1','x','user'),('u2','u2','x','user'),('u3','u3','x','user')",
            "INSERT INTO user_settings (user_id, auto_post) VALUES ('u2', 0)",
            "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise, balance_set_on) VALUES
                ('a1','u1','Spend','bank',1000000,'2026-09-01'),
                ('a2','u1','Reserve','bank',5000000,'2026-09-20'),
                ('a3','u1','No baseline','bank',700,NULL),
                ('b1','u2','Off','bank',100,'2026-09-01'),
                ('c1','u3','Typed today','bank',100,'2026-11-21')",
            "INSERT INTO income_sources (income_id, user_id, label, amount_paise, day_of_month, account_id) VALUES
                ('i1','u1','Salary',9000000,30,'a1'),
                ('i2','u1','Side',100000,5,'a2'),
                ('i3','u1','Unassigned',999,5,NULL),
                ('i4','u1','To untouched account',555,6,'a3'),
                ('j1','u2','Salary',100,5,'b1'),
                ('k1','u3','Salary',100,5,'c1')",
            "INSERT INTO commitments (commitment_id, user_id, label, amount_paise, day_of_month, account_id, to_account_id, bucket, frequency, due_month, every_months, first_month, start_month, end_month) VALUES
                ('m1','u1','Rent',2200000,3,'a2',NULL,'needs','monthly',NULL,1,NULL,NULL,NULL),
                ('m2','u1','SIP',3000000,10,'a1','a2','savings','monthly',NULL,1,NULL,NULL,NULL),
                ('m3','u1','Insurance',2400000,12,'a2',NULL,'needs','yearly',10,1,NULL,NULL,NULL),
                ('m4','u1','Perfume',400000,18,'a1',NULL,'wants','monthly',NULL,4,'2026-09',NULL,NULL),
                ('m5','u1','Deposit in',100000,14,NULL,'a1','savings','monthly',NULL,1,NULL,NULL,NULL),
                ('m6','u1','Course',300000,22,'a1',NULL,'needs','monthly',NULL,1,NULL,'2026-11','2027-02'),
                ('m7','u1','Moved day',150000,9,'a2',NULL,'needs','monthly',NULL,1,NULL,NULL,NULL),
                ('m8','u1','Paused',999000,8,'a1',NULL,'wants','monthly',NULL,1,NULL,NULL,NULL)",
            "UPDATE commitments SET active = 0 WHERE commitment_id = 'm8'",
            // The rent for October already went through on the bank side: the posting must not double it.
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, category, account_id, source, dedupe_key)
             VALUES ('s1','u1','2026-10-04','debit',2200000,'rent','a2','sms','k1')",
            // m7's day was moved: the 9th of November was taken on the 7th, so it must not be taken again.
            "INSERT INTO auto_postings (user_id, source_kind, source_id, account_id, occurs_on, label, amount_paise)
             VALUES ('u1','commitment','m7','a2','2026-11-07','Moved day',-150000)",
        ];
        for s in stmts {
            sqlx::query(s).execute(&db).await.unwrap();
        }
        db
    }

    async fn dump(db: &sqlx::SqlitePool) -> serde_json::Value {
        let accounts: Vec<(String, i64)> = sqlx::query_as("SELECT account_id, balance_paise FROM accounts ORDER BY account_id").fetch_all(db).await.unwrap();
        let postings: Vec<(String, String, String, String, String, i64)> =
            sqlx::query_as("SELECT user_id, source_kind, source_id, account_id || ' ' || occurs_on, label, amount_paise FROM auto_postings ORDER BY user_id, source_id, occurs_on, account_id")
                .fetch_all(db)
                .await
                .unwrap();
        serde_json::json!({ "accounts": accounts, "postings": postings })
    }

    async fn snapshot() -> String {
        let db = seeded().await;
        let mut stages = Vec::new();
        for (name, today) in [("first_run", date!(2026 - 11 - 21)), ("same_day_again", date!(2026 - 11 - 21)), ("a_week_later", date!(2026 - 11 - 28)), ("a_year_later_hits_lookback", date!(2027 - 11 - 21))] {
            for user in ["u1", "u2", "u3", "nobody"] {
                assert!(apply(&db, user, today).await.is_ok());
            }
            stages.push(serde_json::json!({ "after": name, "state": dump(&db).await }));
        }
        serde_json::to_string_pretty(&stages).unwrap() + "\n"
    }

    #[tokio::test]
    async fn autopost_matches_the_pinned_output() {
        let now = snapshot().await;
        if std::env::var("UPDATE_GOLDEN").is_ok() {
            std::fs::write(FILE, &now).unwrap();
            return;
        }
        let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
        assert!(now == pinned, "auto-posting changed; if on purpose, rerun with UPDATE_GOLDEN=1");
    }
}
