//! Slips either of us passed on.

use sqlx::SqlitePool;

// ── vetoes ────────────────────────────────────────────────────────────────

/// (activity id, who passed on it).
pub async fn vetoes(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(i64, i64)>> {
    sqlx::query_as(
        "SELECT v.activity_id, v.person FROM joy_vetoes v JOIN joy_activities a ON a.id = v.activity_id WHERE a.couple_id = ?",
    )
    .bind(couple)
    .fetch_all(db)
    .await
}

pub async fn veto(db: &SqlitePool, couple: &str, activity: i64, person: usize) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO joy_vetoes (activity_id, person)
         SELECT id, ? FROM joy_activities WHERE id = ? AND couple_id = ?",
    )
        .bind(person as i64)
        .bind(activity)
        .bind(couple)
        .execute(db)
        .await?;
    Ok(())
}

/// Everything anyone passed on goes back into the draw. Returns how many slips came back.
pub async fn clear_vetoes(db: &SqlitePool, couple: &str) -> sqlx::Result<u64> {
    let done = sqlx::query("DELETE FROM joy_vetoes WHERE activity_id IN (SELECT id FROM joy_activities WHERE couple_id = ?)")
        .bind(couple)
        .execute(db)
        .await?;
    Ok(done.rows_affected())
}

/// Every archived slip comes back into the jar, except the one-off slips behind dates you added
/// by hand (those were never meant to be drawn). Returns how many came back.
pub async fn restore_all_archived(db: &SqlitePool, couple: &str) -> sqlx::Result<u64> {
    let done = sqlx::query("UPDATE joy_activities SET archived = 0 WHERE couple_id = ? AND archived = 1 AND custom = 0")
        .bind(couple)
        .execute(db)
        .await?;
    Ok(done.rows_affected())
}

/// The "put back" rows that rerolling leaves in Kept. They also make a slip count as having history,
/// so it can only be archived, not deleted. Returns how many went.
pub async fn clear_putbacks(db: &SqlitePool, couple: &str) -> sqlx::Result<u64> {
    let done = sqlx::query("DELETE FROM joy_history WHERE couple_id = ? AND outcome = 'skipped'")
        .bind(couple)
        .execute(db)
        .await?;
    Ok(done.rows_affected())
}

/// Take back `person`'s own pass on a slip. The other one's pass, if there is one, stays.
pub async fn unveto(db: &SqlitePool, couple: &str, activity: i64, person: usize) -> sqlx::Result<()> {
    sqlx::query(
        "DELETE FROM joy_vetoes WHERE activity_id = ? AND person = ? AND activity_id IN (SELECT id FROM joy_activities WHERE couple_id = ?)",
    )
        .bind(activity)
        .bind(person as i64)
        .bind(couple)
        .execute(db)
        .await?;
    Ok(())
}

