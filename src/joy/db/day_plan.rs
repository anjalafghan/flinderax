//! The day-of plan: tasks, Plan B and what is next.

#![allow(unused_imports)]
use super::*;
use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

// ── the day-of plan ───────────────────────────────────────────────────────

pub async fn set_day_kind(db: &SqlitePool, couple: &str, ids: &[i64], kind: &str) -> sqlx::Result<()> {
    for id in ids {
        sqlx::query("UPDATE joy_history SET day_kind = ? WHERE id = ? AND couple_id = ?")
            .bind(kind)
            .bind(id)
            .bind(couple)
            .execute(db)
            .await?;
    }
    Ok(())
}

pub async fn set_plan_b(db: &SqlitePool, couple: &str, history_id: i64, plan_b: Option<i64>) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE joy_history SET plan_b = ?1 WHERE id = ?2 AND couple_id = ?3
           AND (?1 IS NULL OR EXISTS (SELECT 1 FROM joy_activities WHERE id = ?1 AND couple_id = ?3))",
    )
    .bind(plan_b)
    .bind(history_id)
    .bind(couple)
    .execute(db)
    .await?;
    Ok(())
}

/// Go with Plan B instead: the kept date becomes the other slip, and the original becomes its
/// Plan B (so swapping back is one more tap). Done counts follow the slip we're really doing.
pub async fn swap_plan_b(db: &SqlitePool, couple: &str, history_id: i64) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let row: Option<(i64, Option<i64>)> = sqlx::query_as(
        "SELECT activity_id, plan_b FROM joy_history
         WHERE id = ? AND couple_id = ? AND outcome = 'did_it'
           AND plan_b IN (SELECT id FROM joy_activities WHERE couple_id = joy_history.couple_id)",
    )
    .bind(history_id)
    .bind(couple)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((from, Some(to))) = row else {
        tx.rollback().await?;
        return Ok(false);
    };
    sqlx::query("UPDATE joy_history SET activity_id = ?, plan_b = ?, est_cost = (SELECT est_cost FROM joy_activities WHERE id = ?) WHERE id = ? AND couple_id = ?")
        .bind(to)
        .bind(from)
        .bind(to)
        .bind(history_id)
        .bind(couple)
        .execute(&mut *tx)
        .await?;
    for a in [from, to] {
        sqlx::query(
            "UPDATE joy_activities SET
               times_done = (SELECT COUNT(*) FROM joy_history WHERE activity_id = ?1 AND outcome = 'did_it'),
               last_done_at = (SELECT MAX(picked_at) FROM joy_history WHERE activity_id = ?1 AND outcome = 'did_it')
             WHERE id = ?1 AND couple_id = ?2",
        )
        .bind(a)
        .bind(couple)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(true)
}

#[derive(Debug, Clone, FromRow)]
pub struct Task {
    pub id: i64,
    pub history_id: i64,
    pub who: i64,
    pub task: String,
    pub due: String,
    pub done_at: Option<i64>,
}

/// How many jobs each of us has been dealt so far, to keep the dealing fair.
pub async fn task_counts(db: &SqlitePool, couple: &str) -> sqlx::Result<[i64; 2]> {
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT t.who, COUNT(*) FROM joy_tasks t JOIN joy_history h ON h.id = t.history_id WHERE h.couple_id = ? GROUP BY t.who",
    )
    .bind(couple)
    .fetch_all(db)
    .await?;
    let mut out = [0; 2];
    for (who, n) in rows {
        if let Some(slot) = usize::try_from(who).ok().and_then(|w| out.get_mut(w)) {
            *slot = n;
        }
    }
    Ok(out)
}

/// Deal a date's jobs (who, task, due). Only the first deal counts: a double tap adds nothing.
pub async fn add_tasks(db: &SqlitePool, couple: &str, history_id: i64, tasks: &[(usize, String, String)]) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    let already: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM joy_tasks WHERE history_id = ?1)
             OR NOT EXISTS (SELECT 1 FROM joy_history WHERE id = ?1 AND couple_id = ?2)",
    )
        .bind(history_id)
        .bind(couple)
        .fetch_one(&mut *tx)
        .await?;
    if !already {
        for (who, task, due) in tasks {
            sqlx::query("INSERT INTO joy_tasks (history_id, who, task, due) VALUES (?, ?, ?, ?)")
                .bind(history_id)
                .bind(*who as i64)
                .bind(task)
                .bind(due)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await
}

pub async fn tasks_for(db: &SqlitePool, couple: &str, history_id: i64) -> sqlx::Result<Vec<Task>> {
    sqlx::query_as(
        "SELECT t.id, t.history_id, t.who, t.task, t.due, t.done_at FROM joy_tasks t JOIN joy_history h ON h.id = t.history_id
         WHERE h.couple_id = ? AND t.history_id = ? ORDER BY t.who, t.id",
    )
        .bind(couple)
        .bind(history_id)
        .fetch_all(db)
        .await
}

pub async fn task(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<Option<Task>> {
    sqlx::query_as(
        "SELECT t.id, t.history_id, t.who, t.task, t.due, t.done_at FROM joy_tasks t JOIN joy_history h ON h.id = t.history_id
         WHERE h.couple_id = ? AND t.id = ?",
    )
    .bind(couple)
    .bind(id)
    .fetch_optional(db)
    .await
}

/// Tick a job off (or untick it).
pub async fn set_task_done(db: &SqlitePool, couple: &str, id: i64, done: bool) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE joy_tasks SET done_at = CASE WHEN ? THEN COALESCE(done_at, unixepoch()) ELSE NULL END
         WHERE id = ? AND history_id IN (SELECT id FROM joy_history WHERE couple_id = ?)",
    )
        .bind(done)
        .bind(id)
        .bind(couple)
        .execute(db)
        .await?;
    Ok(())
}

/// Local now: ("YYYY-MM-DD", minutes since midnight).
pub async fn local_now(db: &SqlitePool) -> sqlx::Result<(String, i64)> {
    sqlx::query_as(
        "SELECT date('now', 'localtime'),
                CAST(strftime('%H', 'now', 'localtime') AS INTEGER) * 60 + CAST(strftime('%M', 'now', 'localtime') AS INTEGER)",
    )
    .fetch_one(db)
    .await
}

/// The next kept date still to come (from 3 hours ago), soonest first.
pub async fn next_planned(db: &SqlitePool, couple: &str) -> sqlx::Result<Option<HistoryRow>> {
    sqlx::query_as(&format!(
        "SELECT {HISTORY_COLS} FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ? AND h.outcome = 'did_it' AND h.planned_at >= unixepoch() - 3 * 3600
         ORDER BY h.planned_at, h.id LIMIT 1"
    ))
    .bind(couple)
    .fetch_optional(db)
    .await
}

/// The envelope token for a slip or a kept date, made the first time it's shared.
async fn share_token(db: &SqlitePool, couple: &str, table: &str, id: i64) -> sqlx::Result<String> {
    use rand::{distributions::Alphanumeric, Rng};
    let fresh: String = rand::thread_rng().sample_iter(&Alphanumeric).take(24).map(char::from).collect();
    // Every share renews the link: 30 days for a slip, a week past the day for a kept date.
    let expiry = if table == "joy_history" {
        "MAX(COALESCE(planned_at, picked_at), unixepoch()) + 7 * 86400"
    } else {
        "unixepoch() + 30 * 86400"
    };
    sqlx::query(&format!("UPDATE {table} SET share_expires_at = {expiry} WHERE id = ? AND couple_id = ? AND share_token IS NOT NULL"))
        .bind(id)
        .bind(couple)
        .execute(db)
        .await?;
    sqlx::query(&format!(
        "UPDATE {table} SET share_token = ?, share_expires_at = {expiry} WHERE id = ? AND couple_id = ? AND share_token IS NULL"
    ))
        .bind(&fresh)
        .bind(id)
        .bind(couple)
        .execute(db)
        .await?;
    sqlx::query_scalar(&format!("SELECT share_token FROM {table} WHERE id = ? AND couple_id = ?"))
        .bind(id)
        .bind(couple)
        .fetch_one(db)
        .await
}

/// Stop sharing: the link stops opening and the next share makes a new one.
pub async fn unshare(db: &SqlitePool, couple: &str, table: &str, id: i64) -> sqlx::Result<()> {
    let table = if table == "joy_history" { "joy_history" } else { "joy_activities" };
    sqlx::query(&format!("UPDATE {table} SET share_token = NULL, share_expires_at = NULL WHERE id = ? AND couple_id = ?"))
        .bind(id)
        .bind(couple)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn slip_token(db: &SqlitePool, couple: &str, activity_id: i64) -> sqlx::Result<String> {
    share_token(db, couple, "joy_activities", activity_id).await
}

pub async fn date_token(db: &SqlitePool, couple: &str, history_id: i64) -> sqlx::Result<String> {
    share_token(db, couple, "joy_history", history_id).await
}

/// What an envelope holds: a kept date, or a slip one of us is suggesting.
pub enum Envelope {
    Date(HistoryRow),
    Slip(Activity),
}

/// Opened by the unguessable 24-character token alone (the recipient has no login), so this is the
/// one lookup that isn't given a couple: the token's own row says whose it is.
pub async fn envelope(db: &SqlitePool, token: &str) -> sqlx::Result<Option<Envelope>> {
    if token.len() != 24 || !token.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Ok(None);
    }
    if let Some((couple, id)) = sqlx::query_as::<_, (String, i64)>("SELECT couple_id, id FROM joy_history WHERE share_token = ? AND share_expires_at > unixepoch()")
        .bind(token)
        .fetch_optional(db)
        .await?
    {
        return Ok(history_row(db, &couple, id).await?.map(Envelope::Date));
    }
    match sqlx::query_as::<_, (String, i64)>("SELECT couple_id, id FROM joy_activities WHERE share_token = ? AND share_expires_at > unixepoch() AND archived = 0")
        .bind(token)
        .fetch_optional(db)
        .await?
    {
        Some((couple, id)) => Ok(get(db, &couple, id).await?.map(Envelope::Slip)),
        None => Ok(None),
    }
}

