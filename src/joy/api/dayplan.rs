//! The day-of plan for a kept date: the timeline, a Plan B, somewhere to eat after, and the
//! little jobs the jar dealt each of us.
//!
//! Part 1 ports the non-handler half of Joy's `routes/dayplan.rs` (everything decide and
//! envelope call). The handlers (`task_done`, `swap_plan_b`) are for parts 2/3 to add here.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::Date;

use super::{decide::sealed_for, flex, joy, weather_for, AppError};
use crate::joy::{
    ctx::{road_km, Ctx},
    db::{self, Activity, HistoryRow},
    hours::Hours,
    schedule::{self, Fit, Kind, Schedule, Slot},
    weather,
};
use crate::models::AppState;

/// How long a calendar link stays good (a week of reloads, long enough for a planned date).
pub const ICS_TTL_SECS: i64 = 7 * 24 * 3600;

/// A signed link to a kept date's `.ics` invite. The viewer is part of what's signed, so a
/// sealed surprise stays sealed for whoever it's sealed for: the handler must verify
/// `format!("/api/dates/history/{id}/invite.ics")` plus `?who=N` when the query has `who`.
pub fn ics_url(history_id: i64, who: Option<usize>) -> String {
    let path = format!("/api/dates/history/{history_id}/invite.ics");
    // Signed over the /api/dates path; served (and downloaded by the browser) under the public
    // prefix, since a calendar download can't carry the login header.
    super::regular::public_link(super::sign(&match who {
        Some(w) => format!("{path}?who={w}"),
        None => path,
    }, ICS_TTL_SECS))
}

/// Every date we could draw for right now (empty without a schedule).
pub async fn slots(db: &SqlitePool, ctx: &Ctx, n: usize) -> Result<Vec<Slot>, AppError> {
    let Some(sched) = &ctx.schedule else { return Ok(vec![]) };
    let (date, minutes) = db::local_now(db).await?;
    Ok(schedule::parse_date(&date).map(|d| sched.slots(d, minutes, n)).unwrap_or_default())
}

/// Every day of the coming week, for plans that don't follow the usual days.
pub async fn days(db: &SqlitePool, ctx: &Ctx) -> Result<Vec<Slot>, AppError> {
    let Some(sched) = &ctx.schedule else { return Ok(vec![]) };
    let (date, minutes) = db::local_now(db).await?;
    Ok(schedule::parse_date(&date).map(|d| sched.days(d, minutes, 8)).unwrap_or_default())
}

/// The date a draw is for: the one the form names, else the next one, starting from where
/// the form says ("office", "home", or "" for the usual). "any" = no particular date.
pub async fn slot_for(db: &SqlitePool, ctx: &Ctx, when: &str, start: &str) -> Result<Option<Slot>, AppError> {
    if when == "any" {
        return Ok(None);
    }
    let slot = match days(db, ctx).await?.into_iter().find(|x| x.key() == when) {
        Some(sl) => Some(sl),
        None => slots(db, ctx, 1).await?.into_iter().next(),
    };
    Ok(slot.map(|sl| sl.starting_from(start)))
}

/// Exactly the date the form names, if it's still one we could draw for (no falling back).
pub async fn slot_exact(db: &SqlitePool, ctx: &Ctx, when: &str, start: &str) -> Result<Option<Slot>, AppError> {
    Ok(days(db, ctx).await?.into_iter().find(|x| x.key() == when).map(|sl| sl.starting_from(start)))
}

/// The kind of date a day is: what we kept it as, else what our schedule says.
pub fn slot_on(sched: &Schedule, date: Date, kept_as: &str) -> Slot {
    Slot { date, kind: Kind::parse(kept_as).unwrap_or_else(|| sched.usual(date)), today: false }
}

/// A slip to swap to on the day: indoors, close by, about the same money, and fitting the same date.
pub fn choose_plan_b(ctx: &Ctx, slot: Option<&Slot>, a: &Activity, acts: &[Activity]) -> Option<i64> {
    let here = a.latlng()?;
    // A free walk's Plan B can still be a café: allow a bit more than the original.
    let cap = (a.est_cost * 3 / 2).max(a.est_cost + 1000);
    let tags = a.tag_list();
    // Close by, and the same kind of evening: each shared tag counts like being 1.5 km nearer.
    let score = |b: &Activity, km: f64| km - 1.5 * b.tag_list().iter().filter(|t| tags.contains(t)).count() as f64;
    acts.iter()
        .filter(|b| b.id != a.id && b.setting == "local" && !b.outdoor() && b.est_cost <= cap)
        .filter_map(|b| Some((b, road_km(here, b.latlng()?))))
        .filter(|(_, km)| *km <= 4.0)
        .map(|(b, km)| (b, score(b, km)))
        .filter(|(b, _)| match (slot, &ctx.schedule) {
            (Some(slot), Some(sched)) => sched.fit(ctx, slot, b).is_ok(),
            _ => true,
        })
        .min_by(|x, y| x.1.total_cmp(&y.1))
        .map(|(b, _)| b.id)
}

/// The jobs a slip needs before the date: booking it, and what to bring.
pub fn jobs(a: &Activity, place: &str, date: Option<Date>, today: Date) -> Vec<(String, String)> {
    let weekend = date.is_some_and(|d| matches!(d.weekday().number_days_from_sunday(), 0 | 6));
    let mut out = vec![];
    if a.booking == "always" || (a.booking == "weekends" && weekend) {
        let due = date
            .map(|d| {
                let ahead = a.book_ahead_days.unwrap_or(1).clamp(0, 60);
                let by = d.checked_sub(time::Duration::days(ahead)).unwrap_or(d).max(today).min(d);
                ymd(by)
            })
            .unwrap_or_default();
        out.push((format!("Book {place}"), due));
    }
    out.extend(a.bring_list().into_iter().take(4).map(|item| (format!("Bring {item}"), String::new())));
    out
}

/// Deal jobs out one at a time, starting with whoever has had fewer (on a tie, whoever didn't
/// draw, since drawing was their bit). A surprise's jobs all go to whoever planned it.
pub fn deal(jobs: Vec<(String, String)>, counts: [i64; 2], drawn_by: Option<usize>, surprise: bool) -> Vec<(usize, String, String)> {
    if surprise {
        if let Some(d) = drawn_by {
            return jobs.into_iter().map(|(t, due)| (d, t, due)).collect();
        }
    }
    let mut counts = counts;
    jobs.into_iter()
        .map(|(t, due)| {
            let who = match counts[0].cmp(&counts[1]) {
                std::cmp::Ordering::Less => 0,
                std::cmp::Ordering::Greater => 1,
                std::cmp::Ordering::Equal => drawn_by.map_or(0, |d| 1 - d),
            };
            counts[who] += 1;
            (who, t, due)
        })
        .collect()
}

fn ymd(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
}

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// "2026-11-04" → "Wed 4 Nov"; today → "today".
pub fn day_label(ymd: &str, today: Date) -> String {
    match schedule::parse_date(ymd) {
        Some(d) if d == today => "today".into(),
        Some(d) => format!("{} {} {}", DAYS[d.weekday().number_days_from_sunday() as usize], d.day(), MONTHS[d.month() as usize - 1]),
        None => String::new(),
    }
}

#[derive(Debug, Serialize)]
pub struct StepView {
    pub at: String,
    pub minutes: i64,
    pub what: String,
    /// It's about the person asking.
    pub mine: bool,
}

#[derive(Debug, Serialize)]
pub struct TaskView {
    pub id: i64,
    pub task: String,
    /// "by Wed 4 Nov", "by today", or empty.
    pub due: String,
    /// "Diya", or empty when it's mine.
    pub whose: String,
    pub mine: bool,
    pub done: bool,
}

#[derive(Debug, Serialize)]
pub struct PlanB {
    pub title: String,
    pub place: String,
}

/// Everything the day-of card shows.
#[derive(Debug, Serialize)]
pub struct DayPlan {
    pub history_id: i64,
    pub title: String,
    /// "Thursday evening · 5 Nov"
    pub when: String,
    pub steps: Vec<StepView>,
    pub tight: bool,
    /// "Open 12:00 pm – 11:30 pm" for that day, or a nudge to check.
    pub hours_note: String,
    pub plan_b: Option<PlanB>,
    /// Rain chance (%) when it's today, likely, and the date is outdoors.
    pub rain: Option<u8>,
    /// "Kala Ghoda Cafe, 300 m away".
    pub after: String,
    pub tasks: Vec<TaskView>,
    /// A surprise this viewer only gets hints of.
    pub sealed: bool,
    /// Signed link to the calendar invite (see [`ics_url`]).
    pub ics_url: String,
    /// When the date itself starts (minutes after midnight), for the calendar reminder.
    pub start: i64,
    /// Minutes before `start` this viewer has to set off (0 when there's no travel).
    pub leave_before: i64,
    /// How many of the jobs still open are this viewer's.
    pub my_open_tasks: usize,
}

impl DayPlan {
    /// Minutes before the start that this viewer has to set off (0 when there's no travel).
    pub fn leave_before(&self, who: Option<usize>) -> i64 {
        // The earliest step before we're there that's about this viewer (or both of us).
        self.steps
            .iter()
            .filter(|st| st.minutes < self.start && (st.mine || who.is_none() || !st.what.contains(" leaves ")))
            .map(|st| self.start - st.minutes)
            .max()
            .unwrap_or(0)
    }

    pub fn my_open_tasks(&self) -> usize {
        self.tasks.iter().filter(|t| t.mine && !t.done).count()
    }
}

/// Somewhere to eat within a short walk that's open when the date ends.
fn after(ctx: &Ctx, a: &Activity, acts: &[Activity], wd: u8, end: i64) -> String {
    let Some(here) = a.latlng() else { return String::new() };
    acts.iter()
        .filter(|b| b.id != a.id && b.tag_list().iter().any(|t| t == "foodie") && b.setting == "local")
        .filter(|b| Hours::parse(&b.hours).is_none_or(|h| h.covers(wd, end, end + 60)))
        .filter_map(|b| Some((b, road_km(here, b.latlng()?))))
        .filter(|(_, km)| *km <= 1.2)
        .min_by(|x, y| x.1.total_cmp(&y.1))
        .map(|(b, km)| {
            let name = if b.venue.is_empty() { ctx.personalize(&b.title) } else { b.venue.clone() };
            let m = ((km * 1000.0 / 50.0).round() as i64 * 50).max(50);
            format!("{name}, about {m} m away")
        })
        .unwrap_or_default()
}

/// The plan for a kept date that has a time, from this viewer's side. None without a schedule
/// or a time.
pub async fn build(db: &SqlitePool, couple: &str, ctx: &Ctx, row: &HistoryRow, who: Option<usize>) -> Result<Option<DayPlan>, AppError> {
    let (Some(sched), Some(local)) = (&ctx.schedule, row.planned_local.as_deref()) else { return Ok(None) };
    let (Some(date), Some(minutes)) = (
        local.get(..10).and_then(schedule::parse_date),
        local.get(11..16).and_then(|hm| Some(hm.get(..2)?.parse::<i64>().ok()? * 60 + hm.get(3..)?.parse::<i64>().ok()?)),
    ) else {
        return Ok(None);
    };
    let Some(a) = db::get(db, couple, row.activity_id).await? else { return Ok(None) };
    let slot = slot_on(sched, date, &row.day_kind);
    let wd = slot.weekday();
    let fit = match sched.fit(ctx, &slot, &a) {
        Ok(f) => {
            // We planned it for a different time than the jar would: move everything with it.
            let d = minutes - f.start;
            Fit { start: f.start + d, end: f.end + d, ..f }
        }
        Err(_) => Fit { start: minutes, end: minutes + a.minutes().min(12 * 60), tight: false, there: [None; 2], back: [None; 2] },
    };
    let sealed = sealed_for(row, who);
    let steps = schedule::timeline(sched, ctx, &slot, &a, &fit);
    let steps: Vec<StepView> = steps
        .into_iter()
        .filter(|st| !sealed || st.who.is_none() || st.who == who)
        .map(|st| {
            let what = if sealed && st.who.is_none() && st.at != schedule::at(fit.start) { String::new() } else { st.what };
            // On a sealed surprise, how long and how you travel would hint at where: keep just the time.
            let what = if sealed { what.split(',').next().unwrap_or_default().to_string() } else { what };
            StepView { mine: st.who.is_some() && st.who == who, at: st.at, minutes: st.minutes, what }
        })
        .filter(|st| !st.what.is_empty())
        .map(|st| if sealed && st.at == schedule::at(fit.start) && !st.mine { StepView { what: "The surprise starts".into(), ..st } } else { st })
        .collect();
    let (today_s, _) = db::local_now(db).await?;
    let today = schedule::parse_date(&today_s).unwrap_or(date);
    // Only a plan with somewhere to be needs the rest of the jar (for "after").
    let after = if sealed || slot.kind == Kind::Apart || a.latlng().is_none() {
        String::new()
    } else {
        after(ctx, &a, &db::list(db, couple, false).await?, wd, fit.end)
    };
    let plan_b = match (sealed, row.plan_b) {
        (false, Some(id)) => db::get(db, couple, id).await?.map(|b| PlanB {
            title: ctx.personalize(&b.title),
            place: [b.venue.as_str(), b.area.as_str()].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" · "),
        }),
        _ => None,
    };
    let rain = if date == today && a.outdoor() && !sealed {
        weather_for(ctx).tonight().await.filter(|p| *p >= weather::LIKELY_PERCENT)
    } else {
        None
    };
    let tasks = db::tasks_for(db, couple, row.id)
        .await?
        .into_iter()
        .filter(|t| !sealed || Some(t.who as usize) == who)
        .map(|t| TaskView {
            id: t.id,
            due: match day_label(&t.due, today) {
                l if l.is_empty() => String::new(),
                l => format!("by {l}"),
            },
            whose: if Some(t.who as usize) == who { String::new() } else { ctx.people.get(t.who as usize).map(|p| p.0.clone()).unwrap_or_default() },
            mine: Some(t.who as usize) == who,
            done: t.done_at.is_some(),
            task: ctx.personalize(&t.task),
        })
        .collect();
    let mut plan = DayPlan {
        history_id: row.id,
        title: if sealed { "A surprise".into() } else { ctx.personalize(&row.title) },
        when: if date == today { slot_label_today(&slot) } else { format!("{} · {}", slot.label(), day_label(&ymd(date), today)) },
        tight: fit.tight && !sealed,
        hours_note: match Hours::parse(&a.hours) {
            _ if sealed || matches!(a.setting.as_str(), "home" | "apart") => String::new(),
            Some(h) => h.label(wd),
            None => "Hours not confirmed: check before you go".into(),
        },
        after,
        steps,
        plan_b,
        rain,
        tasks,
        sealed,
        ics_url: ics_url(row.id, who),
        start: fit.start,
        leave_before: 0,
        my_open_tasks: 0,
    };
    plan.leave_before = plan.leave_before(who);
    plan.my_open_tasks = plan.my_open_tasks();
    Ok(Some(plan))
}

fn slot_label_today(slot: &Slot) -> String {
    match slot.kind {
        Kind::Weeknight => "Tonight".into(),
        Kind::Weekend => "Today".into(),
        Kind::Apart => "Tonight, apart".into(),
    }
}

/// When a date is kept: pick its Plan B, deal its jobs, and set its time if we drew it for one.
pub async fn on_keep(
    db: &SqlitePool,
    couple: &str,
    ctx: &Ctx,
    ids: &[i64],
    slot: Option<&Slot>,
    who: Option<usize>,
    surprise: bool,
) -> Result<(), AppError> {
    let Some(&first) = ids.first() else { return Ok(()) };
    let Some(row) = db::history_row(db, couple, first).await? else { return Ok(()) };
    let Some(a) = db::get(db, couple, row.activity_id).await? else { return Ok(()) };
    if let (Some(slot), Some(sched)) = (slot, &ctx.schedule) {
        if row.planned_at.is_none() {
            if let Ok(fit) = sched.fit(ctx, slot, &a) {
                db::set_planned(db, couple, ids, Some(&slot.local(fit.start))).await?;
                // Remember an office Saturday or a Monday off, so the plan stays that way.
                db::set_day_kind(db, couple, ids, slot.kind.as_str()).await?;
            }
        }
    }
    let acts = db::list(db, couple, false).await?;
    let vetoed: Vec<i64> = db::vetoes(db, couple).await?.into_iter().map(|(a, _)| a).collect();
    let acts: Vec<Activity> = acts.into_iter().filter(|x| !vetoed.contains(&x.id)).collect();
    if row.plan_b.is_none() {
        db::set_plan_b(db, couple, first, choose_plan_b(ctx, slot, &a, &acts)).await?;
    }
    let (today, _) = db::local_now(db).await?;
    let today = schedule::parse_date(&today);
    let mut all_jobs = vec![];
    for id in ids {
        let Some(r) = db::history_row(db, couple, *id).await? else { continue };
        let Some(act) = db::get(db, couple, r.activity_id).await? else { continue };
        let place = if act.venue.is_empty() { ctx.personalize(&act.title) } else { act.venue.clone() };
        if let Some(today) = today {
            all_jobs.extend(jobs(&act, &place, slot.map(|x| x.date), today));
        }
    }
    let counts = db::task_counts(db, couple).await?;
    db::add_tasks(db, couple, first, &deal(all_jobs, counts, who, surprise)).await?;
    Ok(())
}

// ── handlers (Joy's `task_done` and `swap_plan_b`) ───────────────────────

#[derive(Deserialize)]
pub struct DoneBody {
    /// "1" (or true) ticks the job off; anything else unticks it.
    #[serde(default, deserialize_with = "flex")]
    done: String,
}

/// `POST /tasks/{id}/done`: tick off (or untick) one of my jobs. Only whoever was dealt it may;
/// anyone else gets 403, another couple's job a 404. Returns the refreshed day-of plan.
pub async fn task_done(
    State(s): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Path(id): Path<i64>,
    body: Option<Json<DoneBody>>,
) -> Result<Json<Option<DayPlan>>, AppError> {
    let j = joy(&s, &user_id).await?;
    let Some(t) = db::task(&s.db, &j.couple, id).await? else {
        return Err(AppError(StatusCode::NOT_FOUND, "That job isn't here.".into()));
    };
    if t.who as usize != j.who {
        return Err(AppError(StatusCode::FORBIDDEN, "That job is your partner's.".into()));
    }
    let done = body.is_some_and(|b| b.0.done == "1");
    db::set_task_done(&s.db, &j.couple, id, done).await?;
    plan_of(&s, &j, t.history_id).await
}

/// `POST /history/{id}/plan-b`: go with Plan B instead (or back again). Returns the refreshed plan.
pub async fn swap_plan_b(
    State(s): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Path(id): Path<i64>,
) -> Result<Json<Option<DayPlan>>, AppError> {
    let j = joy(&s, &user_id).await?;
    let Some(row) = db::history_row(&s.db, &j.couple, id).await? else {
        return Err(AppError(StatusCode::NOT_FOUND, "That date isn't here.".into()));
    };
    if sealed_for(&row, Some(j.who)) {
        return Err(AppError(StatusCode::FORBIDDEN, "That's a surprise.".into()));
    }
    db::swap_plan_b(&s.db, &j.couple, id).await?;
    plan_of(&s, &j, id).await
}

/// The plan card on its own (null when the date has no time or the couple no schedule).
async fn plan_of(s: &AppState, j: &super::Joy, history_id: i64) -> Result<Json<Option<DayPlan>>, AppError> {
    let Some(row) = db::history_row(&s.db, &j.couple, history_id).await? else {
        return Err(AppError(StatusCode::NOT_FOUND, "That date isn't here.".into()));
    };
    Ok(Json(build(&s.db, &j.couple, &j.ctx, &row, Some(j.who)).await?))
}
