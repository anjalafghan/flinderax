//! Slips: the wishlist page, the add/edit form, archive/restore/delete, "Not for me" undone, and
//! planning a slip straight to a day. A slip that isn't ours is a 404 everywhere.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{
    dayplan, duration_opts, envelope, flex, go, joy, learned, parse_local, planned_label, ActivityView, AppError, Go, Joy,
    Me, Opt,
};
use crate::{
    handlers::common::not_found,
    joy::{
        ctx::{parse_latlng, Ctx},
        db::{self, Activity, ActivityInput},
        hours::Hours,
        picker::{self, Duration, ENERGIES, SETTINGS},
    },
    models::AppState,
};

type User = Extension<(String, String)>;

/// Where Joy's `/wishlist` went.
const SLIPS: &str = "/joy/slips";

#[derive(Debug, Serialize)]
pub struct Group {
    pub key: String,
    pub label: String,
    pub items: Vec<ActivityView>,
}

/// `GET /slips`
#[derive(Debug, Serialize)]
pub struct WishlistView {
    pub me_name: String,
    pub groups: Vec<Group>,
    pub archived: Vec<ActivityView>,
    pub total: usize,
}

pub async fn page(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<WishlistView>, AppError> {
    let j = joy(&s, &user_id).await?;
    let (db, couple, ctx) = (&s.db, j.couple.as_str(), &j.ctx);
    // A slip with a surprise still sealed from us is not ours to see yet.
    let sealed = db::sealed_from(db, couple, j.who).await?;
    let mut active = db::list(db, couple, false).await?;
    active.retain(|a| !sealed.contains(&a.id));
    let pairs = db::pairs(db, couple).await?;
    let vetoes = db::vetoes(db, couple).await?;
    let learned = learned(db, couple).await?;
    let wants = db::wants(db, couple).await?;
    let view = |a: &Activity| {
        let mut v = ActivityView::new(ctx, a, &learned);
        v.my_want = wants.iter().find(|(id, p, _)| *id == a.id && *p == j.who as i64).map(|(_, _, l)| l.clone()).unwrap_or_default();
        v.both_want = (0..2).all(|p| wants.iter().any(|(id, person, l)| *id == a.id && *person == p && l == "want"));
        v.vetoed_by = vetoes
            .iter()
            .filter(|(act, _)| *act == a.id)
            .filter_map(|(_, p)| ctx.people.get(*p as usize).map(|x| x.0.clone()))
            .collect::<Vec<_>>()
            .join(" & ");
        v.pairs = pairs
            .iter()
            .filter_map(|&(x, y)| if x == a.id { Some(y) } else if y == a.id { Some(x) } else { None })
            .filter_map(|other| active.iter().find(|o| o.id == other))
            .map(|o| ctx.personalize(&o.title))
            .collect();
        v
    };
    let groups = SETTINGS
        .iter()
        .map(|(key, label)| Group {
            key: (*key).into(),
            label: (*label).into(),
            items: active.iter().filter(|a| a.setting == *key).map(view).collect(),
        })
        .filter(|g| !g.items.is_empty())
        .collect();
    let archived = db::list(db, couple, true)
        .await?
        .iter()
        .filter(|a| !sealed.contains(&a.id))
        .map(|a| ActivityView::new(ctx, a, &learned))
        .collect();
    Ok(Json(WishlistView { me_name: me_name(&j), groups, archived, total: active.len() }))
}

// ── plan it straight from a slip ───────────────────────────────────────────

/// What `POST /slips/{id}/plan` answers with.
#[derive(Debug, Serialize)]
pub struct SlipPlanned {
    pub title: String,
    /// "Mon 2 Nov, 7:00 pm".
    pub label: String,
    /// Signed link to the calendar invite.
    pub ics_url: String,
    /// The other one of us ("Diya").
    pub partner: String,
    pub wa_url: String,
    pub share_text: String,
    pub history_id: i64,
    /// Where Joy's non-htmx redirect went: the kept date in the Kept page.
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub struct PlanForm {
    /// From `<input type="datetime-local">`: "2026-11-02T19:00".
    #[serde(default, deserialize_with = "flex")]
    pub at: String,
}

/// 404 when `id` is a sealed surprise the other partner drew for this viewer, so it can't be opened,
/// changed or even confirmed to exist (a 403 would give the surprise away).
async fn unsealed(db: &SqlitePool, j: &Joy, id: i64) -> Result<(), AppError> {
    super::unsealed(db, &j.couple, j.who, id).await
}

/// A slip of ours, or a 404.
async fn mine(db: &SqlitePool, couple: &str, id: i64) -> Result<Activity, AppError> {
    db::get(db, couple, id).await?.ok_or_else(|| not_found("No such slip"))
}

/// Keep a slip as a date at a time we choose, without drawing it from the jar.
pub async fn plan(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<i64>,
    body: Option<Json<PlanForm>>,
) -> Result<Json<SlipPlanned>, AppError> {
    let f = body.map(|j| j.0).unwrap_or(PlanForm { at: String::new() });
    let j = joy(&s, &user_id).await?;
    let (db, couple, ctx) = (&s.db, j.couple.as_str(), &j.ctx);
    // Only a slip that could be drawn can be planned: not archived, passed on or sealed.
    let Some(a) = db::drawable_get(db, couple, j.who, id).await? else { return Err(not_found("No such slip")) };
    let Some(local) = parse_local(&f.at) else {
        return Err(AppError(StatusCode::UNPROCESSABLE_ENTITY, "Pick a day and a time.".into()));
    };
    let who = Some(j.who);
    let hid = db::accept(db, couple, id, who, false).await?;
    db::set_planned(db, couple, &[hid], Some(&local)).await?;
    let row = db::history_row(db, couple, hid).await?;
    let label = row.and_then(|r| r.planned_local).as_deref().map(planned_label).unwrap_or_default();
    let view = ActivityView::new(ctx, &a, &learned(db, couple).await?);
    let share = envelope::wa_text(ctx, &db::date_token(db, couple, hid).await?, &format!("It's a date, {label}! Open the envelope:"));
    Ok(Json(SlipPlanned {
        title: view.title,
        label,
        ics_url: dayplan::ics_url(hid, who),
        partner: Me::new(ctx, who).partner,
        wa_url: envelope::wa_url(&share),
        share_text: share,
        history_id: hid,
        to: format!("/joy/kept#k{hid}"),
    }))
}

// ── the add/edit form ──────────────────────────────────────────────────────

/// The add/edit form (Joy's `activity_form.html`): `GET /slips/new`, `GET /slips/{id}`, and what
/// a refused save answers with (status 422, `error` set).
#[derive(Debug, Serialize)]
pub struct FormView {
    pub id: Option<i64>,
    pub title: String,
    pub notes: String,
    pub est_cost: String,
    pub venue: String,
    pub area: String,
    pub location: String,
    pub tags: String,
    pub currency: String,
    /// "Rohit & Neha", signed on the slip.
    pub couple: String,
    pub error: String,
    pub settings: Vec<Opt>,
    pub durations: Vec<Opt>,
    pub energies: Vec<Opt>,
    pub people: Vec<Opt>,
    /// Every other slip in the jar, ticked if it pairs with this one.
    pub pair_opts: Vec<Opt>,
    pub me_name: String,
    /// Weekdays (value "mon"…, label "Mon"…), ticked if it's closed then.
    pub closed_opts: Vec<Opt>,
    /// Months for the season pickers; `selected` marks the current from/to.
    pub season_from_opts: Vec<Opt>,
    pub season_to_opts: Vec<Opt>,
    pub hours: String,
    /// "Open 12:00 pm – 11:30 pm" style preview of `hours` for today, or empty.
    pub hours_today: String,
    pub best_times: Vec<Opt>,
    pub minutes: String,
    pub bookings: Vec<Opt>,
    pub book_ahead_days: String,
    pub covers: Vec<Opt>,
    pub cover_unknown: bool,
    pub bring: String,
    pub station: String,
    pub station_walk_min: String,
    pub url: String,
    /// Open "More details" when any of them is filled in or wrong.
    pub details_open: bool,
}

async fn form(db: &SqlitePool, j: &Joy, id: Option<i64>, i: &ActivityInput, error: &str) -> Result<FormView, AppError> {
    let ctx = &j.ctx;
    let mut pair_opts = db::visible_list(db, &j.couple, Some(j.who))
        .await?
        .iter()
        .filter(|o| Some(o.id) != id)
        .map(|o| Opt { value: o.id.to_string(), label: ctx.personalize(&o.title), selected: i.pairs.contains(&o.id) })
        .collect::<Vec<_>>();
    // Current pairs first, so they're visible without scrolling.
    pair_opts.sort_by_key(|o| !o.selected);
    let people = ctx.people.iter().map(|(n, _)| (n.as_str(), n.as_str()));
    Ok(FormView {
        id,
        title: i.title.clone(),
        notes: i.notes.clone(),
        est_cost: if id.is_none() && i.est_cost == 0 && i.title.is_empty() { String::new() } else { crate::validate::plain_rupees(i.est_cost) },
        venue: i.venue.clone(),
        area: i.area.clone(),
        location: i.location.clone(),
        tags: i.tags.clone(),
        currency: ctx.currency.clone(),
        couple: format!("{} & {}", ctx.people[0].0, ctx.people[1].0),
        error: error.into(),
        settings: Opt::list(SETTINGS, std::slice::from_ref(&i.setting)),
        durations: duration_opts(&i.duration),
        energies: Opt::list(ENERGIES, std::slice::from_ref(&i.energy)),
        people: {
            let mut p = Opt::list(people, std::slice::from_ref(&i.added_by));
            // Keep an author we don't know (e.g. imported "list") instead of silently overwriting it.
            if !i.added_by.is_empty() && !p.iter().any(|o| o.selected) {
                let label = if i.added_by == "list" { "Both of us".into() } else { i.added_by.clone() };
                p.push(Opt { value: i.added_by.clone(), label, selected: true });
            }
            p
        },
        pair_opts,
        me_name: me_name(j),
        closed_opts: picker::WEEKDAYS
            .iter()
            .map(|(k, name)| Opt { value: (*k).into(), label: name[..3].into(), selected: i.closed.iter().any(|c| c == k) })
            .collect(),
        season_from_opts: month_opts(&i.season_from),
        season_to_opts: month_opts(&i.season_to),
        hours: i.hours.clone(),
        hours_today: match Hours::parse(&i.hours) {
            Some(h) => h.label(db::local_today(db).await?.2),
            None => String::new(),
        },
        best_times: Opt::list(db::BEST_TIMES, std::slice::from_ref(&i.best_time)),
        minutes: i.minutes.clone(),
        bookings: Opt::list(db::BOOKINGS, std::slice::from_ref(&i.booking)),
        book_ahead_days: i.book_ahead_days.clone(),
        covers: Opt::list(db::COVERS, std::slice::from_ref(&i.cover)),
        cover_unknown: !db::COVERS.iter().any(|(k, _)| *k == i.cover),
        bring: i.bring.clone(),
        station: i.station.clone(),
        station_walk_min: i.station_walk_min.clone(),
        url: i.url.clone(),
        details_open: error.contains("Hours")
            || [&i.hours, &i.best_time, &i.minutes, &i.booking, &i.bring, &i.station, &i.url].iter().any(|v| !v.trim().is_empty()),
    })
}

fn month_opts(selected: &str) -> Vec<Opt> {
    (1..=12)
        .map(|m: usize| Opt {
            value: m.to_string(),
            label: picker::MONTHS_SHORT[m - 1].into(),
            selected: selected == m.to_string(),
        })
        .collect()
}

/// What a saved slip answers with: where to go, and what was filled in on the way.
#[derive(Debug, Serialize)]
pub struct SlipSaved {
    pub ok: bool,
    pub to: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub filled: Vec<String>,
}

/// The JSON body of `POST /slips` and `POST /slips/{id}`. Converted to Joy's [`ActivityInput`].
#[derive(Debug, Default, Deserialize)]
pub struct SlipBody {
    pub title: String,
    /// "Save anyway": the title is the same as another slip's and that is fine.
    #[serde(default)]
    pub allow_duplicate: bool,
    #[serde(default)]
    pub notes: String,
    #[serde(default, deserialize_with = "flex")]
    pub est_cost: String,
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
    #[serde(default)]
    pub energy: String,
    #[serde(default)]
    pub added_by: String,
    /// Ids of slips this one pairs well with.
    #[serde(default)]
    pub pairs: Vec<i64>,
    /// Weekday keys it's closed on ("mon", …).
    #[serde(default)]
    pub closed: Vec<String>,
    #[serde(default, deserialize_with = "flex")]
    pub season_from: String,
    #[serde(default, deserialize_with = "flex")]
    pub season_to: String,
    #[serde(default)]
    pub hours: String,
    #[serde(default)]
    pub best_time: String,
    #[serde(default, deserialize_with = "flex")]
    pub minutes: String,
    #[serde(default)]
    pub booking: String,
    #[serde(default, deserialize_with = "flex")]
    pub book_ahead_days: String,
    #[serde(default)]
    pub cover: String,
    #[serde(default)]
    pub bring: String,
    #[serde(default)]
    pub station: String,
    #[serde(default, deserialize_with = "flex")]
    pub station_walk_min: String,
    #[serde(default)]
    pub url: String,
}

impl SlipBody {
    /// Joy's form model. A cost that isn't a whole number is refused later, by [`save`].
    fn input(self) -> (ActivityInput, bool) {
        let cost = self.est_cost.trim();
        let (est_cost, cost_ok) = if cost.is_empty() {
            (0, true)
        } else {
            match crate::validate::paise_from_text(cost) {
                Ok(c) => (c, true),
                Err(_) => (0, false),
            }
        };
        (
            ActivityInput {
                title: self.title,
                notes: self.notes,
                est_cost,
                setting: self.setting,
                duration: self.duration,
                venue: self.venue,
                area: self.area,
                location: self.location,
                tags: self.tags,
                energy: if self.energy.is_empty() { "med".into() } else { self.energy },
                added_by: self.added_by,
                pairs: self.pairs,
                closed: self.closed,
                season_from: self.season_from,
                season_to: self.season_to,
                hours: self.hours,
                best_time: self.best_time,
                minutes: self.minutes,
                booking: self.booking,
                book_ahead_days: self.book_ahead_days,
                cover: self.cover,
                bring: self.bring,
                station: self.station,
                station_walk_min: self.station_walk_min,
                url: self.url,
            },
            cost_ok,
        )
    }
}

fn input_from(ctx: &Ctx, a: &Activity) -> ActivityInput {
    ActivityInput {
        title: ctx.personalize(&a.title),
        notes: ctx.personalize(&a.notes),
        est_cost: a.est_cost,
        setting: a.setting.clone(),
        duration: a.duration.clone(),
        venue: a.venue.clone(),
        area: a.area.clone(),
        location: a.latlng().map(|(lat, lng)| format!("{lat}, {lng}")).unwrap_or_default(),
        tags: a.tags.replace(',', ", "),
        energy: a.energy.clone(),
        added_by: a.added_by.clone(),
        pairs: vec![],
        closed: a.closed_days.split(',').filter(|d| !d.is_empty()).map(str::to_string).collect(),
        season_from: a.season().map(|s| s.0.to_string()).unwrap_or_default(),
        season_to: a.season().map(|s| s.1.to_string()).unwrap_or_default(),
        hours: a.hours.clone(),
        best_time: a.best_time.clone(),
        minutes: a.minutes.map(|m| m.to_string()).unwrap_or_default(),
        booking: a.booking.clone(),
        book_ahead_days: a.book_ahead_days.map(|m| m.to_string()).unwrap_or_default(),
        cover: a.cover.clone(),
        bring: a.bring.clone(),
        station: a.station.clone(),
        station_walk_min: a.station_walk_min.map(|m| m.to_string()).unwrap_or_default(),
        url: a.url.clone(),
    }
}

/// `GET /slips/new`: a blank form.
pub async fn new(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<FormView>, AppError> {
    let j = joy(&s, &user_id).await?;
    let blank = ActivityInput {
        setting: "local".into(),
        duration: "short".into(),
        energy: "med".into(),
        // Signed by whoever is writing it.
        added_by: j.ctx.people[j.who].0.clone(),
        ..Default::default()
    };
    Ok(Json(form(&s.db, &j, None, &blank, "").await?))
}

/// `GET /slips/{id}`: the form filled in with that slip.
pub async fn edit(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<FormView>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    let a = mine(&s.db, &j.couple, id).await?;
    let mut input = input_from(&j.ctx, &a);
    input.pairs = db::partners(&s.db, &j.couple, id).await?.iter().map(|p| p.id).collect();
    Ok(Json(form(&s.db, &j, Some(id), &input, "").await?))
}

/// What is wrong with a slip as typed, in words for the form; empty when it can be saved. The first
/// problem found is the one reported. `cost_ok`: the cost field was a whole number; `latlng`: the
/// location, if it parsed.
fn slip_error(i: &ActivityInput, cost_ok: bool, latlng: Option<(f64, f64)>) -> &'static str {
    if i.title.trim().is_empty() {
        "Give it a name."
    } else if i.title.trim().chars().count() > db::MAX_TITLE {
        "That name is too long: keep it under 120 characters."
    } else if i.venue.trim().chars().count() > db::MAX_PLACE {
        "That place is too long: keep it under 120 characters."
    } else if i.area.trim().chars().count() > db::MAX_PLACE {
        "That area is too long: keep it under 120 characters."
    } else if i.notes.trim().chars().count() > db::MAX_NOTES {
        "Those notes are too long: keep them under 2,000 characters."
    } else if !db::tags_fit(&i.tags) {
        "Use at most 12 tags of up to 24 characters each."
    } else if !db::url_ok(&i.url) {
        "That link should start with http:// or https:// and have no spaces."
    } else if !ENERGIES.iter().any(|(k, _)| *k == i.energy) {
        "Pick an energy: low, medium or high."
    } else if !cost_ok {
        "The cost should be a whole number."
    } else if !i.location.trim().is_empty() && latlng.is_none() {
        "Location should look like “19.13, 72.81”. Leave it blank for at-home plans."
    } else if !i.hours.trim().is_empty() && Hours::parse(&i.hours).is_none() {
        "Hours should look like “tue-sun 12:00-23:30”, with a “;” between different days."
    } else if Duration::parse(&i.duration).is_none() || !SETTINGS.iter().any(|(k, _)| *k == i.setting) {
        "Pick a setting and duration."
    } else {
        ""
    }
}

async fn save(s: AppState, user_id: String, id: Option<i64>, body: SlipBody) -> Result<Response, AppError> {
    let j = joy(&s, &user_id).await?;
    if let Some(id) = id {
        mine(&s.db, &j.couple, id).await?;
    }
    let allow_duplicate = body.allow_duplicate;
    let (i, cost_ok) = body.input();
    let latlng = parse_latlng(&i.location);
    let error = slip_error(&i, cost_ok, latlng);
    let dup = if error.is_empty() && !allow_duplicate { db::duplicate_of(&s.db, &j.couple, &i.title, id).await? } else { None };
    let error = if dup.is_some() { "A slip like this is already in the jar. Save it anyway?" } else { error };
    if !error.is_empty() {
        let view = form(&s.db, &j, id, &i, error).await?;
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(view)).into_response());
    }
    let saved = db::save(&s.db, &j.couple, id, &i, latlng).await?;
    // Blanks the rest of the jar can answer are filled, and the note says what, so it can be undone by editing.
    let filled: Vec<String> = crate::joy::enrich::run_one(&s.db, &j.couple, saved).await?.iter().map(|f| f.describe()).collect();
    Ok(Json(SlipSaved { ok: true, to: SLIPS.into(), filled }).into_response())
}

/// `POST /slips`
pub async fn create(State(s): State<AppState>, Extension((user_id, _)): User, Json(i): Json<SlipBody>) -> Result<Response, AppError> {
    save(s, user_id, None, i).await
}

/// `POST /slips/{id}`
pub async fn update(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<i64>,
    Json(i): Json<SlipBody>,
) -> Result<Response, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    save(s, user_id, Some(id), i).await
}

/// A day we marked by hand (not drawn from the jar), with its one-off slip as it stands.
#[derive(Debug, Serialize)]
pub struct CustomDate {
    pub slip_id: i64,
    pub title: String,
    /// "YYYY-MM-DD" days it was kept on (usually one).
    pub kept_on: Vec<String>,
    /// The slip has been given its details (notes) already.
    pub dressed: bool,
    pub setting: String,
    pub venue: String,
    pub area: String,
    pub location: String,
    pub notes: String,
    pub tags: String,
    pub energy: String,
    pub duration: String,
    pub best_time: String,
    pub minutes: Option<i64>,
    pub cover: String,
    pub bring: String,
    pub station: String,
    pub url: String,
    pub est_cost_paise: i64,
}

/// `GET /custom`: days we marked by hand, so their slips can be filled in like any other.
pub async fn custom_list(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Vec<CustomDate>>, AppError> {
    let j = joy(&s, &user_id).await?;
    let rows = db::custom_slips(&s.db, &j.couple, j.who).await?;
    Ok(Json(
        rows.into_iter()
            .map(|(a, days)| CustomDate {
                slip_id: a.id,
                kept_on: days.split(',').filter(|d| !d.is_empty()).map(str::to_string).collect(),
                dressed: !a.notes.trim().is_empty(),
                location: a.latlng().map(|(lat, lng)| format!("{lat}, {lng}")).unwrap_or_default(),
                title: a.title,
                setting: a.setting,
                venue: a.venue,
                area: a.area,
                notes: a.notes,
                tags: a.tags,
                energy: a.energy,
                duration: a.duration,
                best_time: a.best_time,
                minutes: a.minutes,
                cover: a.cover,
                bring: a.bring,
                station: a.station,
                url: a.url,
                est_cost_paise: a.est_cost,
            })
            .collect(),
    ))
}

/// What may be added to a hand-marked day's slip. Every field is optional.
#[derive(Debug, Default, Deserialize)]
pub struct DressBody {
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub venue: String,
    #[serde(default)]
    pub area: String,
    /// "19.13, 72.81"
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub energy: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub best_time: String,
    #[serde(default, deserialize_with = "flex")]
    pub minutes: String,
    #[serde(default)]
    pub hours: String,
    #[serde(default)]
    pub cover: String,
    #[serde(default)]
    pub bring: String,
    #[serde(default)]
    pub station: String,
    #[serde(default, deserialize_with = "flex")]
    pub station_walk_min: String,
    #[serde(default)]
    pub url: String,
    /// Also put it in the jar, so it can be drawn again.
    #[serde(default)]
    pub in_jar: bool,
}

#[derive(Debug, Serialize)]
pub struct Dressed {
    pub ok: bool,
    /// Fields that were blank and are now filled.
    pub filled: Vec<&'static str>,
    /// Fields sent that already had a value, so were left alone.
    pub kept: Vec<&'static str>,
    pub in_jar: bool,
    /// What the rest of the jar filled in on top (best time, hours…).
    pub also: Vec<String>,
}

/// Fill blanks only: `dst` takes `src` when `dst` is empty and `src` is not.
fn fill(name: &'static str, dst: &mut String, src: &str, filled: &mut Vec<&'static str>, kept: &mut Vec<&'static str>) {
    let src = src.trim();
    if src.is_empty() {
        return;
    }
    if dst.trim().is_empty() {
        *dst = src.to_string();
        filled.push(name);
    } else if dst.trim() != src {
        kept.push(name);
    }
}

/// `POST /slips/{id}/dress`: give a hand-marked day's one-off slip the details a jar slip has, so
/// it reads as well as one. Only blanks are filled; the title is never changed (its keepsake is
/// found by title). Energy and duration were placeholders on a hand-marked day, so they may be set
/// until the slip has notes. `in_jar` also makes it an ordinary slip in the jar.
pub async fn dress(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<i64>,
    Json(b): Json<DressBody>,
) -> Result<Response, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    let a = mine(&s.db, &j.couple, id).await?;
    if !db::is_custom(&s.db, &j.couple, id).await? {
        return Err(AppError(StatusCode::UNPROCESSABLE_ENTITY, "That slip wasn't a day marked by hand; edit it from Slips instead.".into()));
    }
    let mut i = input_from(&j.ctx, &a);
    // The stored text, not the personalised one, goes back in.
    i.title = a.title.clone();
    i.notes = a.notes.clone();
    i.pairs = db::partners(&s.db, &j.couple, id).await?.iter().map(|p| p.id).collect();
    let first = a.notes.trim().is_empty();
    let (mut filled, mut kept) = (Vec::new(), Vec::new());
    fill("notes", &mut i.notes, &b.notes, &mut filled, &mut kept);
    fill("venue", &mut i.venue, &b.venue, &mut filled, &mut kept);
    fill("area", &mut i.area, &b.area, &mut filled, &mut kept);
    fill("location", &mut i.location, &b.location, &mut filled, &mut kept);
    fill("tags", &mut i.tags, &b.tags, &mut filled, &mut kept);
    fill("best_time", &mut i.best_time, &b.best_time, &mut filled, &mut kept);
    fill("minutes", &mut i.minutes, &b.minutes, &mut filled, &mut kept);
    fill("hours", &mut i.hours, &b.hours, &mut filled, &mut kept);
    fill("cover", &mut i.cover, &b.cover, &mut filled, &mut kept);
    fill("bring", &mut i.bring, &b.bring, &mut filled, &mut kept);
    fill("station", &mut i.station, &b.station, &mut filled, &mut kept);
    fill("station_walk_min", &mut i.station_walk_min, &b.station_walk_min, &mut filled, &mut kept);
    fill("url", &mut i.url, &b.url, &mut filled, &mut kept);
    for (name, dst, src) in [("energy", &mut i.energy, b.energy.trim()), ("duration", &mut i.duration, b.duration.trim())] {
        if src.is_empty() || *dst == src {
            continue;
        }
        if first {
            *dst = src.to_string();
            filled.push(name);
        } else {
            kept.push(name);
        }
    }
    let latlng = parse_latlng(&i.location);
    let error = slip_error(&i, true, latlng);
    if !error.is_empty() {
        return Err(AppError(StatusCode::UNPROCESSABLE_ENTITY, error.into()));
    }
    if b.in_jar && db::duplicate_of(&s.db, &j.couple, &i.title, Some(id)).await?.is_some() {
        return Err(AppError(StatusCode::UNPROCESSABLE_ENTITY, "A slip with this name is already in the jar; leave in_jar off.".into()));
    }
    db::save(&s.db, &j.couple, Some(id), &i, latlng).await?;
    if b.in_jar {
        db::custom_to_jar(&s.db, &j.couple, id).await?;
    }
    let also = crate::joy::enrich::run_one(&s.db, &j.couple, id).await?.iter().map(|f| f.describe()).collect();
    Ok(Json(Dressed { ok: true, filled, kept, in_jar: b.in_jar, also }).into_response())
}

/// `POST /slips/{id}/archive`
pub async fn archive(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    mine(&s.db, &j.couple, id).await?;
    db::set_archived(&s.db, &j.couple, id, true).await?;
    Ok(go(SLIPS))
}

#[derive(Deserialize)]
pub struct WantBody {
    #[serde(default)]
    level: String,
}

/// `POST /slips/{id}/want`: quietly mark a slip "want", "fine" or "no" (or "" to take it back). Only
/// the caller's own mark changes, and the other one never sees it.
pub async fn want(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<i64>,
    Json(b): Json<WantBody>,
) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    mine(&s.db, &j.couple, id).await?;
    if !matches!(b.level.as_str(), "" | "want" | "fine" | "no") {
        return Err(AppError(StatusCode::UNPROCESSABLE_ENTITY, "Pick want, fine or no.".into()));
    }
    db::set_want(&s.db, &j.couple, id, j.who, &b.level).await?;
    Ok(go(SLIPS))
}

/// `GET /packs`: the starter packs, and how much of each this couple has.
pub async fn packs(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Vec<db::PackView>>, AppError> {
    let j = joy(&s, &user_id).await?;
    Ok(Json(db::starter_packs(&s.db, &j.couple).await?))
}

#[derive(Deserialize)]
pub struct PackBody {
    pack: String,
}

#[derive(Serialize)]
pub struct PackAdded {
    pub added: u64,
}

/// `POST /packs/add`: put a starter pack's slips in the jar (the ones not already there).
pub async fn add_pack(State(s): State<AppState>, Extension((user_id, _)): User, Json(b): Json<PackBody>) -> Result<Json<PackAdded>, AppError> {
    let j = joy(&s, &user_id).await?;
    match db::add_pack(&s.db, &j.couple, &b.pack).await? {
        Some(added) => Ok(Json(PackAdded { added })),
        None => Err(not_found("No such starter pack")),
    }
}

/// `POST /slips/{id}/unshare`: the envelope link stops opening.
pub async fn unshare(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    mine(&s.db, &j.couple, id).await?;
    db::unshare(&s.db, &j.couple, "joy_activities", id).await?;
    Ok(go(SLIPS))
}

/// `POST /slips/{id}/restore`
pub async fn restore(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    mine(&s.db, &j.couple, id).await?;
    // A hand-added date's one-off slip has no place in the jar.
    if !db::restore_one(&s.db, &j.couple, id).await? {
        return Err(not_found("No such slip"));
    }
    Ok(go(SLIPS))
}

/// `POST /slips/{id}/unveto`: bring a slip someone passed on back into the draw.
pub async fn unveto(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    mine(&s.db, &j.couple, id).await?;
    db::unveto(&s.db, &j.couple, id, j.who).await?;
    Ok(go(SLIPS))
}

/// `POST /slips/vetoes/clear`: every slip anyone passed on goes back into the draw.
pub async fn vetoes_clear(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    db::clear_vetoes(&s.db, &j.couple).await?;
    Ok(go(SLIPS))
}

/// `POST /slips/restore-all`: every archived slip comes back into the jar.
pub async fn restore_all(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    db::restore_all_archived(&s.db, &j.couple).await?;
    Ok(go(SLIPS))
}

/// `POST /slips/{id}/delete`
pub async fn delete(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    mine(&s.db, &j.couple, id).await?;
    // Archived rather than dropped when it has history (see `db::delete`); the answer is the same.
    db::delete(&s.db, &j.couple, id).await?;
    Ok(go(SLIPS))
}

fn me_name(j: &Joy) -> String {
    Me::new(&j.ctx, Some(j.who)).name
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    /// A slip that passes every check.
    fn good() -> ActivityInput {
        ActivityInput { title: "Sunset walk".into(), setting: "local".into(), duration: "short".into(), energy: "med".into(), ..Default::default() }
    }

    fn error_for(edit: impl FnOnce(&mut ActivityInput), cost_ok: bool) -> &'static str {
        let mut i = good();
        edit(&mut i);
        slip_error(&i, cost_ok, parse_latlng(&i.location))
    }

    #[test]
    fn a_good_slip_has_no_error() {
        assert_eq!(error_for(|_| {}, true), "");
    }

    #[test]
    fn each_mistake_gets_its_own_message() {
        let long = |n: usize| "x".repeat(n);
        let cases: Vec<(&str, Box<dyn FnOnce(&mut ActivityInput)>, bool, &str)> = vec![
            ("blank title", Box::new(|i| i.title = "   ".into()), true, "Give it a name."),
            ("long title", Box::new(move |i| i.title = long(db::MAX_TITLE + 1)), true, "That name is too long: keep it under 120 characters."),
            ("long venue", Box::new(move |i| i.venue = long(db::MAX_PLACE + 1)), true, "That place is too long: keep it under 120 characters."),
            ("long area", Box::new(move |i| i.area = long(db::MAX_PLACE + 1)), true, "That area is too long: keep it under 120 characters."),
            ("long notes", Box::new(move |i| i.notes = long(db::MAX_NOTES + 1)), true, "Those notes are too long: keep them under 2,000 characters."),
            ("too many tags", Box::new(|i| i.tags = (0..13).map(|n| format!("t{n}")).collect::<Vec<_>>().join(", ")), true, "Use at most 12 tags of up to 24 characters each."),
            ("bad link", Box::new(|i| i.url = "ftp://x".into()), true, "That link should start with http:// or https:// and have no spaces."),
            ("bad energy", Box::new(|i| i.energy = "wild".into()), true, "Pick an energy: low, medium or high."),
            ("cost not a number", Box::new(|_| {}), false, "The cost should be a whole number."),
            ("bad location", Box::new(|i| i.location = "somewhere".into()), true, "Location should look like “19.13, 72.81”. Leave it blank for at-home plans."),
            ("bad hours", Box::new(|i| i.hours = "whenever".into()), true, "Hours should look like “tue-sun 12:00-23:30”, with a “;” between different days."),
            ("bad duration", Box::new(|i| i.duration = "forever".into()), true, "Pick a setting and duration."),
            ("bad setting", Box::new(|i| i.setting = "moon".into()), true, "Pick a setting and duration."),
        ];
        for (what, edit, cost_ok, want) in cases {
            assert_eq!(error_for(edit, cost_ok), want, "{what}");
        }
    }

    #[test]
    fn limits_are_inclusive_and_blank_optionals_are_fine() {
        assert_eq!(error_for(|i| i.title = "x".repeat(db::MAX_TITLE), true), "");
        assert_eq!(error_for(|i| i.notes = "x".repeat(db::MAX_NOTES), true), "");
        assert_eq!(error_for(|i| { i.location = "19.13, 72.81".into(); i.hours = "tue-sun 12:00-23:30".into(); }, true), "");
        assert_eq!(error_for(|i| i.location = "  ".into(), true), "", "a blank location means an at-home plan");
    }

    #[test]
    fn the_first_problem_found_is_the_one_reported() {
        let both = |i: &mut ActivityInput| { i.title = String::new(); i.url = "nope".into(); i.energy = "wild".into(); };
        assert_eq!(error_for(both, false), "Give it a name.");
        let later = |i: &mut ActivityInput| { i.url = "nope".into(); i.energy = "wild".into(); };
        assert_eq!(error_for(later, false), "That link should start with http:// or https:// and have no spaces.");
        let last = |i: &mut ActivityInput| { i.energy = "wild".into(); i.hours = "whenever".into(); i.duration = "forever".into(); };
        assert_eq!(error_for(last, true), "Pick an energy: low, medium or high.");
    }

    /// A migrated database with one couple (`ua` first, `ub` second) and a slip of theirs.
    async fn couple_db() -> (SqlitePool, i64) {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for u in ["ua", "ub"] {
            sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES (?, ?, 'x', 'user')")
                .bind(u)
                .bind(u)
                .execute(&db)
                .await
                .unwrap();
        }
        sqlx::query("INSERT INTO couples (couple_id, user_0, user_1) VALUES ('C', 'ua', 'ub')").execute(&db).await.unwrap();
        let input = ActivityInput {
            title: "Sunset walk".into(),
            est_cost: 500,
            setting: "local".into(),
            duration: "short".into(),
            energy: "med".into(),
            ..Default::default()
        };
        let id = db::save(&db, "C", None, &input, None).await.unwrap();
        (db, id)
    }

    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("handler failed: {}", e.1),
        }
    }

    fn state(db: SqlitePool) -> AppState {
        let key = rusty_paseto::prelude::PasetoSymmetricKey::from(rusty_paseto::prelude::Key::from(&[7u8; 32]));
        AppState { db, paseto_key: std::sync::Arc::new(key) }
    }

    fn ids(groups: &[Group]) -> Vec<i64> {
        groups.iter().flat_map(|g| g.items.iter().map(|a| a.id)).collect()
    }

    #[tokio::test]
    async fn a_sealed_surprise_slip_is_hidden_from_the_partner_list() {
        let (db, id) = couple_db().await;
        // ua (slot 0) plans it as a surprise.
        db::accept(&db, "C", id, Some(0), true).await.unwrap();

        let planner = ok(page(State(state(db.clone())), Extension(("ua".into(), String::new()))).await).0;
        assert_eq!(ids(&planner.groups), vec![id], "the creator still sees the slip");

        let partner = ok(page(State(state(db.clone())), Extension(("ub".into(), String::new()))).await).0;
        assert!(ids(&partner.groups).is_empty(), "the partner doesn't see the sealed slip");
        assert_eq!(partner.total, 0);

        // Opening or changing it by id is a plain 404 for the partner, so its existence doesn't leak.
        let opened = edit(State(state(db.clone())), Extension(("ub".into(), String::new())), Path(id)).await;
        assert_eq!(opened.err().map(|e| e.0), Some(axum::http::StatusCode::NOT_FOUND));
        let deleted = delete(State(state(db.clone())), Extension(("ub".into(), String::new())), Path(id)).await;
        assert_eq!(deleted.err().map(|e| e.0), Some(axum::http::StatusCode::NOT_FOUND));
        assert!(edit(State(state(db.clone())), Extension(("ua".into(), String::new())), Path(id)).await.is_ok());
    }

    #[tokio::test]
    async fn deleting_a_slip_with_history_moves_it_to_archived() {
        let (db, id) = couple_db().await;
        db::accept(&db, "C", id, Some(0), false).await.unwrap();

        let res = ok(delete(State(state(db.clone())), Extension(("ua".into(), String::new())), Path(id)).await);
        assert_eq!(res.0.to, SLIPS);
        let view = ok(page(State(state(db.clone())), Extension(("ua".into(), String::new()))).await).0;
        assert!(ids(&view.groups).is_empty(), "archived slips stay out of the slip list");
        assert_eq!(view.archived.iter().map(|a| a.id).collect::<Vec<_>>(), vec![id]);
    }

    /// The dress answer as JSON, or the refusal's status.
    async fn dress_as(db: &SqlitePool, user: &str, id: i64, b: DressBody) -> Result<serde_json::Value, StatusCode> {
        match dress(State(state(db.clone())), Extension((user.into(), String::new())), Path(id), Json(b)).await {
            Ok(r) => Ok(serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap()),
            Err(e) => Err(e.0),
        }
    }

    fn names(v: &serde_json::Value, k: &str) -> Vec<String> {
        v[k].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect()
    }

    #[tokio::test]
    async fn a_hand_marked_day_is_listed_and_dressed_filling_only_blanks() {
        let (db, slip) = couple_db().await;
        let id = db::keep_custom(&db, "C", Some(0), "The day we met", "Bandra", "local", "2026-11-20", None).await.unwrap();
        let act: i64 = sqlx::query_scalar("SELECT activity_id FROM joy_history WHERE id = ?").bind(id).fetch_one(&db).await.unwrap();

        let list = ok(custom_list(State(state(db.clone())), Extension(("ub".into(), String::new()))).await).0;
        assert_eq!(list.len(), 1, "only the hand-marked day, not the jar slip");
        assert_eq!((list[0].slip_id, list[0].kept_on.clone(), list[0].dressed), (act, vec!["2026-11-20".to_string()], false));

        let first = DressBody {
            notes: "Coffee that turned into a walk to the sea.".into(),
            venue: "Somewhere else".into(),
            tags: "first, coffee".into(),
            energy: "low".into(),
            duration: "half_day".into(),
            best_time: "evening".into(),
            ..Default::default()
        };
        let d = dress_as(&db, "ua", act, first).await.unwrap();
        let filled = names(&d, "filled");
        assert!(["notes", "energy", "duration", "tags"].iter().all(|f| filled.iter().any(|x| x == f)), "{filled:?}");
        assert_eq!(names(&d, "kept"), vec!["venue"], "a place already typed is left alone");
        let a = db::get(&db, "C", act).await.unwrap().unwrap();
        assert_eq!((a.title.as_str(), a.venue.as_str(), a.energy.as_str(), a.duration.as_str()), ("The day we met", "Bandra", "low", "half_day"));
        assert!(a.tags.starts_with("first"), "tags are saved, in the jar's own words: {}", a.tags);

        // Once dressed, notes and the old placeholders are the couple's: a second pass changes nothing.
        let again = DressBody { notes: "Other words".into(), energy: "high".into(), ..Default::default() };
        let d = dress_as(&db, "ua", act, again).await.unwrap();
        assert!(names(&d, "filled").is_empty());
        let a = db::get(&db, "C", act).await.unwrap().unwrap();
        assert_eq!((a.notes.as_str(), a.energy.as_str()), ("Coffee that turned into a walk to the sea.", "low"));
        assert!(!db::list(&db, "C", false).await.unwrap().iter().any(|x| x.id == act), "still out of the jar");

        // A jar slip is edited from Slips, not here.
        let refused = dress_as(&db, "ua", slip, DressBody { notes: "x".into(), ..Default::default() }).await.unwrap_err();
        assert_eq!(refused, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn a_dressed_day_can_join_the_jar() {
        let (db, _) = couple_db().await;
        let id = db::keep_custom(&db, "C", Some(0), "Picnic at the lake", "", "local", "2026-11-01", None).await.unwrap();
        let act: i64 = sqlx::query_scalar("SELECT activity_id FROM joy_history WHERE id = ?").bind(id).fetch_one(&db).await.unwrap();
        let d = dress_as(&db, "ua", act, DressBody { notes: "Mats, mangoes, a long afternoon.".into(), in_jar: true, ..Default::default() }).await.unwrap();
        assert_eq!(d["in_jar"], true);
        assert!(db::list(&db, "C", false).await.unwrap().iter().any(|x| x.id == act), "now in the jar");
        let list = ok(custom_list(State(state(db.clone())), Extension(("ua".into(), String::new()))).await).0;
        assert!(list.is_empty(), "no longer a hand-marked one-off");

        // A name already in the jar is not put in twice.
        let id2 = db::keep_custom(&db, "C", Some(0), "Sunset walk", "", "local", "2026-11-02", None).await.unwrap();
        let act2: i64 = sqlx::query_scalar("SELECT activity_id FROM joy_history WHERE id = ?").bind(id2).fetch_one(&db).await.unwrap();
        let e = dress_as(&db, "ua", act2, DressBody { in_jar: true, ..Default::default() }).await.unwrap_err();
        assert_eq!(e, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn an_out_of_range_cost_is_refused() {
        let body = |cost: &str| SlipBody { title: "x".into(), est_cost: cost.into(), setting: "local".into(), duration: "short".into(), ..Default::default() };
        assert!(body("1200").input().1, "a whole rupee amount is fine");
        assert!(!body("-5").input().1, "negative cost is refused");
        assert!(!body("1000000000001").input().1, "above the rupee cap is refused");
        assert!(body("12.5").input().1, "paise are fine");
        assert!(!body("12.345").input().1, "more than two decimals is refused");
    }
}
