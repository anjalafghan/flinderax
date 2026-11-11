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
    let view = |a: &Activity| {
        let mut v = ActivityView::new(ctx, a, &learned);
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
    unsealed(&s.db, &j, id).await?;
    let (db, couple, ctx) = (&s.db, j.couple.as_str(), &j.ctx);
    let a = mine(db, couple, id).await?;
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
        est_cost: if id.is_none() && i.est_cost == 0 && i.title.is_empty() { String::new() } else { i.est_cost.to_string() },
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
        details_open: error.contains("Hours")
            || [&i.hours, &i.best_time, &i.minutes, &i.booking, &i.bring, &i.station].iter().any(|v| !v.trim().is_empty()),
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

/// The JSON body of `POST /slips` and `POST /slips/{id}`. Converted to Joy's [`ActivityInput`].
#[derive(Debug, Default, Deserialize)]
pub struct SlipBody {
    pub title: String,
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
}

impl SlipBody {
    /// Joy's form model. A cost that isn't a whole number is refused later, by [`save`].
    fn input(self) -> (ActivityInput, bool) {
        let cost = self.est_cost.trim();
        let (est_cost, cost_ok) = if cost.is_empty() {
            (0, true)
        } else {
            match cost.parse::<i64>().map(crate::validate::rupees) {
                Ok(Ok(c)) => (c, true),
                _ => (0, false),
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

async fn save(s: AppState, user_id: String, id: Option<i64>, body: SlipBody) -> Result<Response, AppError> {
    let j = joy(&s, &user_id).await?;
    if let Some(id) = id {
        mine(&s.db, &j.couple, id).await?;
    }
    let (i, cost_ok) = body.input();
    let latlng = parse_latlng(&i.location);
    let error = if i.title.trim().is_empty() {
        "Give it a name."
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
    };
    if !error.is_empty() {
        let view = form(&s.db, &j, id, &i, error).await?;
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(view)).into_response());
    }
    db::save(&s.db, &j.couple, id, &i, latlng).await?;
    Ok(go(SLIPS).into_response())
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

/// `POST /slips/{id}/archive`
pub async fn archive(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    mine(&s.db, &j.couple, id).await?;
    db::set_archived(&s.db, &j.couple, id, true).await?;
    Ok(go(SLIPS))
}

/// `POST /slips/{id}/restore`
pub async fn restore(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    mine(&s.db, &j.couple, id).await?;
    db::set_archived(&s.db, &j.couple, id, false).await?;
    Ok(go(SLIPS))
}

/// `POST /slips/{id}/unveto`: bring a slip someone passed on back into the draw.
pub async fn unveto(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    unsealed(&s.db, &j, id).await?;
    mine(&s.db, &j.couple, id).await?;
    db::unveto(&s.db, &j.couple, id).await?;
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
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
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

    #[tokio::test]
    async fn an_out_of_range_cost_is_refused() {
        let body = |cost: &str| SlipBody { title: "x".into(), est_cost: cost.into(), setting: "local".into(), duration: "short".into(), ..Default::default() };
        assert!(body("1200").input().1, "a whole rupee amount is fine");
        assert!(!body("-5").input().1, "negative cost is refused");
        assert!(!body("1000000000001").input().1, "above the rupee cap is refused");
        assert!(!body("12.5").input().1, "not a whole number is refused");
    }
}
