//! Linking a kept date to the bank debit that paid for it.

#![allow(unused_imports)]
use super::*;
use serde::Serialize;
use sqlx::SqlitePool;

/// A debit of the asker's that could be what a date cost.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Candidate_ {
    pub txn_id: String,
    pub merchant: String,
    pub amount_paise: i64,
    pub day: String,
}

pub type TxnCandidate = Candidate_;

/// The asker's own debits from two days before the date to three days after it, that no date has
/// claimed yet, closest in day then in amount first. Only their own: the partner's bank data is theirs.
pub async fn txn_candidates(db: &SqlitePool, user_id: &str, history_id: i64) -> sqlx::Result<Vec<TxnCandidate>> {
    sqlx::query_as(
        "SELECT t.txn_id, COALESCE(t.merchant, '') AS merchant, t.amount_paise, t.occurred_on AS day
           FROM joy_history h
           JOIN transactions t ON t.user_id = ?1 AND t.direction = 'debit'
          WHERE h.id = ?2
            AND julianday(t.occurred_on) BETWEEN julianday(date(COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime')) - 2
                                             AND julianday(date(COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime')) + 3
            AND t.txn_id NOT IN (SELECT txn_id FROM joy_history WHERE txn_id IS NOT NULL)
          ORDER BY ABS(julianday(t.occurred_on) - julianday(date(COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime'))),
                   ABS(t.amount_paise / 100 - COALESCE(h.est_cost, 0)), t.txn_id
          LIMIT 8",
    )
    .bind(user_id)
    .bind(history_id)
    .fetch_all(db)
    .await
}

#[derive(Debug, PartialEq, Eq)]
pub enum LinkOutcome {
    Linked,
    /// Not a debit of this person's.
    NoSuchTransaction,
    /// Already paying for another date.
    Taken,
    NoSuchDate,
}

/// Link `txn_id` to the date: its real cost becomes the transaction's amount (rounded up to a whole
/// rupee), and the transaction is labelled a date and locked so a later re-sort leaves it alone.
pub async fn link_txn(db: &SqlitePool, couple: &str, user_id: &str, history_id: i64, txn_id: &str) -> sqlx::Result<LinkOutcome> {
    let mut tx = db.begin().await?;
    let date: Option<i64> = sqlx::query_scalar("SELECT id FROM joy_history WHERE id = ? AND couple_id = ? AND outcome = 'did_it'")
        .bind(history_id)
        .bind(couple)
        .fetch_optional(&mut *tx)
        .await?;
    if date.is_none() {
        return Ok(LinkOutcome::NoSuchDate);
    }
    let paise: Option<i64> = sqlx::query_scalar("SELECT amount_paise FROM transactions WHERE txn_id = ? AND user_id = ? AND direction = 'debit'")
        .bind(txn_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(paise) = paise else { return Ok(LinkOutcome::NoSuchTransaction) };
    let taken: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM joy_history WHERE txn_id = ? AND id <> ?)")
        .bind(txn_id)
        .bind(history_id)
        .fetch_one(&mut *tx)
        .await?;
    if taken {
        return Ok(LinkOutcome::Taken);
    }
    let rupees = (paise + 99) / 100;
    sqlx::query("UPDATE joy_history SET txn_id = ?, actual_cost = ? WHERE id = ? AND couple_id = ?")
        .bind(txn_id)
        .bind(rupees)
        .bind(history_id)
        .bind(couple)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE transactions SET category = 'dates', category_locked = 1 WHERE txn_id = ? AND user_id = ?")
        .bind(txn_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(LinkOutcome::Linked)
}

/// Take the link off. The transaction is unlocked and sorted again later; the date keeps its cost.
pub async fn unlink_txn(db: &SqlitePool, couple: &str, history_id: i64) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let txn: Option<Option<String>> = sqlx::query_scalar("SELECT txn_id FROM joy_history WHERE id = ? AND couple_id = ?")
        .bind(history_id)
        .bind(couple)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(Some(txn_id)) = txn else { return Ok(false) };
    sqlx::query("UPDATE joy_history SET txn_id = NULL WHERE id = ? AND couple_id = ?").bind(history_id).bind(couple).execute(&mut *tx).await?;
    sqlx::query("UPDATE transactions SET category_locked = 0, category = 'uncategorized' WHERE txn_id = ?").bind(&txn_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(true)
}

/// For the Kept page: the linked transactions of the asker's dates, by history id.
pub async fn linked_txns(db: &SqlitePool, couple: &str, user_id: &str) -> sqlx::Result<Vec<(i64, TxnCandidate)>> {
    let rows: Vec<(i64, String, String, i64, String)> = sqlx::query_as(
        "SELECT h.id, t.txn_id, COALESCE(t.merchant, ''), t.amount_paise, t.occurred_on
           FROM joy_history h JOIN transactions t ON t.txn_id = h.txn_id AND t.user_id = ?
          WHERE h.couple_id = ?",
    )
    .bind(user_id)
    .bind(couple)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(|(id, txn_id, merchant, amount_paise, day)| (id, TxnCandidate { txn_id, merchant, amount_paise, day })).collect())
}
