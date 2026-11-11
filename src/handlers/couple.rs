//! Couples: two Flinderax users who share the Dates section. One of you starts it (and gets the
//! starter jar), makes a single-use invite code, and the other joins with it. Only Dates data is
//! shared; accounts, cards and transactions stay per user.

use axum::{extract::State, http::StatusCode, Extension, Json};
use nanoid::nanoid;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    handlers::common::{bad_request, not_found, AppError},
    models::AppState,
};

type User = Extension<(String, String)>;

const INVITE_DAYS: i64 = 7;
/// No 0/O/1/I, so a code read out loud can't be misheard.
const CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// The couple a user belongs to and their slot in it (0 = started it, 1 = joined).
pub async fn couple_of(db: &sqlx::SqlitePool, user_id: &str) -> Result<Option<(String, usize)>, AppError> {
    let row = sqlx::query!(
        r#"SELECT couple_id AS "couple_id!", COALESCE(user_0 = ?, 0) AS "first!: bool" FROM couples WHERE user_0 = ? OR user_1 = ?"#,
        user_id,
        user_id,
        user_id
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|r| (r.couple_id, if r.first { 0 } else { 1 })))
}

/// Like [`couple_of`], but a 403 for a user who isn't in a couple yet.
pub async fn require_couple(db: &sqlx::SqlitePool, user_id: &str) -> Result<(String, usize), AppError> {
    couple_of(db, user_id)
        .await?
        .ok_or_else(|| AppError(StatusCode::FORBIDDEN, "Start Dates or join your partner first".into()))
}

fn hash_code(code: &str) -> String {
    let clean: String = code.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_uppercase();
    Sha256::digest(clean.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

fn new_code() -> String {
    let mut rng = rand::thread_rng();
    let raw: String = (0..8).map(|_| CODE_CHARS[rng.gen_range(0..CODE_CHARS.len())] as char).collect();
    format!("{}-{}", &raw[..4], &raw[4..])
}

#[derive(Serialize)]
pub struct CoupleStatus {
    pub in_couple: bool,
    /// Your slot: 0 = started it, 1 = joined.
    pub me: Option<usize>,
    pub partner_name: Option<String>,
    /// Waiting for the partner to join.
    pub waiting: bool,
    /// The couple's money symbol ("₹" unless they set another).
    pub currency: String,
    /// This user has put any money data into the app (an account, card, income, fixed payment or
    /// transaction). False for a partner who only came for Dates: they get a Dates home screen.
    pub uses_money: bool,
}

/// Whether the user has any money data at all.
async fn uses_money(db: &sqlx::SqlitePool, user_id: &str) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM accounts WHERE user_id = ?1)
             OR EXISTS (SELECT 1 FROM cards WHERE user_id = ?1)
             OR EXISTS (SELECT 1 FROM income_sources WHERE user_id = ?1)
             OR EXISTS (SELECT 1 FROM commitments WHERE user_id = ?1)
             OR EXISTS (SELECT 1 FROM transactions WHERE user_id = ?1)",
    )
    .bind(user_id)
    .fetch_one(db)
    .await?)
}

pub async fn status(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<CoupleStatus>, AppError> {
    let Some((couple, me)) = couple_of(&state.db, &user_id).await? else {
        return Ok(Json(CoupleStatus { in_couple: false, me: None, partner_name: None, waiting: false, currency: "₹".into(), uses_money: false }));
    };
    let me_slot = me as i64;
    let partner = sqlx::query_scalar!(
        r#"SELECT u.user_name FROM couples c JOIN users u ON u.user_id = CASE WHEN ? = 0 THEN c.user_1 ELSE c.user_0 END
           WHERE c.couple_id = ?"#,
        me_slot,
        couple
    )
    .fetch_optional(&state.db)
    .await?;
    let currency: Option<String> = sqlx::query_scalar("SELECT value FROM joy_settings WHERE couple_id = ? AND key = 'CURRENCY_SYMBOL'")
        .bind(&couple)
        .fetch_optional(&state.db)
        .await?;
    let currency = currency.map(|c| c.trim().to_string()).filter(|c| !c.is_empty()).unwrap_or_else(|| "₹".into());
    Ok(Json(CoupleStatus { in_couple: true, me: Some(me), waiting: partner.is_none(), partner_name: partner, currency, uses_money: uses_money(&state.db, &user_id).await? }))
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

/// Starts Dates for this user: a new couple with the starter jar of slips.
pub async fn create(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Done>, AppError> {
    if couple_of(&state.db, &user_id).await?.is_some() {
        return Err(bad_request("You already have Dates set up"));
    }
    let couple = nanoid!();
    let name = sqlx::query_scalar!("SELECT user_name FROM users WHERE user_id = ?", user_id).fetch_one(&state.db).await?;
    let mut tx = state.db.begin().await?;
    sqlx::query!("INSERT INTO couples (couple_id, user_0) VALUES (?, ?)", couple, user_id).execute(&mut *tx).await?;
    sqlx::query!("INSERT INTO joy_settings (couple_id, key, value) VALUES (?, 'PERSON_A', ?)", couple, name)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO joy_activities (couple_id, title, notes, est_cost, setting, area, venue, lat, lng, duration, tags, energy,
                added_by, generated, closed_days, season_from, season_to, hours, best_time, minutes, booking, book_ahead_days,
                cover, bring, station, station_walk_min)
         SELECT ?, title, notes, est_cost, setting, area, venue, lat, lng, duration, tags, energy,
                added_by, generated, closed_days, season_from, season_to, hours, best_time, minutes, booking, book_ahead_days,
                cover, bring, station, station_walk_min
         FROM joy_seed_activities ORDER BY id",
        couple
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(Done { status: true }))
}

#[derive(Serialize)]
pub struct Invite {
    /// Shown once, e.g. "K7PQ-2XMB".
    pub code: String,
    pub expires_in_days: i64,
}

/// A new single-use code for the partner (older unused codes stop working).
pub async fn invite(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Invite>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    let full = sqlx::query_scalar!(
        r#"SELECT (user_0 IS NOT NULL AND user_1 IS NOT NULL) AS "full!: bool" FROM couples WHERE couple_id = ?"#,
        couple
    )
        .fetch_one(&state.db)
        .await?;
    if full {
        return Err(bad_request("Your partner has already joined"));
    }
    let code = new_code();
    let hash = hash_code(&code);
    let expires = format!("+{INVITE_DAYS} days");
    let mut tx = state.db.begin().await?;
    sqlx::query!("DELETE FROM couple_invites WHERE couple_id = ? AND used_at IS NULL", couple).execute(&mut *tx).await?;
    sqlx::query!(
        "INSERT INTO couple_invites (code_hash, couple_id, expires_at) VALUES (?, ?, datetime('now', ?))",
        hash,
        couple,
        expires
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(Invite { code, expires_in_days: INVITE_DAYS }))
}

#[derive(Deserialize)]
pub struct JoinPayload {
    pub code: String,
}

pub async fn join(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<JoinPayload>,
) -> Result<Json<Done>, AppError> {
    if couple_of(&state.db, &user_id).await?.is_some() {
        return Err(bad_request("You're already in Dates. Leave it first to join someone else"));
    }
    // Guessing codes is slow: a few tries a minute per account.
    if !crate::ratelimit::allow(&format!("join-user:{user_id}"), 5) {
        return Err(AppError(StatusCode::TOO_MANY_REQUESTS, "Too many tries. Wait a minute and check the code.".into()));
    }
    let hash = hash_code(&p.code);
    let mut tx = state.db.begin().await?;
    let couple = sqlx::query_scalar!(
        r#"SELECT couple_id AS "couple_id!" FROM couple_invites
           WHERE code_hash = ? AND used_at IS NULL AND expires_at > datetime('now')"#,
        hash
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| bad_request("That code didn't work. Ask for a new one"))?;
    // The empty slot: 1 normally, 0 if the one who started it has left.
    let slot = sqlx::query_scalar!(
        r#"SELECT CASE WHEN user_1 IS NULL THEN 1 WHEN user_0 IS NULL THEN 0 END AS "slot?: i64" FROM couples WHERE couple_id = ?"#,
        couple
    )
    .fetch_one(&mut *tx)
    .await?
    .ok_or_else(|| bad_request("That Dates already has two people"))?;
    // Surprises left behind in this slot (from before this change, or from a leaver) stay sealed.
    crate::joy::db::orphan_surprises(&mut tx, &couple, slot).await?;
    if slot == 1 {
        sqlx::query!("UPDATE couples SET user_1 = ? WHERE couple_id = ?", user_id, couple).execute(&mut *tx).await?;
    } else {
        sqlx::query!("UPDATE couples SET user_0 = ? WHERE couple_id = ?", user_id, couple).execute(&mut *tx).await?;
    }
    sqlx::query!("UPDATE couple_invites SET used_at = CURRENT_TIMESTAMP WHERE code_hash = ?", hash).execute(&mut *tx).await?;
    let name = sqlx::query_scalar!("SELECT user_name FROM users WHERE user_id = ?", user_id).fetch_one(&mut *tx).await?;
    let key = if slot == 0 { "PERSON_A" } else { "PERSON_B" };
    sqlx::query!(
        "INSERT INTO joy_settings (couple_id, key, value) VALUES (?, ?, ?)
         ON CONFLICT(couple_id, key) DO UPDATE SET value = excluded.value",
        couple,
        key,
        name
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(Done { status: true }))
}

/// Takes `user_id` out of their couple, inside `tx`: the slot empties and the shared history stays
/// with the partner. When nobody is left the couple row goes, and with it everything in Dates.
/// Returns `None` for someone who is not in a couple, otherwise `Some(emptied)` where `emptied` is
/// the couple's id when it was deleted, so the caller can remove its photo folder after the commit.
pub async fn leave_in_tx(tx: &mut sqlx::SqliteConnection, user_id: &str) -> Result<Option<Option<String>>, AppError> {
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT couple_id, user_0 FROM couples WHERE user_0 = ? OR user_1 = ?")
            .bind(user_id)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some((couple, user_0)) = row else { return Ok(None) };
    let me = if user_0.as_deref() == Some(user_id) { 0 } else { 1 };
    let slot = if me == 0 { "user_0" } else { "user_1" };
    // What the leaver was still planning in secret stays sealed from everyone, so whoever takes
    // the empty slot next can't become its planner.
    crate::joy::db::orphan_surprises(&mut *tx, &couple, me).await?;
    sqlx::query(&format!("UPDATE couples SET {slot} = NULL WHERE couple_id = ?"))
        .bind(&couple)
        .execute(&mut *tx)
        .await?;
    let gone = sqlx::query("DELETE FROM couples WHERE couple_id = ? AND user_0 IS NULL AND user_1 IS NULL")
        .bind(&couple)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    Ok(Some((gone > 0).then_some(couple)))
}

/// Deletes a couple's photo folder (`<photos root>/<couple id>`). A folder that is already gone is fine.
pub async fn remove_couple_photos(couple: &str) {
    // Couple ids are nanoids; anything else must never become part of a path.
    if couple.is_empty() || !couple.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return;
    }
    let dir = crate::import_joy::photos_root().join(couple);
    if let Err(e) = tokio::fs::remove_dir_all(&dir).await
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::error!("could not remove photos of a deleted couple: {e}");
    }
}

/// Leaves the couple: your slot empties and the shared history stays with your partner. When
/// nobody is left, Dates is deleted with everything in it, photos included.
pub async fn leave(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Done>, AppError> {
    let mut tx = state.db.begin().await?;
    let emptied = leave_in_tx(&mut tx, &user_id).await?.ok_or_else(|| not_found("You're not in Dates"))?;
    tx.commit().await?;
    if let Some(couple) = emptied {
        remove_couple_photos(&couple).await;
    }
    Ok(Json(Done { status: true }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_readable_and_hash_the_same_however_typed() {
        let c = new_code();
        assert_eq!(c.len(), 9);
        assert!(c.chars().all(|ch| ch == '-' || CODE_CHARS.contains(&(ch as u8))));
        assert_eq!(hash_code(&c), hash_code(&c.to_lowercase().replace('-', " ")));
    }
}
