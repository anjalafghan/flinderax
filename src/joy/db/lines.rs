//! The two lines each of us writes about a date.

use sqlx::SqlitePool;

// ── two lines ─────────────────────────────────────────────────────────────

/// Every line written on a kept date: (history id, who, line).
pub async fn memories(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(i64, i64, String)>> {
    sqlx::query_as(
        "SELECT m.history_id, m.who, m.line FROM joy_memories m JOIN joy_history h ON h.id = m.history_id WHERE h.couple_id = ?",
    )
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Write (or rewrite) one person's line on a kept date; a blank line takes it back.
pub async fn set_memory(db: &SqlitePool, couple: &str, history_id: i64, who: usize, line: &str) -> sqlx::Result<()> {
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.is_empty() {
        sqlx::query(
            "DELETE FROM joy_memories WHERE history_id = ? AND who = ?
               AND history_id IN (SELECT id FROM joy_history WHERE couple_id = ?)",
        )
        .bind(history_id)
        .bind(who as i64)
        .bind(couple)
        .execute(db)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO joy_memories (history_id, who, line) SELECT id, ?, ? FROM joy_history WHERE id = ? AND couple_id = ?
             ON CONFLICT (history_id, who) DO UPDATE SET line = excluded.line, at = unixepoch()",
        )
        .bind(who as i64)
        .bind(line)
        .bind(history_id)
        .bind(couple)
        .execute(db)
        .await?;
    }
    Ok(())
}

