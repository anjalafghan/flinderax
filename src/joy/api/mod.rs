//! Dates (Joy) as a JSON API. Joy's server logic is kept as it was; only the transport changed:
//! each Askama template became a `Serialize` view struct, each form post a JSON body, and each
//! redirect `{"ok": true, "to": "/joy/..."}` (the SPA path the redirect pointed at).
//!
//! Part 1 lives in `decide`, `middle`, `wishlist`, `envelope`, plus `dayplan` and `regular`, which
//! hold the non-handler halves of Joy's modules of the same names (decide and envelope need them).
//! Parts 2/3 extend those two files with their handlers.
//!
//! SPA paths used in links this module builds:
//! - envelope link (what goes to WhatsApp): `{PUBLIC_URL}/joy/e/{token}`; the SPA page reads
//!   `GET /api/public/dates/e/{token}`.
//! - calendar invite: signed `GET /api/dates/history/{id}/invite.ics?who=N&exp=..&sig=..`
//!   (see [`dayplan::ics_url`]).
//! - a regular's night photo: signed over `/api/dates/nights/{id}/photo?v=..` and served, public,
//!   at `GET /api/public/dates/nights/{id}/photo?v=..&exp=..&sig=..` (see [`regular::public_link`]).

// part 2/3: add `pub mod history; pub mod month; pub mod regular_routes; pub mod year;` etc. here
pub mod dayplan;
pub mod decide;
pub mod envelope;
pub mod history;
pub mod middle;
pub mod month;
pub mod regular;
pub mod wishlist;
pub mod year;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{http::StatusCode, Json};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::{
    handlers::{
        common::AppError,
        couple::require_couple,
    },
    joy::{
        ctx::{self, Ctx},
        db::Activity,
        picker,
        weather::{self, Weather},
    },
    models::AppState,
};

/// Everything a Dates request needs to know about who is asking.
pub struct Joy {
    pub couple: String,
    /// The caller's slot in the couple (0/1): what Joy's "who is holding the phone" cookie was.
    pub who: usize,
    pub ctx: Ctx,
}

/// The caller's couple, slot and settings. A 403 for someone who hasn't started or joined one.
pub async fn joy(state: &AppState, user_id: &str) -> Result<Joy, AppError> {
    let (couple, who) = require_couple(&state.db, user_id).await?;
    let ctx = load_ctx(&state.db, &couple).await?;
    Ok(Joy { couple, who, ctx })
}

/// A couple's settings with the usual public URL and photo root.
pub async fn load_ctx(db: &SqlitePool, couple: &str) -> Result<Ctx, AppError> {
    let public_url = std::env::var("PUBLIC_URL").unwrap_or_else(|_| "https://joy-jar.fly.dev".into());
    Ok(ctx::load(db, couple, &public_url, &crate::import_joy::photos_root()).await?)
}

pub fn internal(e: impl std::fmt::Display) -> AppError {
    AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

/// `{"ok": true, "to": "/joy/slips"}`: what a Joy redirect became.
#[derive(Debug, Serialize)]
pub struct Go {
    pub ok: bool,
    pub to: String,
}

pub fn go(to: impl Into<String>) -> Json<Go> {
    Json(Go { ok: true, to: to.into() })
}

// ── lenient JSON fields ───────────────────────────────────────────────────
//
// Joy's forms were all strings. A JSON client may send a number, a bool or null instead; these
// turn any of them back into the string Joy's logic expects ("" for null/false, "1" for true).

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Flex {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null(()),
}

fn flex_text(v: Flex) -> String {
    match v {
        Flex::Str(s) => s,
        Flex::Int(i) => i.to_string(),
        Flex::Float(f) if f.fract() == 0.0 && f.abs() < 1e15 => (f as i64).to_string(),
        Flex::Float(f) => f.to_string(),
        Flex::Bool(true) => "1".into(),
        Flex::Bool(false) | Flex::Null(()) => String::new(),
    }
}

/// A string, number, bool or null, as a string. For `#[serde(default, deserialize_with = ...)]`.
pub fn flex<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(flex_text(<Flex as serde::Deserialize>::deserialize(d)?))
}

/// A list of ids as `"1,2,3"`: accepts that string or an array of numbers/strings.
pub fn flex_list<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum List {
        Many(Vec<Flex>),
        One(Flex),
    }
    Ok(match <List as serde::Deserialize>::deserialize(d)? {
        List::Many(v) => v.into_iter().map(flex_text).collect::<Vec<_>>().join(","),
        List::One(v) => flex_text(v),
    })
}

// ── weather ───────────────────────────────────────────────────────────────

/// One forecast cache per spot on the map (the middle of a couple's homes), shared by the process.
pub fn weather_for(ctx: &Ctx) -> Arc<Weather> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Weather>>>> = OnceLock::new();
    let at = weather::midpoint([ctx.people[0].1, ctx.people[1].1]);
    let key = format!("{at:?}");
    let mut map = CACHE.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
    map.entry(key)
        .or_insert_with(|| {
            let fake = std::env::var("WEATHER_FAKE_PERCENT").ok().and_then(|v| v.parse().ok());
            Arc::new(Weather::new(at, fake))
        })
        .clone()
}

// ── shared view pieces (Joy's routes/mod.rs) ──────────────────────────────

/// A <select>/<checkbox> option with its selected state precomputed.
#[derive(Debug, Serialize)]
pub struct Opt {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

impl Opt {
    pub fn list<'a>(items: impl IntoIterator<Item = (&'a str, &'a str)>, selected: &[String]) -> Vec<Opt> {
        items
            .into_iter()
            .map(|(v, l)| Opt { value: v.into(), label: l.into(), selected: selected.iter().any(|s| s == v) })
            .collect()
    }
}

pub fn duration_opts(selected: &str) -> Vec<Opt> {
    Opt::list(picker::Duration::ALL.iter().map(|d| (d.key(), d.label())), &[selected.into()])
}

/// An activity ready for display.
#[derive(Debug, Serialize)]
pub struct ActivityView {
    pub id: i64,
    pub title: String,
    pub notes: String,
    pub place: String,
    pub cost: String,
    pub duration: String,
    pub setting: String,
    /// home | apart | local | day_trip | travel: for per-setting colour.
    pub setting_key: String,
    pub tags: Vec<String>,
    pub distances: Vec<String>,
    pub maps_url: Option<String>,
    pub times_done: i64,
    /// Suggested by the app ("generated for you").
    pub generated: bool,
    /// Titles of slips this pairs with (filled in where shown).
    pub pairs: Vec<String>,
    /// One-way cab and train estimates from each of our homes (empty when we can't tell).
    pub travel: Vec<TravelLine>,
    /// "calibrated from 4 real rides", when cab estimates have learned from us.
    pub travel_note: String,
    /// Why `cost` differs from the slip's own estimate, if it does.
    pub cost_note: String,
    /// "Closed Mondays · Nov–May only", or empty.
    pub closed_label: String,
    /// Who passed on this slip ("Neha"), when someone did (filled in where shown).
    pub vetoed_by: String,
    /// Opens Uber with this place as the drop-off, to see the real fare.
    pub uber_url: Option<String>,
    /// "about 6 h 30 min door to door", when getting there takes a while.
    pub door: String,
}

#[derive(Debug, Serialize)]
pub struct TravelLine {
    /// "Rohit"
    pub name: String,
    /// "~₹380"
    pub cab: String,
    /// "35 min"
    pub cab_time: String,
    pub train: String,
    pub train_time: String,
}

impl ActivityView {
    pub fn new(ctx: &Ctx, a: &Activity, learned: &Learned) -> Self {
        let distances = ctx
            .distances(a.latlng())
            .iter()
            .zip(&ctx.people)
            .filter_map(|(d, (name, _))| d.map(|km| format!("{name} ~{} km", km.round())))
            .collect();
        let place = [a.venue.as_str(), a.area.as_str()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        Self {
            id: a.id,
            title: ctx.personalize(&a.title),
            notes: ctx.personalize(&a.notes),
            place,
            cost: ctx.money(a.effective_cost(learned.cost)),
            cost_note: match learned.cost {
                Some(_) if !a.est_from_actual && a.effective_cost(learned.cost) != a.est_cost => {
                    format!("slip says {}; adjusted for how your dates usually go", ctx.money(a.est_cost))
                }
                _ if a.est_from_actual => "from what it really cost last time".into(),
                _ => String::new(),
            },
            closed_label: closed_label(a),
            duration: a.duration().label().into(),
            setting: picker::setting_label(&a.setting).into(),
            setting_key: a.setting.clone(),
            tags: a.tag_list(),
            distances,
            maps_url: a.latlng().map(|(lat, lng)| format!("https://www.google.com/maps/search/?api=1&query={lat},{lng}")),
            times_done: a.times_done,
            generated: a.generated,
            pairs: vec![],
            vetoed_by: String::new(),
            travel_note: learned
                .cab
                .map(|_| format!("calibrated from {} real ride{}", learned.rides, if learned.rides == 1 { "" } else { "s" }))
                .unwrap_or_default(),
            travel: ctx
                .distances(a.latlng())
                .iter()
                .zip(&ctx.people)
                .filter_map(|(d, (name, _))| {
                    let km = (*d)?;
                    let mut cab = crate::joy::travel::cab(km);
                    if let Some(f) = learned.cab {
                        cab.rupees = ((cab.rupees as f64 * f) / 10.0).round() as i64 * 10;
                    }
                    let train = crate::joy::travel::train(km);
                    Some(TravelLine {
                        name: name.clone(),
                        cab: format!("~{}", ctx.money(cab.rupees)),
                        cab_time: crate::joy::travel::duration_label(cab.minutes),
                        train: format!("~{}", ctx.money(train.rupees)),
                        train_time: crate::joy::travel::duration_label(train.minutes),
                    })
                })
                .collect(),
            door: match door_minutes(ctx, a) {
                Some(m) if m >= a.minutes() + 30 => {
                    format!("about {} door to door", crate::joy::travel::duration_label((m + 15) / 30 * 30))
                }
                _ => String::new(),
            },
            uber_url: a.latlng().map(|(lat, lng)| {
                format!(
                    "https://m.uber.com/ul/?action=setPickup&pickup=my_location\
                     &dropoff%5Blatitude%5D={lat}&dropoff%5Blongitude%5D={lng}&dropoff%5Bnickname%5D={}",
                    url_encode(if a.venue.is_empty() { &a.title } else { &a.venue })
                )
            }),
        }
    }
}

/// How long each of us travels one way to get there, in minutes: by our usual train or metro
/// (calibrated to our real commutes when the schedule knows them). At home together, one of
/// us comes over; long distance, nobody travels.
pub fn travel_minutes(ctx: &Ctx, a: &Activity) -> [Option<i64>; 2] {
    let leg = |who: usize, place: (f64, f64)| -> Option<i64> {
        match &ctx.schedule {
            Some(sched) => sched.home_leg(ctx, who, place).map(|l| l.minutes),
            None => Some(crate::joy::travel::train(ctx::road_km(ctx.people[who].1?, place)).minutes),
        }
    };
    match a.setting.as_str() {
        "apart" => [None, None],
        // Whoever has the shorter trip comes over; the other stays put.
        "home" => match (ctx.people[0].1, ctx.people[1].1) {
            (Some(h0), Some(h1)) => {
                let (m0, m1) = (leg(0, h1), leg(1, h0));
                if m0.unwrap_or(i64::MAX) <= m1.unwrap_or(i64::MAX) { [m0, Some(0)] } else { [Some(0), m1] }
            }
            _ => [None, None],
        },
        _ => match a.latlng() {
            Some(p) => [leg(0, p), leg(1, p)],
            None => [None, None],
        },
    }
}

/// The whole date for whichever of us travels longest: there, the date itself, and back.
/// None when nobody travels (or we can't tell).
pub fn door_minutes(ctx: &Ctx, a: &Activity) -> Option<i64> {
    let longest = travel_minutes(ctx, a).into_iter().flatten().max()?;
    Some(a.minutes() + 2 * longest)
}

/// Distance the "how far" filter compares against: from one person's home,
/// or (`None`) whichever of us is farther.
pub fn distance_from(ctx: &Ctx, a: &Activity, from: Option<usize>) -> Option<f64> {
    let d = ctx.distances(a.latlng());
    match from {
        Some(i) => d.get(i).copied().flatten(),
        None => d.into_iter().flatten().reduce(f64::max),
    }
}

/// Which of us is asking, with names for display.
#[derive(Debug, Serialize)]
pub struct Me {
    pub who: Option<usize>,
    /// "Rohit", or empty when we don't know yet.
    pub name: String,
    /// The other one of us ("Neha"), or "your partner" when unknown.
    pub partner: String,
}

impl Me {
    pub fn new(ctx: &Ctx, who: Option<usize>) -> Self {
        Self {
            who,
            name: who.map(|i| ctx.people[i].0.clone()).unwrap_or_default(),
            partner: who.map(|i| ctx.people[1 - i].0.clone()).unwrap_or_else(|| "your partner".into()),
        }
    }
}

/// "₹4,200 of ₹6,000 this month".
#[derive(Debug, Serialize)]
pub struct BudgetView {
    pub spent: String,
    /// This month's budget, including anything rolled over. Empty when none is set.
    pub budget: String,
    /// Raw standing amount for the edit field ("6000" or "").
    pub budget_raw: String,
    /// 0–100, for a progress bar.
    pub pct: u8,
    pub over: bool,
    /// "includes ₹1,800 from last month", or empty.
    pub carry_note: String,
    /// Last month's leftover, offered until we choose to roll it over or start fresh.
    pub rollover: Option<Rollover>,
}

#[derive(Debug, Serialize)]
pub struct Rollover {
    /// "October".
    pub month: String,
    pub spent: String,
    pub budget: String,
    pub left: String,
}

/// A finished month in the "Past months" list.
#[derive(Debug, Serialize)]
pub struct PastMonth {
    pub label: String,
    pub spent: String,
    /// Empty when no budget was set.
    pub budget: String,
}

const MONTHS: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// "2026-10" → "October" (plus the year when it isn't this one).
pub fn month_name(ym: &str, this_year: &str) -> String {
    let (y, m) = ym.split_once('-').unwrap_or_default();
    let name = m.parse::<usize>().ok().and_then(|m| MONTHS.get(m.wrapping_sub(1))).copied().unwrap_or_default();
    if y == this_year { name.into() } else { format!("{name} {y}") }
}

pub fn standing_budget(v: Option<String>) -> Option<i64> {
    v.and_then(|v| v.parse().ok()).filter(|b| *b > 0)
}

/// What last month left over, if it's still ours to roll over: not yet decided this month,
/// a budget is set, and last month came in under it.
pub async fn pending_rollover(db: &SqlitePool, couple: &str, standing: Option<i64>) -> Result<Option<(String, i64, i64, i64)>, AppError> {
    use crate::joy::db;
    let Some(standing) = standing else { return Ok(None) };
    let (this, prev) = (db::month_offset(db, 0).await?, db::month_offset(db, 1).await?);
    if db::month_budget(db, couple, &this).await?.is_some() {
        return Ok(None);
    }
    let spent = db::spent_in_month(db, couple, &prev).await?;
    let prev_row = db::month_budget(db, couple, &prev).await?;
    if spent == 0 && prev_row.is_none() {
        return Ok(None);
    }
    let budget = prev_row.map(|(b, c)| b + c).unwrap_or(standing);
    Ok(Some((prev, spent, budget, (budget - spent).max(0))).filter(|r| r.3 > 0))
}

pub async fn budget_view(db: &SqlitePool, couple: &str, ctx: &Ctx) -> Result<BudgetView, AppError> {
    use crate::joy::db;
    let spent = db::spent_this_month(db, couple).await?;
    let standing = standing_budget(db::setting(db, couple, "monthly_budget").await?);
    let this = db::month_offset(db, 0).await?;
    let carry = db::month_budget(db, couple, &this).await?.map(|(_, c)| c).unwrap_or(0);
    let budget = standing.map(|b| b + carry);
    let year = this.get(..4).unwrap_or_default();
    let rollover = pending_rollover(db, couple, standing).await?.map(|(prev, spent, budget, left)| Rollover {
        month: month_name(&prev, year),
        spent: ctx.money(spent),
        budget: ctx.money(budget),
        left: ctx.money(left),
    });
    Ok(BudgetView {
        spent: if spent == 0 { format!("{}0", ctx.currency) } else { ctx.money(spent) },
        budget: budget.map(|b| ctx.money(b)).unwrap_or_default(),
        budget_raw: standing.map(|b| b.to_string()).unwrap_or_default(),
        pct: budget.map(|b| ((spent * 100) / b).clamp(0, 100) as u8).unwrap_or(0),
        over: budget.is_some_and(|b| spent > b),
        carry_note: if carry > 0 && standing.is_some() { format!("includes {} rolled over", ctx.money(carry)) } else { String::new() },
        rollover,
    })
}

/// Finished months, newest first, each against the budget it had (or today's, if we never decided).
pub async fn past_months(db: &SqlitePool, couple: &str, ctx: &Ctx) -> Result<Vec<PastMonth>, AppError> {
    use crate::joy::db as jdb;
    let standing = standing_budget(jdb::setting(db, couple, "monthly_budget").await?);
    let this = jdb::month_offset(db, 0).await?;
    let year = this.get(..4).unwrap_or_default();
    let mut out = vec![];
    for (month, spent) in jdb::past_months(db, couple).await? {
        let budget = jdb::month_budget(db, couple, &month).await?.map(|(b, c)| b + c).or(standing);
        out.push(PastMonth {
            label: month_name(&month, year),
            spent: ctx.money(spent),
            budget: budget.map(|b| ctx.money(b)).unwrap_or_default(),
        });
    }
    Ok(out)
}

/// What to do with last month's leftover.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Leftover {
    /// Add it to this month's budget.
    Roll,
    /// Put it in the savings jar.
    Save,
    /// Let it go.
    Fresh,
}

/// Settle this month's start with last month's leftover. Does nothing if already decided.
pub async fn settle_month(db: &SqlitePool, couple: &str, choice: Leftover) -> Result<(), AppError> {
    use crate::joy::db as jdb;
    let standing = standing_budget(jdb::setting(db, couple, "monthly_budget").await?);
    let Some(base) = standing else { return Ok(()) };
    let this = jdb::month_offset(db, 0).await?;
    let pending = pending_rollover(db, couple, standing).await?;
    let carry = match pending {
        Some((_, _, _, left)) if choice == Leftover::Roll => left,
        _ => 0,
    };
    let settled = jdb::decide_month(db, couple, &this, base, carry).await?;
    if let (true, Leftover::Save, Some((prev, _, _, left))) = (settled, choice, pending) {
        let year = this.get(..4).unwrap_or_default();
        jdb::add_saving(db, couple, left, "leftover", &format!("{}'s leftover", month_name(&prev, year))).await?;
    }
    Ok(())
}

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MON: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// "YYYY-MM-DD HH:MM W" (from SQLite) → "Sat 18 Oct, 7:00 pm".
pub fn planned_label(local: &str) -> String {
    let parse = || -> Option<String> {
        let (date, rest) = local.split_once(' ')?;
        let (hm, wd) = rest.split_once(' ')?;
        let mut d = date.split('-').skip(1);
        let (m, day): (usize, u32) = (d.next()?.parse().ok()?, d.next()?.parse().ok()?);
        let (h, min): (u32, u32) = {
            let (h, m) = hm.split_once(':')?;
            (h.parse().ok()?, m.parse().ok()?)
        };
        let (h12, ampm) = match h {
            0 => (12, "am"),
            1..=11 => (h, "am"),
            12 => (12, "pm"),
            _ => (h - 12, "pm"),
        };
        Some(format!("{} {day} {}, {h12}:{min:02} {ampm}", WEEKDAYS.get(wd.parse::<usize>().ok()?)?, MON.get(m.checked_sub(1)?)?))
    };
    parse().unwrap_or_default()
}

/// "YYYY-MM-DD HH:MM W" → the value a `<input type="datetime-local">` expects.
pub fn planned_input(local: &str) -> String {
    local.get(..16).map(|s| s.replacen(' ', "T", 1)).unwrap_or_default()
}

/// `<input type="datetime-local">` text ("2026-11-02T19:00") → "2026-11-02 19:00", or None if it
/// isn't one. (Joy's `history::parse_local`.)
pub fn parse_local(at: &str) -> Option<String> {
    let at = at.trim();
    let ok = at.len() >= 16
        && at.is_char_boundary(16)
        && at.as_bytes()[4] == b'-'
        && at.as_bytes()[7] == b'-'
        && matches!(at.as_bytes()[10], b'T' | b' ')
        && at.as_bytes()[13] == b':'
        && at[..16].chars().enumerate().all(|(i, c)| [4, 7, 10, 13].contains(&i) || c.is_ascii_digit());
    ok.then(|| format!("{} {}", &at[..10], &at[11..16]))
}

/// Minimal percent-encoding for a URL query value.
pub fn url_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// What the jar has learned from real costs. Worked out per request (a couple of tiny queries).
#[derive(Debug, Clone, Default)]
pub struct Learned {
    /// Multiply plan estimates by this (None until there are enough real costs).
    pub cost: Option<f64>,
    /// Real ÷ estimated over all dates with a real cost, and how many there are.
    pub cost_ratio: f64,
    pub cost_samples: i64,
    /// Multiply cab estimates by this (None until there are enough real rides).
    pub cab: Option<f64>,
    pub rides: i64,
}

/// Learned factors are clamped so one odd night can't wreck every estimate.
const FACTOR_RANGE: (f64, f64) = (0.5, 3.0);

pub async fn learned(db: &SqlitePool, couple: &str) -> Result<Learned, AppError> {
    use crate::joy::db;
    let (ratio, n) = db::cost_ratio(db, couple).await?;
    let (cab_ratio, rides) = db::cab_ratio(db, couple).await?;
    let usable = |r: f64, n: i64| (n >= db::MIN_SAMPLES).then(|| r.clamp(FACTOR_RANGE.0, FACTOR_RANGE.1));
    Ok(Learned { cost: usable(ratio, n), cost_ratio: ratio, cost_samples: n, cab: usable(cab_ratio, rides), rides })
}

/// "Closed Mondays & Wednesdays · Nov–May only".
pub fn closed_label(a: &Activity) -> String {
    let days: Vec<&str> = picker::parse_closed(&a.closed_days)
        .into_iter()
        .filter_map(|d| picker::WEEKDAYS.get(d as usize).map(|(_, n)| *n))
        .collect();
    let mut parts = vec![];
    if !days.is_empty() {
        let days: Vec<String> = days.iter().map(|d| format!("{d}s")).collect();
        let list = match days.split_last() {
            Some((last, rest)) if !rest.is_empty() => format!("{} & {last}", rest.join(", ")),
            _ => days.join(""),
        };
        parts.push(format!("Closed {list}"));
    }
    if let Some((from, to)) = a.season() {
        parts.push(format!("{} only", picker::season_label(from, to)));
    }
    parts.join(" · ")
}

// ── signed URLs ───────────────────────────────────────────────────────────
//
// An `<img>` or a `.ics` download can't carry a bearer header, so the API hands out URLs that
// carry their own proof: `path?exp=<unix>&sig=<hex HMAC-SHA256 of "path\nexp">`. Anything that
// depends on who is looking (a sealed surprise) must put that in the signed path itself
// (e.g. `?who=1`), so the viewer can't be swapped by editing the URL.

fn unix_now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

/// The signing key: `DATES_URL_KEY`, else SHA-256 of `PASETO_KEY`, else a random per-process key
/// (links then stop working on restart, which beats having no key).
fn url_key() -> &'static [u8] {
    static KEY: OnceLock<Vec<u8>> = OnceLock::new();
    KEY.get_or_init(|| {
        if let Some(k) = std::env::var("DATES_URL_KEY").ok().filter(|k| !k.is_empty()) {
            return k.into_bytes();
        }
        if let Some(k) = std::env::var("PASETO_KEY").ok().filter(|k| !k.is_empty()) {
            return Sha256::digest(k.as_bytes()).to_vec();
        }
        tracing::warn!("neither DATES_URL_KEY nor PASETO_KEY is set: signed Dates links use a random key");
        let mut k = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut k);
        k
    })
}

/// HMAC-SHA256 (RFC 2104) built from plain SHA-256.
pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut block = [0u8; 64];
    if key.len() > 64 {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let (mut ipad, mut opad) = ([0x36u8; 64], [0x5cu8; 64]);
    for i in 0..64 {
        ipad[i] ^= block[i];
        opad[i] ^= block[i];
    }
    let inner = Sha256::new().chain_update(ipad).chain_update(msg).finalize();
    Sha256::new().chain_update(opad).chain_update(inner).finalize().into()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Equal without stopping at the first difference.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn mac(key: &[u8], path: &str, exp: i64) -> String {
    hex(&hmac_sha256(key, format!("{path}\n{exp}").as_bytes()))
}

fn sign_with(key: &[u8], path: &str, exp: i64) -> String {
    let sep = if path.contains('?') { '&' } else { '?' };
    format!("{path}{sep}exp={exp}&sig={}", mac(key, path, exp))
}

fn verify_with(key: &[u8], path: &str, exp: i64, sig: &str, now: i64) -> bool {
    now <= exp && constant_time_eq(mac(key, path, exp).as_bytes(), sig.as_bytes())
}

/// `path?exp=..&sig=..`, good for `ttl_secs`. `path` may already carry a query (`?who=1`), in
/// which case `exp` and `sig` are appended with `&` and the query is part of what's signed.
pub fn sign(path: &str, ttl_secs: i64) -> String {
    sign_with(url_key(), path, unix_now() + ttl_secs)
}

/// Is `sig` the signature [`sign`] made for exactly this `path` and `exp`, and not yet expired?
/// Pass the same `path` (including any query that was signed) that was given to `sign`.
pub fn verify(path: &str, exp: i64, sig: &str) -> bool {
    verify_with(url_key(), path, exp, sig, unix_now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joy::picker::Duration;

    #[test]
    fn a_date_far_from_both_of_us_takes_most_of_the_day() {
        let mut cfg = Ctx::for_tests();
        cfg.people = [("Anjal".into(), Some((19.0, 73.05))), ("Diya".into(), Some((19.28, 72.87)))];
        let a = Activity { setting: "local".into(), lat: Some(19.136), lng: Some(72.83), duration: "short".into(), ..Default::default() };
        let [ulwe, mira] = travel_minutes(&cfg, &a);
        assert!(ulwe.unwrap() > 60 && mira.unwrap() > 30, "{ulwe:?} {mira:?}");
        let door = door_minutes(&cfg, &a).unwrap();
        assert_eq!(door, a.minutes() + 2 * ulwe.unwrap().max(mira.unwrap()));
        assert!(Duration::of_minutes(door) > Duration::Short, "two hours in Andheri isn't a couple of hours: {door}");
        // At home together, one of us comes over; long distance, nobody travels.
        let home = Activity { setting: "home".into(), ..a.clone() };
        assert!(travel_minutes(&cfg, &home).contains(&Some(0)));
        let apart = Activity { setting: "apart".into(), ..a };
        assert_eq!(door_minutes(&cfg, &apart), None);
    }

    #[test]
    fn labels_planned_times() {
        assert_eq!(planned_label("2026-10-18 19:00 0"), "Sun 18 Oct, 7:00 pm");
        assert_eq!(planned_label("2026-01-03 00:05 6"), "Sat 3 Jan, 12:05 am");
        assert_eq!(planned_label("garbage"), "");
        assert_eq!(planned_input("2026-10-18 19:00 0"), "2026-10-18T19:00");
    }

    #[test]
    fn encodes_for_whatsapp_links() {
        assert_eq!(url_encode("Cat café & chai\n"), "Cat%20caf%C3%A9%20%26%20chai%0A");
    }

    #[test]
    fn parses_datetime_local() {
        assert_eq!(parse_local("2026-11-02T19:00").as_deref(), Some("2026-11-02 19:00"));
        assert_eq!(parse_local(" 2026-11-02 19:00:00 ").as_deref(), Some("2026-11-02 19:00"));
        assert_eq!(parse_local(""), None);
        assert_eq!(parse_local("2026-11-02"), None);
        assert_eq!(parse_local("2026/11/02T19:00"), None);
        assert_eq!(parse_local("2026-11-02T1é:00"), None);
    }

    #[test]
    fn hmac_matches_the_rfc_4231_vectors() {
        // Test case 1.
        assert_eq!(
            hex(&hmac_sha256(&[0x0b; 20], b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        // Test case 6: a key longer than the block size is hashed first.
        assert_eq!(
            hex(&hmac_sha256(&[0xaa; 131], b"Test Using Larger Than Block-Size Key - Hash Key First")),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn signed_links_verify_until_they_expire() {
        let key = b"test key";
        let url = sign_with(key, "/api/dates/history/5/invite.ics?who=1", 2_000);
        assert!(url.starts_with("/api/dates/history/5/invite.ics?who=1&exp=2000&sig="), "{url}");
        let sig = url.rsplit("sig=").next().unwrap();
        assert!(verify_with(key, "/api/dates/history/5/invite.ics?who=1", 2_000, sig, 1_999));
        assert!(verify_with(key, "/api/dates/history/5/invite.ics?who=1", 2_000, sig, 2_000));
        assert!(!verify_with(key, "/api/dates/history/5/invite.ics?who=1", 2_000, sig, 2_001), "expired");
        // Without a query the first separator is `?`.
        assert!(sign_with(key, "/p", 1).starts_with("/p?exp=1&sig="));
    }

    #[test]
    fn tampering_breaks_the_signature() {
        let key = b"test key";
        let path = "/api/dates/history/5/invite.ics?who=1";
        let sig = mac(key, path, 2_000);
        assert!(verify_with(key, path, 2_000, &sig, 0));
        assert!(!verify_with(key, "/api/dates/history/6/invite.ics?who=1", 2_000, &sig, 0), "other path");
        assert!(!verify_with(key, "/api/dates/history/5/invite.ics?who=0", 2_000, &sig, 0), "other viewer");
        assert!(!verify_with(key, path, 9_999, &sig, 0), "longer life");
        assert!(!verify_with(b"other key", path, 2_000, &sig, 0), "other key");
        let mut bad = sig.clone();
        bad.replace_range(..1, if sig.starts_with('0') { "1" } else { "0" });
        assert!(!verify_with(key, path, 2_000, &bad, 0), "flipped digit");
        assert!(!verify_with(key, path, 2_000, &sig[..sig.len() - 1], 0), "truncated");
        assert!(!verify_with(key, path, 2_000, "", 0), "empty");
        assert!(!verify_with(key, path, 2_000, &format!("{sig}00"), 0), "padded");
        assert!(!verify_with(key, "/a\n1", 1, &mac(key, "/a", 11), 0), "exp can't be smuggled into the path");
    }

    #[test]
    fn the_process_key_signs_and_verifies() {
        let url = sign("/api/dates/nights/3/photo?v=7", 60);
        let (path, rest) = url.split_once("&exp=").unwrap();
        let (exp, sig) = rest.split_once("&sig=").unwrap();
        assert!(verify(path, exp.parse().unwrap(), sig));
        assert!(!verify(path, exp.parse::<i64>().unwrap() + 1, sig));
        let past = sign("/x", -5);
        let (exp, sig) = past.split_once("?exp=").unwrap().1.split_once("&sig=").unwrap();
        assert!(!verify("/x", exp.parse().unwrap(), sig), "already expired");
    }
}
