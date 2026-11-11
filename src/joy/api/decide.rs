//! The jar page and the draw: pick, live match counts, and "it's a date". The server works out
//! the peek cap, whose turn it is, and what a sealed surprise may show, every time.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{
    budget_view, dayplan, distance_from, door_minutes, duration_opts, envelope, flex, flex_list, internal, joy,
    learned, middle::{self, Middle}, regular, weather_for, ActivityView, AppError, BudgetView, Learned, Me, Opt,
};
use crate::{
    handlers::common::not_found,
    joy::{
        ctx::{road_km, Ctx},
        db,
        picker::{self, Candidate, Duration, Filters, Hint, PickResult, Today, ENERGIES, MAX_REROLLS, SETTINGS},
        schedule::{self, Slot},
        weather,
    },
    models::AppState,
};

const KM_CHOICES: [(&str, &str); 5] =
    [("15", "15 km"), ("30", "30 km"), ("45", "45 km"), ("60", "60 km"), ("100", "100 km")];

/// One slip in the jar, as the page's jar drawing sees it.
#[derive(Debug, Serialize)]
pub struct Slip {
    pub id: i64,
    pub title: String,
    pub setting: String,
    /// Sinks in the jar when rain is likely.
    pub outdoor: bool,
}

/// `GET /jar`: everything the jar page shows.
#[derive(Debug, Serialize)]
pub struct DecideView {
    pub couple: String,
    /// "Cello Triumph" when we have an office to start from, else empty.
    pub office_name: String,
    pub currency: String,
    pub count: usize,
    /// Every slip in the jar that can be drawn.
    pub slips: Vec<Slip>,
    pub durations: Vec<Opt>,
    pub settings: Vec<Opt>,
    pub tags: Vec<Opt>,
    pub energies: Vec<Opt>,
    pub kms: Vec<Opt>,
    pub froms: Vec<Opt>,
    /// Who's asking ("Rohit").
    pub me_name: String,
    /// Whose turn it is to draw ("Neha"), empty when it's anyone's.
    pub turn_name: String,
    /// It's the asker's turn: they get a bonus peek.
    pub my_turn: bool,
    /// Tonight's rain chance (%) when rain is likely.
    pub rain: Option<u8>,
    /// A kept date from about a year ago this week.
    pub memory: Option<Memory>,
    pub budget: BudgetView,
    /// A gentle line: "Tomorrow, 7:00 pm: Saltt + Long drive", or "It's been 3 weeks…". Empty = none.
    pub nudge: String,
    /// The nudge is about a date coming up (vs. a gap since the last one).
    pub nudge_soon: bool,
    /// "Meet in the middle": tonight's private check-ins, from the asker's side.
    pub middle: middle::MiddleView,
    /// The dates we could draw for ("Thursday evening"), the next one selected; empty without a schedule.
    pub whens: Vec<Opt>,
    /// The next kept date's day-of plan, until it's over.
    pub next_plan: Option<dayplan::DayPlan>,
    /// Tonight's question for a regular ("Blue Tokai?"), and what we could still spend on them.
    pub regulars: regular::Card,
}

/// Weeks without a date before the jar says so.
const NUDGE_AFTER_DAYS: i64 = 21;

/// The gentle nudge for the jar page. A surprise sealed for this viewer shows only who it's from.
async fn nudge(db: &SqlitePool, couple: &str, ctx: &Ctx, who: Option<usize>) -> Result<(String, bool), AppError> {
    let (today, tomorrow, _, _) = db::local_today(db).await?;
    let upcoming = db::upcoming(db, couple).await?;
    if let Some(first) = upcoming.first() {
        let together: Vec<&db::HistoryRow> = upcoming.iter().filter(|r| r.planned_at == first.planned_at).collect();
        let local = first.planned_local.clone().unwrap_or_default();
        let hour: u32 = local.get(11..13).and_then(|h| h.parse().ok()).unwrap_or(0);
        let day = match local.get(..10) {
            Some(d) if d == today && hour >= 17 => "Tonight",
            Some(d) if d == today => "Today",
            Some(d) if d == tomorrow => "Tomorrow",
            _ => "Soon",
        };
        let time = super::planned_label(&local).split(", ").nth(1).unwrap_or("").to_string();
        let what: Vec<String> = together
            .iter()
            .map(|r| {
                if sealed_for(r, who) {
                    let from = r.drawn_by.and_then(|d| ctx.people.get(d as usize)).map_or("one of you", |p| p.0.as_str());
                    format!("a surprise from {from}")
                } else {
                    ctx.personalize(&r.title)
                }
            })
            .collect::<Vec<_>>();
        let mut what = what;
        what.dedup();
        return Ok((format!("{day}, {time}: {}", what.join(" + ")), true));
    }
    if !db::anything_planned(db, couple).await?
        && let Some(days) = db::days_since_last_date(db, couple).await?.filter(|d| *d >= NUDGE_AFTER_DAYS) {
            let weeks = days / 7;
            return Ok((format!("It's been {weeks} weeks since your last date."), false));
        }
    Ok((String::new(), false))
}

#[derive(Debug, Serialize)]
pub struct Memory {
    pub title: String,
    /// "14 Oct 2025"
    pub when: String,
}

/// Whose turn it is to draw: the other one of us from whoever kept the last date.
pub async fn turn(db: &SqlitePool, couple: &str) -> Result<Option<usize>, AppError> {
    Ok(db::last_drawer(db, couple).await?.and_then(|w| usize::try_from(1 - w).ok()))
}

/// Is a kept date a surprise this viewer shouldn't see yet? Only whoever planned it sees it
/// until it's revealed; someone who hasn't said who they are sees nothing (fail closed).
pub fn sealed_for(row: &db::HistoryRow, me: Option<usize>) -> bool {
    row.surprise && !row.revealed && (me.is_none() || row.drawn_by != me.map(|m| m as i64))
}

/// Tonight's rain chance, only when it's likely.
async fn rain(ctx: &Ctx) -> Option<u8> {
    weather_for(ctx).tonight().await.filter(|p| *p >= weather::LIKELY_PERCENT)
}

/// `GET /jar`
pub async fn page(State(s): State<AppState>, Extension((user_id, _)): Extension<(String, String)>) -> Result<Json<DecideView>, AppError> {
    let j = joy(&s, &user_id).await?;
    let (db, couple, ctx) = (&s.db, j.couple.as_str(), &j.ctx);
    let who = Some(j.who);
    let me = Me::new(ctx, who);
    let turn = turn(db, couple).await?;
    let memory = db::year_ago(db, couple)
        .await?
        .into_iter()
        .find(|r| !sealed_for(r, who))
        .map(|r| Memory { title: ctx.personalize(&r.title), when: format!("{} {}", r.day_label(), r.day.get(..4).unwrap_or("")) });
    let nudge = nudge(db, couple, ctx, who).await?;
    let middle = middle::view(db, couple, ctx, who, &me.partner).await?;
    let jar = jar_data(db, couple, ctx, j.who, None).await?;
    let acts = &jar.acts;
    // Every day this week, so a holiday Monday or an office Saturday can be planned too. The
    // next date we usually meet is picked to start with.
    let first = dayplan::slots(db, ctx, 1).await?.first().map(|sl| sl.key());
    let whens: Vec<Opt> = dayplan::days(db, ctx)
        .await?
        .iter()
        .map(|sl| Opt { value: sl.key(), label: sl.label(), selected: Some(sl.key()) == first })
        .collect();
    let next_plan = match db::next_planned(db, couple).await? {
        Some(r) => dayplan::build(db, couple, ctx, &r, who).await?,
        None => None,
    };
    let mut tags: Vec<String> = acts.iter().flat_map(|a| a.tag_list()).collect();
    tags.sort();
    tags.dedup();
    let slips: Vec<Slip> = acts
        .iter()
        .map(|a| Slip {
            id: a.id,
            title: ctx.personalize(&a.title),
            setting: a.setting.clone(),
            outdoor: a.tag_list().iter().any(|t| t == "outdoors"),
        })
        .collect();
    Ok(Json(DecideView {
        couple: format!("{} & {}", ctx.people[0].0, ctx.people[1].0),
        office_name: ctx.schedule.as_ref().filter(|sc| sc.office.is_some()).map(|sc| sc.office_name.clone()).unwrap_or_default(),
        currency: ctx.currency.clone(),
        count: acts.len(),
        slips,
        durations: duration_opts(""),
        settings: Opt::list(SETTINGS, &[]),
        tags: tags.iter().map(|t| Opt { value: t.clone(), label: t.clone(), selected: false }).collect(),
        energies: Opt::list(ENERGIES, &[]),
        kms: Opt::list(KM_CHOICES, &[]),
        froms: ctx
            .people
            .iter()
            .enumerate()
            .filter(|(_, (_, home))| home.is_some())
            .map(|(i, (name, _))| Opt { value: i.to_string(), label: format!("{name}'s place"), selected: false })
            .chain(
                ctx.schedule
                    .as_ref()
                    .filter(|sc| sc.office.is_some())
                    .map(|sc| Opt { value: "office".into(), label: sc.office_name.clone(), selected: false }),
            )
            .collect(),
        me_name: me.name,
        turn_name: turn.map(|t| ctx.people[t].0.clone()).unwrap_or_default(),
        my_turn: turn.is_some() && turn == who,
        rain: rain(ctx).await,
        memory,
        budget: budget_view(db, couple, ctx).await?,
        nudge: nudge.0,
        nudge_soon: nudge.1,
        middle,
        whens,
        next_plan,
        regulars: regular::card(db, couple, ctx).await?,
    }))
}

/// Everything the pick card round-trips, so rerolls are stateless. Every field is a string, as
/// Joy's form fields were; numbers, booleans and null are accepted too (`true` means "1").
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PickForm {
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub budget: String,
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub max_duration: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<String>,
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub tag: String,
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub energy: String,
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub max_km: String,
    /// "" = whoever's farther, "0"/"1" = that person's home, "office".
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub from: String,
    /// Comma-separated ids already peeked at (an array is accepted too).
    #[serde(deserialize_with = "flex_list", skip_serializing_if = "String::is_empty")]
    pub excluded: String,
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub used: String,
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub current: String,
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub reroll: String,
    /// "1": the asker passes on the current slip for good ("Not for me").
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub veto: String,
    /// "1": draw from tonight's check-ins ("Meet in the middle") instead of these filters.
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub middle: String,
    /// The date we're drawing for ("2026-11-05"), "any", or empty for the next one.
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub when: String,
    /// Where we start that day when it isn't the usual: "office" or "home" ("" = usual).
    #[serde(deserialize_with = "flex", skip_serializing_if = "String::is_empty")]
    pub start: String,
}

/// A hidden form field of Joy's pick card (kept for fidelity; `state` is the ready-to-post form).
#[derive(Debug, Serialize)]
pub struct Hidden {
    pub name: String,
    pub value: String,
}

fn hidden_of(fields: Vec<(String, String)>) -> Vec<Hidden> {
    fields.into_iter().map(|(name, value)| Hidden { name, value }).collect()
}

impl PickForm {
    pub fn filters(&self) -> Filters {
        let opt = |s: &str| Some(s.trim().to_string()).filter(|s| !s.is_empty());
        Filters {
            budget: self.budget.trim().parse().unwrap_or(i64::MAX),
            max_duration: Duration::parse(&self.max_duration),
            settings: self.settings.iter().filter(|s| !s.is_empty()).cloned().collect(),
            tag: opt(&self.tag),
            energy: opt(&self.energy),
            max_km: self.max_km.trim().parse().ok(),
            rain_likely: false,
            today: None,
        }
    }

    pub fn is_middle(&self) -> bool {
        self.middle == "1"
    }

    /// These filters replaced by where tonight's two check-ins meet.
    fn met(&self, m: &Middle) -> PickForm {
        PickForm {
            budget: m.budget.map(|b| b.to_string()).unwrap_or_default(),
            max_duration: m.max_duration.map(|d| d.key().to_string()).unwrap_or_default(),
            settings: if m.apart {
                vec!["apart".into()]
            } else if m.stay_in {
                vec!["home".into()]
            } else {
                // Together tonight: anything but the long-distance ones.
                SETTINGS.iter().map(|(k, _)| k.to_string()).filter(|k| k != "apart").collect()
            },
            tag: String::new(),
            energy: String::new(),
            max_km: m.max_km.map(|k| k.to_string()).unwrap_or_default(),
            from: String::new(),
            ..self.clone()
        }
    }

    fn from(&self) -> Option<usize> {
        self.from.parse().ok().filter(|i| *i < 2)
    }

    /// "office", "home", or "" for the usual start that day.
    fn start(&self) -> &'static str {
        match self.start.as_str() {
            "office" => "office",
            "home" => "home",
            _ => "",
        }
    }

    fn excluded(&self) -> Vec<i64> {
        self.excluded.split(',').filter_map(|s| s.trim().parse().ok()).collect()
    }

    /// The filter fields only. Drawing from the middle round-trips just that flag: the server
    /// works the filters out again each time, so neither of us can read the other's answers
    /// out of the card.
    pub fn filter_fields(&self) -> Vec<(String, String)> {
        if self.is_middle() {
            return vec![("middle".into(), "1".into()), ("when".into(), self.when.clone()), ("start".into(), self.start().into())];
        }
        let mut h: Vec<(String, String)> = vec![
            ("when".into(), self.when.clone()),
            ("start".into(), self.start().into()),
            ("budget".into(), self.budget.clone()),
            ("max_duration".into(), self.max_duration.clone()),
            ("tag".into(), self.tag.clone()),
            ("energy".into(), self.energy.clone()),
            ("max_km".into(), self.max_km.clone()),
            ("from".into(), self.from.clone()),
        ];
        h.extend(self.settings.iter().map(|s| ("settings".into(), s.clone())));
        h
    }

    fn hidden(&self, excluded: &[i64], used: u32, current: i64) -> Vec<(String, String)> {
        let mut h = self.filter_fields();
        h.push(("excluded".into(), excluded.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")));
        h.push(("used".into(), used.to_string()));
        h.push(("current".into(), current.to_string()));
        h
    }

    /// Just the fields [`filter_fields`](Self::filter_fields) keeps, as a form to post back.
    pub fn kept(&self) -> PickForm {
        if self.is_middle() {
            return PickForm { middle: "1".into(), when: self.when.clone(), start: self.start().into(), ..Default::default() };
        }
        PickForm {
            when: self.when.clone(),
            start: self.start().into(),
            budget: self.budget.clone(),
            max_duration: self.max_duration.clone(),
            tag: self.tag.clone(),
            energy: self.energy.clone(),
            max_km: self.max_km.clone(),
            from: self.from.clone(),
            settings: self.settings.clone(),
            ..Default::default()
        }
    }

    /// The form to post back for the next peek: the kept filters plus where the draw has got to.
    fn state(&self, excluded: &[i64], used: u32, current: i64) -> PickForm {
        PickForm {
            excluded: excluded.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","),
            used: used.to_string(),
            current: current.to_string(),
            ..self.kept()
        }
    }
}

/// What can be drawn right now: slips in the jar that neither of us passed on, what we've
/// learned about real costs, and today's date (for closed days and seasons). Every place that
/// suggests or counts slips uses this, so they all agree.
pub struct JarData {
    pub acts: Vec<db::Activity>,
    pub learned: Learned,
    pub today: Today,
    /// The date we're drawing for, when we have a schedule and didn't say "any time".
    pub slot: Option<Slot>,
}

pub async fn jar_data(db: &SqlitePool, couple: &str, _ctx: &Ctx, who: usize, slot: Option<Slot>) -> Result<JarData, AppError> {
    let vetoed: Vec<i64> = db::vetoes(db, couple).await?.into_iter().map(|(a, _)| a).collect();
    let acts = db::visible_list(db, couple, Some(who)).await?.into_iter().filter(|a| !vetoed.contains(&a.id)).collect();
    // Closed days and seasons are about the day we're drawing for, not necessarily today.
    let today = match &slot {
        Some(sl) => Today { weekday: sl.weekday(), month: sl.month() },
        None => {
            let (_, _, weekday, month) = db::local_today(db).await?;
            Today { weekday, month }
        }
    };
    Ok(JarData { acts, learned: learned(db, couple).await?, today, slot })
}

impl JarData {
    fn candidates(&self, ctx: &Ctx, f: &PickForm) -> Vec<Candidate> {
        self.acts
            .iter()
            .map(|a| {
                let km = match (f.from.as_str(), ctx.schedule.as_ref().and_then(|sc| sc.office)) {
                    ("office", Some(office)) => a.latlng().map(|p| road_km(office, p)),
                    _ => distance_from(ctx, a, f.from()),
                };
                let mut c = a.candidate(km, self.learned.cost);
                // "The time we have" is door to door: two hours in Andheri is most of a day from
                // home. An evening from the office has its own clock (the fit above).
                let from_office = self.slot.is_some_and(|sl| sl.kind == schedule::Kind::Weeknight);
                if let Some(m) = door_minutes(ctx, a).filter(|_| !from_office) {
                    c.duration = c.duration.max(Duration::of_minutes(m));
                }
                c.title = ctx.personalize(&c.title);
                if let (Some(slot), Some(sched)) = (&self.slot, &ctx.schedule) {
                    c.window_miss = sched.fit(ctx, slot, a).err();
                }
                c
            })
            .collect()
    }

    fn filters(&self, f: &PickForm, rain_likely: bool) -> Filters {
        Filters { rain_likely, today: Some(self.today), ..f.filters() }
    }
}

#[derive(Debug, Serialize)]
pub struct MatchCount {
    pub count: usize,
    pub total: usize,
    pub ids: Vec<i64>,
}

/// `POST /match`: which slips fit the current filters, so the jar reacts live as filters change.
pub async fn matching(
    State(s): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    body: Option<Json<PickForm>>,
) -> Result<Json<MatchCount>, AppError> {
    let f = body.map(|j| j.0).unwrap_or_default();
    let j = joy(&s, &user_id).await?;
    let (db, couple, ctx) = (&s.db, j.couple.as_str(), &j.ctx);
    let jar = jar_data(db, couple, ctx, j.who, dayplan::slot_for(db, ctx, &f.when, f.start()).await?).await?;
    let cands = jar.candidates(ctx, &f);
    let filters = jar.filters(&f, rain(ctx).await.is_some());
    let ids: Vec<i64> = cands.iter().filter(|c| picker::matches(c, &filters)).map(|c| c.id).collect();
    Ok(Json(MatchCount { count: ids.len(), total: cands.len(), ids }))
}

/// A slip that pairs with the drawn one, offered as "make it a pair".
#[derive(Debug, Serialize)]
pub struct PairOffer {
    pub id: i64,
    pub title: String,
    pub cost: String,
    pub duration: String,
    /// Combined cost of both, e.g. "₹4,300".
    pub together: String,
    /// e.g. "₹2,300 over tonight's budget", or empty.
    pub over: String,
}

/// The card `POST /pick` and `POST /accept/{id}` answer with (Joy's `pick_card.html`).
#[derive(Debug, Serialize)]
pub struct PickCard {
    pub activity: Option<ActivityView>,
    /// Shown when nothing fits (empty when there is a pick).
    pub message: String,
    /// One-tap fix for the no-match case: button label + the loosened filters.
    pub relax_label: String,
    /// The loosened filters as Joy's hidden fields.
    pub relax_hidden: Vec<Hidden>,
    /// The loosened filters as a form to post to `/pick`, or None when there's nothing to loosen.
    pub relax: Option<PickForm>,
    pub done: bool,
    /// For the stamp on an accepted pick, e.g. "14 · 10 · 2026".
    pub stamp_date: String,
    pub rerolls_left: u32,
    pub max_rerolls: u32,
    /// The drawn slip was generated for us, so putting it back is free.
    pub free_putback: bool,
    pub pairs: Vec<PairOffer>,
    /// On an accepted pair: the second slip.
    pub paired: Option<ActivityView>,
    /// Joy's hidden fields for a reroll.
    pub hidden: Vec<Hidden>,
    /// The form to post to `/pick` (with `reroll`/`veto` set) to peek again; None on a done card.
    pub state: Option<PickForm>,
    /// This draw has a bonus peek because it's this person's turn.
    pub bonus_peek: bool,
    /// The slip can be kept as a surprise for the other.
    pub can_surprise: bool,
    /// They can pass on this slip ("Not for me").
    pub can_veto: bool,
    /// The other one of us ("Neha").
    pub partner: String,
    /// Accepted as a surprise (done card).
    pub surprise: bool,
    /// History row ids of the accepted date (two for a pair), comma separated, for "when is it?".
    pub history_ids: String,
    /// WhatsApp share: the message, and a ready wa.me link.
    pub share_text: String,
    pub wa_url: String,
    /// Drawn from tonight's check-ins.
    pub middle: bool,
    /// The date this draw is for ("2026-11-05" or "any"), passed on to "It's a date".
    pub when: String,
    /// "office"/"home" when the day starts somewhere other than usual, for accept.
    pub start: String,
    /// "Thursday evening", or empty.
    pub when_label: String,
    /// On a drawn card: the plan in a line ("leave 7:30 pm · home by 10:40 pm").
    pub plan_line: String,
    /// On a drawn card: the day's timeline, before we say yes to it.
    pub route: Vec<dayplan::StepView>,
    /// "Open 12:00 pm – 11:30 pm" that day, or a nudge to check.
    pub route_note: String,
    /// On a kept card: the whole day-of plan.
    pub plan: Option<dayplan::DayPlan>,
}

impl PickCard {
    fn blank() -> Self {
        Self {
            activity: None,
            message: String::new(),
            relax_label: String::new(),
            relax_hidden: vec![],
            relax: None,
            done: false,
            stamp_date: String::new(),
            rerolls_left: 0,
            max_rerolls: MAX_REROLLS,
            free_putback: false,
            middle: false,
            pairs: vec![],
            paired: None,
            hidden: vec![],
            state: None,
            bonus_peek: false,
            can_surprise: false,
            can_veto: false,
            partner: String::new(),
            surprise: false,
            history_ids: String::new(),
            share_text: String::new(),
            wa_url: String::new(),
            when: String::new(),
            start: String::new(),
            route: vec![],
            route_note: String::new(),
            when_label: String::new(),
            plan_line: String::new(),
            plan: None,
        }
    }
}

/// Who's drawing and how many peeks they get. Worked out on the server every time.
struct Drawer {
    me: Me,
    max: u32,
}

async fn drawer(db: &SqlitePool, couple: &str, ctx: &Ctx, who: usize) -> Result<Drawer, AppError> {
    let turn = turn(db, couple).await?;
    Ok(Drawer { me: Me::new(ctx, Some(who)), max: picker::max_peeks(turn == Some(who)) })
}

/// `POST /pick`: draw a slip, peek again (`reroll`), or pass on one for good (`veto`).
pub async fn pick(
    State(s): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    body: Option<Json<PickForm>>,
) -> Result<Json<PickCard>, AppError> {
    let f = body.map(|j| j.0).unwrap_or_default();
    let j = joy(&s, &user_id).await?;
    let (db, couple, ctx) = (&s.db, j.couple.as_str(), &j.ctx);
    let d = drawer(db, couple, ctx, j.who).await?;
    let middle = if f.is_middle() { middle::tonight(db, couple).await? } else { None };
    if f.is_middle() && middle.is_none() {
        return Ok(Json(PickCard {
            message: "Once you've both checked in for tonight, the jar draws from the middle.".into(),
            ..PickCard::blank()
        }));
    }
    let f = match &middle {
        Some(m) => f.met(m),
        None => f,
    };
    let mut excluded = f.excluded();
    let mut used: u32 = f.used.parse().unwrap_or(0);

    // "Not for me": out of the draw for both of us, and it doesn't cost a peek. A slip that
    // isn't ours does nothing (the veto is scoped to the couple).
    if f.veto == "1" {
        if let Ok(cur) = f.current.parse::<i64>() {
            super::unsealed(db, couple, j.who, cur).await?;
            db::veto(db, couple, cur, j.who).await?;
            excluded.push(cur);
        }
    } else if f.reroll == "1"
        && let Ok(cur) = f.current.parse::<i64>() {
            super::unsealed(db, couple, j.who, cur).await?;
            let current = db::get(db, couple, cur).await?;
            match picker::put_back(used, current.as_ref().is_some_and(|a| a.generated), d.max) {
                Some(next) => {
                    db::record(db, couple, cur, false).await?;
                    excluded.push(cur);
                    used = next;
                }
                // Out of peeks: the card already hides the button, so just show it again.
                None => {
                    if let Some(a) = current {
                        let learned = learned(db, couple).await?;
                        let slot = dayplan::slot_for(db, ctx, &f.when, f.start()).await?;
                        return Ok(Json(card(db, couple, ctx, &d, &f, Some(&a), &excluded, used, &learned, slot).await?));
                    }
                }
            }
        }

    let jar = jar_data(db, couple, ctx, j.who, dayplan::slot_for(db, ctx, &f.when, f.start()).await?).await?;
    let mut cands = jar.candidates(ctx, &f);
    if let Some(m) = &middle {
        cands.retain(|c| m.allows(&c.energy, &c.tags));
    }
    let filters = jar.filters(&f, rain(ctx).await.is_some());
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(internal)?.as_secs() as i64;
    let result = picker::pick(&cands, &filters, &excluded, now, &mut thread_rng());

    let card_out = match result {
        PickResult::Pick(id) => {
            card(db, couple, ctx, &d, &f, jar.acts.iter().find(|a| a.id == id), &excluded, used, &jar.learned, jar.slot).await?
        }
        PickResult::NoMatch(Hint::OutsideWindow { title, why }) if middle.is_none() => {
            let label = jar.slot.map(|sl| sl.label()).unwrap_or_default();
            let relax = PickForm { when: "any".into(), excluded: String::new(), used: String::new(), current: String::new(), reroll: String::new(), ..f.clone() };
            PickCard {
                message: format!("Nothing in the jar fits {label}. Closest: {title}, but it's {why}."),
                relax_label: "Draw for any day".into(),
                relax_hidden: hidden_of(relax.filter_fields()),
                relax: Some(relax.kept()),
                ..PickCard::blank()
            }
        }
        // Saying which filter to loosen would give away one of our answers.
        PickResult::NoMatch(h) if middle.is_some() => PickCard {
            message: match h {
                Hint::Exhausted => "You've peeked at every slip that fits you both. One of you could change your check-in.".into(),
                _ => "Nothing in the jar fits you both tonight. One of you could change your check-in, or draw on your own.".into(),
            },
            ..PickCard::blank()
        },
        PickResult::NoMatch(h) => {
            let (relax_label, relax) = relax(ctx, &f, &h);
            PickCard {
                message: hint_text(ctx, &f, &filters, h),
                relax_label,
                relax_hidden: relax.as_ref().map(|r| hidden_of(r.filter_fields())).unwrap_or_default(),
                relax: relax.map(|r| r.kept()),
                ..PickCard::blank()
            }
        }
    };
    Ok(Json(card_out))
}

#[allow(clippy::too_many_arguments)]
async fn card(
    db: &SqlitePool,
    couple: &str,
    ctx: &Ctx,
    d: &Drawer,
    f: &PickForm,
    a: Option<&db::Activity>,
    excluded: &[i64],
    used: u32,
    learned: &Learned,
    slot: Option<Slot>,
) -> Result<PickCard, AppError> {
    let pairs = match a {
        Some(a) => pair_offers(db, couple, ctx, d.me.who, f, a, learned).await?,
        None => vec![],
    };
    let mut view = a.map(|a| ActivityView::new(ctx, a, learned));
    // From the office, door to door from home doesn't apply: the timeline has the real times.
    if let (Some(v), Some(schedule::Kind::Weeknight)) = (view.as_mut(), slot.map(|sl| sl.kind)) {
        v.door.clear();
    }
    // "Send to Diya" sends an envelope to open, not the slip spelled out in the chat.
    let share = match a {
        Some(a) => envelope::wa_text(ctx, &db::slip_token(db, couple, a.id).await?, "Found one in our jar for us. Open the envelope:"),
        None => String::new(),
    };
    let fit = match (a, &slot, &ctx.schedule) {
        (Some(a), Some(slot), Some(sched)) => sched.fit(ctx, slot, a).ok(),
        _ => None,
    };
    let plan_line = match (&fit, &slot) {
        (Some(fit), Some(slot)) => plan_line(ctx, slot, fit),
        _ => String::new(),
    };
    // The day as it would go, shown before "It's a date" so we can decide with it.
    let (route, route_note) = match (a, &slot, &ctx.schedule, &fit) {
        (Some(a), Some(slot), Some(sched), Some(fit)) => (
            schedule::timeline(sched, ctx, slot, a, fit)
                .into_iter()
                .map(|st| dayplan::StepView { mine: st.who.is_some() && st.who == d.me.who, at: st.at, minutes: st.minutes, what: st.what })
                .collect(),
            match crate::joy::hours::Hours::parse(&a.hours) {
                _ if matches!(a.setting.as_str(), "home" | "apart") => String::new(),
                Some(h) => h.label(slot.weekday()),
                None => "Hours not confirmed: check before you go".into(),
            },
        ),
        _ => (vec![], String::new()),
    };
    let current = a.map_or(0, |a| a.id);
    Ok(PickCard {
        when: slot.map(|sl| sl.key()).unwrap_or_else(|| "any".into()),
        start: f.start().into(),
        route,
        route_note,
        when_label: slot.map(|sl| sl.label()).unwrap_or_default(),
        plan_line,
        hidden: hidden_of(f.hidden(excluded, used, current)),
        state: Some(f.state(excluded, used, current)),
        rerolls_left: picker::rerolls_left(used, d.max),
        max_rerolls: d.max,
        bonus_peek: d.max > MAX_REROLLS,
        free_putback: a.is_some_and(|a| a.generated),
        can_surprise: d.me.who.is_some(),
        can_veto: d.me.who.is_some(),
        partner: d.me.partner.clone(),
        wa_url: envelope::wa_url(&share),
        middle: f.is_middle(),
        share_text: share,
        activity: view,
        pairs,
        ..PickCard::blank()
    })
}

/// The plan in a line, for a drawn slip: when to go and when each of us is back.
fn plan_line(ctx: &Ctx, slot: &Slot, fit: &schedule::Fit) -> String {
    use schedule::{at, Kind};
    match slot.kind {
        Kind::Apart => format!("{} on a call", at(fit.start)),
        Kind::Weeknight => {
            let leave = fit.start - fit.there[0].map_or(0, |t| t.minutes);
            let homes: Vec<String> = (0..2)
                .filter_map(|w| Some(format!("{} back {}", ctx.people[w].0, at(fit.end + fit.back[w]?.minutes))))
                .collect();
            let mut parts = vec![format!("leave {}", at(leave)), format!("there {}", at(fit.start))];
            parts.extend(homes);
            parts.join(" · ")
        }
        Kind::Weekend => {
            let mut parts = vec![format!("{} – {}", at(fit.start), at(fit.end))];
            parts.extend((0..2).filter_map(|w| Some(format!("{} {} each way", ctx.people[w].0, crate::joy::travel::duration_label(fit.there[w]?.minutes)))));
            parts.join(" · ")
        }
    }
}

/// Up to two partners for the drawn slip, with the combined cost against tonight's budget.
async fn pair_offers(db: &SqlitePool, couple: &str, ctx: &Ctx, who: Option<usize>, f: &PickForm, a: &db::Activity, learned: &Learned) -> Result<Vec<PairOffer>, AppError> {
    let budget = f.filters().budget;
    let vetoed: Vec<i64> = db::vetoes(db, couple).await?.into_iter().map(|(a, _)| a).collect();
    let sealed = match who {
        Some(w) => db::sealed_from(db, couple, w).await?,
        None => db::sealed_any(db, couple).await?,
    };
    Ok(db::partners(db, couple, a.id)
        .await?
        .iter()
        .filter(|p| !sealed.contains(&p.id))
        .filter(|p| !vetoed.contains(&p.id))
        .take(2)
        .map(|p| {
            // Same costs the jar uses everywhere else.
            let (mine, theirs) = (a.effective_cost(learned.cost), p.effective_cost(learned.cost));
            let together = mine + theirs;
            PairOffer {
                id: p.id,
                title: ctx.personalize(&p.title),
                cost: ctx.money(theirs),
                duration: p.duration().label().into(),
                together: ctx.money(together),
                over: if budget != i64::MAX && together > budget {
                    format!("{} over tonight's budget", ctx.money(together - budget))
                } else {
                    String::new()
                },
            }
        })
        .collect())
}

fn hint_text(ctx: &Ctx, form: &PickForm, f: &Filters, h: Hint) -> String {
    match h {
        Hint::EmptyWishlist => "The jar is empty. Write the first slip.".into(),
        Hint::Exhausted => "You've peeked at every slip that fits. Loosen something, or go with one you put back.".into(),
        Hint::OverBudget { title, cost } if f.budget == 0 => {
            format!("Nothing in the jar is free. Closest: {title} at {}.", ctx.money(cost))
        }
        Hint::OverBudget { title, cost } => format!(
            "Nothing in the jar fits under {}. Closest: {title} at {}.",
            ctx.money(f.budget),
            ctx.money(cost)
        ),
        Hint::NeedsMoreTime { title, duration } => {
            format!("Nothing fits that time. {title} needs {}.", duration.label().to_lowercase())
        }
        Hint::TooFar { title, km } => format!(
            "Nothing within {} km {}. Nearest: {title} (~{km} km).",
            f.max_km.unwrap_or_default(),
            match form.from() {
                Some(i) => format!("of {}'s place", ctx.people[i].0),
                None if form.from == "office" => format!("of {}", ctx.schedule.as_ref().map_or("the office", |sc| sc.office_name.as_str())),
                None => "for both of you".into(),
            }
        ),
        Hint::OtherSetting { title, setting } => format!(
            "Nothing in those places. {title} ({}) would fit.",
            picker::setting_label(&setting).to_lowercase()
        ),
        Hint::ClosedToday { title, weekday } => format!(
            "{title} would fit, but it's closed on {}s.",
            picker::WEEKDAYS.get(weekday as usize).map_or("this day", |d| d.1)
        ),
        Hint::OutOfSeason { title, from, to } => {
            format!("{title} would fit, but it's only on {}.", picker::season_label(from, to))
        }
        Hint::OutsideWindow { title, why } => format!("Nothing fits that day. Closest: {title}, but it's {why}."),
        Hint::NoMoodMatch => "Nothing matches that vibe or energy.".into(),
    }
}

/// The smallest change to the filters that would make the hinted slip fit.
fn relax(ctx: &Ctx, f: &PickForm, h: &Hint) -> (String, Option<PickForm>) {
    let mut r = PickForm { excluded: String::new(), used: String::new(), current: String::new(), reroll: String::new(), ..f.clone() };
    let label = match h {
        Hint::OverBudget { cost, .. } => {
            r.budget = cost.to_string();
            format!("Stretch to {}", ctx.money(*cost))
        }
        Hint::NeedsMoreTime { duration, .. } => {
            r.max_duration = duration.key().into();
            format!("Make it {}", duration.label().to_lowercase())
        }
        Hint::TooFar { km, .. } => {
            let km = (km + 4) / 5 * 5;
            r.max_km = km.to_string();
            format!("Go up to {km} km")
        }
        Hint::OtherSetting { setting, .. } => {
            r.settings.push(setting.clone());
            format!("Include {}", picker::setting_label(setting).to_lowercase())
        }
        // Nothing to loosen: we can't change the day or the season.
        Hint::ClosedToday { .. } | Hint::OutOfSeason { .. } => return (String::new(), None),
        Hint::OutsideWindow { .. } => {
            r.when = "any".into();
            "Draw for any day".into()
        }
        Hint::NoMoodMatch => {
            r.tag.clear();
            r.energy.clear();
            "Any vibe, any energy".into()
        }
        Hint::Exhausted => {
            return ("Start over with these filters".into(), Some(r));
        }
        Hint::EmptyWishlist => return (String::new(), None),
    };
    (label, Some(r))
}

/// Options for `POST /accept/{id}`, read from the query string and/or a JSON body (a non-empty
/// body value wins). Every value is a string (numbers and `true` are accepted).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AcceptOpts {
    /// Accept this partner slip too, as one date.
    #[serde(deserialize_with = "flex")]
    pub with: String,
    /// Keep it as a surprise for the other one of us ("1" or true).
    #[serde(deserialize_with = "flex")]
    pub surprise: String,
    /// The date it was drawn for ("2026-11-05"), so the jar can set the time and plan the day.
    #[serde(deserialize_with = "flex")]
    pub when: String,
    /// Where we start that day, when it isn't the usual ("office" or "home").
    #[serde(deserialize_with = "flex")]
    pub start: String,
}

fn or(first: String, second: String) -> String {
    if second.is_empty() { first } else { second }
}

/// `POST /accept/{id}`: "It's a date". Keeps the slip (and a paired one) as a date.
pub async fn accept(
    State(s): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Path(id): Path<i64>,
    Query(q): Query<AcceptOpts>,
    body: Option<Json<AcceptOpts>>,
) -> Result<Json<PickCard>, AppError> {
    let b = body.map(|j| j.0).unwrap_or_default();
    let q = AcceptOpts {
        with: or(q.with, b.with),
        surprise: or(q.surprise, b.surprise),
        when: or(q.when, b.when),
        start: or(q.start, b.start),
    };
    let j = joy(&s, &user_id).await?;
    let (db, couple, ctx) = (&s.db, j.couple.as_str(), &j.ctx);
    let who = Some(j.who);
    // A slip that isn't ours isn't here.
    let Some(a) = db::visible_get(db, couple, j.who, id).await? else { return Err(not_found("No such slip")) };
    let surprise = q.surprise == "1";
    let learned = learned(db, couple).await?;
    let mut ids = vec![db::accept(db, couple, id, who, surprise).await?];
    let mut paired = None;
    if let Ok(with) = q.with.trim().parse::<i64>() {
        // Only a real partner of this slip can ride along.
        if let Some(p) = db::visible_partners(db, couple, j.who, id).await?.into_iter().find(|p| p.id == with) {
            ids.push(db::accept(db, couple, p.id, who, surprise).await?);
            paired = Some(ActivityView::new(ctx, &p, &learned));
        }
    }
    // Only the exact date it was drawn for: a card left open overnight shouldn't plan another day.
    let slot = dayplan::slot_exact(db, ctx, &q.when, &q.start).await?;
    dayplan::on_keep(db, couple, ctx, &ids, slot.as_ref(), who, surprise).await?;
    let plan = match db::history_row(db, couple, ids[0]).await? {
        Some(r) => dayplan::build(db, couple, ctx, &r, who).await?,
        None => None,
    };
    let mut view = Some(ActivityView::new(ctx, &a, &learned));
    if let (Some(v), Some(schedule::Kind::Weeknight)) = (view.as_mut(), slot.map(|sl| sl.kind)) {
        v.door.clear();
    }
    // The kept date goes over as an envelope: the plan is inside, and a surprise stays sealed.
    let line = if surprise { "I've planned something for us. Open the envelope (it stays sealed till the day):" } else { "It's a date! Open the envelope:" };
    let share = envelope::wa_text(ctx, &db::date_token(db, couple, ids[0]).await?, line);
    Ok(Json(PickCard {
        plan,
        done: true,
        stamp_date: db::today(db).await?,
        paired,
        surprise,
        partner: Me::new(ctx, who).partner,
        history_ids: ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","),
        wa_url: envelope::wa_url(&share),
        share_text: share,
        activity: view,
        ..PickCard::blank()
    }))
}
