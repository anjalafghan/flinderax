use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

use crate::joy::picker::{Candidate, Duration};

#[derive(Debug, Clone, Default, FromRow)]
pub struct Activity {
    pub id: i64,
    pub title: String,
    pub notes: String,
    pub est_cost: i64,
    pub setting: String,
    pub area: String,
    pub venue: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub duration: String,
    pub tags: String,
    pub energy: String,
    pub added_by: String,
    pub times_done: i64,
    pub last_done_at: Option<i64>,
    pub last_skipped_at: Option<i64>,
    /// Suggested by the app rather than written by us ("generated for you").
    pub generated: bool,
    /// "mon,wed": days it's closed.
    pub closed_days: String,
    pub season_from: Option<i64>,
    pub season_to: Option<i64>,
    /// The estimate came from a real cost, so the learned factor leaves it alone.
    pub est_from_actual: bool,
    /// Opening hours, "tue-sun 12:00-23:30; mon 18:00-23:00". Empty = unknown.
    pub hours: String,
    /// morning | afternoon | evening | night | "" (any time).
    pub best_time: String,
    /// How long the date takes, not counting travel. None = from `duration`.
    pub minutes: Option<i64>,
    /// none | weekends | always | "" (unknown).
    pub booking: String,
    pub book_ahead_days: Option<i64>,
    /// indoor | outdoor | mixed | "" (unknown).
    pub cover: String,
    /// "closed shoes, cash".
    pub bring: String,
    /// Nearest local or metro station, and the walk from it.
    pub station: String,
    pub station_walk_min: Option<i64>,
}

impl Activity {
    pub fn latlng(&self) -> Option<(f64, f64)> {
        Some((self.lat?, self.lng?))
    }

    pub fn tag_list(&self) -> Vec<String> {
        self.tags.split(',').map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty()).collect()
    }

    pub fn duration(&self) -> Duration {
        Duration::parse(&self.duration).unwrap_or(Duration::Short)
    }

    pub fn season(&self) -> Option<(u8, u8)> {
        let ok = |m: i64| u8::try_from(m).ok().filter(|m| (1..=12).contains(m));
        Some((ok(self.season_from?)?, ok(self.season_to?)?))
    }

    /// The estimate as the jar should use it: adjusted by what our dates really cost,
    /// unless it was already set from a real cost.
    pub fn effective_cost(&self, factor: Option<f64>) -> i64 {
        match factor {
            Some(f) if !self.est_from_actual && self.est_cost > 0 => ((self.est_cost as f64 * f) / 50.0).round() as i64 * 50,
            _ => self.est_cost,
        }
    }

    /// Minutes the date itself takes: what the slip says, else a guess from its duration.
    pub fn minutes(&self) -> i64 {
        self.minutes.filter(|m| *m > 0).unwrap_or_else(|| crate::joy::ics::minutes_for(&self.duration))
    }

    /// Would rain spoil it? Unknown slips fall back to the "outdoors" tag.
    pub fn outdoor(&self) -> bool {
        match self.cover.as_str() {
            "outdoor" => true,
            "indoor" | "mixed" => false,
            _ => self.tag_list().iter().any(|t| t == "outdoors"),
        }
    }

    /// The things to remember to take, one per item.
    pub fn bring_list(&self) -> Vec<String> {
        self.bring.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
    }

    pub fn candidate(&self, distance_km: Option<f64>, factor: Option<f64>) -> Candidate {
        Candidate {
            id: self.id,
            title: self.title.clone(),
            est_cost: self.effective_cost(factor),
            setting: self.setting.clone(),
            duration: self.duration(),
            tags: self.tag_list(),
            energy: self.energy.clone(),
            last_done_at: self.last_done_at,
            last_skipped_at: self.last_skipped_at,
            distance_km,
            closed_days: crate::joy::picker::parse_closed(&self.closed_days),
            season: self.season(),
            window_miss: None,
        }
    }
}

/// Fields a person edits on the add/edit form.
#[derive(Debug, Default, Deserialize)]
pub struct ActivityInput {
    pub title: String,
    #[serde(default)]
    pub notes: String,
    #[serde(deserialize_with = "blank_as_zero")]
    pub est_cost: i64,
    pub setting: String,
    pub duration: String,
    #[serde(default)]
    pub venue: String,
    #[serde(default)]
    pub area: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub tags: String,
    pub energy: String,
    #[serde(default)]
    pub added_by: String,
    /// Ids of slips this one pairs well with.
    #[serde(default)]
    pub pairs: Vec<i64>,
    /// Checked weekday keys it's closed on ("mon", …).
    #[serde(default)]
    pub closed: Vec<String>,
    /// Season months "1"–"12", or empty for all year.
    #[serde(default)]
    pub season_from: String,
    #[serde(default)]
    pub season_to: String,
    #[serde(default)]
    pub hours: String,
    #[serde(default)]
    pub best_time: String,
    /// Blank = guess from the duration.
    #[serde(default)]
    pub minutes: String,
    #[serde(default)]
    pub booking: String,
    #[serde(default)]
    pub book_ahead_days: String,
    #[serde(default)]
    pub cover: String,
    #[serde(default)]
    pub bring: String,
    #[serde(default)]
    pub station: String,
    #[serde(default)]
    pub station_walk_min: String,
}

pub const BEST_TIMES: [(&str, &str); 4] = [("morning", "Morning"), ("afternoon", "Afternoon"), ("evening", "Evening"), ("night", "Late night")];
pub const BOOKINGS: [(&str, &str); 3] = [("none", "Walk in"), ("weekends", "Book for weekends"), ("always", "Always book")];
pub const COVERS: [(&str, &str); 3] = [("indoor", "Indoors"), ("outdoor", "Outdoors"), ("mixed", "Bit of both")];

/// Keep a form value only when it's one of `allowed`, so the database never holds anything else.
fn one_of(v: &str, allowed: &[(&str, &str)]) -> String {
    let v = v.trim();
    if allowed.iter().any(|(k, _)| *k == v) { v.to_string() } else { String::new() }
}

/// At most `max` characters, cut on a character boundary.
fn capped(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// A small positive number from a form field, or None.
fn small_number(v: &str, max: i64) -> Option<i64> {
    v.trim().parse::<i64>().ok().filter(|n| (1..=max).contains(n))
}

/// An empty cost field means free, not a bad request.
fn blank_as_zero<'de, D: serde::Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    let s = String::deserialize(d)?;
    let s = s.trim();
    if s.is_empty() {
        return Ok(0);
    }
    s.parse().map_err(serde::de::Error::custom)
}

const COLS: &str = "id, title, notes, est_cost, setting, area, venue, lat, lng, duration, tags, energy, \
                    added_by, times_done, last_done_at, last_skipped_at, generated, \
                    closed_days, season_from, season_to, est_from_actual, \
                    hours, best_time, minutes, booking, book_ahead_days, cover, bring, station, station_walk_min";

pub async fn list(db: &SqlitePool, couple: &str, archived: bool) -> sqlx::Result<Vec<Activity>> {
    sqlx::query_as(&format!(
        "SELECT {COLS} FROM joy_activities WHERE couple_id = ? AND archived = ? ORDER BY est_cost, title"
    ))
    .bind(couple)
    .bind(archived)
    .fetch_all(db)
    .await
}

pub async fn get(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<Option<Activity>> {
    sqlx::query_as(&format!("SELECT {COLS} FROM joy_activities WHERE couple_id = ? AND id = ?"))
        .bind(couple)
        .bind(id)
        .fetch_optional(db)
        .await
}

/// Insert or update a slip and its pairs; returns its id.
pub async fn save(db: &SqlitePool, couple: &str, id: Option<i64>, a: &ActivityInput, latlng: Option<(f64, f64)>) -> sqlx::Result<i64> {
    let tags = a
        .tags
        .split(',')
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    let sql = match id {
        None => "INSERT INTO joy_activities (title, notes, est_cost, setting, duration, venue, area, lat, lng, tags, energy, added_by,
                 closed_days, season_from, season_to, hours, best_time, minutes, booking, book_ahead_days, cover, bring,
                 station, station_walk_min, id, couple_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?17, ?18, ?19, ?20, ?21, ?22, ?23,
                 ?24, ?25, ?16, ?26)",
        // A hand-edited estimate is ours again, so the learned factor applies to it.
        Some(_) => "UPDATE joy_activities SET title = ?1, notes = ?2,
                    est_from_actual = CASE WHEN est_cost = ?3 THEN est_from_actual ELSE 0 END, est_cost = ?3,
                    setting = ?4, duration = ?5, venue = ?6, area = ?7, lat = ?8, lng = ?9, tags = ?10, energy = ?11,
                    added_by = ?12, closed_days = ?13, season_from = ?14, season_to = ?15, hours = ?17, best_time = ?18,
                    minutes = ?19, booking = ?20, book_ahead_days = ?21, cover = ?22, bring = ?23, station = ?24,
                    station_walk_min = ?25 WHERE id = ?16 AND couple_id = ?26",
    };
    let closed = crate::joy::picker::WEEKDAYS
        .iter()
        .map(|(k, _)| *k)
        .filter(|k| a.closed.iter().any(|c| c == k))
        .collect::<Vec<_>>()
        .join(",");
    let month = |s: &str| s.trim().parse::<i64>().ok().filter(|m| (1..=12).contains(m));
    let season = month(&a.season_from).zip(month(&a.season_to));
    let done = sqlx::query(sql)
        .bind(a.title.split_whitespace().collect::<Vec<_>>().join(" "))
        .bind(a.notes.trim())
        .bind(a.est_cost.max(0))
        .bind(&a.setting)
        .bind(&a.duration)
        .bind(a.venue.trim())
        .bind(a.area.trim())
        .bind(latlng.map(|l| l.0))
        .bind(latlng.map(|l| l.1))
        .bind(tags)
        .bind(&a.energy)
        .bind(a.added_by.trim())
        .bind(closed)
        .bind(season.map(|s| s.0))
        .bind(season.map(|s| s.1))
        .bind(id)
        .bind(crate::joy::hours::Hours::parse(&a.hours).map(|h| h.to_string()).unwrap_or_default())
        .bind(one_of(&a.best_time, &BEST_TIMES))
        .bind(small_number(&a.minutes, 7 * 24 * 60))
        .bind(one_of(&a.booking, &BOOKINGS))
        .bind(small_number(&a.book_ahead_days, 90))
        .bind(one_of(&a.cover, &COVERS))
        .bind(capped(&a.bring.split(',').map(str::trim).filter(|t| !t.is_empty()).collect::<Vec<_>>().join(", "), 200))
        .bind(capped(&a.station.split_whitespace().collect::<Vec<_>>().join(" "), 60))
        .bind(small_number(&a.station_walk_min, 120))
        .bind(couple)
        .execute(db)
        .await?;
    // Editing a slip that isn't this couple's matches nothing.
    if id.is_some() && done.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    // From this statement's own result: a separate query could run on another pool connection.
    let id = id.unwrap_or(done.last_insert_rowid());
    set_pairs(db, couple, id, &a.pairs).await?;
    Ok(id)
}

/// All pairs as (smaller id, larger id).
pub async fn pairs(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(i64, i64)>> {
    sqlx::query_as(
        "SELECT p.a_id, p.b_id FROM joy_pairs p
           JOIN joy_activities a ON a.id = p.a_id AND a.couple_id = ?1
           JOIN joy_activities b ON b.id = p.b_id AND b.couple_id = ?1",
    )
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Slips still in the jar that pair with `id`.
pub async fn partners(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<Vec<Activity>> {
    sqlx::query_as(&format!(
        "SELECT {COLS} FROM joy_activities WHERE couple_id = ?2 AND archived = 0 AND id IN
           (SELECT b_id FROM joy_pairs WHERE a_id = ?1 UNION SELECT a_id FROM joy_pairs WHERE b_id = ?1)
         ORDER BY est_cost, title"
    ))
    .bind(id)
    .bind(couple)
    .fetch_all(db)
    .await
}

/// `partners`, minus slips sealed from `who`.
pub async fn visible_partners(db: &SqlitePool, couple: &str, who: usize, id: i64) -> sqlx::Result<Vec<Activity>> {
    let sealed = sealed_from(db, couple, who).await?;
    let mut p = partners(db, couple, id).await?;
    p.retain(|a| !sealed.contains(&a.id));
    Ok(p)
}

/// Replace everything `id` pairs with.
/// Both ends must be this couple's slips; anything else is skipped.
pub async fn set_pairs(db: &SqlitePool, couple: &str, id: i64, with: &[i64]) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    let mine: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM joy_activities WHERE id = ? AND couple_id = ?)")
        .bind(id)
        .bind(couple)
        .fetch_one(&mut *tx)
        .await?;
    if !mine {
        return tx.rollback().await;
    }
    sqlx::query("DELETE FROM joy_pairs WHERE a_id = ?1 OR b_id = ?1").bind(id).execute(&mut *tx).await?;
    for &other in with.iter().filter(|&&o| o != id) {
        sqlx::query(
            "INSERT OR IGNORE INTO joy_pairs (a_id, b_id)
             SELECT ?1, ?2 WHERE EXISTS (SELECT 1 FROM joy_activities WHERE id = ?4 AND couple_id = ?3)",
        )
        .bind(id.min(other))
        .bind(id.max(other))
        .bind(couple)
        .bind(other)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

pub async fn set_archived(db: &SqlitePool, couple: &str, id: i64, archived: bool) -> sqlx::Result<()> {
    sqlx::query("UPDATE joy_activities SET archived = ? WHERE id = ? AND couple_id = ?")
        .bind(archived)
        .bind(id)
        .bind(couple)
        .execute(db)
        .await?;
    Ok(())
}

/// Remove a slip. A slip that has any history (a date, a kept or skipped pick, or a plan-B link)
/// is archived instead, so its dates, memories, tasks and jar spend survive. Returns `true` when
/// the row was really deleted and `false` when it was archived.
pub async fn delete(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let has_history: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM joy_history WHERE activity_id = ?1 OR plan_b = ?1)",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    let deleted = if has_history {
        sqlx::query("UPDATE joy_activities SET archived = 1 WHERE id = ? AND couple_id = ?")
            .bind(id)
            .bind(couple)
            .execute(&mut *tx)
            .await?;
        false
    } else {
        sqlx::query("DELETE FROM joy_activities WHERE id = ? AND couple_id = ?")
            .bind(id)
            .bind(couple)
            .execute(&mut *tx)
            .await?;
        true
    };
    tx.commit().await?;
    Ok(deleted)
}

/// Slips that carry a surprise still sealed for `who`: the partner (anyone but the planner, who is
/// the `drawn_by` of the surprise) sees neither the slip nor its date until it is revealed.
pub async fn sealed_from(db: &SqlitePool, couple: &str, who: usize) -> sqlx::Result<Vec<i64>> {
    sqlx::query_scalar(
        "SELECT DISTINCT activity_id FROM joy_history
         WHERE couple_id = ? AND surprise = 1 AND revealed = 0 AND COALESCE(drawn_by, -1) <> ?",
    )
    .bind(couple)
    .bind(who as i64)
    .fetch_all(db)
    .await
}

/// Slips with an unrevealed surprise on them, whoever planned it: for lists shown to both of us
/// (a Plan B suggestion), where neither of us may be handed a title that is still a secret.
pub async fn sealed_any(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<i64>> {
    sqlx::query_scalar("SELECT DISTINCT activity_id FROM joy_history WHERE couple_id = ? AND surprise = 1 AND revealed = 0")
        .bind(couple)
        .fetch_all(db)
        .await
}

/// The couple's active slips as `who` may see them: `list` minus anything sealed from them. `None`
/// (no viewer) fails closed and drops every unrevealed surprise.
pub async fn visible_list(db: &SqlitePool, couple: &str, who: Option<usize>) -> sqlx::Result<Vec<Activity>> {
    let sealed = match who {
        Some(w) => sealed_from(db, couple, w).await?,
        None => sealed_any(db, couple).await?,
    };
    let mut acts = list(db, couple, false).await?;
    acts.retain(|a| !sealed.contains(&a.id));
    Ok(acts)
}

/// `get`, but a slip sealed from `who` is not found, the same as one that doesn't exist.
pub async fn visible_get(db: &SqlitePool, couple: &str, who: usize, id: i64) -> sqlx::Result<Option<Activity>> {
    if sealed_from(db, couple, who).await?.contains(&id) {
        return Ok(None);
    }
    get(db, couple, id).await
}

/// A joiner takes over an empty slot. Whatever the person who left was still planning in secret
/// belongs to nobody now: `drawn_by` is cleared, which `sealed_from`/`sealed_for` read as sealed
/// from everyone (the new partner can't be its planner, and can't reveal or read it).
pub async fn orphan_surprises(tx: &mut sqlx::SqliteConnection, couple: &str, slot: i64) -> sqlx::Result<()> {
    sqlx::query("UPDATE joy_history SET drawn_by = NULL WHERE couple_id = ? AND surprise = 1 AND revealed = 0 AND drawn_by = ?")
        .bind(couple)
        .bind(slot)
        .execute(&mut *tx)
        .await?;
    Ok(())
}

pub async fn record(db: &SqlitePool, couple: &str, id: i64, did_it: bool) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    let inserted = sqlx::query(
        "INSERT INTO joy_history (couple_id, activity_id, outcome)
         SELECT ?1, id, ?3 FROM joy_activities WHERE id = ?2 AND couple_id = ?1",
    )
    .bind(couple)
    .bind(id)
    .bind(if did_it { "did_it" } else { "skipped" })
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if inserted == 0 {
        return tx.rollback().await;
    }
    let sql = if did_it {
        "UPDATE joy_activities SET times_done = times_done + 1, last_done_at = unixepoch() WHERE id = ? AND couple_id = ?"
    } else {
        "UPDATE joy_activities SET last_skipped_at = unixepoch() WHERE id = ? AND couple_id = ?"
    };
    sqlx::query(sql).bind(id).bind(couple).execute(&mut *tx).await?;
    tx.commit().await
}

/// Accept a pick and return its history row. A double-tap within a minute only counts once
/// (the second tap gets the same row back).
pub async fn accept(db: &SqlitePool, couple: &str, id: i64, drawn_by: Option<usize>, surprise: bool) -> sqlx::Result<i64> {
    let mut tx = db.begin().await?;
    // One statement, so two concurrent taps can't both pass the "recent?" check.
    let inserted = sqlx::query(
        "INSERT INTO joy_history (couple_id, activity_id, outcome, drawn_by, surprise, est_cost)
         SELECT ?4, ?1, 'did_it', ?2, ?3, est_cost FROM joy_activities WHERE id = ?1 AND couple_id = ?4
         AND NOT EXISTS (SELECT 1 FROM joy_history WHERE activity_id = ?1 AND outcome = 'did_it'
                         AND picked_at > unixepoch() - 60)",
    )
    .bind(id)
    .bind(drawn_by.map(|w| w as i64))
    .bind(surprise)
    .bind(couple)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if inserted == 1 {
        sqlx::query("UPDATE joy_activities SET times_done = times_done + 1, last_done_at = unixepoch() WHERE id = ? AND couple_id = ?")
            .bind(id)
            .bind(couple)
            .execute(&mut *tx)
            .await?;
    }
    let row: i64 = sqlx::query_scalar(
        "SELECT id FROM joy_history WHERE couple_id = ? AND activity_id = ? AND outcome = 'did_it' ORDER BY id DESC LIMIT 1",
    )
    .bind(couple)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

/// A date we had that never was a slip: kept straight into Kept on `on` ("YYYY-MM-DD", local
/// noon), with its own one-off slip that is archived, so it never turns up in the jar. Returns
/// the kept date's id.
pub async fn keep_custom(
    db: &SqlitePool,
    couple: &str,
    drawn_by: Option<usize>,
    title: &str,
    place: &str,
    setting: &str,
    on: &str,
    cost: Option<i64>,
) -> sqlx::Result<i64> {
    let mut tx = db.begin().await?;
    let activity = sqlx::query(
        "INSERT INTO joy_activities (couple_id, title, est_cost, setting, venue, duration, added_by, archived, custom, times_done, last_done_at)
         VALUES (?, ?, ?, ?, ?, 'short', 'us', 1, 1, 1, CAST(strftime('%s', ? || ' 12:00:00', 'utc') AS INTEGER))",
    )
    .bind(couple)
    .bind(title)
    .bind(cost.unwrap_or(0))
    .bind(setting)
    .bind(place)
    .bind(on)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    let id = sqlx::query(
        "INSERT INTO joy_history (couple_id, activity_id, picked_at, outcome, drawn_by, est_cost, actual_cost)
         VALUES (?, ?, CAST(strftime('%s', ? || ' 12:00:00', 'utc') AS INTEGER), 'did_it', ?, ?, ?)",
    )
    .bind(couple)
    .bind(activity)
    .bind(on)
    .bind(drawn_by.map(|w| w as i64))
    .bind(cost.unwrap_or(0))
    .bind(cost)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    tx.commit().await?;
    Ok(id)
}

/// Today's local date for the "it's a date" stamp, e.g. "14 · 10 · 2026".
pub async fn today(db: &SqlitePool) -> sqlx::Result<String> {
    sqlx::query_scalar("SELECT strftime('%d · %m · %Y', 'now', 'localtime')").fetch_one(db).await
}

#[derive(Debug, Clone, FromRow)]
pub struct HistoryRow {
    pub id: i64,
    pub activity_id: i64,
    /// Unix seconds when it was drawn/kept.
    pub picked_at: i64,
    pub title: String,
    pub notes: String,
    pub setting: String,
    pub place: String,
    pub duration: String,
    pub energy: String,
    pub tags: String,
    /// The slip's estimate when it was kept (total for both of us).
    pub est_cost: i64,
    /// What it really cost, when someone filled it in.
    pub actual_cost: Option<i64>,
    /// Real one-way cab fares per person, when filled in.
    pub cab_fare_0: Option<i64>,
    pub cab_fare_1: Option<i64>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    /// "YYYY-MM-DD"
    pub day: String,
    pub outcome: String,
    pub rating: Option<i64>,
    pub note: String,
    pub drawn_by: Option<i64>,
    pub planned_at: Option<i64>,
    /// Local time of `planned_at` as "YYYY-MM-DD HH:MM W" (W = weekday, 0 = Sunday).
    pub planned_local: Option<String>,
    pub surprise: bool,
    pub revealed: bool,
    pub has_photo: bool,
    /// The slip to swap to if plans fall through.
    pub plan_b: Option<i64>,
    /// "weeknight", "weekend" or "apart" when we kept it for a particular kind of day; empty
    /// means whatever the schedule says for that date.
    pub day_kind: String,
}

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
    "December",
];

impl HistoryRow {
    /// "14 Oct"
    pub fn day_label(&self) -> String {
        let mut parts = self.day.split('-').skip(1);
        let (m, d) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
        let month = MONTHS.get(m.parse::<usize>().unwrap_or(1).saturating_sub(1)).copied().unwrap_or("");
        format!("{} {}", d.trim_start_matches('0'), &month[..month.len().min(3)])
    }

    /// "October 2026"
    pub fn month_label(&self) -> String {
        let mut parts = self.day.split('-');
        let (y, m) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
        let month = MONTHS.get(m.parse::<usize>().unwrap_or(1).saturating_sub(1)).copied().unwrap_or("");
        format!("{month} {y}")
    }
}

const HISTORY_COLS: &str = "h.id, h.activity_id, h.picked_at, a.title, a.notes, a.setting,
    CASE WHEN a.venue != '' THEN a.venue ELSE a.area END AS place,
    a.duration, a.energy, a.tags, COALESCE(h.est_cost, a.est_cost) AS est_cost, h.actual_cost, h.cab_fare_0, h.cab_fare_1, a.lat, a.lng,
    date(h.picked_at, 'unixepoch', 'localtime') AS day, h.outcome, h.rating, h.note,
    h.drawn_by, h.planned_at,
    strftime('%Y-%m-%d %H:%M %w', h.planned_at, 'unixepoch', 'localtime') AS planned_local,
    h.surprise, h.revealed, h.has_photo, h.plan_b, h.day_kind";

pub async fn history(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<HistoryRow>> {
    sqlx::query_as(&format!(
        "SELECT {HISTORY_COLS} FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ?
         ORDER BY h.picked_at DESC, h.id DESC LIMIT 300"
    ))
    .bind(couple)
    .fetch_all(db)
    .await
}

pub async fn history_row(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<Option<HistoryRow>> {
    sqlx::query_as(&format!(
        "SELECT {HISTORY_COLS} FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ? AND h.id = ?"
    ))
        .bind(couple)
        .bind(id)
        .fetch_optional(db)
        .await
}

/// Kept dates from about this week, one year ago.
pub async fn year_ago(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<HistoryRow>> {
    sqlx::query_as(&format!(
        "SELECT {HISTORY_COLS} FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ? AND h.outcome = 'did_it'
           AND date(h.picked_at, 'unixepoch', 'localtime')
               BETWEEN date('now', 'localtime', '-1 year', '-3 days') AND date('now', 'localtime', '-1 year', '+3 days')
         ORDER BY h.picked_at"
    ))
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Set when these dates happen, from a local "YYYY-MM-DD HH:MM" (or clear it with None).
pub async fn set_planned(db: &SqlitePool, couple: &str, ids: &[i64], local: Option<&str>) -> sqlx::Result<()> {
    for id in ids {
        sqlx::query("UPDATE joy_history SET planned_at = unixepoch(?, 'utc') WHERE id = ? AND couple_id = ? AND outcome = 'did_it'")
            .bind(local)
            .bind(id)
            .bind(couple)
            .execute(db)
            .await?;
    }
    Ok(())
}

/// Remove a kept date. The slip stays in the jar; its "done" count and last-done
/// time are recalculated from what's left, so the picker's weighting stays right.
/// Its two lines go with it, and anything the savings jar paid goes back in.
pub async fn delete_history(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    let activity: Option<i64> = sqlx::query_scalar("SELECT activity_id FROM joy_history WHERE id = ? AND couple_id = ?")
        .bind(id)
        .bind(couple)
        .fetch_optional(&mut *tx)
        .await?;
    // Not this couple's date: nothing to remove.
    if activity.is_none() {
        return tx.rollback().await;
    }
    // Money the savings jar paid for this date goes back in the jar.
    sqlx::query(
        "INSERT INTO joy_savings (couple_id, amount, kind, note)
         SELECT h.couple_id, h.from_savings, 'back', a.title FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.id = ? AND h.couple_id = ? AND h.from_savings > 0",
    )
    .bind(id)
    .bind(couple)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM joy_memories WHERE history_id = ?").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM joy_tasks WHERE history_id = ?").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM joy_history_photos WHERE history_id = ?").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM joy_history WHERE id = ? AND couple_id = ?").bind(id).bind(couple).execute(&mut *tx).await?;
    if let Some(a) = activity {
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
        // A date that never was a slip leaves a one-off archived slip behind; with its only date
        // gone, that slip has no reason to exist (it would sit in the archive, and Restore would
        // put it in the jar).
        sqlx::query(
            "DELETE FROM joy_activities WHERE id = ?1 AND couple_id = ?2 AND custom = 1 AND archived = 1
               AND NOT EXISTS (SELECT 1 FROM joy_history WHERE activity_id = ?1)",
        )
        .bind(a)
        .bind(couple)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

/// Record what a kept date really cost (None clears it). Optionally teach the slip:
/// set its estimate to the real cost, so future draws and budget filters are closer.
pub async fn set_actual(db: &SqlitePool, couple: &str, id: i64, actual: Option<i64>, update_estimate: bool) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query("UPDATE joy_history SET actual_cost = ? WHERE id = ? AND couple_id = ? AND outcome = 'did_it'")
        .bind(actual)
        .bind(id)
        .bind(couple)
        .execute(&mut *tx)
        .await?;
    if let (Some(cost), true) = (actual, update_estimate) {
        sqlx::query(
            "UPDATE joy_activities SET est_cost = ?, est_from_actual = 1
             WHERE couple_id = ? AND id = (SELECT activity_id FROM joy_history WHERE id = ? AND couple_id = ?)",
        )
            .bind(cost)
            .bind(couple)
            .bind(id)
            .bind(couple)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}

pub async fn reveal(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<()> {
    sqlx::query("UPDATE joy_history SET revealed = 1 WHERE id = ? AND couple_id = ?").bind(id).bind(couple).execute(db).await?;
    Ok(())
}

/// Most photos one kept date can hold.
pub const MAX_PHOTOS_PER_DATE: i64 = 24;

/// One stored photo of a kept date: its id and its file name in the couple's photos folder.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct DatePhoto {
    pub id: i64,
    pub history_id: i64,
    pub file: String,
}

/// Every photo of this couple's kept dates, oldest first within each date.
pub async fn all_photos(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<DatePhoto>> {
    sqlx::query_as(
        "SELECT p.id, p.history_id, p.file FROM joy_history_photos p JOIN joy_history h ON h.id = p.history_id
         WHERE h.couple_id = ? ORDER BY p.history_id, p.id",
    )
    .bind(couple)
    .fetch_all(db)
    .await
}

/// The photos of one kept date, oldest first (the first is its cover).
pub async fn photos_of(db: &SqlitePool, couple: &str, history_id: i64) -> sqlx::Result<Vec<DatePhoto>> {
    sqlx::query_as(
        "SELECT p.id, p.history_id, p.file FROM joy_history_photos p JOIN joy_history h ON h.id = p.history_id
         WHERE h.couple_id = ? AND p.history_id = ? ORDER BY p.id",
    )
    .bind(couple)
    .bind(history_id)
    .fetch_all(db)
    .await
}

/// One photo, with the couple whose date it is (the signed public link has no login to ask).
pub async fn photo_by_id(db: &SqlitePool, history_id: i64, photo_id: i64) -> sqlx::Result<Option<(String, DatePhoto)>> {
    let row: Option<(String, i64, i64, String)> = sqlx::query_as(
        "SELECT h.couple_id, p.id, p.history_id, p.file FROM joy_history_photos p JOIN joy_history h ON h.id = p.history_id
         WHERE p.id = ? AND p.history_id = ?",
    )
    .bind(photo_id)
    .bind(history_id)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(c, id, history_id, file)| (c, DatePhoto { id, history_id, file })))
}

/// Records a stored file as the date's newest photo. `None` when the date isn't this couple's or
/// it already holds [`MAX_PHOTOS_PER_DATE`].
pub async fn add_photo(db: &SqlitePool, couple: &str, history_id: i64, file: &str) -> sqlx::Result<Option<i64>> {
    let mut tx = db.begin().await?;
    let n: Option<i64> = sqlx::query_scalar(
        "SELECT (SELECT COUNT(*) FROM joy_history_photos WHERE history_id = h.id) FROM joy_history h WHERE h.id = ? AND h.couple_id = ?",
    )
    .bind(history_id)
    .bind(couple)
    .fetch_optional(&mut *tx)
    .await?;
    if n.is_none_or(|n| n >= MAX_PHOTOS_PER_DATE) {
        tx.rollback().await?;
        return Ok(None);
    }
    let id = sqlx::query("INSERT INTO joy_history_photos (history_id, file) VALUES (?, ?)")
        .bind(history_id)
        .bind(file)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
    sqlx::query("UPDATE joy_history SET has_photo = 1 WHERE id = ?").bind(history_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Some(id))
}

/// Forgets one photo; returns its file so the caller can remove it. Keeps `has_photo` true while
/// any photo is left.
pub async fn remove_photo(db: &SqlitePool, couple: &str, history_id: i64, photo_id: i64) -> sqlx::Result<Option<String>> {
    let mut tx = db.begin().await?;
    let file: Option<String> = sqlx::query_scalar(
        "SELECT p.file FROM joy_history_photos p JOIN joy_history h ON h.id = p.history_id
         WHERE p.id = ? AND p.history_id = ? AND h.couple_id = ?",
    )
    .bind(photo_id)
    .bind(history_id)
    .bind(couple)
    .fetch_optional(&mut *tx)
    .await?;
    if file.is_some() {
        sqlx::query("DELETE FROM joy_history_photos WHERE id = ?").bind(photo_id).execute(&mut *tx).await?;
        sqlx::query("UPDATE joy_history SET has_photo = EXISTS (SELECT 1 FROM joy_history_photos WHERE history_id = ?1) WHERE id = ?1")
            .bind(history_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(file)
}

/// Who kept the most recent date (0 or 1), if we know.
pub async fn last_drawer(db: &SqlitePool, couple: &str) -> sqlx::Result<Option<i64>> {
    sqlx::query_scalar(
        "SELECT drawn_by FROM joy_history WHERE couple_id = ? AND outcome = 'did_it' AND drawn_by IS NOT NULL ORDER BY id DESC LIMIT 1",
    )
    .bind(couple)
    .fetch_optional(db)
    .await
}

/// What we really spent on dates this calendar month (local time). Only dates with an
/// actual cost filled in count: a kept slip that hasn't happened yet isn't money spent.
/// A date counts in the month it happens (its planned time), else the month it was drawn.
/// Spending logged outside a slip counts in the month of its date.
pub async fn spent_this_month(db: &SqlitePool, couple: &str) -> sqlx::Result<i64> {
    spent_in_month(db, couple, &month_offset(db, 0).await?).await
}

/// "YYYY-MM" of this calendar month (local time), `back` months ago.
pub async fn month_offset(db: &SqlitePool, back: i64) -> sqlx::Result<String> {
    sqlx::query_scalar("SELECT strftime('%Y-%m', 'now', 'localtime', 'start of month', '-' || ? || ' months')")
        .bind(back)
        .fetch_one(db)
        .await
}

/// Spent in the given "YYYY-MM" (see `spent_this_month`). What the savings jar paid for a date
/// doesn't count: that money was set aside in an earlier month.
pub async fn spent_in_month(db: &SqlitePool, couple: &str, month: &str) -> sqlx::Result<i64> {
    sqlx::query_scalar(
        "SELECT
           (SELECT COALESCE(SUM(MAX(actual_cost - COALESCE(from_savings, 0), 0)), 0) FROM joy_history
             WHERE couple_id = ?2 AND outcome = 'did_it' AND actual_cost IS NOT NULL
               AND strftime('%Y-%m', COALESCE(planned_at, picked_at), 'unixepoch', 'localtime') = ?1)
         + (SELECT COALESCE(SUM(amount), 0) FROM joy_other_spending WHERE couple_id = ?2 AND strftime('%Y-%m', spent_on) = ?1)
         + (SELECT COALESCE(SUM(v.amount), 0) FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
             WHERE r.couple_id = ?2 AND v.status = 'went' AND strftime('%Y-%m', v.day) = ?1)",
    )
    .bind(month)
    .bind(couple)
    .fetch_one(db)
    .await
}

/// What we spent in each month of `year` (index 0 = January), for the year in review: real costs of
/// dates both of us may see (no unrevealed surprises, nothing still to come) plus spending that
/// wasn't a slip.
pub async fn spent_by_month(db: &SqlitePool, couple: &str, year: i32) -> sqlx::Result<[i64; 12]> {
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT m, SUM(a) FROM (
           SELECT CAST(strftime('%m', COALESCE(planned_at, picked_at), 'unixepoch', 'localtime') AS INTEGER) AS m,
                  MAX(actual_cost - COALESCE(from_savings, 0), 0) AS a
             FROM joy_history
            WHERE couple_id = ?2 AND outcome = 'did_it' AND actual_cost IS NOT NULL AND (surprise = 0 OR revealed = 1)
              AND COALESCE(planned_at, picked_at) <= unixepoch()
              AND CAST(strftime('%Y', COALESCE(planned_at, picked_at), 'unixepoch', 'localtime') AS INTEGER) = ?1
           UNION ALL
           SELECT CAST(strftime('%m', spent_on) AS INTEGER), amount FROM joy_other_spending
            WHERE couple_id = ?2 AND CAST(strftime('%Y', spent_on) AS INTEGER) = ?1
           UNION ALL
           SELECT CAST(strftime('%m', v.day) AS INTEGER), v.amount FROM joy_regular_visits v
              JOIN joy_regulars r ON r.id = v.regular_id
            WHERE r.couple_id = ?2 AND v.status = 'went' AND CAST(strftime('%Y', v.day) AS INTEGER) = ?1
         ) GROUP BY m",
    )
    .bind(year)
    .bind(couple)
    .fetch_all(db)
    .await?;
    let mut out = [0; 12];
    for (m, total) in rows {
        if let Some(slot) = usize::try_from(m - 1).ok().and_then(|i| out.get_mut(i)) {
            *slot = total;
        }
    }
    Ok(out)
}

/// What we decided at the start of a month: the budget then, and money rolled over into it.
pub async fn month_budget(db: &SqlitePool, couple: &str, month: &str) -> sqlx::Result<Option<(i64, i64)>> {
    sqlx::query_as("SELECT base, carry FROM joy_month_budget WHERE couple_id = ? AND month = ?")
        .bind(couple)
        .bind(month)
        .fetch_optional(db)
        .await
}

/// Settle a month's start: rolling over `carry` (0 = start fresh). Pressing twice changes nothing.
/// True when this call was the one that settled it.
pub async fn decide_month(db: &SqlitePool, couple: &str, month: &str, base: i64, carry: i64) -> sqlx::Result<bool> {
    let done = sqlx::query("INSERT OR IGNORE INTO joy_month_budget (couple_id, month, base, carry) VALUES (?, ?, ?, ?)")
        .bind(couple)
        .bind(month)
        .bind(base)
        .bind(carry)
        .execute(db)
        .await?;
    Ok(done.rows_affected() == 1)
}

/// Finished months with anything spent, newest first: (month "YYYY-MM", spent).
pub async fn past_months(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(String, i64)>> {
    sqlx::query_as(
        "SELECT m, SUM(a) FROM (
           SELECT strftime('%Y-%m', COALESCE(planned_at, picked_at), 'unixepoch', 'localtime') AS m,
                  MAX(actual_cost - COALESCE(from_savings, 0), 0) AS a
             FROM joy_history WHERE couple_id = ?1 AND outcome = 'did_it' AND actual_cost IS NOT NULL
           UNION ALL
           SELECT strftime('%Y-%m', spent_on), amount FROM joy_other_spending WHERE couple_id = ?1
           UNION ALL
           SELECT strftime('%Y-%m', v.day), v.amount FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
            WHERE r.couple_id = ?1 AND v.status = 'went'
         ) WHERE m < strftime('%Y-%m', 'now', 'localtime') GROUP BY m HAVING SUM(a) > 0 ORDER BY m DESC LIMIT 12",
    )
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Log money spent on a date that wasn't a slip. `spent_on` is "YYYY-MM-DD"; blank means today.
pub async fn add_other_spending(db: &SqlitePool, couple: &str, amount: i64, note: &str, spent_on: &str) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO joy_other_spending (couple_id, amount, note, spent_on) VALUES (?, ?, ?, COALESCE(NULLIF(date(?), ''), date('now', 'localtime')))")
        .bind(couple)
        .bind(amount)
        .bind(note.trim())
        .bind(spent_on.trim())
        .execute(db)
        .await?;
    Ok(())
}

pub async fn delete_other_spending(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM joy_other_spending WHERE id = ? AND couple_id = ?").bind(id).bind(couple).execute(db).await?;
    Ok(())
}

/// This month's non-slip spending, newest first: (id, amount, note, spent_on).
pub async fn other_spending_this_month(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(i64, i64, String, String)>> {
    sqlx::query_as(
        "SELECT id, amount, note, spent_on FROM joy_other_spending
         WHERE couple_id = ? AND strftime('%Y-%m', spent_on) = strftime('%Y-%m', 'now', 'localtime')
         ORDER BY spent_on DESC, id DESC",
    )
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Non-slip spending in a "YYYY-MM", oldest first: (amount, note, spent_on).
pub async fn other_spending_in_month(db: &SqlitePool, couple: &str, month: &str) -> sqlx::Result<Vec<(i64, String, String)>> {
    sqlx::query_as(
        "SELECT amount, note, spent_on FROM joy_other_spending WHERE couple_id = ? AND strftime('%Y-%m', spent_on) = ? ORDER BY spent_on, id",
    )
    .bind(couple)
    .bind(month)
    .fetch_all(db)
    .await
}

pub async fn setting(db: &SqlitePool, couple: &str, key: &str) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar("SELECT value FROM joy_settings WHERE couple_id = ? AND key = ?")
        .bind(couple)
        .bind(key)
        .fetch_optional(db)
        .await
}

pub async fn set_setting(db: &SqlitePool, couple: &str, key: &str, value: Option<&str>) -> sqlx::Result<()> {
    match value {
        Some(v) => sqlx::query(
            "INSERT INTO joy_settings (couple_id, key, value) VALUES (?, ?, ?)
             ON CONFLICT(couple_id, key) DO UPDATE SET value = excluded.value",
        )
        .bind(couple)
        .bind(key)
        .bind(v)
        .execute(db)
        .await?,
        None => sqlx::query("DELETE FROM joy_settings WHERE couple_id = ? AND key = ?")
            .bind(couple)
            .bind(key)
            .execute(db)
            .await?,
    };
    Ok(())
}

pub async fn rate(db: &SqlitePool, couple: &str, id: i64, rating: Option<i64>, note: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE joy_history SET rating = ?, note = ? WHERE id = ? AND couple_id = ?")
        .bind(rating.map(|r| r.clamp(1, 5)))
        .bind(note.trim())
        .bind(id)
        .bind(couple)
        .execute(db)
        .await?;
    Ok(())
}

// ── learned from real costs ───────────────────────────────────────────────

/// Fewest samples before we trust what we've learned.
pub const MIN_SAMPLES: i64 = 3;

/// How real plan costs compare to estimates: (sum actual ÷ sum estimate, samples).
pub async fn cost_ratio(db: &SqlitePool, couple: &str) -> sqlx::Result<(f64, i64)> {
    let (actual, est, n): (Option<i64>, Option<i64>, i64) = sqlx::query_as(
        "SELECT SUM(actual_cost), SUM(est_cost), COUNT(*) FROM joy_history
         WHERE couple_id = ? AND outcome = 'did_it' AND actual_cost IS NOT NULL AND est_cost > 0",
    )
    .bind(couple)
    .fetch_one(db)
    .await?;
    Ok(match (actual, est) {
        (Some(a), Some(e)) if e > 0 => (a as f64 / e as f64, n),
        _ => (1.0, 0),
    })
}

/// Real cab fares vs the model's estimates at the time: (ratio, rides).
pub async fn cab_ratio(db: &SqlitePool, couple: &str) -> sqlx::Result<(f64, i64)> {
    let (fare, est, n): (Option<i64>, Option<i64>, i64) = sqlx::query_as(
        "SELECT SUM(fare), SUM(est), COUNT(*) FROM (
           SELECT cab_fare_0 AS fare, cab_est_0 AS est FROM joy_history WHERE couple_id = ?1 AND cab_fare_0 > 0 AND cab_est_0 > 0
           UNION ALL
           SELECT cab_fare_1, cab_est_1 FROM joy_history WHERE couple_id = ?1 AND cab_fare_1 > 0 AND cab_est_1 > 0)",
    )
    .bind(couple)
    .fetch_one(db)
    .await?;
    Ok(match (fare, est) {
        (Some(f), Some(e)) if e > 0 => (f as f64 / e as f64, n),
        _ => (1.0, 0),
    })
}

/// Record a real one-way cab fare for one of us, next to the model's estimate for that trip.
pub async fn set_cab_fare(db: &SqlitePool, couple: &str, id: i64, person: usize, fare: Option<i64>, model_est: Option<i64>) -> sqlx::Result<()> {
    let sql = if person == 0 {
        "UPDATE joy_history SET cab_fare_0 = ?, cab_est_0 = ? WHERE id = ? AND couple_id = ? AND outcome = 'did_it'"
    } else {
        "UPDATE joy_history SET cab_fare_1 = ?, cab_est_1 = ? WHERE id = ? AND couple_id = ? AND outcome = 'did_it'"
    };
    sqlx::query(sql).bind(fare).bind(fare.and(model_est)).bind(id).bind(couple).execute(db).await?;
    Ok(())
}

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

pub async fn unveto(db: &SqlitePool, couple: &str, activity: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM joy_vetoes WHERE activity_id = ? AND activity_id IN (SELECT id FROM joy_activities WHERE couple_id = ?)")
        .bind(activity)
        .bind(couple)
        .execute(db)
        .await?;
    Ok(())
}

// ── gentle nudges ─────────────────────────────────────────────────────────

/// Today and tomorrow as local "YYYY-MM-DD", plus today's weekday (0 = Sunday) and month.
pub async fn local_today(db: &SqlitePool) -> sqlx::Result<(String, String, u8, u8)> {
    let (today, tomorrow, wd, m): (String, String, i64, i64) = sqlx::query_as(
        "SELECT date('now', 'localtime'), date('now', 'localtime', '+1 day'),
                CAST(strftime('%w', 'now', 'localtime') AS INTEGER), CAST(strftime('%m', 'now', 'localtime') AS INTEGER)",
    )
    .fetch_one(db)
    .await?;
    Ok((today, tomorrow, wd as u8, m as u8))
}

/// Kept dates planned from now until the end of tomorrow (local), soonest first.
pub async fn upcoming(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<HistoryRow>> {
    sqlx::query_as(&format!(
        "SELECT {HISTORY_COLS} FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ? AND h.outcome = 'did_it' AND h.planned_at >= unixepoch() - 3 * 3600
           AND date(h.planned_at, 'unixepoch', 'localtime') <= date('now', 'localtime', '+1 day')
         ORDER BY h.planned_at, h.id"
    ))
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Is anything planned for later (beyond tomorrow too)?
pub async fn anything_planned(db: &SqlitePool, couple: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM joy_history WHERE couple_id = ? AND outcome = 'did_it' AND planned_at > unixepoch())",
    )
        .bind(couple)
        .fetch_one(db)
        .await
}

/// Whole days since our last date happened (its planned time if set, else when it was kept).
pub async fn days_since_last_date(db: &SqlitePool, couple: &str) -> sqlx::Result<Option<i64>> {
    sqlx::query_scalar(
        "SELECT CAST((unixepoch() - MAX(COALESCE(planned_at, picked_at))) / 86400 AS INTEGER) FROM joy_history
         WHERE couple_id = ? AND outcome = 'did_it' AND COALESCE(planned_at, picked_at) <= unixepoch()",
    )
    .bind(couple)
    .fetch_one(db)
    .await
}

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
pub async fn delete_saving(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM joy_savings WHERE id = ? AND couple_id = ? AND kind = 'add'").bind(id).bind(couple).execute(db).await?;
    Ok(())
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
    sqlx::query(&format!("UPDATE {table} SET share_token = ? WHERE id = ? AND couple_id = ? AND share_token IS NULL"))
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
    if let Some((couple, id)) = sqlx::query_as::<_, (String, i64)>("SELECT couple_id, id FROM joy_history WHERE share_token = ?")
        .bind(token)
        .fetch_optional(db)
        .await?
    {
        return Ok(history_row(db, &couple, id).await?.map(Envelope::Date));
    }
    match sqlx::query_as::<_, (String, i64)>("SELECT couple_id, id FROM joy_activities WHERE share_token = ?")
        .bind(token)
        .fetch_optional(db)
        .await?
    {
        Some((couple, id)) => Ok(get(db, &couple, id).await?.map(Envelope::Slip)),
        None => Ok(None),
    }
}

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
}

pub async fn regular(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<Option<Regular>> {
    sqlx::query_as("SELECT * FROM joy_regulars WHERE couple_id = ? AND id = ? AND active = 1").bind(couple).bind(id).fetch_optional(db).await
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
         SELECT r.id, d.day, strftime('%w', d.day)
           FROM d JOIN joy_regulars r ON r.active = 1 AND r.couple_id = ?
          WHERE d.day >= r.since
            AND instr(',' || r.weekdays || ',', ',' || strftime('%w', d.day) || ',') > 0
            AND NOT EXISTS (SELECT 1 FROM joy_regular_visits v WHERE v.regular_id = r.id AND v.day = d.day)
          ORDER BY d.day DESC, r.id",
    )
    .bind(couple)
    .fetch_all(db)
    .await?;
    let mut out = vec![];
    for (id, day, _) in rows {
        if let Some(r) = regular(db, couple, id).await? {
            let label: String = sqlx::query_scalar(
                "SELECT substr('SunMonTueWedThuFriSat', 1 + 3 * CAST(strftime('%w', ?1) AS INTEGER), 3) || ' ' ||
                        CAST(strftime('%d', ?1) AS INTEGER) || ' ' ||
                        substr('JanFebMarAprMayJunJulAugSepOctNovDec', 1 + 3 * (CAST(strftime('%m', ?1) AS INTEGER) - 1), 3)",
            )
            .bind(&day)
            .fetch_one(db)
            .await?;
            out.push((r, day, label));
        }
    }
    Ok(out)
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
    let mut out = vec![];
    for v in visits {
        if let Some(r) = regular(db, couple, v.regular_id).await? {
            out.push((r, v));
        }
    }
    Ok(out)
}

/// Say whether we went on `day`. `status` is "went" or "skipped"; a blank `place` means the
/// regular's own place. Days in the future, or more than a month back, are refused (false).
/// Answering again changes the answer; a photo already on the night is kept.
pub async fn record_visit(db: &SqlitePool, couple: &str, regular_id: i64, day: &str, status: &str, place: &str, amount: i64) -> sqlx::Result<bool> {
    let ok: i64 = sqlx::query_scalar(
        "SELECT date(?1) IS NOT NULL AND date(?1) = ?1
            AND date(?1) <= date('now', 'localtime') AND date(?1) >= date('now', 'localtime', '-31 days')
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
    Ok(true)
}

/// The nights we went, newest first, across every regular (stopped ones too) with the regular's
/// name; at most `limit`.
pub async fn went_nights(db: &SqlitePool, couple: &str, limit: i64) -> sqlx::Result<Vec<(Visit, String)>> {
    let rows: Vec<Visit> = sqlx::query_as(
        "SELECT v.* FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
          WHERE r.couple_id = ? AND v.status = 'went' ORDER BY v.day DESC, v.id DESC LIMIT ?",
    )
    .bind(couple)
    .bind(limit)
    .fetch_all(db)
    .await?;
    let mut out = vec![];
    for v in rows {
        let title = regular(db, couple, v.regular_id).await?.map(|r| r.title).unwrap_or_default();
        out.push((v, title));
    }
    Ok(out)
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
          WHERE r.couple_id = ? AND v.status = 'went' AND strftime('%Y-%m', v.day) = ? ORDER BY v.day, v.id",
    )
    .bind(couple)
    .bind(month)
    .fetch_all(db)
    .await?;
    let mut out = vec![];
    for v in rows {
        let title = regular(db, couple, v.regular_id).await?.map(|r| r.title).unwrap_or_default();
        out.push((v, title));
    }
    Ok(out)
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

/// Fill a new couple's jar: copies every starter slip in `joy_seed_activities` into their
/// `joy_activities`. Returns how many slips were added.
pub async fn seed_new_couple(db: &SqlitePool, couple: &str) -> sqlx::Result<u64> {
    let done = sqlx::query(
        "INSERT INTO joy_activities (couple_id, title, notes, est_cost, setting, area, venue, lat, lng, duration, tags, energy,
                added_by, generated, closed_days, season_from, season_to, hours, best_time, minutes, booking,
                book_ahead_days, cover, bring, station, station_walk_min)
         SELECT ?, title, notes, est_cost, setting, area, venue, lat, lng, duration, tags, energy,
                added_by, generated, closed_days, season_from, season_to, hours, best_time, minutes, booking,
                book_ahead_days, cover, bring, station, station_walk_min
           FROM joy_seed_activities ORDER BY id",
    )
    .bind(couple)
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    /// A migrated in-memory database with two users, each the first half of their own couple.
    async fn test_db() -> SqlitePool {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for (u, c) in [("ua", "A"), ("ub", "B")] {
            sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES (?, ?, 'x', 'user')")
                .bind(u)
                .bind(u)
                .execute(&db)
                .await
                .unwrap();
            sqlx::query("INSERT INTO couples (couple_id, user_0) VALUES (?, ?)").bind(c).bind(u).execute(&db).await.unwrap();
        }
        db
    }

    async fn add_slip(db: &SqlitePool, couple: &str, title: &str) -> i64 {
        let input = ActivityInput {
            title: title.into(),
            est_cost: 500,
            setting: "local".into(),
            duration: "short".into(),
            energy: "med".into(),
            ..Default::default()
        };
        save(db, couple, None, &input, None).await.unwrap()
    }

    #[tokio::test]
    async fn deleting_a_slip_with_history_archives_it_and_keeps_the_history() {
        let db = test_db().await;
        let id = add_slip(&db, "A", "Sunset walk").await;
        let hid = accept(&db, "A", id, Some(0), false).await.unwrap();
        sqlx::query("INSERT INTO joy_memories (history_id, who, line) VALUES (?, 0, 'loved it')")
            .bind(hid)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO joy_tasks (history_id, who, task) VALUES (?, 1, 'book cab')")
            .bind(hid)
            .execute(&db)
            .await
            .unwrap();

        assert!(!delete(&db, "A", id).await.unwrap(), "a slip with history is archived, not deleted");

        assert!(list(&db, "A", false).await.unwrap().iter().all(|a| a.id != id), "gone from the slip list");
        assert!(list(&db, "A", true).await.unwrap().iter().any(|a| a.id == id), "kept as archived");
        let history: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM joy_history WHERE activity_id = ?").bind(id).fetch_one(&db).await.unwrap();
        let memories: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM joy_memories WHERE history_id = ?").bind(hid).fetch_one(&db).await.unwrap();
        let tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM joy_tasks WHERE history_id = ?").bind(hid).fetch_one(&db).await.unwrap();
        assert_eq!((history, memories, tasks), (1, 1, 1));
    }

    #[tokio::test]
    async fn deleting_a_slip_without_history_really_deletes_it() {
        let db = test_db().await;
        let id = add_slip(&db, "A", "Idle idea").await;
        assert!(delete(&db, "A", id).await.unwrap());
        assert!(get(&db, "A", id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn a_sealed_surprise_is_hidden_from_the_partner_only() {
        let db = test_db().await;
        let id = add_slip(&db, "A", "Secret rooftop").await;
        // Slot 0 planned it as a surprise: slot 1 must not see it until it's revealed.
        accept(&db, "A", id, Some(0), true).await.unwrap();
        assert_eq!(sealed_from(&db, "A", 1).await.unwrap(), vec![id]);
        assert!(sealed_from(&db, "A", 0).await.unwrap().is_empty(), "the planner still sees it");
        sqlx::query("UPDATE joy_history SET revealed = 1 WHERE activity_id = ?").bind(id).execute(&db).await.unwrap();
        assert!(sealed_from(&db, "A", 1).await.unwrap().is_empty(), "revealed surprises are visible");
    }

    #[tokio::test]
    async fn seed_copies_every_starter_slip() {
        let db = test_db().await;
        assert_eq!(seed_new_couple(&db, "A").await.unwrap(), 333);
        assert_eq!(list(&db, "A", false).await.unwrap().len(), 333);
        assert!(list(&db, "B", false).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn spending_ignores_the_other_couple() {
        let db = test_db().await;
        let month = month_offset(&db, 0).await.unwrap();
        add_other_spending(&db, "A", 700, "chai", "").await.unwrap();
        add_other_spending(&db, "B", 9000, "dinner", "").await.unwrap();
        let slip_b = add_slip(&db, "B", "B's slip").await;
        let kept_b = accept(&db, "B", slip_b, Some(0), false).await.unwrap();
        set_actual(&db, "B", kept_b, Some(1200), false).await.unwrap();

        assert_eq!(spent_in_month(&db, "A", &month).await.unwrap(), 700);
        assert_eq!(spent_in_month(&db, "B", &month).await.unwrap(), 9000 + 1200);
        assert_eq!(spent_this_month(&db, "A").await.unwrap(), 700);
        assert_eq!(other_spending_this_month(&db, "A").await.unwrap().len(), 1);
        // A can't delete B's row either.
        let b_row = other_spending_this_month(&db, "B").await.unwrap()[0].0;
        delete_other_spending(&db, "A", b_row).await.unwrap();
        assert_eq!(other_spending_this_month(&db, "B").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn memories_tasks_and_visits_stay_with_their_couple() {
        let db = test_db().await;
        let slip = add_slip(&db, "B", "B's slip").await;
        let kept = accept(&db, "B", slip, Some(1), false).await.unwrap();
        set_memory(&db, "B", kept, 0, "we laughed").await.unwrap();
        add_tasks(&db, "B", kept, &[(0, "book".into(), "".into())]).await.unwrap();
        let reg = add_regular(&db, "B", "Blue Tokai", "", "", "2,4", 400, "2020-01-01").await.unwrap();
        let day = sqlx::query_scalar::<_, String>("SELECT date('now', 'localtime')").fetch_one(&db).await.unwrap();
        assert!(record_visit(&db, "B", reg, &day, "went", "", 400).await.unwrap());
        let visit_id = went_visits(&db, "B", reg).await.unwrap()[0].id;
        let task_id = tasks_for(&db, "B", kept).await.unwrap()[0].id;

        // B sees them all.
        assert_eq!(memories(&db, "B").await.unwrap().len(), 1);
        assert_eq!(task_counts(&db, "B").await.unwrap(), [1, 0]);
        assert!(visit(&db, "B", visit_id).await.unwrap().is_some());

        // A sees none, and can't write to them.
        assert!(memories(&db, "A").await.unwrap().is_empty());
        assert!(tasks_for(&db, "A", kept).await.unwrap().is_empty());
        assert!(task(&db, "A", task_id).await.unwrap().is_none());
        assert_eq!(task_counts(&db, "A").await.unwrap(), [0, 0]);
        assert!(regular(&db, "A", reg).await.unwrap().is_none());
        assert!(regulars(&db, "A").await.unwrap().is_empty());
        assert!(went_visits(&db, "A", reg).await.unwrap().is_empty());
        assert!(visit(&db, "A", visit_id).await.unwrap().is_none());
        assert!(visits_in_month(&db, "A", &day[..7]).await.unwrap().is_empty());
        assert!(!record_visit(&db, "A", reg, &day, "skipped", "", 0).await.unwrap());
        set_memory(&db, "A", kept, 0, "hijack").await.unwrap();
        set_memory(&db, "A", kept, 1, "hijack").await.unwrap();
        add_tasks(&db, "A", kept + 1000, &[(0, "x".into(), "".into())]).await.unwrap();
        set_task_done(&db, "A", task_id, true).await.unwrap();
        delete_visit(&db, "A", visit_id).await.unwrap();
        set_visit_photo(&db, "A", visit_id, true).await.unwrap();
        assert_eq!(memories(&db, "B").await.unwrap(), vec![(kept, 0, "we laughed".to_string())]);
        assert!(task(&db, "B", task_id).await.unwrap().unwrap().done_at.is_none());
        let v = visit(&db, "B", visit_id).await.unwrap().unwrap();
        assert_eq!(v.status, "went");
        assert!(!v.has_photo);
        // Photos: B's date takes them, A can neither add to it nor remove from it.
        let p1 = add_photo(&db, "B", kept, "x-1.jpg").await.unwrap().unwrap();
        let p2 = add_photo(&db, "B", kept, "x-2.jpg").await.unwrap().unwrap();
        assert!(add_photo(&db, "A", kept, "evil.jpg").await.unwrap().is_none());
        assert!(remove_photo(&db, "A", kept, p1).await.unwrap().is_none());
        assert!(photos_of(&db, "A", kept).await.unwrap().is_empty());
        assert!(all_photos(&db, "A").await.unwrap().is_empty());
        assert_eq!(photos_of(&db, "B", kept).await.unwrap().iter().map(|p| p.id).collect::<Vec<_>>(), vec![p1, p2]);
        assert_eq!(photo_by_id(&db, kept, p2).await.unwrap().unwrap().0, "B");
        assert!(photo_by_id(&db, kept + 1, p2).await.unwrap().is_none(), "a photo only answers to its own date");
        assert!(history_row(&db, "B", kept).await.unwrap().unwrap().has_photo);
        assert_eq!(remove_photo(&db, "B", kept, p1).await.unwrap().as_deref(), Some("x-1.jpg"));
        assert!(history_row(&db, "B", kept).await.unwrap().unwrap().has_photo, "one photo is still there");
        remove_photo(&db, "B", kept, p2).await.unwrap();
        assert!(!history_row(&db, "B", kept).await.unwrap().unwrap().has_photo);
        add_photo(&db, "B", kept, "x-3.jpg").await.unwrap().unwrap();
        // A can't delete B's kept date either.
        delete_history(&db, "A", kept).await.unwrap();
        assert!(history_row(&db, "B", kept).await.unwrap().is_some());
        assert!(history_row(&db, "A", kept).await.unwrap().is_none());
        // Deleting the date forgets its photos.
        delete_history(&db, "B", kept).await.unwrap();
        assert!(all_photos(&db, "B").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_custom_date_is_kept_on_its_day_and_stays_out_of_the_jar() {
        let db = test_db().await;
        let id = keep_custom(&db, "B", Some(0), "Picnic at the lake", "Powai", "local", "2026-03-14", Some(1200)).await.unwrap();
        let row = history_row(&db, "B", id).await.unwrap().unwrap();
        assert_eq!((row.title.as_str(), row.day.as_str(), row.outcome.as_str()), ("Picnic at the lake", "2026-03-14", "did_it"));
        assert_eq!((row.place.as_str(), row.actual_cost, row.est_cost), ("Powai", Some(1200), 1200));
        assert!(list(&db, "B", false).await.unwrap().iter().all(|a| a.title != "Picnic at the lake"), "not a slip in the jar");
        assert!(history_row(&db, "A", id).await.unwrap().is_none(), "the other couple never sees it");
        let free = keep_custom(&db, "B", None, "Walk", "", "home", "2026-03-15", None).await.unwrap();
        assert_eq!(history_row(&db, "B", free).await.unwrap().unwrap().actual_cost, None);
    }

    #[tokio::test]
    async fn a_date_holds_a_limited_number_of_photos() {
        let db = test_db().await;
        let slip = add_slip(&db, "B", "B's slip").await;
        let kept = accept(&db, "B", slip, Some(1), false).await.unwrap();
        for i in 0..MAX_PHOTOS_PER_DATE {
            assert!(add_photo(&db, "B", kept, &format!("{i}.jpg")).await.unwrap().is_some());
        }
        assert!(add_photo(&db, "B", kept, "one-too-many.jpg").await.unwrap().is_none());
        assert_eq!(photos_of(&db, "B", kept).await.unwrap().len() as i64, MAX_PHOTOS_PER_DATE);
    }
}
