use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

use crate::joy::picker::{Candidate, Duration};

mod learned;
mod txn_link;
pub use txn_link::*;
pub use learned::*;
mod vetoes;
pub use vetoes::*;
mod nudges;
pub use nudges::*;
mod lines;
pub use lines::*;
mod savings;
pub use savings::*;
mod day_plan;
pub use day_plan::*;
mod regulars;
pub use regulars::*;
mod packs;
pub use packs::*;


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
    /// Came from a starter pack rather than being written by the couple.
    pub starter: bool,
    /// A booking page, menu or post: http(s) only.
    pub url: String,
    /// Unix seconds a person last saved this slip (None for starter slips nobody has touched).
    pub verified_at: Option<i64>,
}

impl Activity {
    pub fn latlng(&self) -> Option<(f64, f64)> {
        Some((self.lat?, self.lng?))
    }

    pub fn tag_list(&self) -> Vec<String> {
        self.tags.split(',').map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty()).collect()
    }

    /// Tags for the draw and the taste model: the ones written plus effort and time implied by
    /// energy and duration. Not for display or storage.
    pub fn draw_tags(&self) -> Vec<String> {
        crate::joy::tags::with_implied(self.tag_list(), self.duration(), &self.energy)
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
            Some(f) if !self.est_from_actual && self.est_cost > 0 => ((self.est_cost as f64 * f) / 5_000.0).round() as i64 * 5_000,
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
            tags: self.draw_tags(),
            energy: self.energy.clone(),
            last_done_at: self.last_done_at,
            last_skipped_at: self.last_skipped_at,
            distance_km,
            closed_days: crate::joy::picker::parse_closed(&self.closed_days),
            season: self.season(),
            window_miss: None,
            taste: 1.0,
            outdoor: self.outdoor(),
            like_recent: false,
            both_want: false,
            any_no: false,
            starter: self.starter,
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
    #[serde(default)]
    pub url: String,
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

/// `times_done` counts kept dates whose day has passed. `last_done_at` is the latest kept date,
/// planned or done, so a slip that is booked is not handed out again. Both are derived from the
/// history; the stored columns of the same names are no longer read.
const COLS: &str = "id, title, notes, est_cost, setting, area, venue, lat, lng, duration, tags, energy, \
                    added_by, \
                    (SELECT COUNT(*) FROM joy_history h WHERE h.activity_id = joy_activities.id AND h.outcome = 'did_it' \
                       AND COALESCE(h.planned_at, h.picked_at) <= unixepoch()) AS times_done, \
                    (SELECT MAX(COALESCE(h.planned_at, h.picked_at)) FROM joy_history h \
                      WHERE h.activity_id = joy_activities.id AND h.outcome = 'did_it') AS last_done_at, \
                    last_skipped_at, generated, \
                    closed_days, season_from, season_to, est_from_actual, \
                    hours, best_time, minutes, booking, book_ahead_days, cover, bring, station, station_walk_min, url, verified_at, starter";

pub async fn list(db: &SqlitePool, couple: &str, archived: bool) -> sqlx::Result<Vec<Activity>> {
    sqlx::query_as(&format!(
        "SELECT {COLS} FROM joy_activities WHERE couple_id = ? AND archived = ? AND (archived = 0 OR custom = 0) ORDER BY est_cost, title"
    ))
    .bind(couple)
    .bind(archived)
    .fetch_all(db)
    .await
}

/// Raw lookup with NO seal check. Anything reached through a client-sent id goes through
/// `visible_get`, `drawable_get` or `api::slip` instead; a slip behind one of our own history rows
/// goes through `row_activity`.
pub async fn get(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<Option<Activity>> {
    sqlx::query_as(&format!("SELECT {COLS} FROM joy_activities WHERE couple_id = ? AND id = ?"))
        .bind(couple)
        .bind(id)
        .fetch_optional(db)
        .await
}

/// Mark a slip "want", "fine" or "no" for `person`; "" takes the mark back. Quietly: it never
/// touches the other one's mark.
pub async fn set_want(db: &SqlitePool, couple: &str, id: i64, person: usize, level: &str) -> sqlx::Result<()> {
    if level.is_empty() {
        sqlx::query("DELETE FROM joy_wants WHERE activity_id = ? AND person = ? AND activity_id IN (SELECT id FROM joy_activities WHERE couple_id = ?)")
            .bind(id)
            .bind(person as i64)
            .bind(couple)
            .execute(db)
            .await?;
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO joy_wants (activity_id, person, level)
         SELECT id, ?2, ?3 FROM joy_activities WHERE id = ?1 AND couple_id = ?4
         ON CONFLICT(activity_id, person) DO UPDATE SET level = excluded.level",
    )
    .bind(id)
    .bind(person as i64)
    .bind(level)
    .bind(couple)
    .execute(db)
    .await?;
    Ok(())
}

/// Every mark in the couple's jar: (slip id, person, level). Callers must only ever show a person
/// their own marks, plus "we both want it".
pub async fn wants(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(i64, i64, String)>> {
    sqlx::query_as(
        "SELECT w.activity_id, w.person, w.level FROM joy_wants w JOIN joy_activities a ON a.id = w.activity_id WHERE a.couple_id = ?",
    )
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Both partners (the two distinct slots) marked this slip "want". Takes `wants` as fetched once per
/// request; only ever answers yes or no, never who or how much.
pub fn both_want(wants: &[(i64, i64, String)], id: i64) -> bool {
    (0..2).all(|p| wants.iter().any(|(i, person, l)| *i == id && *person == p && l == "want"))
}

/// Another slip in the jar with the same title (ignoring case and spacing), if any.
pub async fn duplicate_of(db: &SqlitePool, couple: &str, title: &str, except: Option<i64>) -> sqlx::Result<Option<i64>> {
    let norm = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    let want = norm(title);
    let rows: Vec<(i64, String)> = sqlx::query_as("SELECT id, title FROM joy_activities WHERE couple_id = ? AND archived = 0")
        .bind(couple)
        .fetch_all(db)
        .await?;
    Ok(rows.into_iter().find(|(id, t)| Some(*id) != except && norm(t) == want).map(|(id, _)| id))
}

/// Insert or update a slip and its pairs; returns its id.
pub async fn save(db: &SqlitePool, couple: &str, id: Option<i64>, a: &ActivityInput, latlng: Option<(f64, f64)>) -> sqlx::Result<i64> {
    let tags = crate::joy::tags::normalise(&a.tags);
    let sql = match id {
        None => "INSERT INTO joy_activities (title, notes, est_cost, setting, duration, venue, area, lat, lng, tags, energy, added_by,
                 closed_days, season_from, season_to, hours, best_time, minutes, booking, book_ahead_days, cover, bring,
                 station, station_walk_min, url, verified_at, id, couple_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?17, ?18, ?19, ?20, ?21, ?22, ?23,
                 ?24, ?25, ?27, unixepoch(), ?16, ?26)",
        // A hand-edited estimate is ours again, so the learned factor applies to it.
        Some(_) => "UPDATE joy_activities SET title = ?1, notes = ?2,
                    est_from_actual = CASE WHEN est_cost = ?3 THEN est_from_actual ELSE 0 END, est_cost = ?3,
                    setting = ?4, duration = ?5, venue = ?6, area = ?7, lat = ?8, lng = ?9, tags = ?10, energy = ?11,
                    added_by = ?12, closed_days = ?13, season_from = ?14, season_to = ?15, hours = ?17, best_time = ?18,
                    minutes = ?19, booking = ?20, book_ahead_days = ?21, cover = ?22, bring = ?23, station = ?24,
                    station_walk_min = ?25, url = ?27, verified_at = unixepoch() WHERE id = ?16 AND couple_id = ?26",
    };
    let closed = crate::joy::picker::WEEKDAYS
        .iter()
        .map(|(k, _)| *k)
        .filter(|k| a.closed.iter().any(|c| c == k))
        .collect::<Vec<_>>()
        .join(",");
    let month = |s: &str| s.trim().parse::<i64>().ok().filter(|m| (1..=12).contains(m));
    let season = month(&a.season_from).zip(month(&a.season_to));
    // The slip and its pairs are saved together or not at all.
    let mut tx = db.begin().await?;
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
        .bind(a.url.trim())
        .execute(&mut *tx)
        .await?;
    // Editing a slip that isn't this couple's matches nothing.
    if id.is_some() && done.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    // From this statement's own result: a separate query could run on another pool connection.
    let id = id.unwrap_or(done.last_insert_rowid());
    set_pairs_in(&mut tx, couple, id, &a.pairs).await?;
    tx.commit().await?;
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
    set_pairs_in(&mut tx, couple, id, with).await?;
    tx.commit().await
}

async fn set_pairs_in(tx: &mut sqlx::SqliteConnection, couple: &str, id: i64, with: &[i64]) -> sqlx::Result<()> {
    let mine: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM joy_activities WHERE id = ? AND couple_id = ?)")
        .bind(id)
        .bind(couple)
        .fetch_one(&mut *tx)
        .await?;
    if !mine {
        return Ok(());
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
    Ok(())
}

pub async fn restore_one(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<bool> {
    let done = sqlx::query("UPDATE joy_activities SET archived = 0 WHERE id = ? AND couple_id = ? AND custom = 0")
        .bind(id)
        .bind(couple)
        .execute(db)
        .await?;
    Ok(done.rows_affected() > 0)
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

/// A slip that can be drawn, kept or planned right now: in the jar (not archived), not passed on
/// by either of us, and not sealed from `who`. Anything else is not found, the same as one that
/// doesn't exist.
pub async fn drawable_get(db: &SqlitePool, couple: &str, who: usize, id: i64) -> sqlx::Result<Option<Activity>> {
    let Some(a) = visible_get(db, couple, who, id).await? else { return Ok(None) };
    let archived: bool = sqlx::query_scalar("SELECT archived FROM joy_activities WHERE id = ? AND couple_id = ?")
        .bind(id)
        .bind(couple)
        .fetch_one(db)
        .await?;
    let vetoed: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM joy_vetoes WHERE activity_id = ?)").bind(id).fetch_one(db).await?;
    Ok((!archived && !vetoed).then_some(a))
}

/// `get`, but a slip sealed from `who` is not found, the same as one that doesn't exist.
pub async fn visible_get(db: &SqlitePool, couple: &str, who: usize, id: i64) -> sqlx::Result<Option<Activity>> {
    if sealed_from(db, couple, who).await?.contains(&id) {
        return Ok(None);
    }
    get(db, couple, id).await
}

/// `visible_get` for a viewer who may be unknown: no viewer fails closed, as in `visible_list`.
pub async fn visible_get_opt(db: &SqlitePool, couple: &str, who: Option<usize>, id: i64) -> sqlx::Result<Option<Activity>> {
    match who {
        Some(w) => visible_get(db, couple, w, id).await,
        None if sealed_any(db, couple).await?.contains(&id) => Ok(None),
        None => get(db, couple, id).await,
    }
}

/// The slip behind one of the couple's own history rows (never a client-sent id). It is not hidden
/// from a viewer a surprise is sealed from: the caller masks what it shows with `decide::sealed_for`.
pub async fn row_activity(db: &SqlitePool, couple: &str, row: &HistoryRow) -> sqlx::Result<Option<Activity>> {
    get(db, couple, row.activity_id).await
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

/// Server-side limits on what a slip may hold (the form's `maxLength` is only a courtesy).
pub const MAX_TITLE: usize = 120;
pub const MAX_PLACE: usize = 120;
pub const MAX_NOTES: usize = 2000;
pub const MAX_URL: usize = 300;

/// Empty, or an http(s) link of reasonable length.
pub fn url_ok(u: &str) -> bool {
    let u = u.trim();
    u.is_empty() || (u.len() <= MAX_URL && (u.starts_with("https://") || u.starts_with("http://")) && !u.contains(char::is_whitespace))
}
const MAX_TAGS: usize = 12;
const MAX_TAG_LEN: usize = 24;

/// At most 12 tags of at most 24 characters each.
pub fn tags_fit(tags: &str) -> bool {
    let normalised = crate::joy::tags::normalise(tags);
    let list: Vec<&str> = normalised.split(',').filter(|t| !t.is_empty()).collect();
    list.len() <= MAX_TAGS && list.iter().all(|t| t.chars().count() <= MAX_TAG_LEN)
}

/// The one-tap reasons for putting a slip back. Anything else is stored as no reason.
pub const PUT_BACK_REASONS: [&str; 4] = ["not_tonight", "too_far", "too_pricey", "not_my_thing"];

/// Remember that the jar showed this slip, and the chance it had (see `picker::draw_probabilities`).
/// Only the draw's own numbers are kept: nothing here says why anyone liked or disliked it.
pub async fn log_draw(db: &SqlitePool, couple: &str, id: i64, who: usize, probability: f64, pool_size: usize) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO joy_draws (couple_id, activity_id, who, probability, pool_size)
         SELECT ?1, id, ?3, ?4, ?5 FROM joy_activities WHERE id = ?2 AND couple_id = ?1",
    )
    .bind(couple)
    .bind(id)
    .bind(who as i64)
    .bind(probability)
    .bind(pool_size as i64)
    .execute(db)
    .await
    .map(|_| ())
}

/// Put a slip back while drawing. This is a peek, not a date: it never touches the history, and
/// it only damps the slip for a few days (`last_skipped_at`).
pub async fn put_back(db: &SqlitePool, couple: &str, id: i64, who: usize, reason: &str) -> sqlx::Result<()> {
    let reason = if PUT_BACK_REASONS.contains(&reason) { reason } else { "" };
    let mut tx = db.begin().await?;
    let n = sqlx::query(
        "INSERT INTO joy_peeks (couple_id, activity_id, who, reason)
         SELECT ?1, id, ?3, ?4 FROM joy_activities WHERE id = ?2 AND couple_id = ?1",
    )
    .bind(couple)
    .bind(id)
    .bind(who as i64)
    .bind(reason)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n > 0 {
        sqlx::query("UPDATE joy_activities SET last_skipped_at = unixepoch() WHERE id = ? AND couple_id = ?")
            .bind(id)
            .bind(couple)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}

/// Accept a pick and return its history row. A double-tap within a minute only counts once
/// (the second tap gets the same row back).
pub async fn accept(db: &SqlitePool, couple: &str, id: i64, drawn_by: Option<usize>, surprise: bool) -> sqlx::Result<i64> {
    accept_many(db, couple, &[id], drawn_by, surprise).await?.into_iter().next().ok_or(sqlx::Error::RowNotFound)
}

/// Accept several slips atomically. A paired pick must never leave only one half in history.
pub async fn accept_many(
    db: &SqlitePool,
    couple: &str,
    ids: &[i64],
    drawn_by: Option<usize>,
    surprise: bool,
) -> sqlx::Result<Vec<i64>> {
    let mut tx = db.begin().await?;
    let mut accepted = Vec::with_capacity(ids.len());
    for id in ids {
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
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
        accepted.push(row);
    }
    tx.commit().await?;
    Ok(accepted)
}

/// The (setting, main tag) of the last two dates that happened, newest first: what we just did.
pub async fn recent_kinds(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<(String, String)>> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT a.setting, a.tags FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ? AND h.outcome = 'did_it' AND COALESCE(h.planned_at, h.picked_at) <= unixepoch()
         ORDER BY COALESCE(h.planned_at, h.picked_at) DESC, h.id DESC LIMIT 2",
    )
    .bind(couple)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(|(s, t)| (s, t.split(',').next().unwrap_or("").trim().to_lowercase())).collect())
}

/// The most recent kept date whose day has passed and that nobody has confirmed, rated or priced:
/// the "did it happen?" question. Sealed surprises are left to their planner (the caller filters).
pub async fn to_close(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<HistoryRow>> {
    sqlx::query_as(&format!(
        "SELECT {HISTORY_COLS} FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ? AND h.outcome = 'did_it' AND h.confirmed_at IS NULL
           AND h.rating IS NULL AND h.actual_cost IS NULL AND a.custom = 0
           AND COALESCE(h.planned_at, h.picked_at) <= unixepoch() - 3 * 3600
         ORDER BY COALESCE(h.planned_at, h.picked_at) DESC LIMIT 5"
    ))
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Close out a kept date whose day has passed. `true` confirms it; `false` cancels it (outcome
/// 'cancelled'), which takes it out of Kept, the recap, the budget and what the jar has learned.
/// Returns false when there is no such date, or its day has not come yet.
pub async fn set_happened(db: &SqlitePool, couple: &str, id: i64, happened: bool) -> sqlx::Result<bool> {
    let sql = if happened {
        "UPDATE joy_history SET confirmed_at = unixepoch() WHERE id = ? AND couple_id = ? AND outcome = 'did_it'
           AND COALESCE(planned_at, picked_at) <= unixepoch()"
    } else {
        "UPDATE joy_history SET outcome = 'cancelled' WHERE id = ? AND couple_id = ? AND outcome = 'did_it'
           AND COALESCE(planned_at, picked_at) <= unixepoch()"
    };
    Ok(sqlx::query(sql).bind(id).bind(couple).execute(db).await?.rows_affected() == 1)
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
         VALUES (?, ?, MIN(CAST(strftime('%s', ? || ' 12:00:00', 'utc') AS INTEGER), unixepoch()), 'did_it', ?, ?, ?)",
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

/// The one-off slips behind days we marked by hand (`keep_custom`), newest first, with the days
/// they were kept on. Slips sealed from `who` are left out, like everywhere else.
pub async fn custom_slips(db: &SqlitePool, couple: &str, who: usize) -> sqlx::Result<Vec<(Activity, String)>> {
    let sealed = sealed_from(db, couple, who).await?;
    let acts: Vec<Activity> = sqlx::query_as(&format!("SELECT {COLS} FROM joy_activities WHERE couple_id = ? AND custom = 1 ORDER BY id DESC"))
        .bind(couple)
        .fetch_all(db)
        .await?;
    let mut out = Vec::new();
    for a in acts.into_iter().filter(|a| !sealed.contains(&a.id)) {
        let days: Option<String> = sqlx::query_scalar(
            "SELECT group_concat(day, ',') FROM (SELECT date(picked_at, 'unixepoch', 'localtime') AS day FROM joy_history
              WHERE activity_id = ? AND couple_id = ? AND outcome = 'did_it' ORDER BY picked_at)",
        )
        .bind(a.id)
        .bind(couple)
        .fetch_one(db)
        .await?;
        out.push((a, days.unwrap_or_default()));
    }
    Ok(out)
}

/// Whether this is the one-off slip of a day we marked by hand.
pub async fn is_custom(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<bool> {
    let c: Option<bool> = sqlx::query_scalar("SELECT custom = 1 FROM joy_activities WHERE id = ? AND couple_id = ?")
        .bind(id)
        .bind(couple)
        .fetch_optional(db)
        .await?;
    Ok(c.unwrap_or(false))
}

/// A hand-marked day's slip becomes an ordinary slip in the jar, so it can be drawn again. Deleting
/// its kept date no longer removes it.
pub async fn custom_to_jar(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<()> {
    sqlx::query("UPDATE joy_activities SET custom = 0, archived = 0 WHERE id = ? AND couple_id = ? AND custom = 1")
        .bind(id)
        .bind(couple)
        .execute(db)
        .await?;
    Ok(())
}

/// Moves a kept date to the day it really happened (`on`, "YYYY-MM-DD"). False when it isn't this
/// couple's kept date, or it is a regular night and that regular already has a night on `on`.
pub async fn set_kept_day(db: &SqlitePool, couple: &str, id: i64, on: &str) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let moved = sqlx::query(
        "UPDATE joy_history SET picked_at = MIN(CAST(strftime('%s', ?1 || ' 12:00:00', 'utc') AS INTEGER), unixepoch()), planned_at = NULL
          WHERE id = ?2 AND couple_id = ?3 AND outcome = 'did_it'",
    )
    .bind(on)
    .bind(id)
    .bind(couple)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if moved == 0 {
        tx.rollback().await?;
        return Ok(false);
    }
    // A regular night keeps its day in step; two nights of one regular can't share a day.
    let clash = sqlx::query("UPDATE joy_regular_visits SET day = ?1 WHERE history_id = ?2").bind(on).bind(id).execute(&mut *tx).await;
    if clash.is_err() {
        tx.rollback().await?;
        return Ok(false);
    }
    tx.commit().await?;
    Ok(true)
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

use crate::joy::names::MONTHS_LONG as MONTHS;

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

/// Every photo of this couple's kept dates: within each date the hero first, then oldest first.
pub async fn all_photos(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<DatePhoto>> {
    sqlx::query_as(
        "SELECT p.id, p.history_id, p.file FROM joy_history_photos p JOIN joy_history h ON h.id = p.history_id
         WHERE h.couple_id = ? ORDER BY p.history_id, COALESCE(p.id = h.hero_photo_id, 0) DESC, p.id",
    )
    .bind(couple)
    .fetch_all(db)
    .await
}

/// The photos of one kept date: the hero we chose first (it is the cover), then oldest first.
pub async fn photos_of(db: &SqlitePool, couple: &str, history_id: i64) -> sqlx::Result<Vec<DatePhoto>> {
    sqlx::query_as(
        "SELECT p.id, p.history_id, p.file FROM joy_history_photos p JOIN joy_history h ON h.id = p.history_id
         WHERE h.couple_id = ? AND p.history_id = ? ORDER BY COALESCE(p.id = h.hero_photo_id, 0) DESC, p.id",
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
        sqlx::query(
            "UPDATE joy_history SET has_photo = EXISTS (SELECT 1 FROM joy_history_photos WHERE history_id = ?1),
                    hero_photo_id = CASE WHEN hero_photo_id = ?2 THEN NULL ELSE hero_photo_id END
             WHERE id = ?1",
        )
        .bind(history_id)
        .bind(photo_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(file)
}

/// Makes one of a date's photos its hero (the one the Month and Year pages show), or with `None`
/// goes back to the oldest. False when the date isn't this couple's or the photo isn't the date's.
pub async fn set_hero_photo(db: &SqlitePool, couple: &str, history_id: i64, photo_id: Option<i64>) -> sqlx::Result<bool> {
    let done = sqlx::query(
        "UPDATE joy_history SET hero_photo_id = ?3
         WHERE id = ?1 AND couple_id = ?2
           AND (?3 IS NULL OR EXISTS (SELECT 1 FROM joy_history_photos WHERE id = ?3 AND history_id = ?1))",
    )
    .bind(history_id)
    .bind(couple)
    .bind(photo_id)
    .execute(db)
    .await?;
    Ok(done.rows_affected() > 0)
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
             WHERE r.couple_id = ?2 AND v.status = 'went' AND v.history_id IS NULL AND strftime('%Y-%m', v.day) = ?1)",
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
            WHERE r.couple_id = ?2 AND v.status = 'went' AND v.history_id IS NULL AND CAST(strftime('%Y', v.day) AS INTEGER) = ?1
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
/// `leftover` ((amount, note) for the savings jar) goes in with it in one transaction, so a failure
/// in between can't lose the month's leftover.
pub async fn decide_month(db: &SqlitePool, couple: &str, month: &str, base: i64, carry: i64, leftover: Option<(i64, &str)>) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let done = sqlx::query("INSERT OR IGNORE INTO joy_month_budget (couple_id, month, base, carry) VALUES (?, ?, ?, ?)")
        .bind(couple)
        .bind(month)
        .bind(base)
        .bind(carry)
        .execute(&mut *tx)
        .await?;
    let settled = done.rows_affected() == 1;
    if let (true, Some((amount, note))) = (settled, leftover)
        && amount != 0 {
            sqlx::query("INSERT INTO joy_savings (couple_id, amount, kind, note) VALUES (?, ?, 'leftover', ?)")
                .bind(couple)
                .bind(amount)
                .bind(note.trim())
                .execute(&mut *tx)
                .await?;
        }
    tx.commit().await?;
    Ok(settled)
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
            WHERE r.couple_id = ?1 AND v.status = 'went' AND v.history_id IS NULL
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
    async fn paired_accept_rolls_back_both_history_rows_if_any_slip_is_missing() {
        let db = test_db().await;
        let first = add_slip(&db, "A", "First half").await;
        assert!(accept_many(&db, "A", &[first, i64::MAX], Some(0), false).await.is_err());
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM joy_history WHERE couple_id = 'A'")
            .fetch_one(&db).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn a_future_planned_keep_is_reserved_but_not_counted_as_done() {
        let db = test_db().await;
        let id = add_slip(&db, "A", "Future walk").await;
        let history_id = accept(&db, "A", id, Some(0), false).await.unwrap();
        sqlx::query("UPDATE joy_history SET planned_at = unixepoch('now', '+30 days') WHERE id = ?")
            .bind(history_id).execute(&db).await.unwrap();
        let row = get(&db, "A", id).await.unwrap().unwrap();
        assert_eq!(row.times_done, 0);
        assert!(row.last_done_at.is_some_and(|at| at > time::OffsetDateTime::now_utc().unix_timestamp()));
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
    async fn packs_split_the_starter_slips_and_adding_one_twice_adds_nothing_new() {
        let db = test_db().await;
        let packs = starter_packs(&db, "A").await.unwrap();
        assert_eq!(packs.iter().map(|p| p.key.as_str()).collect::<Vec<_>>(), ["home", "apart", "together", "mumbai"]);
        assert_eq!(packs.iter().map(|p| p.total).sum::<i64>(), 398, "every starter slip is in exactly one pack");
        assert!(packs.iter().all(|p| p.total > 0 && p.added == 0));

        let n = add_pack(&db, "A", "home").await.unwrap().unwrap();
        assert_eq!(n, packs[0].total as u64);
        assert_eq!(add_pack(&db, "A", "home").await.unwrap(), Some(0), "already there");
        let after = starter_packs(&db, "A").await.unwrap();
        assert_eq!((after[0].added, after[1].added), (after[0].total, 0));
        assert!(list(&db, "A", false).await.unwrap().len() as i64 == after[0].total);
        let marked: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM joy_activities WHERE couple_id = 'A' AND starter = 1").fetch_one(&db).await.unwrap();
        assert_eq!(marked, after[0].total);
        assert_eq!(add_pack(&db, "A", "nonsense").await.unwrap(), None);
    }

    #[tokio::test]
    async fn seed_copies_every_starter_slip() {
        let db = test_db().await;
        assert_eq!(seed_new_couple(&db, "A").await.unwrap(), 398);
        assert_eq!(list(&db, "A", false).await.unwrap().len(), 398);
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

    #[test]
    fn both_want_needs_both_slots_to_say_want() {
        let w = |id: i64, person: i64, l: &str| (id, person, l.to_string());
        let marks = vec![w(1, 0, "want"), w(1, 1, "want"), w(2, 0, "want"), w(2, 1, "fine"), w(3, 1, "want")];
        assert!(both_want(&marks, 1));
        assert!(!both_want(&marks, 2) && !both_want(&marks, 3) && !both_want(&marks, 4));
    }
}
