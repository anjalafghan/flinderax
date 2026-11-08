//! Regulars: a standing date ("Blue Tokai, Tue and Thu"). The jar page asks one question on those
//! evenings (went / somewhere else / not tonight).
//!
//! Part 1 ported what the jar page shows (`Card`, `card`); part 3 adds the regulars' own pages and
//! the answering handlers from Joy's `routes/regular.rs`. A regular or night that isn't ours is a
//! 404 everywhere. Photo links are signed over `/api/dates/nights/{id}/photo?v=..` but served from
//! the public router (an `<img>` can't send a bearer token), so the link handed out starts with
//! `/api/public/dates/` (see [`public_link`]).

use std::time::UNIX_EPOCH;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{flex, go, internal, joy, month, year, AppError, Go, Me};
use crate::{
    handlers::common::not_found,
    joy::{ctx::Ctx, db, photos},
    models::AppState,
};

type User = Extension<(String, String)>;

/// How long a night's photo link stays good.
const PHOTO_TTL_SECS: i64 = 7 * 24 * 3600;

/// A regular night nobody has answered yet.
#[derive(Debug, Serialize)]
pub struct OpenNight {
    pub regular_id: i64,
    pub title: String,
    pub maps_url: String,
    /// "₹300"
    pub usual: String,
    /// The usual amount as digits, for the form field.
    pub usual_raw: i64,
    /// "2026-11-05"
    pub day: String,
    /// "Thu 5 Nov"
    pub label: String,
    pub today: bool,
}

/// A night we went tonight: shown as done, with a spot for a photo.
#[derive(Debug, Serialize)]
pub struct WentTonight {
    pub visit_id: i64,
    pub title: String,
    pub place: String,
    pub amount: String,
    /// Signed photo link, or empty.
    pub photo: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Card {
    pub open: Vec<OpenNight>,
    pub went: Vec<WentTonight>,
    /// "up to ₹1,200 more", or empty.
    pub ceiling: String,
    /// There is something to show (Joy's `Card::show()`).
    pub show: bool,
}

impl Card {
    pub fn show(&self) -> bool {
        !self.open.is_empty() || !self.went.is_empty()
    }
}

pub async fn card(db: &SqlitePool, couple: &str, ctx: &Ctx) -> Result<Card, AppError> {
    let (today, ..) = db::local_today(db).await?;
    let open = db::open_nights(db, couple)
        .await?
        .into_iter()
        .map(|(r, day, label)| OpenNight {
            regular_id: r.id,
            title: r.title,
            maps_url: r.maps_url,
            usual: ctx.money(r.usual_cost),
            usual_raw: r.usual_cost,
            today: day == today,
            day,
            label,
        })
        .collect();
    let went = db::went_today(db, couple)
        .await?
        .into_iter()
        .map(|(r, v)| WentTonight {
            visit_id: v.id,
            title: r.title.clone(),
            place: if v.place.is_empty() { String::new() } else { v.place.clone() },
            amount: ctx.money(v.amount),
            photo: if v.has_photo { visit_photo_url(ctx, v.id) } else { String::new() },
        })
        .collect();
    let ceiling = db::regulars_ceiling(db, couple).await?;
    let mut card = Card { open, went, ceiling: if ceiling > 0 { ctx.money(ceiling) } else { String::new() }, show: false };
    card.show = card.show();
    Ok(card)
}

/// Signed link to a night's photo (the file's mtime busts the cache): verify
/// `/api/dates/nights/{id}/photo?v={v}` with `exp` and `sig`.
pub fn visit_photo_url(ctx: &Ctx, id: i64) -> String {
    let v = mtime_secs(&photos::visit_path(&ctx.photos_dir, id));
    public_link(super::sign(&format!("/api/dates/nights/{id}/photo?v={v}"), PHOTO_TTL_SECS))
}

/// The file's modified time in seconds (0 when it isn't there): the cache-busting `v`.
pub fn mtime_secs(path: &std::path::Path) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs())
}

/// A signed `/api/dates/...` link as the browser should request it: the public router serves
/// anything that can't carry a bearer token under `/api/public/dates/...`. The signature still
/// covers the `/api/dates/...` path, which is what the handler verifies.
pub fn public_link(signed: String) -> String {
    match signed.strip_prefix("/api/dates/") {
        Some(rest) => format!("/api/public/dates/{rest}"),
        None => signed,
    }
}

/// `GET /regulars`, `POST /regulars`, `/regulars/{id}`, answering and photos, plus the month and
/// year pages. Behind login; mounted under `/api/dates`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/regulars", get(index).post(create))
        .route("/regulars/{id}", get(spread))
        .route("/regulars/{id}/went", post(went))
        .route("/regulars/{id}/skip", post(skip))
        .route("/nights/{id}/delete", post(delete_night))
        .route("/nights/{id}/photo", post(upload_photo).layer(DefaultBodyLimit::max(photos::MAX_UPLOAD_BYTES + 64 * 1024)))
        .route("/nights/{id}/photo/delete", post(delete_photo))
        .route("/year", get(year::current))
        .route("/year/{year}", get(year::page))
        .route("/year/{year}/{month}", get(month::page))
}

/// No login, the signature is the proof: mounted under `/api/public/dates`.
pub fn public_router() -> Router<AppState> {
    Router::new().route("/nights/{id}/photo", get(photo))
}

fn refuse(msg: impl Into<String>) -> Response {
    (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": msg.into() }))).into_response()
}

/// Where to go after answering: the caller's own `back` if it is a path on this site.
fn back_to(back: &str, otherwise: &str) -> Json<Go> {
    go(if back.starts_with('/') && !back.starts_with("//") { back } else { otherwise })
}

// ── answering ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WentBody {
    #[serde(default, deserialize_with = "flex")]
    day: String,
    /// Blank = the usual amount.
    #[serde(default, deserialize_with = "flex")]
    amount: String,
    /// Blank = the regular's own place.
    #[serde(default, deserialize_with = "flex")]
    place: String,
    #[serde(default, deserialize_with = "flex")]
    back: String,
}

/// `POST /regulars/{id}/went`
pub async fn went(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<i64>,
    Json(f): Json<WentBody>,
) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let r = db::regular(&s.db, &j.couple, id).await?.ok_or_else(|| not_found("No such regular"))?;
    let typed = f.amount.trim().replace(',', "");
    let amount = if typed.is_empty() { Some(r.usual_cost) } else { typed.parse::<i64>().ok().filter(|a| *a >= 0) };
    if let Some(amount) = amount {
        let place = f.place.trim().chars().take(80).collect::<String>();
        // Typing the regular's own name as "somewhere else" is just the regular.
        let place = if place.eq_ignore_ascii_case(&r.title) || place.eq_ignore_ascii_case(&r.place) { String::new() } else { place };
        db::record_visit(&s.db, &j.couple, id, &f.day, "went", &place, amount).await?;
    }
    Ok(back_to(&f.back, "/joy"))
}

#[derive(Debug, Deserialize)]
pub struct SkipBody {
    #[serde(default, deserialize_with = "flex")]
    day: String,
    #[serde(default, deserialize_with = "flex")]
    back: String,
}

/// `POST /regulars/{id}/skip`
pub async fn skip(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<i64>,
    Json(f): Json<SkipBody>,
) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    db::regular(&s.db, &j.couple, id).await?.ok_or_else(|| not_found("No such regular"))?;
    db::record_visit(&s.db, &j.couple, id, &f.day, "skipped", "", 0).await?;
    Ok(back_to(&f.back, "/joy"))
}

#[derive(Debug, Default, Deserialize)]
pub struct BackBody {
    #[serde(default, deserialize_with = "flex")]
    back: String,
}

/// `POST /nights/{id}/delete`: the night and its photo.
pub async fn delete_night(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(vid): Path<i64>,
    body: Option<Json<BackBody>>,
) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let v = db::visit(&s.db, &j.couple, vid).await?.ok_or_else(|| not_found("No such night"))?;
    let _ = tokio::fs::remove_file(photos::visit_path(&j.ctx.photos_dir, vid)).await;
    db::delete_visit(&s.db, &j.couple, vid).await?;
    Ok(back_to(&body.unwrap_or_default().0.back, &spread_path(v.regular_id)))
}

fn spread_path(regular_id: i64) -> String {
    format!("/joy/regulars/{regular_id}")
}

// ── photos ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PhotoSaved {
    pub ok: bool,
    /// Signed link to the new photo.
    pub photo: String,
}

/// `POST /nights/{id}/photo`: multipart field `photo`, same rules as a kept date's photo. A file
/// that can't be used is a 422 `{error}` with the reason.
pub async fn upload_photo(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(vid): Path<i64>,
    mut form: Multipart,
) -> Result<Response, AppError> {
    let j = joy(&s, &user_id).await?;
    if db::visit(&s.db, &j.couple, vid).await?.filter(|v| v.status == "went").is_none() {
        return Err(not_found("No such night"));
    }
    let mut bytes = None;
    while let Ok(Some(field)) = form.next_field().await {
        if field.name() == Some("photo") {
            match field.bytes().await {
                Ok(b) => bytes = Some(b),
                Err(_) => return Ok(refuse("That photo is over 15 MB. Try a smaller one.")),
            }
        }
    }
    let Some(bytes) = bytes.filter(|b| !b.is_empty()) else {
        return Ok(refuse("That file doesn't look like a photo."));
    };
    let jpeg = match tokio::task::spawn_blocking(move || photos::process(&bytes)).await.map_err(internal)? {
        Ok(jpeg) => jpeg,
        Err(e) => return Ok(refuse(e.0)),
    };
    tokio::fs::create_dir_all(&j.ctx.photos_dir).await.map_err(internal)?;
    tokio::fs::write(photos::visit_path(&j.ctx.photos_dir, vid), jpeg).await.map_err(internal)?;
    db::set_visit_photo(&s.db, &j.couple, vid, true).await?;
    Ok(Json(PhotoSaved { ok: true, photo: visit_photo_url(&j.ctx, vid) }).into_response())
}

/// `POST /nights/{id}/photo/delete`
pub async fn delete_photo(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(vid): Path<i64>,
    body: Option<Json<BackBody>>,
) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let v = db::visit(&s.db, &j.couple, vid).await?.ok_or_else(|| not_found("No such night"))?;
    let _ = tokio::fs::remove_file(photos::visit_path(&j.ctx.photos_dir, vid)).await;
    db::set_visit_photo(&s.db, &j.couple, vid, false).await?;
    Ok(back_to(&body.unwrap_or_default().0.back, &spread_path(v.regular_id)))
}

#[derive(Debug, Deserialize)]
pub struct PhotoQuery {
    v: Option<String>,
    exp: Option<i64>,
    sig: Option<String>,
}

/// `GET /nights/{id}/photo?v=..&exp=..&sig=..` (public): the signature is the login. Anything
/// that doesn't verify is a 403; a night without a photo is a 404.
pub async fn photo(State(s): State<AppState>, Path(vid): Path<i64>, Query(q): Query<PhotoQuery>) -> Result<Response, AppError> {
    let (Some(v), Some(exp), Some(sig)) = (q.v, q.exp, q.sig) else {
        return Err(AppError(StatusCode::FORBIDDEN, "Link not valid".into()));
    };
    if !super::verify(&format!("/api/dates/nights/{vid}/photo?v={v}"), exp, &sig) {
        return Err(AppError(StatusCode::FORBIDDEN, "Link not valid or expired".into()));
    }
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT r.couple_id FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id WHERE v.id = ? AND v.has_photo = 1",
    )
    .bind(vid)
    .fetch_optional(&s.db)
    .await?;
    let Some((couple,)) = row else { return Err(not_found("No photo")) };
    let path = photos::visit_path(&crate::import_joy::photos_root().join(&couple), vid);
    let Ok(bytes) = tokio::fs::read(&path).await else { return Err(not_found("No photo")) };
    // The link carries the file's modified time, so a matching ?v= can be cached for good.
    let fresh = v == mtime_secs(&path).to_string();
    let cache = if fresh { "private, max-age=31536000, immutable" } else { "private, no-cache" };
    Ok(([(header::CONTENT_TYPE, "image/jpeg"), (header::CACHE_CONTROL, cache)], bytes).into_response())
}

// ── the regular's spread ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Stamp {
    pub id: i64,
    /// "3 Nov"
    pub day: String,
    pub year: String,
    /// Where we went when it wasn't the usual place.
    pub elsewhere: String,
    pub amount: String,
    /// Signed photo link, or empty.
    pub photo: String,
    /// -2..=2, so each print sits at its own angle.
    pub tilt: i64,
}

/// `GET /regulars/{id}`
#[derive(Debug, Serialize)]
pub struct SpreadView {
    pub me_name: String,
    pub regular_id: i64,
    pub title: String,
    pub place: String,
    pub maps_url: String,
    /// "Tuesdays and Thursdays"
    pub days: String,
    pub count: usize,
    pub elsewhere: usize,
    pub total: String,
    pub average: String,
    pub stamps: Vec<Stamp>,
}

/// "2,4" → "Tuesdays and Thursdays"
pub fn weekdays_label(weekdays: &str) -> String {
    const NAMES: [&str; 7] = ["Sundays", "Mondays", "Tuesdays", "Wednesdays", "Thursdays", "Fridays", "Saturdays"];
    let names: Vec<&str> = weekdays.split(',').filter_map(|d| d.trim().parse::<usize>().ok()).filter_map(|d| NAMES.get(d).copied()).collect();
    match names.as_slice() {
        [] => String::new(),
        [one] => (*one).into(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

pub async fn spread(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<SpreadView>, AppError> {
    let j = joy(&s, &user_id).await?;
    let ctx = &j.ctx;
    let r = db::regular(&s.db, &j.couple, id).await?.ok_or_else(|| not_found("No such regular"))?;
    let visits = db::went_visits(&s.db, &j.couple, id).await?;
    let total: i64 = visits.iter().map(|v| v.amount).sum();
    let count = visits.len();
    let stamps = visits
        .iter()
        .enumerate()
        .map(|(i, v)| Stamp {
            id: v.id,
            day: month::day_label(&v.day),
            year: v.day.get(..4).unwrap_or("").to_string(),
            elsewhere: v.place.clone(),
            amount: ctx.money(v.amount),
            photo: if v.has_photo { visit_photo_url(ctx, v.id) } else { String::new() },
            tilt: [-2, 1, -1, 2, 0][i % 5],
        })
        .collect();
    Ok(Json(SpreadView {
        me_name: Me::new(ctx, Some(j.who)).name,
        regular_id: id,
        title: r.title,
        place: r.place,
        maps_url: r.maps_url,
        days: weekdays_label(&r.weekdays),
        count,
        elsewhere: visits.iter().filter(|v| !v.place.is_empty()).count(),
        total: ctx.money(total),
        average: if count > 0 { ctx.money(total / count as i64) } else { String::new() },
        stamps,
    }))
}

// ── the list, and adding one ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ListItem {
    pub id: i64,
    pub title: String,
    pub place: String,
    pub days: String,
    pub usual: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct DayBox {
    pub value: usize,
    pub label: &'static str,
    pub checked: bool,
}

/// `GET /regulars`: every regular, and the day boxes for the add form.
#[derive(Debug, Serialize)]
pub struct RegularsView {
    pub me_name: String,
    pub items: Vec<ListItem>,
    pub days: Vec<DayBox>,
}

pub async fn index(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<RegularsView>, AppError> {
    let j = joy(&s, &user_id).await?;
    let mut items = vec![];
    for r in db::regulars(&s.db, &j.couple).await? {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM joy_regular_visits WHERE regular_id = ? AND status = 'went'")
            .bind(r.id)
            .fetch_one(&s.db)
            .await?;
        items.push(ListItem { id: r.id, title: r.title, place: r.place, days: weekdays_label(&r.weekdays), usual: j.ctx.money(r.usual_cost), count });
    }
    const LABELS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let days = LABELS.iter().enumerate().map(|(i, l)| DayBox { value: i, label: l, checked: i == 2 || i == 4 }).collect();
    Ok(Json(RegularsView { me_name: Me::new(&j.ctx, Some(j.who)).name, items, days }))
}

#[derive(Debug, Deserialize)]
pub struct NewRegular {
    #[serde(default, deserialize_with = "flex")]
    title: String,
    #[serde(default, deserialize_with = "flex")]
    place: String,
    #[serde(default, deserialize_with = "flex")]
    maps_url: String,
    /// Weekday numbers 0 (Sunday) to 6.
    #[serde(default, deserialize_with = "flex_days")]
    days: Vec<String>,
    #[serde(default, deserialize_with = "flex")]
    cost: String,
    /// "YYYY-MM-DD"; blank = today.
    #[serde(default, deserialize_with = "flex")]
    since: String,
}

/// An array of numbers/strings, or one number/string.
fn flex_days<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(super::flex_list(d)?.split(',').map(str::to_string).collect())
}

/// The weekdays as "2,4": only 0-6, each once, in order.
fn clean_weekdays(days: &[String]) -> String {
    let mut d: Vec<u8> = days.iter().filter_map(|d| d.trim().parse().ok()).filter(|d| *d <= 6).collect();
    d.sort_unstable();
    d.dedup();
    d.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

/// `POST /regulars` → `Go` to the new regular's spread; a missing name or day is a 422 `{error}`.
pub async fn create(State(s): State<AppState>, Extension((user_id, _)): User, Json(f): Json<NewRegular>) -> Result<Response, AppError> {
    let j = joy(&s, &user_id).await?;
    let title: String = f.title.trim().chars().take(60).collect();
    if title.is_empty() {
        return Ok(refuse("Give it a name."));
    }
    let weekdays = clean_weekdays(&f.days);
    if weekdays.is_empty() {
        return Ok(refuse("Pick at least one day."));
    }
    // The link is shown as an href, so only a web address is kept.
    let maps = f.maps_url.trim();
    let maps = if maps.starts_with("https://") || maps.starts_with("http://") { maps } else { "" };
    let cost = f.cost.trim().replace(',', "").parse::<i64>().ok().filter(|c| *c >= 0).unwrap_or(0);
    let place: String = f.place.trim().chars().take(120).collect();
    let id = db::add_regular(&s.db, &j.couple, &title, &place, maps, &weekdays, cost, &f.since).await?;
    Ok(go(spread_path(id)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_days() {
        assert_eq!(weekdays_label("2,4"), "Tuesdays and Thursdays");
        assert_eq!(weekdays_label("5"), "Fridays");
        assert_eq!(weekdays_label("1,3,5"), "Mondays, Wednesdays and Fridays");
        assert_eq!(weekdays_label(""), "");
    }

    #[test]
    fn keeps_only_real_weekdays_once_each() {
        let d = |v: &[&str]| clean_weekdays(&v.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(d(&["4", "2", "4", "9", "x"]), "2,4");
        assert_eq!(d(&[]), "");
    }

    #[test]
    fn photo_links_are_served_from_the_public_router() {
        assert_eq!(public_link("/api/dates/nights/3/photo?v=1&exp=2&sig=x".into()), "/api/public/dates/nights/3/photo?v=1&exp=2&sig=x");
        assert_eq!(public_link("/elsewhere".into()), "/elsewhere");
    }

    #[test]
    fn back_stays_on_this_site() {
        assert_eq!(back_to("/joy/year", "/joy").0.to, "/joy/year");
        assert_eq!(back_to("//evil.example", "/joy").0.to, "/joy");
        assert_eq!(back_to("https://evil.example", "/joy").0.to, "/joy");
    }
}
