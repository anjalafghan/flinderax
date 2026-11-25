//! Regulars: standing dates ("Blue Tokai, Tue and Thu").

use super::*;
use sqlx::{FromRow, SqlitePool};

// ── regulars: standing dates ("Blue Tokai, Tue and Thu") ──────────────────

#[derive(Debug, Clone, FromRow)]
pub struct Regular {
    pub id: i64,
    pub title: String,
    pub place: String,
    pub maps_url: String,
    /// SQLite %w numbers, comma separated ("2,4" = Tue and Thu).
    pub weekdays: String,
    pub usual_cost: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct Visit {
    pub id: i64,
    pub regular_id: i64,
    /// Local "YYYY-MM-DD".
    pub day: String,
    /// went | skipped
    pub status: String,
    /// Where we actually went; empty means the regular's own place.
    pub place: String,
    pub amount: i64,
    pub has_photo: bool,
    /// The kept date this night is (a night we went to always has one).
    pub history_id: Option<i64>,
}

pub async fn regular(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<Option<Regular>> {
    sqlx::query_as("SELECT * FROM joy_regulars WHERE couple_id = ? AND id = ? AND active = 1").bind(couple).bind(id).fetch_optional(db).await
}

/// The active regulars keyed by id, fetched once so a loop over visits needs no query per row.
async fn regulars_by_id(db: &SqlitePool, couple: &str) -> sqlx::Result<std::collections::HashMap<i64, Regular>> {
    Ok(regulars(db, couple).await?.into_iter().map(|r| (r.id, r)).collect())
}

/// How many nights we went, per regular (those never gone to are absent).
pub async fn went_counts(db: &SqlitePool, couple: &str) -> sqlx::Result<std::collections::HashMap<i64, i64>> {
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT v.regular_id, count(*) FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
          WHERE r.couple_id = ? AND v.status = 'went' GROUP BY v.regular_id",
    )
    .bind(couple)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().collect())
}

pub async fn regulars(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<Regular>> {
    sqlx::query_as("SELECT * FROM joy_regulars WHERE couple_id = ? AND active = 1 ORDER BY id").bind(couple).fetch_all(db).await
}

/// Days from the last week (today first) on which a regular was due and nobody has said
/// whether we went: (regular, "YYYY-MM-DD", "Tue 3 Nov"). Nothing is assumed, so a night we never
/// answer simply stays blank.
pub async fn open_nights(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(Regular, String, String)>> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "WITH RECURSIVE d(day) AS (
           SELECT date('now', 'localtime', '-6 days')
           UNION ALL SELECT date(day, '+1 day') FROM d WHERE day < date('now', 'localtime'))
         SELECT r.id, d.day,
                substr('SunMonTueWedThuFriSat', 1 + 3 * CAST(strftime('%w', d.day) AS INTEGER), 3) || ' ' ||
                CAST(strftime('%d', d.day) AS INTEGER) || ' ' ||
                substr('JanFebMarAprMayJunJulAugSepOctNovDec', 1 + 3 * (CAST(strftime('%m', d.day) AS INTEGER) - 1), 3)
           FROM d JOIN joy_regulars r ON r.active = 1 AND r.couple_id = ?
          WHERE d.day >= r.since
            AND instr(',' || r.weekdays || ',', ',' || strftime('%w', d.day) || ',') > 0
            AND NOT EXISTS (SELECT 1 FROM joy_regular_visits v WHERE v.regular_id = r.id AND v.day = d.day)
          ORDER BY d.day DESC, r.id",
    )
    .bind(couple)
    .fetch_all(db)
    .await?;
    let by_id = regulars_by_id(db, couple).await?;
    Ok(rows.into_iter().filter_map(|(id, day, label)| by_id.get(&id).map(|r| (r.clone(), day, label))).collect())
}

/// Tonight's answered visits (we went), so the card can say so and offer a photo.
pub async fn went_today(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(Regular, Visit)>> {
    let visits: Vec<Visit> = sqlx::query_as(
        "SELECT v.* FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
          WHERE r.couple_id = ? AND v.day = date('now', 'localtime') AND v.status = 'went' AND r.active = 1 ORDER BY v.id",
    )
    .bind(couple)
    .fetch_all(db)
    .await?;
    let by_id = regulars_by_id(db, couple).await?;
    Ok(visits.into_iter().filter_map(|v| by_id.get(&v.regular_id).map(|r| (r.clone(), v))).collect())
}

/// Say whether we went on `day`. `status` is "went" or "skipped"; a blank `place` means the
/// regular's own place. Days in the future (or before 2000) are refused (false); any past day is fine, so a journal can be filled in later.
/// Answering again changes the answer; a photo already on the night is kept.
pub async fn record_visit(db: &SqlitePool, couple: &str, regular_id: i64, day: &str, status: &str, place: &str, amount: i64) -> sqlx::Result<bool> {
    let ok: i64 = sqlx::query_scalar(
        "SELECT date(?1) IS NOT NULL AND date(?1) = ?1
            AND date(?1) <= date('now', 'localtime') AND date(?1) >= '2000-01-01'
            AND EXISTS (SELECT 1 FROM joy_regulars WHERE id = ?2 AND couple_id = ?3 AND active = 1)",
    )
    .bind(day)
    .bind(regular_id)
    .bind(couple)
    .fetch_one(db)
    .await?;
    if ok == 0 || !matches!(status, "went" | "skipped") {
        return Ok(false);
    }
    let (place, amount) = if status == "went" { (place.trim(), amount.max(0)) } else { ("", 0) };
    sqlx::query(
        "INSERT INTO joy_regular_visits (regular_id, day, status, place, amount) VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(regular_id, day) DO UPDATE SET status = excluded.status, place = excluded.place, amount = excluded.amount",
    )
    .bind(regular_id)
    .bind(day)
    .bind(status)
    .bind(place)
    .bind(amount)
    .execute(db)
    .await?;
    sync_visit_history(db, couple, regular_id, day).await?;
    Ok(true)
}

/// Keeps the kept date behind a regular night in step with the visit: going creates it (or updates
/// its cost and place), skipping the night removes it. After this the night is an ordinary kept
/// date that takes photos, a rating and lines.
pub async fn sync_visit_history(db: &SqlitePool, couple: &str, regular_id: i64, day: &str) -> sqlx::Result<()> {
    let Some(v): Option<Visit> = sqlx::query_as("SELECT * FROM joy_regular_visits WHERE regular_id = ? AND day = ?")
        .bind(regular_id)
        .bind(day)
        .fetch_optional(db)
        .await?
    else {
        return Ok(());
    };
    if v.status != "went" {
        if let Some(h) = v.history_id {
            delete_history(db, couple, h).await?;
            sqlx::query("UPDATE joy_regular_visits SET history_id = NULL WHERE id = ?").bind(v.id).execute(db).await?;
        }
        return Ok(());
    }
    let Some(r) = regular(db, couple, regular_id).await? else { return Ok(()) };
    let place = if v.place.is_empty() { r.place.clone() } else { v.place.clone() };
    match v.history_id {
        Some(h) => {
            sqlx::query("UPDATE joy_history SET actual_cost = ?1, est_cost = ?1 WHERE id = ?2 AND couple_id = ?3")
                .bind(v.amount)
                .bind(h)
                .bind(couple)
                .execute(db)
                .await?;
            sqlx::query("UPDATE joy_activities SET venue = ? WHERE id = (SELECT activity_id FROM joy_history WHERE id = ? AND couple_id = ?)")
                .bind(&place)
                .bind(h)
                .bind(couple)
                .execute(db)
                .await?;
        }
        None => {
            let h = keep_custom(db, couple, None, &r.title, &place, "local", day, Some(v.amount)).await?;
            sqlx::query("UPDATE joy_regular_visits SET history_id = ? WHERE id = ?").bind(h).bind(v.id).execute(db).await?;
        }
    }
    Ok(())
}

/// `joy backfill-visits`, safe to repeat: gives every night we went to that has no kept date one,
/// copying its photo across so nothing is lost.
pub async fn backfill_visit_history(db: &SqlitePool, photos_root: &std::path::Path) -> sqlx::Result<()> {
    let rows: Vec<(i64, String, i64, String, bool)> = sqlx::query_as(
        "SELECT v.id, r.couple_id, v.regular_id, v.day, v.has_photo FROM joy_regular_visits v
           JOIN joy_regulars r ON r.id = v.regular_id WHERE v.status = 'went' AND v.history_id IS NULL",
    )
    .fetch_all(db)
    .await?;
    for (vid, couple, regular_id, day, has_photo) in rows {
        sync_visit_history(db, &couple, regular_id, &day).await?;
        if !has_photo {
            continue;
        }
        let h: Option<i64> = sqlx::query_scalar("SELECT history_id FROM joy_regular_visits WHERE id = ?").bind(vid).fetch_one(db).await?;
        let Some(h) = h else { continue };
        let dir = photos_root.join(&couple);
        let file = crate::joy::photos::new_date_file(h);
        if let Some(dest) = crate::joy::photos::date_path(&dir, &file)
            && tokio::fs::copy(crate::joy::photos::visit_path(&dir, vid), &dest).await.is_ok() {
                add_photo(db, &couple, h, &file).await?;
            }
    }
    Ok(())
}

/// Changes what a night we went to cost, and where (blank = the regular's own place). False when
/// there is no such night of ours that we went to.
pub async fn set_visit_amount(db: &SqlitePool, couple: &str, id: i64, place: &str, amount: i64) -> sqlx::Result<bool> {
    let res = sqlx::query(
        "UPDATE joy_regular_visits SET amount = ?, place = ? WHERE id = ? AND status = 'went'
            AND regular_id IN (SELECT id FROM joy_regulars WHERE couple_id = ?)",
    )
    .bind(amount.max(0))
    .bind(place.trim())
    .bind(id)
    .bind(couple)
    .execute(db)
    .await?;
    let _ = sqlx::query(
        "UPDATE joy_history SET actual_cost = ?1, est_cost = ?1 WHERE id IN (SELECT history_id FROM joy_regular_visits WHERE id = ?2) AND couple_id = ?3",
    )
    .bind(amount.max(0))
    .bind(id)
    .bind(couple)
    .execute(db)
    .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn visit(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<Option<Visit>> {
    sqlx::query_as(
        "SELECT v.* FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id WHERE r.couple_id = ? AND v.id = ?",
    )
    .bind(couple)
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn delete_visit(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<()> {
    if let Some(v) = visit(db, couple, id).await?
        && let Some(h) = v.history_id {
            delete_history(db, couple, h).await?;
        }
    sqlx::query(
        "DELETE FROM joy_regular_visits WHERE id = ? AND regular_id IN (SELECT id FROM joy_regulars WHERE couple_id = ?)",
    )
    .bind(id)
    .bind(couple)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn set_visit_photo(db: &SqlitePool, couple: &str, id: i64, has: bool) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE joy_regular_visits SET has_photo = ? WHERE id = ? AND regular_id IN (SELECT id FROM joy_regulars WHERE couple_id = ?)",
    )
    .bind(has)
    .bind(id)
    .bind(couple)
    .execute(db)
    .await?;
    Ok(())
}

/// Nights we went to a regular, newest first.
pub async fn went_visits(db: &SqlitePool, couple: &str, regular_id: i64) -> sqlx::Result<Vec<Visit>> {
    sqlx::query_as(
        "SELECT v.* FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
          WHERE r.couple_id = ? AND v.regular_id = ? AND v.status = 'went' ORDER BY v.day DESC, v.id DESC",
    )
        .bind(couple)
        .bind(regular_id)
        .fetch_all(db)
        .await
}

/// Nights we went in a "YYYY-MM", oldest first, with the regular's title.
pub async fn visits_in_month(db: &SqlitePool, couple: &str, month: &str) -> sqlx::Result<Vec<(Visit, String)>> {
    let rows: Vec<Visit> = sqlx::query_as(
        "SELECT v.* FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
          WHERE r.couple_id = ? AND v.status = 'went' AND v.history_id IS NULL AND strftime('%Y-%m', v.day) = ? ORDER BY v.day, v.id",
    )
    .bind(couple)
    .bind(month)
    .fetch_all(db)
    .await?;
    let by_id = regulars_by_id(db, couple).await?;
    Ok(rows.into_iter().map(|v| { let title = by_id.get(&v.regular_id).map(|r| r.title.clone()).unwrap_or_default(); (v, title) }).collect())
}

/// The most we could still spend this month if every remaining regular night happened and cost
/// its usual amount (today counts until it's answered). A ceiling to plan around, never taken
/// off what's left, because most weeks we won't go to every one.
pub async fn regulars_ceiling(db: &SqlitePool, couple: &str) -> sqlx::Result<i64> {
    sqlx::query_scalar(
        "WITH RECURSIVE d(day) AS (
           SELECT date('now', 'localtime')
           UNION ALL SELECT date(day, '+1 day') FROM d
            WHERE day < date('now', 'localtime', 'start of month', '+1 month', '-1 day'))
         SELECT COALESCE(SUM(r.usual_cost), 0)
           FROM d JOIN joy_regulars r ON r.active = 1 AND r.couple_id = ?
          WHERE d.day >= r.since
            AND instr(',' || r.weekdays || ',', ',' || strftime('%w', d.day) || ',') > 0
            AND NOT EXISTS (SELECT 1 FROM joy_regular_visits v WHERE v.regular_id = r.id AND v.day = d.day)",
    )
    .bind(couple)
    .fetch_one(db)
    .await
}

/// Add a regular. `weekdays` is "2,4"-style (SQLite %w); `since` blank means today.
pub async fn add_regular(db: &SqlitePool, couple: &str, title: &str, place: &str, maps_url: &str, weekdays: &str, usual_cost: i64, since: &str) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO joy_regulars (couple_id, title, place, maps_url, weekdays, usual_cost, since)
         VALUES (?, ?, ?, ?, ?, ?, COALESCE(NULLIF(date(?), ''), date('now', 'localtime'))) RETURNING id",
    )
    .bind(couple)
    .bind(title)
    .bind(place)
    .bind(maps_url)
    .bind(weekdays)
    .bind(usual_cost)
    .bind(since.trim())
    .fetch_one(db)
    .await?;
    Ok(row.0)
}

/// Change a regular's details (the nights already stamped stay as they are). Scoped by couple.
pub async fn update_regular(db: &SqlitePool, couple: &str, id: i64, title: &str, place: &str, maps_url: &str, weekdays: &str, usual_cost: i64) -> sqlx::Result<bool> {
    let done = sqlx::query(
        "UPDATE joy_regulars SET title = ?, place = ?, maps_url = ?, weekdays = ?, usual_cost = ?
         WHERE couple_id = ? AND id = ? AND active = 1",
    )
    .bind(title)
    .bind(place)
    .bind(maps_url)
    .bind(weekdays)
    .bind(usual_cost)
    .bind(couple)
    .bind(id)
    .execute(db)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Stop tracking a regular: archived (`active = 0`), never erased. False when it isn't ours.
/// Nights you skipped at a regular are hidden everywhere; this forgets them, so those days are open
/// again. Returns how many went.
pub async fn clear_skipped_nights(db: &SqlitePool, couple: &str, regular: i64) -> sqlx::Result<u64> {
    let done = sqlx::query(
        "DELETE FROM joy_regular_visits WHERE regular_id = ?1 AND status = 'skipped'
           AND regular_id IN (SELECT id FROM joy_regulars WHERE couple_id = ?2)",
    )
    .bind(regular)
    .bind(couple)
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

/// Start tracking a stopped regular again.
pub async fn restore_regular(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<bool> {
    let done = sqlx::query("UPDATE joy_regulars SET active = 1 WHERE couple_id = ? AND id = ? AND active = 0").bind(couple).bind(id).execute(db).await?;
    Ok(done.rows_affected() > 0)
}

/// The regulars you stopped tracking: (id, title).
pub async fn stopped_regulars(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(i64, String)>> {
    sqlx::query_as("SELECT id, title FROM joy_regulars WHERE couple_id = ? AND active = 0 ORDER BY title").bind(couple).fetch_all(db).await
}

pub async fn archive_regular(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<bool> {
    let done = sqlx::query("UPDATE joy_regulars SET active = 0 WHERE couple_id = ? AND id = ? AND active = 1").bind(couple).bind(id).execute(db).await?;
    Ok(done.rows_affected() > 0)
}
