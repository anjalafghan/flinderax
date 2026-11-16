//! Starter packs: ready-made slips a couple can add.

#![allow(unused_imports)]
use super::*;
use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

/// A pack of starter slips a couple can add: "At home", "Long distance", "Together days", "Mumbai".
pub const PACKS: [(&str, &str, &str); 4] = [
    ("home", "At home", "Cooking, games and cosy plans for a night in."),
    ("apart", "Long distance", "Things to do together while you are in different places."),
    ("together", "Together days", "Evenings and whole days when you can finally meet, in and around Mumbai."),
    ("mumbai", "Mumbai", "Places and outings in and around Mumbai, with hours and prices."),
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct PackView {
    pub key: String,
    pub label: String,
    pub blurb: String,
    pub total: i64,
    /// How many of its slips this couple already has.
    pub added: i64,
}

pub async fn starter_packs(db: &SqlitePool, couple: &str) -> sqlx::Result<Vec<PackView>> {
    let mut out = vec![];
    for (key, label, blurb) in PACKS {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM joy_seed_activities WHERE pack = ?").bind(key).fetch_one(db).await?;
        let added: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM joy_activities a WHERE a.couple_id = ? AND a.starter = 1
               AND EXISTS (SELECT 1 FROM joy_seed_activities s WHERE s.pack = ? AND s.title = a.title)",
        )
        .bind(couple)
        .bind(key)
        .fetch_one(db)
        .await?;
        out.push(PackView { key: key.into(), label: label.into(), blurb: blurb.into(), total, added });
    }
    Ok(out)
}

/// Add a pack's slips that this couple does not have yet (matched by title). `None` for an unknown
/// pack. Returns how many were added.
pub async fn add_pack(db: &SqlitePool, couple: &str, pack: &str) -> sqlx::Result<Option<u64>> {
    let mut conn = db.acquire().await?;
    add_pack_in(&mut conn, couple, pack).await
}

pub async fn add_pack_in(conn: &mut sqlx::SqliteConnection, couple: &str, pack: &str) -> sqlx::Result<Option<u64>> {
    if !PACKS.iter().any(|(k, _, _)| *k == pack) {
        return Ok(None);
    }
    let done = sqlx::query(
        "INSERT INTO joy_activities (couple_id, title, notes, est_cost, setting, area, venue, lat, lng, duration, tags, energy,
                added_by, generated, closed_days, season_from, season_to, hours, best_time, minutes, booking,
                book_ahead_days, cover, bring, station, station_walk_min, starter)
         SELECT ?1, title, notes, est_cost, setting, area, venue, lat, lng, duration, tags, energy,
                added_by, generated, closed_days, season_from, season_to, hours, best_time, minutes, booking,
                book_ahead_days, cover, bring, station, station_walk_min, 1
           FROM joy_seed_activities s WHERE s.pack = ?2
            AND NOT EXISTS (SELECT 1 FROM joy_activities a WHERE a.couple_id = ?1 AND a.title = s.title)
          ORDER BY s.id",
    )
    .bind(couple)
    .bind(pack)
    .execute(&mut *conn)
    .await?;
    Ok(Some(done.rows_affected()))
}

/// Fill a new couple's jar: copies every starter slip in `joy_seed_activities` into their
/// `joy_activities`. Returns how many slips were added.
pub async fn seed_new_couple(db: &SqlitePool, couple: &str) -> sqlx::Result<u64> {
    let mut conn = db.acquire().await?;
    seed_new_couple_in(&mut conn, couple).await
}

/// The one copy of the starter-slip SQL, usable inside a larger transaction (creating a couple).
pub async fn seed_new_couple_in(conn: &mut sqlx::SqliteConnection, couple: &str) -> sqlx::Result<u64> {
    let mut added = 0;
    for (key, _, _) in PACKS {
        added += add_pack_in(conn, couple, key).await?.unwrap_or(0);
    }
    Ok(added)
}
