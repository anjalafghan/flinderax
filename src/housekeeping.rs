//! Small daily clean-ups, run by the first dashboard load of the day (see `backup::maybe_backup`).
//! Each is safe to run twice and must never fail the page that triggered it.

use sqlx::SqlitePool;

/// How long the full text of a bank message is kept when no statement has confirmed it.
pub const RAW_TEXT_DAYS: i64 = 90;

/// Forget the original bank-alert text once it has done its job: when a statement row has
/// confirmed the transaction, or after 90 days. The transaction itself stays. Returns how many
/// rows were cleared.
pub async fn purge_raw_text(db: &SqlitePool) -> sqlx::Result<u64> {
    let done = sqlx::query(
        "UPDATE transactions SET raw_text = NULL
         WHERE raw_text IS NOT NULL AND (on_statement = 1 OR created_at < datetime('now', ?))",
    )
    .bind(format!("-{RAW_TEXT_DAYS} days"))
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

/// The day the clean-up last ran in this process, so the check is free after the first call.
static LAST: std::sync::Mutex<Option<time::Date>> = std::sync::Mutex::new(None);

/// Runs [`daily`] the first time it is called each day.
pub async fn daily_once(db: &SqlitePool, today: time::Date) {
    {
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        if *last == Some(today) {
            return;
        }
        *last = Some(today);
    }
    daily(db).await;
}

/// The daily run: failures are logged, not returned.
pub async fn daily(db: &SqlitePool) {
    if let Err(e) = purge_raw_text(db).await {
        tracing::error!("housekeeping: purging old alert text failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn db() -> SqlitePool {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'uu', 'x', 'user')").execute(&db).await.unwrap();
        db
    }

    async fn txn(db: &SqlitePool, id: &str, age_days: i64, on_statement: i64) {
        sqlx::query(
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, source, raw_text, dedupe_key, on_statement, created_at)
             VALUES (?, 'u', '2026-01-01', 'debit', 100, 'sms', 'Rs 1 debited at SHOP', ?, ?, datetime('now', ?))",
        )
        .bind(id)
        .bind(format!("k-{id}"))
        .bind(on_statement)
        .bind(format!("-{age_days} days"))
        .execute(db)
        .await
        .unwrap();
    }

    async fn kept(db: &SqlitePool, id: &str) -> bool {
        sqlx::query_scalar::<_, Option<String>>("SELECT raw_text FROM transactions WHERE txn_id = ?").bind(id).fetch_one(db).await.unwrap().is_some()
    }

    #[tokio::test]
    async fn alert_text_goes_after_90_days_or_once_a_statement_confirms_it() {
        let db = db().await;
        txn(&db, "fresh", 5, 0).await;
        txn(&db, "old", 120, 0).await;
        txn(&db, "confirmed", 5, 1).await;
        assert_eq!(purge_raw_text(&db).await.unwrap(), 2);
        assert!(kept(&db, "fresh").await, "a recent, unconfirmed alert keeps its text");
        assert!(!kept(&db, "old").await);
        assert!(!kept(&db, "confirmed").await);
        // The transactions themselves are untouched, and a second run changes nothing.
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM transactions").fetch_one(&db).await.unwrap();
        assert_eq!(n, 3);
        assert_eq!(purge_raw_text(&db).await.unwrap(), 0);
    }
}
