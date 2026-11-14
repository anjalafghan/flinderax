//! Kept: the dates we've had, as a JSON API. Ports Joy's `routes/history.rs`.
//!
//! Sealing: a surprise is sealed for anyone but its planner until the planner reveals it. Every
//! handler that acts on a kept date refuses (403) when it is sealed for the caller, `reveal` is
//! the planner's alone, and the two public, signed endpoints (the `.ics` invite and the photo)
//! put the viewer in the signed path or treat the date as sealed for nobody-in-particular.
//!
//! Signed links keep the paths [`super::dayplan::ics_url`] signs (`/api/dates/history/{id}/...`);
//! the links this module hands out use the `/api/public/dates/` prefix (no bearer token) while the
//! signature covers the `/api/dates/...` path, which is what the handlers verify.

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{
    budget_view, dayplan, flex, flex_list, go, internal, joy, learned, load_ctx, past_months, planned_input, planned_label,
    regular::public_link, settle_month, sign, verify, AppError, BudgetView, Go, Learned, Leftover, Me, Opt, PastMonth,
};
use crate::{
    joy::{
        ctx::Ctx,
        db::{self, DatePhoto, HistoryRow},
        ics, photos,
        picker::Duration,
        weather,
    },
    models::AppState,
};

use super::decide::sealed_for;

type User = Extension<(String, String)>;

/// How long a signed photo link stays good.
pub const PHOTO_TTL_SECS: i64 = 3600;
/// Longest line, in characters.
const MAX_LINE: usize = 280;
/// Slips listed as things to save for.
const GOAL_CHOICES: usize = 20;
/// Headroom over the photo limit for multipart framing.
const UPLOAD_SLACK: usize = 64 * 1024;

fn forbidden(msg: &str) -> AppError {
    AppError(StatusCode::FORBIDDEN, msg.into())
}

fn missing() -> AppError {
    AppError(StatusCode::NOT_FOUND, "That date isn't here.".into())
}

fn unprocessable(msg: &str) -> AppError {
    AppError(StatusCode::UNPROCESSABLE_ENTITY, msg.into())
}

/// One of this couple's history rows (anyone else's is a 404).
async fn row_of(db: &SqlitePool, couple: &str, id: i64) -> Result<HistoryRow, AppError> {
    db::history_row(db, couple, id).await?.ok_or_else(missing)
}

/// A kept date this viewer may act on: 404 if it isn't ours, 403 if it's a surprise sealed for them.
async fn open_row(db: &SqlitePool, couple: &str, id: i64, who: Option<usize>) -> Result<HistoryRow, AppError> {
    let row = row_of(db, couple, id).await?;
    if sealed_for(&row, who) {
        return Err(forbidden("That's a surprise."));
    }
    Ok(row)
}

fn kept_to(id: i64) -> String {
    format!("/joy/kept#k{id}")
}

// ── the page ──────────────────────────────────────────────────────────────

/// One kept date as the viewer is allowed to see it.
#[derive(Debug, Serialize)]
pub struct KeptView {
    pub id: i64,
    /// A surprise for this viewer: everything below except `hints`, `from_name`,
    /// `planned_label` and `day_label` is blank.
    pub sealed: bool,
    pub title: String,
    /// Setting key ("home", "local", ...) for colour. The hints already say this much.
    pub setting: String,
    pub place: String,
    /// "YYYY-MM-DD"
    pub day: String,
    pub day_label: String,
    pub rating: Option<i64>,
    pub note: String,
    /// Who planned it ("Rohit"), when we know.
    pub from_name: String,
    /// On the planner's own device, for an unrevealed surprise: who it's for ("Neha").
    pub surprise_for: String,
    /// Planned by the person asking, and not yet revealed: they may reveal it.
    pub can_reveal: bool,
    /// For a sealed surprise: what to know without spoiling it.
    pub hints: Vec<String>,
    /// "Sat 18 Oct, 7:00 pm", or empty when no time is set.
    pub planned_label: String,
    /// Value for `<input type="datetime-local">`.
    pub planned_input: String,
    /// Signed calendar invite, when a time is set.
    pub ics_url: String,
    /// The cover (first) photo's signed link (1 hour), never a bare path.
    pub photo_url: Option<String>,
    /// Every photo, oldest first; empty when sealed.
    pub photos: Vec<KeptPhoto>,
    /// More photos fit on this date.
    pub can_add_photo: bool,
    /// "₹1,000": the slip's estimate.
    pub cost_est: String,
    /// "₹5,000", or empty when the real cost isn't filled in.
    pub cost_actual: String,
    /// Value for the cost field ("5000" or "").
    pub cost_actual_raw: String,
    /// "₹4,000 more", "₹200 less" or "spot on"; empty without a real cost.
    pub cost_diff: String,
    pub cost_over: bool,
    /// Per person: name and the real one-way cab fare as entered (or ""), for the cost editor.
    pub cabs: Vec<Cab>,
    /// Comma-separated history ids that share a time: this one, plus its pair if it was
    /// kept as a pair, so "change time" moves both.
    pub pair_ids: String,
    pub lines: Lines,
}

#[derive(Debug, Serialize)]
pub struct Cab {
    pub name: String,
    pub fare: String,
}

/// The two lines on a kept date, as this viewer may see them.
#[derive(Debug, Serialize)]
pub struct Lines {
    /// Who's asking is known, so they can write theirs.
    pub can_write: bool,
    pub mine: String,
    /// The other one of us's line: only once both are written.
    pub theirs: String,
    /// The other one of us has written theirs (it shows once mine is in too).
    pub theirs_waiting: bool,
    /// The other one of us ("Neha").
    pub partner: String,
}

#[derive(Debug, Serialize)]
pub struct Month {
    pub label: String,
    /// "/joy/year/2026/10": that month's spread.
    pub href: String,
    pub items: Vec<KeptView>,
}

#[derive(Debug, Serialize)]
pub struct SkippedView {
    pub title: String,
    pub day_label: String,
}

/// The savings jar: money put aside for one slip too big for a month.
#[derive(Debug, Serialize)]
pub struct SavingsView {
    /// "₹6,200".
    pub saved: String,
    pub saved_raw: i64,
    pub goal: Option<Goal>,
    /// Slips we could save for, priciest first ("Weekend in Lonavala · ₹18,000").
    pub choices: Vec<Opt>,
    pub recent: Vec<SavingLine>,
}

#[derive(Debug, Serialize)]
pub struct Goal {
    pub title: String,
    pub cost: String,
    /// 0–100, for the bar.
    pub pct: u8,
    /// "₹11,800 to go", or empty once there's enough.
    pub to_go: String,
    /// There's enough in the jar to keep it.
    pub ready: bool,
}

#[derive(Debug, Serialize)]
pub struct SavingLine {
    pub id: i64,
    /// "+₹2,000" or "−₹18,000".
    pub amount: String,
    pub note: String,
    pub on: String,
    /// Put in by hand, so it can be taken back.
    pub removable: bool,
}

#[derive(Debug, Serialize)]
pub struct OtherSpend {
    pub id: i64,
    pub amount: String,
    pub note: String,
    pub on: String,
}

#[derive(Debug, Serialize)]
pub struct HistoryView {
    /// Kept dates, newest month first.
    pub months: Vec<Month>,
    pub kept: usize,
    pub skipped: Vec<SkippedView>,
    pub budget: BudgetView,
    /// Who's asking ("Rohit").
    pub me_name: String,
    /// "₹", for the cost field.
    pub currency: String,
    /// "Your dates usually run about 40% over the estimate (5 dates).", once there's enough to say.
    pub cost_insight: String,
    /// Finished months, newest first.
    pub past: Vec<PastMonth>,
    /// This month's spending that wasn't a slip.
    pub other_spend: Vec<OtherSpend>,
    pub savings: SavingsView,
}

/// History rows kept together as a pair with `r` (same request, so within a few seconds).
fn pair_ids(r: &HistoryRow, done: &[HistoryRow], pairs: &[(i64, i64)]) -> String {
    let paired = |x: i64, y: i64| pairs.contains(&(x.min(y), x.max(y)));
    std::iter::once(r.id)
        .chain(
            done.iter()
                .filter(|o| o.id != r.id && (o.picked_at - r.picked_at).abs() <= 5 && paired(o.activity_id, r.activity_id))
                .map(|o| o.id),
        )
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Each line stays sealed from the other one of us until both lines are written.
fn lines(ctx: &Ctx, history_id: i64, who: Option<usize>, all: &[(i64, i64, String)]) -> Lines {
    let line = |w: usize| all.iter().find(|(h, p, _)| *h == history_id && *p == w as i64).map(|(_, _, l)| l.clone());
    let me = Me::new(ctx, who);
    let (mine, theirs) = match who {
        Some(w) => (line(w), line(1 - w)),
        None => (None, None),
    };
    Lines {
        can_write: who.is_some(),
        theirs_waiting: mine.is_none() && theirs.is_some(),
        theirs: if mine.is_some() { theirs.unwrap_or_default() } else { String::new() },
        mine: mine.unwrap_or_default(),
        partner: me.partner,
    }
}

/// One photo of a kept date, as the page shows it.
#[derive(Serialize, Debug)]
pub struct KeptPhoto {
    pub id: i64,
    /// Signed link (1 hour).
    pub url: String,
}

/// A signed link to one photo of a kept date, good for an hour, on the public prefix (the
/// signature covers the `/api/dates/...` path). Photo files are never replaced, so the photo id
/// is enough to keep caches honest. A surprise that isn't revealed yet puts the viewer in the
/// signed path, so the handler can keep it sealed from the other one of us.
pub fn photo_url(r: &HistoryRow, photo_id: i64, who: Option<usize>) -> String {
    let who = if r.surprise && !r.revealed { who } else { None };
    public_link(sign(&photo_path(r.id, photo_id, who), PHOTO_TTL_SECS))
}

/// The same link for a date both of us may see (the Year and Month pages), good for `ttl`.
pub fn open_photo_url(history_id: i64, photo_id: i64, ttl: i64) -> String {
    public_link(sign(&photo_path(history_id, photo_id, None), ttl))
}

/// The path a photo link signs (what [`photo`] checks the signature against).
fn photo_path(id: i64, photo_id: i64, who: Option<usize>) -> String {
    match who {
        Some(w) => format!("/api/dates/history/{id}/photos/{photo_id}?who={w}"),
        None => format!("/api/dates/history/{id}/photos/{photo_id}"),
    }
}

fn view(ctx: &Ctx, r: &HistoryRow, who: Option<usize>, rain: bool, pair_ids: String, memories: &[(i64, i64, String)], pics: &[DatePhoto]) -> KeptView {
    let sealed = sealed_for(r, who);
    let planned = r.planned_local.as_deref().map(planned_label).unwrap_or_default();
    let from_name = r.drawn_by.and_then(|d| ctx.people.get(d as usize)).map(|p| p.0.clone()).unwrap_or_default();
    let mine_unrevealed = r.surprise && !r.revealed && !sealed;
    KeptView {
        id: r.id,
        sealed,
        title: if sealed { String::new() } else { ctx.personalize(&r.title) },
        setting: r.setting.clone(),
        place: if sealed { String::new() } else { r.place.clone() },
        day: r.day.clone(),
        day_label: r.day_label(),
        rating: if sealed { None } else { r.rating },
        note: if sealed { String::new() } else { r.note.clone() },
        surprise_for: if mine_unrevealed {
            r.drawn_by.and_then(|d| ctx.people.get(1 - d as usize)).map(|p| p.0.clone()).unwrap_or_default()
        } else {
            String::new()
        },
        can_reveal: mine_unrevealed,
        hints: if sealed { hints(r, &planned, rain) } else { vec![] },
        planned_input: r.planned_local.as_deref().map(planned_input).unwrap_or_default(),
        ics_url: if r.planned_at.is_some() { public_link(dayplan::ics_url(r.id, who)) } else { String::new() },
        photo_url: pics.first().filter(|_| !sealed).map(|p| photo_url(r, p.id, who)),
        photos: if sealed { vec![] } else { pics.iter().map(|p| KeptPhoto { id: p.id, url: photo_url(r, p.id, who) }).collect() },
        can_add_photo: !sealed && r.outcome == "did_it" && (pics.len() as i64) < db::MAX_PHOTOS_PER_DATE,
        pair_ids,
        cost_est: if sealed { String::new() } else { ctx.money(r.est_cost) },
        cost_actual: r.actual_cost.filter(|_| !sealed).map(|c| ctx.money(c)).unwrap_or_default(),
        cost_actual_raw: r.actual_cost.filter(|_| !sealed).map(|c| c.to_string()).unwrap_or_default(),
        cost_diff: match r.actual_cost.filter(|_| !sealed).map(|c| c - r.est_cost) {
            None => String::new(),
            Some(0) => "spot on".into(),
            Some(d) if d > 0 => format!("{} more", ctx.money(d)),
            Some(d) => format!("{} less", ctx.money(-d)),
        },
        cost_over: !sealed && r.actual_cost.is_some_and(|c| c > r.est_cost),
        cabs: ctx
            .people
            .iter()
            .zip([r.cab_fare_0, r.cab_fare_1])
            .map(|((name, _), fare)| Cab { name: name.clone(), fare: fare.filter(|_| !sealed).map(|f| f.to_string()).unwrap_or_default() })
            .collect(),
        planned_label: planned,
        from_name,
        lines: lines(ctx, r.id, who, if sealed { &[] } else { memories }),
    }
}

/// What the other one of us gets to know about a surprise.
fn hints(r: &HistoryRow, planned: &str, rain: bool) -> Vec<String> {
    let tags: Vec<&str> = r.tags.split(',').map(str::trim).collect();
    let mut h = vec![if planned.is_empty() { "When: still being planned".to_string() } else { format!("When: {planned}") }];
    h.push(format!("Keep {} free", Duration::parse(&r.duration).unwrap_or(Duration::Short).label().to_lowercase()));
    h.push(
        match r.setting.as_str() {
            "home" => "We're staying in",
            "apart" => "From your own place, on a call",
            "day_trip" => "We're heading out of town",
            "travel" => "Pack a bag",
            _ => "We're going out",
        }
        .into(),
    );
    if tags.contains(&"romantic") {
        h.push("Dress up a little".into());
    } else if r.energy == "high" || tags.contains(&"active") {
        h.push("Wear comfy shoes".into());
    }
    if tags.contains(&"outdoors") {
        h.push(if rain { "Bring an umbrella" } else { "Bring a light jacket" }.into());
    }
    if tags.contains(&"foodie") {
        h.push("Come hungry".into());
    }
    h
}

/// What the real costs say, in plain words.
fn cost_insight(l: &Learned) -> String {
    if l.cost_samples < db::MIN_SAMPLES {
        return String::new();
    }
    let pct = ((l.cost_ratio - 1.0) * 100.0).round() as i64;
    let how = match pct {
        -5..=5 => "usually cost about what the slips say".to_string(),
        p if p > 0 => format!("usually run about {p}% over the estimate"),
        p => format!("usually come in about {}% under the estimate", -p),
    };
    let mut line = format!("Your dates {how} ({} dates with real costs).", l.cost_samples);
    if l.cost.is_some() && !(-5..=5).contains(&pct) {
        line.push_str(" The jar now adjusts estimates to match.");
    }
    line
}

async fn savings_view(db: &SqlitePool, couple: &str, ctx: &Ctx, who: usize) -> Result<SavingsView, AppError> {
    let saved = db::savings_balance(db, couple).await?;
    let learned = learned(db, couple).await?;
    let mut slips: Vec<_> = db::visible_list(db, couple, Some(who)).await?.into_iter().filter(|a| a.est_cost > 0).collect();
    slips.sort_by_key(|a| std::cmp::Reverse(a.effective_cost(learned.cost)));
    let goal_id: Option<i64> = db::setting(db, couple, "saving_for").await?.and_then(|v| v.parse().ok());
    let goal = match goal_id {
        Some(id) => db::visible_get(db, couple, who, id).await?.map(|a| {
            let cost = a.effective_cost(learned.cost);
            Goal {
                title: ctx.personalize(&a.title),
                cost: ctx.money(cost),
                pct: if cost > 0 { ((saved * 100) / cost).clamp(0, 100) as u8 } else { 100 },
                to_go: if saved >= cost { String::new() } else { format!("{} to go", ctx.money(cost - saved)) },
                ready: saved >= cost,
            }
        }),
        None => None,
    };
    let day = |on: &str| planned_label(&format!("{on} 00:00 0")).split(',').next().unwrap_or_default().to_string();
    Ok(SavingsView {
        saved: if saved == 0 { format!("{}0", ctx.currency) } else { ctx.money(saved) },
        saved_raw: saved,
        goal,
        choices: slips
            .iter()
            .enumerate()
            .filter(|(i, a)| *i < GOAL_CHOICES || goal_id == Some(a.id))
            .map(|(_, a)| Opt {
                value: a.id.to_string(),
                label: format!("{} · {}", ctx.personalize(&a.title), ctx.money(a.effective_cost(learned.cost))),
                selected: goal_id == Some(a.id),
            })
            .collect(),
        recent: db::savings_recent(db, couple)
            .await?
            .into_iter()
            .map(|(id, amount, kind, note, on)| SavingLine {
                id,
                amount: if amount < 0 { format!("−{}", ctx.money(-amount)) } else { format!("+{}", ctx.money(amount)) },
                note: match kind.as_str() {
                    "kept" => format!("Kept: {}", ctx.personalize(&note)),
                    "back" => format!("Back from {}", ctx.personalize(&note)),
                    _ => note,
                },
                on: day(&on),
                removable: kind == "add",
            })
            .collect(),
    })
}

/// `GET /kept`
pub async fn page(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<HistoryView>, AppError> {
    let j = joy(&s, &user_id).await?;
    let (db, couple, ctx, who) = (&s.db, j.couple.as_str(), &j.ctx, Some(j.who));
    let rain = super::weather_for(ctx).tonight().await.is_some_and(|p| p >= weather::LIKELY_PERCENT);
    let (done, skipped): (Vec<_>, Vec<_>) = db::history(db, couple).await?.into_iter().partition(|r| r.outcome == "did_it");
    // A slip that's now a sealed surprise for this viewer mustn't show up in the
    // "put back" footnote either, or it would give the surprise away.
    let sealed_titles: Vec<&str> = done.iter().filter(|r| sealed_for(r, who)).map(|r| r.title.as_str()).collect();
    let skipped: Vec<SkippedView> = skipped
        .iter()
        .filter(|r| !sealed_titles.contains(&r.title.as_str()))
        .map(|r| SkippedView { title: ctx.personalize(&r.title), day_label: r.day_label() })
        .collect();
    let pairs = db::pairs(db, couple).await?;
    let memories = db::memories(db, couple).await?;
    let mut pics: HashMap<i64, Vec<DatePhoto>> = HashMap::new();
    for p in db::all_photos(db, couple).await? {
        pics.entry(p.history_id).or_default().push(p);
    }
    let mut months: Vec<Month> = vec![];
    for row in &done {
        let label = row.month_label();
        let v = view(ctx, row, who, rain, pair_ids(row, &done, &pairs), &memories, pics.get(&row.id).map_or(&[], Vec::as_slice));
        match months.last_mut() {
            Some(m) if m.label == label => m.items.push(v),
            _ => {
                let href = format!("/joy/year/{}/{}", row.day.get(..4).unwrap_or(""), row.day.get(5..7).unwrap_or("").trim_start_matches('0'));
                months.push(Month { label, href, items: vec![v] })
            }
        }
    }
    let other_spend = db::other_spending_this_month(db, couple)
        .await?
        .into_iter()
        .map(|(id, amount, note, on)| OtherSpend {
            id,
            amount: ctx.money(amount),
            note,
            on: planned_label(&format!("{on} 00:00 0")).split(',').next().unwrap_or_default().into(),
        })
        .collect();
    Ok(Json(HistoryView {
        months,
        kept: done.len(),
        skipped,
        budget: budget_view(db, couple, ctx).await?,
        me_name: Me::new(ctx, who).name,
        currency: ctx.currency.clone(),
        cost_insight: cost_insight(&learned(db, couple).await?),
        past: past_months(db, couple, ctx).await?,
        other_spend,
        savings: savings_view(db, couple, ctx, j.who).await?,
    }))
}

// ── rating ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RateBody {
    #[serde(default, deserialize_with = "flex")]
    rating: String,
    #[serde(default, deserialize_with = "flex")]
    note: String,
}

/// `POST /history/{id}`: stars and a note.
pub async fn rate(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>, Json(f): Json<RateBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    open_row(&s.db, &j.couple, id, Some(j.who)).await?;
    db::rate(&s.db, &j.couple, id, f.rating.trim().parse().ok(), &f.note).await?;
    Ok(go(kept_to(id)))
}

// ── when is it? ───────────────────────────────────────────────────────────

/// What "change time" shows afterwards.
#[derive(Debug, Serialize)]
pub struct PlannedView {
    /// Same comma-separated ids that were posted, so the form can be changed again.
    pub ids: String,
    /// "Sat 18 Oct, 7:00 pm", or empty after clearing.
    pub label: String,
    /// Value for `<input type="datetime-local">`.
    pub input: String,
    /// Signed invite for the person asking.
    pub ics_url: String,
    /// The invite to send the other one of us: signed for them, so a surprise stays sealed.
    pub send_url: String,
    /// The other one of us ("Neha").
    pub partner: String,
}

#[derive(Deserialize)]
pub struct WhenBody {
    /// Comma-separated history ids (two for a pair); an array works too.
    #[serde(deserialize_with = "flex_list")]
    ids: String,
    /// From `<input type="datetime-local">`: "2026-10-18T19:00", or empty to clear.
    #[serde(default, deserialize_with = "flex")]
    at: String,
}

/// `POST /history/when`
pub async fn when(State(s): State<AppState>, Extension((user_id, _)): User, Json(f): Json<WhenBody>) -> Result<Json<PlannedView>, AppError> {
    let j = joy(&s, &user_id).await?;
    let (db, couple) = (&s.db, j.couple.as_str());
    let ids: Vec<i64> = f.ids.split(',').filter_map(|i| i.trim().parse().ok()).collect();
    let Some(&first) = ids.first() else { return Err(missing()) };
    // Every id must be ours and not a surprise sealed for us.
    let mut rows = vec![];
    for id in &ids {
        rows.push(open_row(db, couple, *id, Some(j.who)).await?);
    }
    let local = super::parse_local(&f.at);
    if !f.at.trim().is_empty() && local.is_none() {
        return Err(unprocessable("That date and time didn't look right."));
    }
    // Moved to another day: an office Saturday or a day off was about the old date.
    let old_day = rows[0].planned_local.as_deref().and_then(|l| l.get(..10));
    if old_day != local.as_deref().and_then(|l| l.get(..10)) {
        db::set_day_kind(db, couple, &ids, "").await?;
    }
    db::set_planned(db, couple, &ids, local.as_deref()).await?;
    let planned_local = db::history_row(db, couple, first).await?.and_then(|r| r.planned_local);
    let planned = planned_local.is_some();
    Ok(Json(PlannedView {
        ids: f.ids,
        label: planned_local.as_deref().map(planned_label).unwrap_or_default(),
        input: planned_local.as_deref().map(planned_input).unwrap_or_default(),
        ics_url: if planned { public_link(dayplan::ics_url(first, Some(j.who))) } else { String::new() },
        send_url: if planned { public_link(dayplan::ics_url(first, Some(1 - j.who))) } else { String::new() },
        partner: Me::new(&j.ctx, Some(j.who)).partner,
    }))
}

// ── the invite (public, signed) ───────────────────────────────────────────

#[derive(Deserialize)]
pub struct SignedQuery {
    who: Option<usize>,
    exp: Option<i64>,
    sig: Option<String>,
}

/// The couple a history row belongs to (the signed public endpoints have no login to ask).
async fn couple_of_history(db: &SqlitePool, id: i64) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar("SELECT couple_id FROM joy_history WHERE id = ?").bind(id).fetch_optional(db).await?)
}

/// `GET /history/{id}/invite.ics` (public): needs the `exp`/`sig` [`dayplan::ics_url`] made over
/// `/api/dates/history/{id}/invite.ics` plus `?who=N` when the link names a viewer. Sealed for
/// that viewer (or for nobody-in-particular), a surprise gets hints only.
pub async fn invite(State(s): State<AppState>, Path(id): Path<i64>, Query(q): Query<SignedQuery>) -> Result<Response, AppError> {
    let base = format!("/api/dates/history/{id}/invite.ics");
    let signed = match q.who {
        Some(w) => format!("{base}?who={w}"),
        None => base,
    };
    let (Some(exp), Some(sig)) = (q.exp, q.sig.as_deref()) else { return Err(forbidden("This link is missing its signature.")) };
    if !verify(&signed, exp, sig) {
        return Err(forbidden("This link has expired or isn't valid."));
    }
    let who = q.who.filter(|w| *w < 2);
    let Some(couple) = couple_of_history(&s.db, id).await? else { return Err(missing()) };
    let Some(r) = db::history_row(&s.db, &couple, id).await?.filter(|r| r.planned_at.is_some()) else { return Err(missing()) };
    let ctx = load_ctx(&s.db, &couple).await?;
    let sealed = sealed_for(&r, who);
    let from = r.drawn_by.and_then(|d| ctx.people.get(d as usize)).map(|p| p.0.as_str()).unwrap_or("us");
    let (summary, location, description) = if sealed {
        (format!("A surprise from {from}"), String::new(), hints(&r, "", false).join("\n"))
    } else {
        let maps = r.lat.zip(r.lng).map(|(a, b)| format!("\nhttps://www.google.com/maps/search/?api=1&query={a},{b}"));
        (ctx.personalize(&r.title), r.place.clone(), format!("{}{}", ctx.personalize(&r.notes), maps.unwrap_or_default()))
    };
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(internal)?.as_secs() as i64;
    // With a day-of plan: the timeline goes in the notes and the reminder is for when to leave.
    let plan = if sealed { None } else { dayplan::build(&s.db, &couple, &ctx, &r, who).await? };
    let (description, alarm) = match &plan {
        Some(p) if !p.steps.is_empty() => {
            let lines: Vec<String> = p.steps.iter().map(|st| format!("{}  {}", st.at, st.what)).collect();
            let before = p.leave_before(who);
            (format!("{}\n\n{description}", lines.join("\n")), if before > 0 { (before + 15, "Time to get ready to leave") } else { (120, "Date soon") })
        }
        _ => (description, (120, "Date soon")),
    };
    let body = ics::render(&ics::Event {
        uid: format!("joy-{id}@joy"),
        start: r.planned_at.unwrap_or(now),
        minutes: ics::minutes_for(&r.duration),
        summary: &summary,
        location: &location,
        description: &description,
        now,
        alarm,
    });
    Ok((
        [
            (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"joy-date.ics\""),
            (header::CACHE_CONTROL, "private, no-store"),
        ],
        body,
    )
        .into_response())
}

// ── reveal, delete, cost, line ────────────────────────────────────────────

/// `POST /history/{id}/reveal`: only whoever planned the surprise may spoil it.
pub async fn reveal(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let row = row_of(&s.db, &j.couple, id).await?;
    if row.drawn_by != Some(j.who as i64) {
        return Err(forbidden("Only whoever planned it can reveal it."));
    }
    db::reveal(&s.db, &j.couple, id).await?;
    Ok(go(kept_to(id)))
}

/// `POST /history/{id}/delete`
pub async fn delete(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    // A surprise still sealed for you isn't yours to throw away.
    open_row(&s.db, &j.couple, id, Some(j.who)).await?;
    let pics = db::photos_of(&s.db, &j.couple, id).await?;
    db::delete_history(&s.db, &j.couple, id).await?;
    for p in pics {
        if let Some(path) = photos::date_path(&j.ctx.photos_dir, &p.file) {
            let _ = tokio::fs::remove_file(path).await;
        }
    }
    Ok(go("/joy/kept"))
}

#[derive(Deserialize)]
pub struct CostBody {
    /// What it really cost; blank clears it.
    #[serde(default, deserialize_with = "flex")]
    actual: String,
    /// "1" (or true): also set the slip's estimate to this.
    #[serde(default, deserialize_with = "flex")]
    update_estimate: String,
    /// Real one-way cab fares per person (blank = didn't take one / don't know).
    #[serde(default, deserialize_with = "flex")]
    cab_0: String,
    #[serde(default, deserialize_with = "flex")]
    cab_1: String,
}

/// `POST /history/{id}/cost`
pub async fn set_cost(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>, Json(f): Json<CostBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let (db, couple, ctx) = (&s.db, j.couple.as_str(), &j.ctx);
    let row = open_row(db, couple, id, Some(j.who)).await?;
    let amount = |v: &str| -> Result<Option<i64>, String> {
        let raw: String = v.trim().trim_start_matches(ctx.currency.as_str()).chars().filter(|c| !matches!(c, ',' | ' ')).collect();
        if raw.is_empty() {
            return Ok(None);
        }
        let n = raw.parse::<i64>().map_err(|_| "That amount didn't look right.".to_string())?;
        crate::validate::rupees(n).map(Some)
    };
    let (actual, cab_0, cab_1) = match (amount(&f.actual), amount(&f.cab_0), amount(&f.cab_1)) {
        (Ok(a), Ok(b), Ok(c)) => (a, b, c),
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => return Err(unprocessable(&e)),
    };
    db::set_actual(db, couple, id, actual, f.update_estimate == "1").await?;
    // A regular night is this same date: its own record of the cost follows.
    if let Some(a) = actual {
        sqlx::query("UPDATE joy_regular_visits SET amount = ? WHERE history_id = ?").bind(a).bind(id).execute(db).await?;
    }
    // Each real fare is saved next to what the model would have said for that trip (uncalibrated),
    // so later changes to rates or home locations can't skew what we learn.
    let km = ctx.distances(row.lat.zip(row.lng));
    for (person, fare) in [(0, cab_0), (1, cab_1)] {
        let model = km[person].map(|k| crate::joy::travel::cab(k).rupees);
        db::set_cab_fare(db, couple, id, person, fare.filter(|f| *f > 0), model).await?;
    }
    Ok(go(kept_to(id)))
}

#[derive(Deserialize)]
pub struct DayBody {
    /// "YYYY-MM-DD": the day it really happened. Never in the future.
    #[serde(default, deserialize_with = "flex")]
    on: String,
}

/// `POST /history/{id}/day`: backdate (or re-date) a kept date, like a journal entry.
pub async fn set_day(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>, Json(f): Json<DayBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    open_row(&s.db, &j.couple, id, Some(j.who)).await?;
    let on = time::Date::parse(f.on.trim(), time::macros::format_description!("[year]-[month]-[day]"))
        .map_err(|_| unprocessable("That date didn't look right."))?;
    if on > crate::handlers::common::today_local() {
        return Err(unprocessable("Pick a day that has already happened."));
    }
    if on.year() < 2000 {
        return Err(unprocessable("That date is too far back."));
    }
    if !db::set_kept_day(&s.db, &j.couple, id, &on.to_string()).await? {
        return Err(unprocessable("That date can't move there: it may already have a night on that day."));
    }
    Ok(go(kept_to(id)))
}

#[derive(Deserialize)]
pub struct LineBody {
    #[serde(default, deserialize_with = "flex")]
    line: String,
}

/// `POST /history/{id}/line`: write your own line on a kept date.
pub async fn set_line(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>, Json(f): Json<LineBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let row = open_row(&s.db, &j.couple, id, Some(j.who)).await?;
    if row.outcome != "did_it" {
        return Err(missing());
    }
    let line: String = f.line.chars().take(MAX_LINE).collect();
    db::set_memory(&s.db, &j.couple, id, j.who, &line).await?;
    Ok(go(kept_to(id)))
}

// ── photos ────────────────────────────────────────────────────────────────

/// `POST /history/{id}/photo`: multipart field `photo`, added after the date's other photos.
/// 422 + a message when it can't be used or the date is full.
pub async fn upload_photo(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<i64>,
    mut form: Multipart,
) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let row = open_row(&s.db, &j.couple, id, Some(j.who)).await?;
    if row.outcome != "did_it" {
        return Err(missing());
    }
    if db::photos_of(&s.db, &j.couple, id).await?.len() as i64 >= db::MAX_PHOTOS_PER_DATE {
        return Err(unprocessable(&full_message()));
    }
    let mut bytes = None;
    while let Ok(Some(field)) = form.next_field().await {
        if field.name() == Some("photo") {
            match field.bytes().await {
                Ok(b) => bytes = Some(b),
                Err(_) => return Err(unprocessable("That photo is over 15 MB. Try a smaller one.")),
            }
        }
    }
    let Some(bytes) = bytes.filter(|b| !b.is_empty()) else {
        return Err(unprocessable("That file doesn't look like a photo."));
    };
    let jpeg = match photos::process_limited(bytes.to_vec()).await.map_err(internal)? {
        Ok(j) => j,
        Err(e) => return Err(unprocessable(e.0)),
    };
    tokio::fs::create_dir_all(&j.ctx.photos_dir).await.map_err(internal)?;
    let file = photos::new_date_file(id);
    let path = photos::date_path(&j.ctx.photos_dir, &file).ok_or_else(|| internal("bad photo file name"))?;
    tokio::fs::write(&path, jpeg).await.map_err(internal)?;
    if db::add_photo(&s.db, &j.couple, id, &file).await?.is_none() {
        // Filled up by the other one of us while this one was going up.
        let _ = tokio::fs::remove_file(&path).await;
        return Err(unprocessable(&full_message()));
    }
    Ok(go(kept_to(id)))
}

fn full_message() -> String {
    format!("This date already has {} photos. Remove one to add another.", db::MAX_PHOTOS_PER_DATE)
}

/// `POST /history/{id}/photos/{photo_id}/delete`: takes that one photo off the date.
pub async fn delete_photo(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path((id, photo_id)): Path<(i64, i64)>,
) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    open_row(&s.db, &j.couple, id, Some(j.who)).await?;
    if let Some(file) = db::remove_photo(&s.db, &j.couple, id, photo_id).await?
        && let Some(path) = photos::date_path(&j.ctx.photos_dir, &file)
    {
        let _ = tokio::fs::remove_file(path).await;
    }
    Ok(go(kept_to(id)))
}

/// `GET /history/{id}/photos/{photo_id}?exp=..&sig=..` (public): the signed link [`photo_url`]
/// made. A sealed surprise (for the viewer the link names, else for anyone) is a 404.
pub async fn photo(State(s): State<AppState>, Path((id, photo_id)): Path<(i64, i64)>, Query(q): Query<SignedQuery>) -> Result<Response, AppError> {
    let (Some(exp), Some(sig)) = (q.exp, q.sig.as_deref()) else {
        return Err(forbidden("This link is missing its signature."));
    };
    if !verify(&photo_path(id, photo_id, q.who), exp, sig) {
        return Err(forbidden("This link has expired or isn't valid."));
    }
    let who = q.who.filter(|w| *w < 2);
    let Some((couple, pic)) = db::photo_by_id(&s.db, id, photo_id).await? else { return Err(missing()) };
    if db::history_row(&s.db, &couple, id).await?.is_none_or(|r| sealed_for(&r, who)) {
        return Err(missing());
    }
    let ctx = load_ctx(&s.db, &couple).await?;
    let Some(path) = photos::date_path(&ctx.photos_dir, &pic.file) else { return Err(missing()) };
    let Ok(bytes) = tokio::fs::read(path).await else { return Err(missing()) };
    // A photo id always means the same picture, so it may be cached for as long as the link lives.
    Ok(([(header::CONTENT_TYPE, "image/jpeg"), (header::CACHE_CONTROL, "private, max-age=3000")], bytes).into_response())
}

// ── monthly budget ────────────────────────────────────────────────────────

/// A typed whole-rupee amount: blank or zero is "none"; anything else must be a number within the
/// shared Dates cap (`rupees`), so a huge or negative value is a 422 instead of a silent no-op.
fn money_in(v: &str) -> Result<Option<i64>, String> {
    let t = v.trim().replace(',', "");
    if t.is_empty() {
        return Ok(None);
    }
    let n = t.parse::<i64>().map_err(|_| "That amount didn't look right.".to_string())?;
    if n == 0 {
        return Ok(None);
    }
    crate::validate::rupees(n).map(Some)
}

#[derive(Deserialize)]
pub struct BudgetBody {
    /// Blank clears it.
    #[serde(default, deserialize_with = "flex")]
    amount: String,
    /// Where to go afterwards ("/joy" or "/joy/kept").
    #[serde(default, deserialize_with = "flex")]
    back: String,
}

/// `POST /budget`
pub async fn set_budget(State(s): State<AppState>, Extension((user_id, _)): User, Json(f): Json<BudgetBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let value = money_in(&f.amount).map_err(|e| unprocessable(&e))?.map(|a| a.to_string());
    db::set_setting(&s.db, &j.couple, "monthly_budget", value.as_deref()).await?;
    Ok(go(if f.back.starts_with('/') && !f.back.starts_with("//") { f.back } else { "/joy/kept".into() }))
}

#[derive(Deserialize)]
pub struct RolloverBody {
    /// "roll" carries last month's leftover into this month, "save" puts it in the savings
    /// jar; anything else starts fresh.
    #[serde(default, deserialize_with = "flex")]
    choice: String,
}

/// `POST /budget/rollover`
pub async fn rollover(State(s): State<AppState>, Extension((user_id, _)): User, Json(f): Json<RolloverBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let choice = match f.choice.as_str() {
        "roll" => Leftover::Roll,
        "save" => Leftover::Save,
        _ => Leftover::Fresh,
    };
    settle_month(&s.db, &j.couple, choice).await?;
    Ok(go("/joy/kept"))
}

#[derive(Deserialize)]
pub struct SpendBody {
    #[serde(default, deserialize_with = "flex")]
    amount: String,
    #[serde(default, deserialize_with = "flex")]
    note: String,
    /// "YYYY-MM-DD"; blank means today.
    #[serde(default, deserialize_with = "flex")]
    on: String,
    /// Optional: the kept date this money was about. Not ours is a 404; sealed for you is a 403.
    #[serde(default, deserialize_with = "flex")]
    history_id: String,
}

#[derive(Deserialize)]
pub struct CustomBody {
    #[serde(default, deserialize_with = "flex")]
    title: String,
    /// Where, optional.
    #[serde(default, deserialize_with = "flex")]
    place: String,
    /// One of the slip settings; blank is "local".
    #[serde(default, deserialize_with = "flex")]
    setting: String,
    /// "YYYY-MM-DD"; blank means today. Never in the future.
    #[serde(default, deserialize_with = "flex")]
    on: String,
    /// What it cost, in rupees; blank for unknown.
    #[serde(default, deserialize_with = "flex")]
    cost: String,
}

/// `POST /history/custom`: a date we had that never was a slip, straight into Kept, where it
/// takes photos, a rating and lines like any other. Its slip is archived, so the jar never draws it.
pub async fn add_custom(State(s): State<AppState>, Extension((user_id, _)): User, Json(f): Json<CustomBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let title = f.title.trim();
    if title.is_empty() || title.chars().count() > 80 {
        return Err(unprocessable("Give the date a name, up to 80 characters."));
    }
    let place: String = f.place.trim().chars().take(80).collect();
    let setting = match f.setting.trim() {
        "" => "local",
        s if crate::joy::picker::SETTINGS.iter().any(|(k, _)| *k == s) => s,
        _ => return Err(unprocessable("That kind of date isn't one we know.")),
    };
    let today = crate::handlers::common::today_local();
    let on = match f.on.trim() {
        "" => today,
        d => time::Date::parse(d, time::macros::format_description!("[year]-[month]-[day]"))
            .map_err(|_| unprocessable("That date didn't look right."))?,
    };
    if on > today {
        return Err(unprocessable("Pick a day that has already happened. Plan future dates from the jar."));
    }
    if on.year() < 2000 {
        return Err(unprocessable("That date is too far back."));
    }
    let cost = money_in(&f.cost).map_err(|e| unprocessable(&e))?;
    let id = db::keep_custom(&s.db, &j.couple, Some(j.who), title, &place, setting, &on.to_string(), cost).await?;
    Ok(go(kept_to(id)))
}

/// If the body names a kept date, it must be ours and not sealed from us.
async fn guard_history(j: &super::Joy, db: &SqlitePool, history_id: &str) -> Result<(), AppError> {
    match history_id.trim().parse::<i64>() {
        Ok(id) => open_row(db, &j.couple, id, Some(j.who)).await.map(|_| ()),
        Err(_) => Ok(()),
    }
}

/// `POST /budget/spend`: money spent on a date that wasn't one of our slips.
pub async fn add_spend(State(s): State<AppState>, Extension((user_id, _)): User, Json(f): Json<SpendBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    guard_history(&j, &s.db, &f.history_id).await?;
    if let Some(amount) = money_in(&f.amount).map_err(|e| unprocessable(&e))? {
        db::add_other_spending(&s.db, &j.couple, amount, &f.note, &f.on).await?;
    }
    Ok(go("/joy/kept"))
}

/// `POST /budget/spend/{id}/delete`
pub async fn delete_spend(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    db::delete_other_spending(&s.db, &j.couple, id).await?;
    Ok(go("/joy/kept"))
}

// ── the savings jar ───────────────────────────────────────────────────────

const SAVINGS_TO: &str = "/joy/kept#k-savings";

#[derive(Deserialize)]
pub struct GoalBody {
    /// A slip's id; blank stops saving for anything (the money stays in the jar).
    #[serde(default, deserialize_with = "flex")]
    id: String,
}

/// `POST /savings/goal`
pub async fn set_goal(State(s): State<AppState>, Extension((user_id, _)): User, Json(f): Json<GoalBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let id = match f.id.trim().parse::<i64>() {
        Ok(id) => {
            super::unsealed(&s.db, &j.couple, j.who, id).await?;
            db::visible_get(&s.db, &j.couple, j.who, id).await?.map(|a| a.id.to_string())
        }
        Err(_) => None,
    };
    db::set_setting(&s.db, &j.couple, "saving_for", id.as_deref()).await?;
    Ok(go(SAVINGS_TO))
}

/// `POST /savings/add`
pub async fn add_saving(State(s): State<AppState>, Extension((user_id, _)): User, Json(f): Json<SpendBody>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    guard_history(&j, &s.db, &f.history_id).await?;
    if let Some(amount) = money_in(&f.amount).map_err(|e| unprocessable(&e))? {
        db::add_saving(&s.db, &j.couple, amount, "add", &f.note).await?;
    }
    Ok(go(SAVINGS_TO))
}

/// `POST /savings/{id}/delete`
pub async fn delete_saving(State(s): State<AppState>, Extension((user_id, _)): User, Path(id): Path<i64>) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    db::delete_saving(&s.db, &j.couple, id).await?;
    Ok(go(SAVINGS_TO))
}

/// `POST /savings/keep`: the jar has enough, so keep the slip we were saving for, paid from it.
pub async fn keep_goal(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    let (db, couple) = (&s.db, j.couple.as_str());
    let Some(a) = db::setting(db, couple, "saving_for").await?.and_then(|v| v.parse().ok()) else { return Ok(go(SAVINGS_TO)) };
    let Some(a) = db::visible_get(db, couple, j.who, a).await? else { return Ok(go(SAVINGS_TO)) };
    let cost = a.effective_cost(learned(db, couple).await?.cost);
    Ok(match db::keep_from_savings(db, couple, a.id, cost, Some(j.who)).await? {
        Some(h) => go(kept_to(h)),
        None => go(SAVINGS_TO),
    })
}

// ── routes ────────────────────────────────────────────────────────────────

/// Behind login: Kept, its edits, the budget and the savings jar, plus the day-of plan's
/// Plan B swap and job ticking.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/kept", get(page))
        .route("/history/when", post(when))
        .route("/history/custom", post(add_custom))
        .route("/history/{id}", post(rate))
        .route("/history/{id}/reveal", post(reveal))
        .route("/history/{id}/delete", post(delete))
        .route("/putbacks/clear", post(clear_putbacks))
        .route("/history/{id}/cost", post(set_cost))
        .route("/history/{id}/day", post(set_day))
        .route("/history/{id}/line", post(set_line))
        .route("/history/{id}/photo", post(upload_photo).layer(DefaultBodyLimit::max(photos::MAX_UPLOAD_BYTES + UPLOAD_SLACK)))
        .route("/history/{id}/photos/{photo_id}/delete", post(delete_photo))
        .route("/history/{id}/plan-b", post(dayplan::swap_plan_b))
        .route("/tasks/{id}/done", post(dayplan::task_done))
        .route("/budget", post(set_budget))
        .route("/budget/rollover", post(rollover))
        .route("/budget/spend", post(add_spend))
        .route("/budget/spend/{id}/delete", post(delete_spend))
        .route("/savings/goal", post(set_goal))
        .route("/savings/add", post(add_saving))
        .route("/savings/{id}/delete", post(delete_saving))
        .route("/savings/keep", post(keep_goal))
}

/// `POST /putbacks/clear`: clears the "put back" list that rerolling leaves in Kept.
pub async fn clear_putbacks(State(s): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Go>, AppError> {
    let j = joy(&s, &user_id).await?;
    db::clear_putbacks(&s.db, &j.couple).await?;
    Ok(go("/joy/kept"))
}

/// No login, but each needs the signature its link carries.
pub fn public_router() -> Router<AppState> {
    Router::new().route("/history/{id}/invite.ics", get(invite)).route("/history/{id}/photos/{photo_id}", get(photo))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64, activity_id: i64, picked_at: i64) -> HistoryRow {
        HistoryRow {
            id,
            activity_id,
            picked_at,
            title: "Dinner".into(),
            notes: String::new(),
            setting: "local".into(),
            place: "Cafe".into(),
            duration: "short".into(),
            energy: "med".into(),
            tags: "foodie, outdoors".into(),
            est_cost: 1000,
            actual_cost: None,
            cab_fare_0: None,
            cab_fare_1: None,
            lat: None,
            lng: None,
            day: "2026-10-18".into(),
            outcome: "did_it".into(),
            rating: None,
            note: String::new(),
            drawn_by: Some(0),
            planned_at: None,
            planned_local: None,
            surprise: true,
            revealed: false,
            has_photo: false,
            plan_b: None,
            day_kind: String::new(),
        }
    }

    #[test]
    fn hints_give_the_shape_of_a_surprise_and_nothing_else() {
        let h = hints(&row(1, 1, 0), "Sat 18 Oct, 7:00 pm", true);
        assert_eq!(h[0], "When: Sat 18 Oct, 7:00 pm");
        assert!(h.contains(&"Bring an umbrella".to_string()) && h.contains(&"Come hungry".to_string()));
        assert!(!h.join(" ").contains("Dinner") && !h.join(" ").contains("Cafe"));
        assert_eq!(hints(&row(1, 1, 0), "", false)[0], "When: still being planned");
    }

    #[test]
    fn pairs_move_together_but_strangers_do_not() {
        let done = [row(1, 10, 100), row(2, 11, 102), row(3, 12, 103), row(4, 10, 500)];
        assert_eq!(pair_ids(&done[0], &done, &[(10, 11)]), "1,2");
        assert_eq!(pair_ids(&done[1], &done, &[(10, 11)]), "2,1");
        assert_eq!(pair_ids(&done[2], &done, &[(10, 11)]), "3");
        assert_eq!(pair_ids(&done[3], &done, &[(10, 11)]), "4", "same slip, much later");
    }

    #[test]
    fn lines_stay_sealed_until_both_are_written() {
        let ctx = Ctx::for_tests();
        let mine = vec![(7, 0, "ours".to_string())];
        let l = lines(&ctx, 7, Some(1), &mine);
        assert!(l.theirs_waiting && l.theirs.is_empty() && l.mine.is_empty());
        let both = vec![(7, 0, "ours".to_string()), (7, 1, "mine".to_string())];
        let l = lines(&ctx, 7, Some(1), &both);
        assert_eq!((l.mine.as_str(), l.theirs.as_str(), l.theirs_waiting), ("mine", "ours", false));
        assert!(!lines(&ctx, 7, None, &both).can_write);
    }

    #[test]
    fn cost_insight_waits_for_enough_dates() {
        assert_eq!(cost_insight(&Learned { cost_samples: 1, cost_ratio: 2.0, ..Default::default() }), "");
        let l = Learned { cost: Some(1.4), cost_samples: 5, cost_ratio: 1.4, ..Default::default() };
        assert_eq!(cost_insight(&l), "Your dates usually run about 40% over the estimate (5 dates with real costs). The jar now adjusts estimates to match.");
    }

    #[test]
    fn photo_links_sign_the_viewer_only_for_unrevealed_surprises() {
        let mut r = row(9, 1, 0);
        let url = photo_url(&r, 4, Some(0));
        assert!(url.starts_with("/api/public/dates/history/9/photos/4?who=0&exp="), "{url}");
        r.revealed = true;
        let url = photo_url(&r, 4, Some(0));
        assert!(url.starts_with("/api/public/dates/history/9/photos/4?exp="), "{url}");
        assert!(!url.contains("who"));
    }

    #[test]
    fn a_date_shows_all_its_photos_unless_sealed() {
        let ctx = Ctx::for_tests();
        let pics = [DatePhoto { id: 4, history_id: 9, file: "9.jpg".into() }, DatePhoto { id: 7, history_id: 9, file: "9-ab.jpg".into() }];
        let mut r = row(9, 1, 0);
        r.revealed = true;
        let v = view(&ctx, &r, Some(1), false, "9".into(), &[], &pics);
        assert_eq!(v.photos.iter().map(|p| p.id).collect::<Vec<_>>(), vec![4, 7]);
        assert!(v.photo_url.as_deref().is_some_and(|u| u.contains("/photos/4?")), "the first photo is the cover");
        assert!(v.can_add_photo);
        r.revealed = false;
        let v = view(&ctx, &r, Some(1), false, "9".into(), &[], &pics);
        assert!(v.sealed && v.photos.is_empty() && v.photo_url.is_none() && !v.can_add_photo);
    }

    #[test]
    fn amounts_accept_commas_and_refuse_zero() {
        assert_eq!(money_in(" 6,000 "), Ok(Some(6000)));
        assert_eq!(money_in("0"), Ok(None));
        assert_eq!(money_in(""), Ok(None));
        assert!(money_in("-5").is_err());
        assert!(money_in("1000000000000000").is_err());
        assert!(money_in("abc").is_err());
    }
}
