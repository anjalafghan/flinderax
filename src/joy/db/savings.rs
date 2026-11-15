//! The savings jar.

#![allow(unused_imports)]
use super::*;
use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

// ── the savings jar ───────────────────────────────────────────────────────

/// What's in the savings jar now.
pub async fn savings_balance(db: &SqlitePool, couple: &str) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT COALESCE(SUM(amount), 0) FROM joy_savings WHERE couple_id = ?").bind(couple).fetch_one(db).await
}

/// Put money in (or, negative, take it out). `kind` is add | leftover | kept | back.
pub async fn add_saving(db: &SqlitePool, couple: &str, amount: i64, kind: &str, note: &str) -> sqlx::Result<()> {
    if amount == 0 {
        return Ok(());
    }
    sqlx::query("INSERT INTO joy_savings (couple_id, amount, kind, note) VALUES (?, ?, ?, ?)")
        .bind(couple)
        .bind(amount)
        .bind(kind)
        .bind(note.trim())
        .execute(db)
        .await?;
    Ok(())
}

/// The latest money in and out, newest first: (id, amount, kind, note, on_day).
pub async fn savings_recent(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(i64, i64, String, String, String)>> {
    sqlx::query_as("SELECT id, amount, kind, note, on_day FROM joy_savings WHERE couple_id = ? ORDER BY id DESC LIMIT 30")
        .bind(couple)
        .fetch_all(db)
        .await
}

/// Undo money put in by hand. Leftovers and paid dates stay, so the jar's sums stay true.
/// False (and nothing deleted) when the jar no longer holds that money: removing the deposit would
/// leave it below zero because part of it has already paid for a date.
pub async fn delete_saving(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<bool> {
    let done = sqlx::query(
        "DELETE FROM joy_savings WHERE id = ?1 AND couple_id = ?2 AND kind = 'add'
           AND (SELECT COALESCE(SUM(amount), 0) FROM joy_savings WHERE couple_id = ?2) - amount >= 0",
    )
    .bind(id)
    .bind(couple)
    .execute(db)
    .await?;
    if done.rows_affected() == 1 {
        return Ok(true);
    }
    // Nothing deleted: either it isn't a deposit of ours (fine, nothing to do) or the jar can't spare it.
    let is_deposit: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM joy_savings WHERE id = ? AND couple_id = ? AND kind = 'add')")
        .bind(id)
        .bind(couple)
        .fetch_one(db)
        .await?;
    Ok(!is_deposit)
}

/// Keep the slip we were saving for, paid from the jar: stamps it as a date, takes `cost`
/// out of the jar and stops saving for it. None (and nothing changed) when we've stopped
/// saving for `activity`, or there isn't enough in the jar yet. A double tap keeps it once.
pub async fn keep_from_savings(db: &SqlitePool, couple: &str, activity: i64, cost: i64, drawn_by: Option<usize>) -> sqlx::Result<Option<i64>> {
    let mut tx = db.begin().await?;
    let stopped = sqlx::query("DELETE FROM joy_settings WHERE couple_id = ? AND key = 'saving_for' AND value = ?")
        .bind(couple)
        .bind(activity.to_string())
        .execute(&mut *tx)
        .await?
        .rows_affected()
        == 0;
    let balance: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(amount), 0) FROM joy_savings WHERE couple_id = ?")
        .bind(couple)
        .fetch_one(&mut *tx)
        .await?;
    if stopped || balance < cost {
        tx.rollback().await?;
        return Ok(None);
    }
    let kept = sqlx::query(
        "INSERT INTO joy_history (couple_id, activity_id, outcome, drawn_by, est_cost, from_savings)
         SELECT ?1, id, 'did_it', ?2, est_cost, ?3 FROM joy_activities WHERE id = ?4 AND couple_id = ?1",
    )
    .bind(couple)
    .bind(drawn_by.map(|w| w as i64))
    .bind(cost)
    .bind(activity)
    .execute(&mut *tx)
    .await?;
    // Not one of this couple's slips: nothing kept.
    if kept.rows_affected() == 0 {
        tx.rollback().await?;
        return Ok(None);
    }
    let row = kept.last_insert_rowid();
    sqlx::query("UPDATE joy_activities SET times_done = times_done + 1, last_done_at = unixepoch() WHERE id = ? AND couple_id = ?")
        .bind(activity)
        .bind(couple)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO joy_savings (couple_id, amount, kind, note) SELECT ?1, ?2, 'kept', title FROM joy_activities WHERE id = ?3 AND couple_id = ?1")
        .bind(couple)
        .bind(-cost)
        .bind(activity)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Some(row))
}

