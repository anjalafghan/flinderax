//! `/api/diet`: weekly diet habits, food money, body marks. Phase 5. All private to one user:
//! every query filters by `user_id`, and an unknown or foreign id is a 404.

use std::collections::{BTreeMap, HashMap, HashSet};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date, Duration};

use crate::{
    handlers::common::{not_found, today_local, AppError},
    models::AppState,
    planner::{
        checkin::Period,
        diet::{
            add_notes, is_delivery, per_week_x100, share_bps, DietHabitRate, DietIn, DietReport, HabitIn,
            MAX_ACTIVE, PRESETS,
        },
    },
};

type User = Extension<(String, String)>;

fn unprocessable(msg: impl Into<String>) -> AppError {
    AppError(StatusCode::UNPROCESSABLE_ENTITY, msg.into())
}

fn parse_day(s: &str) -> Option<Date> {
    Date::parse(s, format_description!("[year]-[month]-[day]")).ok()
}

fn month_key(d: Date) -> String {
    format!("{:04}-{:02}", d.year(), d.month() as u8)
}

fn first_of_month(d: Date) -> Date {
    d - Duration::days(i64::from(d.day()) - 1)
}

/// Weekday number with Sunday as 0, the same as `user_settings.week_start`.
fn weekday_num(d: Date) -> i64 {
    i64::from(d.weekday().number_days_from_sunday())
}

/// The first day of the week `d` falls in.
fn week_start_of(d: Date, week_start: i64) -> Date {
    d - Duration::days((weekday_num(d) - week_start).rem_euclid(7))
}

async fn week_start_setting(db: &sqlx::SqlitePool, user_id: &str) -> Result<i64, AppError> {
    let ws: Option<i64> = sqlx::query_scalar("SELECT week_start FROM user_settings WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await?;
    Ok(ws.unwrap_or(1))
}

// ---------------------------------------------------------------------------------------------
// Types

#[derive(Debug, Serialize)]
pub struct Preset {
    pub key: &'static str,
    pub label: &'static str,
    pub target_days: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DietHabit {
    pub id: String,
    pub label: String,
    pub preset_key: Option<String>,
    pub target_days: i64,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub habit_id: String,
    pub days_hit: i64,
}

#[derive(Debug, Serialize)]
pub struct Week {
    pub week_start: String,
    pub feel: Option<i64>,
    pub note: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
pub struct RecentWeek {
    pub week_start: String,
    pub logged: bool,
    pub hit_bps: Option<i64>,
}

#[derive(Debug, Serialize, Default)]
pub struct MonthMoney {
    pub month: String,
    pub eating_out_paise: i64,
    pub groceries_paise: i64,
    pub delivery_orders: i64,
}

#[derive(Debug, Serialize)]
pub struct FoodMoney {
    pub months: Vec<MonthMoney>,
    pub eating_out_share_bps: Option<i64>,
    pub delivery_per_week_x100: i64,
    pub cap_paise: Option<i64>,
}

#[derive(Debug, Serialize, Default)]
pub struct Body {
    pub weight_x10: Option<i64>,
    pub weight_on: Option<String>,
    pub prev_weight_x10: Option<i64>,
    pub waist_x10: Option<i64>,
    pub last_checkup_on: Option<String>,
    pub checkup_due_on: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DietOverview {
    pub presets: Vec<Preset>,
    pub habits: Vec<DietHabit>,
    pub week: Week,
    pub recent: Vec<RecentWeek>,
    pub money: FoodMoney,
    pub body: Body,
}

#[derive(Debug, Serialize)]
pub struct Mark {
    pub id: String,
    pub kind: String,
    pub measured_on: String,
    pub value_x10: Option<i64>,
    pub note: String,
}

// ---------------------------------------------------------------------------------------------
// Habits

type HabitRow = (String, String, Option<String>, i64, i64);

fn habit_of(r: HabitRow) -> DietHabit {
    DietHabit { id: r.0, label: r.1, preset_key: r.2, target_days: r.3, active: r.4 != 0 }
}

async fn load_habits(db: &sqlx::SqlitePool, user_id: &str, active_only: bool) -> Result<Vec<DietHabit>, AppError> {
    let sql = if active_only {
        "SELECT habit_id, label, preset_key, target_days, active FROM diet_habits
         WHERE user_id = ? AND active = 1 ORDER BY sort, created_at, habit_id"
    } else {
        "SELECT habit_id, label, preset_key, target_days, active FROM diet_habits
         WHERE user_id = ? ORDER BY sort, created_at, habit_id"
    };
    let rows: Vec<HabitRow> = sqlx::query_as(sql).bind(user_id).fetch_all(db).await?;
    Ok(rows.into_iter().map(habit_of).collect())
}

async fn find_habit(db: &sqlx::SqlitePool, user_id: &str, id: &str) -> Result<DietHabit, AppError> {
    let row: Option<HabitRow> = sqlx::query_as(
        "SELECT habit_id, label, preset_key, target_days, active FROM diet_habits WHERE habit_id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(db)
    .await?;
    row.map(habit_of).ok_or_else(|| not_found("That habit was not found."))
}

async fn active_count(db: &sqlx::SqlitePool, user_id: &str, except: Option<&str>) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM diet_habits WHERE user_id = ? AND active = 1 AND habit_id != ?")
        .bind(user_id)
        .bind(except.unwrap_or(""))
        .fetch_one(db)
        .await?)
}

fn limit_reached() -> AppError {
    AppError(StatusCode::CONFLICT, "You can track up to three at a time.".to_string())
}

fn check_label(label: &str) -> Result<String, AppError> {
    let label = label.trim();
    if label.is_empty() || label.chars().count() > 60 {
        return Err(unprocessable("Give the habit a name of up to 60 characters."));
    }
    Ok(label.to_string())
}

fn check_target(days: i64) -> Result<i64, AppError> {
    if (1..=7).contains(&days) {
        Ok(days)
    } else {
        Err(unprocessable("Days a week must be between 1 and 7."))
    }
}

#[derive(Deserialize)]
pub struct NewHabit {
    pub label: String,
    pub preset_key: Option<String>,
    pub target_days: i64,
}

pub async fn create_habit(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<NewHabit>,
) -> Result<Json<DietHabit>, AppError> {
    let db = &state.db;
    let label = check_label(&p.label)?;
    let target = check_target(p.target_days)?;
    if let Some(key) = &p.preset_key
        && !PRESETS.iter().any(|(k, ..)| k == key)
    {
        return Err(unprocessable("That is not one of the ready-made habits."));
    }
    if active_count(db, &user_id, None).await? >= MAX_ACTIVE {
        return Err(limit_reached());
    }
    let id = nanoid!();
    let next_sort: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(sort), -1) + 1 FROM diet_habits WHERE user_id = ?")
        .bind(&user_id)
        .fetch_one(db)
        .await?;
    sqlx::query(
        "INSERT INTO diet_habits (habit_id, user_id, label, preset_key, target_days, active, sort) VALUES (?, ?, ?, ?, ?, 1, ?)",
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&label)
    .bind(&p.preset_key)
    .bind(target)
    .bind(next_sort)
    .execute(db)
    .await?;
    Ok(Json(find_habit(db, &user_id, &id).await?))
}

#[derive(Deserialize)]
pub struct HabitPatch {
    pub label: Option<String>,
    pub target_days: Option<i64>,
    pub active: Option<bool>,
    pub sort: Option<i64>,
}

pub async fn update_habit(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
    Json(p): Json<HabitPatch>,
) -> Result<Json<DietHabit>, AppError> {
    let db = &state.db;
    let cur = find_habit(db, &user_id, &id).await?;
    let label = match &p.label {
        Some(l) => check_label(l)?,
        None => cur.label.clone(),
    };
    let target = match p.target_days {
        Some(t) => check_target(t)?,
        None => cur.target_days,
    };
    let active = p.active.unwrap_or(cur.active);
    if active && !cur.active && active_count(db, &user_id, Some(&id)).await? >= MAX_ACTIVE {
        return Err(limit_reached());
    }
    sqlx::query(
        "UPDATE diet_habits SET label = ?, target_days = ?, active = ?, sort = COALESCE(?, sort)
         WHERE habit_id = ? AND user_id = ?",
    )
    .bind(&label)
    .bind(target)
    .bind(i64::from(active))
    .bind(p.sort)
    .bind(&id)
    .bind(&user_id)
    .execute(db)
    .await?;
    Ok(Json(find_habit(db, &user_id, &id).await?))
}

pub async fn delete_habit(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    find_habit(&state.db, &user_id, &id).await?;
    sqlx::query("UPDATE diet_habits SET active = 0 WHERE habit_id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------------------------
// Weeks

async fn week_out(db: &sqlx::SqlitePool, user_id: &str, week_start: &str) -> Result<Week, AppError> {
    let head: Option<(Option<i64>, String)> =
        sqlx::query_as("SELECT feel, note FROM diet_weeks WHERE user_id = ? AND week_start = ?")
            .bind(user_id)
            .bind(week_start)
            .fetch_optional(db)
            .await?;
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT w.habit_id, w.days_hit FROM diet_week_habits w JOIN diet_habits h ON h.habit_id = w.habit_id
         WHERE w.user_id = ? AND w.week_start = ? ORDER BY h.sort, h.created_at, h.habit_id",
    )
    .bind(user_id)
    .bind(week_start)
    .fetch_all(db)
    .await?;
    let (feel, note) = head.unwrap_or((None, String::new()));
    Ok(Week {
        week_start: week_start.to_string(),
        feel,
        note,
        entries: rows.into_iter().map(|(habit_id, days_hit)| Entry { habit_id, days_hit }).collect(),
    })
}

#[derive(Deserialize)]
pub struct EntryIn {
    pub habit_id: String,
    pub days_hit: i64,
}

#[derive(Deserialize)]
pub struct WeekPayload {
    pub feel: Option<i64>,
    pub note: Option<String>,
    #[serde(default)]
    pub entries: Vec<EntryIn>,
}

/// `GET /weeks/{week_start}`: what was logged for that week (empty entries when nothing was).
pub async fn get_week(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(week_start): Path<String>,
) -> Result<Json<Week>, AppError> {
    let start = parse_day(&week_start).ok_or_else(|| unprocessable("The week must look like 2026-11-09."))?;
    Ok(Json(week_out(&state.db, &user_id, &start.to_string()).await?))
}

pub async fn put_week(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(week_start): Path<String>,
    Json(p): Json<WeekPayload>,
) -> Result<Json<Week>, AppError> {
    Ok(Json(put_week_at(&state.db, &user_id, &week_start, p, today_local()).await?))
}

async fn put_week_at(
    db: &sqlx::SqlitePool,
    user_id: &str,
    week_start: &str,
    p: WeekPayload,
    today: Date,
) -> Result<Week, AppError> {
    let ws = week_start_setting(db, user_id).await?;
    let start = parse_day(week_start).ok_or_else(|| unprocessable("The week must look like 2026-11-09."))?;
    if weekday_num(start) != ws {
        return Err(unprocessable("That date is not the first day of a week."));
    }
    let current = week_start_of(today, ws);
    if start > current {
        return Err(unprocessable("That week has not started yet."));
    }
    if start < current - Duration::days(56) {
        return Err(unprocessable("You can fill in weeks up to eight weeks back."));
    }
    if p.feel.is_some_and(|f| !(1..=5).contains(&f)) {
        return Err(unprocessable("How it felt must be between 1 and 5."));
    }
    let note = p.note.unwrap_or_default();
    if note.chars().count() > 280 {
        return Err(unprocessable("The note can be up to 280 characters."));
    }
    let mut seen = HashSet::new();
    for e in &p.entries {
        if !(0..=7).contains(&e.days_hit) {
            return Err(unprocessable("Days must be between 0 and 7."));
        }
        if !seen.insert(e.habit_id.as_str()) {
            return Err(unprocessable("Each habit can appear once."));
        }
    }
    let own: HashSet<String> = load_habits(db, user_id, false).await?.into_iter().map(|h| h.id).collect();
    if p.entries.iter().any(|e| !own.contains(&e.habit_id)) {
        return Err(not_found("That habit was not found."));
    }

    let mut tx = db.begin().await?;
    sqlx::query(
        "INSERT INTO diet_weeks (user_id, week_start, feel, note, updated_at) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(user_id, week_start) DO UPDATE SET feel = excluded.feel, note = excluded.note, updated_at = CURRENT_TIMESTAMP",
    )
    .bind(user_id)
    .bind(week_start)
    .bind(p.feel)
    .bind(&note)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM diet_week_habits WHERE user_id = ? AND week_start = ?")
        .bind(user_id)
        .bind(week_start)
        .execute(&mut *tx)
        .await?;
    for e in &p.entries {
        sqlx::query("INSERT INTO diet_week_habits (user_id, week_start, habit_id, days_hit) VALUES (?, ?, ?, ?)")
            .bind(user_id)
            .bind(week_start)
            .bind(&e.habit_id)
            .bind(e.days_hit)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    week_out(db, user_id, week_start).await
}

// ---------------------------------------------------------------------------------------------
// Marks

type MarkRow = (String, String, String, Option<i64>, String);

fn mark_of(r: MarkRow) -> Mark {
    Mark { id: r.0, kind: r.1, measured_on: r.2, value_x10: r.3, note: r.4 }
}

#[derive(Deserialize)]
pub struct NewMark {
    pub kind: String,
    pub measured_on: String,
    pub value_x10: Option<i64>,
    pub note: Option<String>,
}

pub async fn create_mark(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<NewMark>,
) -> Result<Json<Mark>, AppError> {
    Ok(Json(create_mark_at(&state.db, &user_id, p, today_local()).await?))
}

async fn create_mark_at(db: &sqlx::SqlitePool, user_id: &str, p: NewMark, today: Date) -> Result<Mark, AppError> {
    let range = match p.kind.as_str() {
        "weight" => Some((200, 3000)),
        "waist" => Some((300, 2000)),
        "checkup" => None,
        _ => return Err(unprocessable("The kind must be weight, waist or checkup.")),
    };
    let on = parse_day(&p.measured_on).ok_or_else(|| unprocessable("The date must look like 2026-11-09."))?;
    if on > today {
        return Err(unprocessable("The date cannot be in the future."));
    }
    match (range, p.value_x10) {
        (Some((lo, hi)), Some(v)) if (lo..=hi).contains(&v) => {}
        (Some(_), Some(_)) => return Err(unprocessable("That number looks off. Check it and try again.")),
        (Some(_), None) => return Err(unprocessable("Add the number for this measurement.")),
        (None, Some(_)) => return Err(unprocessable("A check-up has a date only.")),
        (None, None) => {}
    }
    let note = p.note.unwrap_or_default();
    if note.chars().count() > 140 {
        return Err(unprocessable("The note can be up to 140 characters."));
    }
    let id = nanoid!();
    sqlx::query("INSERT INTO health_marks (mark_id, user_id, kind, measured_on, value_x10, note) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(user_id)
        .bind(&p.kind)
        .bind(on.to_string())
        .bind(p.value_x10)
        .bind(&note)
        .execute(db)
        .await?;
    Ok(Mark { id, kind: p.kind, measured_on: on.to_string(), value_x10: p.value_x10, note })
}

#[derive(Deserialize)]
pub struct MarksQuery {
    pub kind: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct MarksOut {
    pub marks: Vec<Mark>,
}

pub async fn list_marks(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Query(q): Query<MarksQuery>,
) -> Result<Json<MarksOut>, AppError> {
    let limit = q.limit.unwrap_or(12);
    if !(1..=50).contains(&limit) {
        return Err(unprocessable("The limit must be between 1 and 50."));
    }
    if q.kind.as_deref().is_some_and(|k| !["weight", "waist", "checkup"].contains(&k)) {
        return Err(unprocessable("The kind must be weight, waist or checkup."));
    }
    let rows: Vec<MarkRow> = sqlx::query_as(
        "SELECT mark_id, kind, measured_on, value_x10, note FROM health_marks
         WHERE user_id = ? AND (? IS NULL OR kind = ?)
         ORDER BY measured_on DESC, created_at DESC, mark_id LIMIT ?",
    )
    .bind(&user_id)
    .bind(&q.kind)
    .bind(&q.kind)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(MarksOut { marks: rows.into_iter().map(mark_of).collect() }))
}

pub async fn delete_mark(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let done = sqlx::query("DELETE FROM health_marks WHERE mark_id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    if done.rows_affected() == 0 {
        return Err(not_found("That entry was not found."));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------------------------
// Settings

#[derive(Deserialize, Serialize)]
pub struct SettingsPayload {
    #[serde(default)]
    pub eating_out_cap_paise: Option<i64>,
}

pub async fn put_settings(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<SettingsPayload>,
) -> Result<Json<SettingsPayload>, AppError> {
    if p.eating_out_cap_paise.is_some_and(|c| c <= 0) {
        return Err(unprocessable("The cap must be more than zero, or empty for none."));
    }
    sqlx::query(
        "INSERT INTO user_settings (user_id, eating_out_cap_paise) VALUES (?, ?)
         ON CONFLICT(user_id) DO UPDATE SET eating_out_cap_paise = excluded.eating_out_cap_paise",
    )
    .bind(&user_id)
    .bind(p.eating_out_cap_paise)
    .execute(&state.db)
    .await?;
    Ok(Json(p))
}

async fn cap_of(db: &sqlx::SqlitePool, user_id: &str) -> Result<Option<i64>, AppError> {
    let cap: Option<Option<i64>> = sqlx::query_scalar("SELECT eating_out_cap_paise FROM user_settings WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await?;
    Ok(cap.flatten())
}

// ---------------------------------------------------------------------------------------------
// Food money

/// One food or groceries debit.
struct Spend {
    on: String,
    eating_out: bool,
    delivery: bool,
    paise: i64,
}

async fn spend_rows(db: &sqlx::SqlitePool, user_id: &str, from: Date, to: Date) -> Result<Vec<Spend>, AppError> {
    let rows: Vec<(String, String, i64, Option<String>)> = sqlx::query_as(
        "SELECT substr(occurred_on, 1, 10), category, amount_paise, merchant FROM transactions
         WHERE user_id = ? AND direction = 'debit' AND category IN ('food', 'groceries')
           AND substr(occurred_on, 1, 10) >= ? AND substr(occurred_on, 1, 10) <= ?",
    )
    .bind(user_id)
    .bind(from.to_string())
    .bind(to.to_string())
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(on, category, paise, merchant)| {
            let eating_out = category == "food";
            Spend { on, eating_out, delivery: eating_out && is_delivery(merchant.as_deref().unwrap_or("")), paise }
        })
        .collect())
}

#[derive(Default, Clone, Copy)]
struct Totals {
    eating_out: i64,
    groceries: i64,
    orders: i64,
}

impl Totals {
    fn any(&self) -> bool {
        self.eating_out + self.groceries > 0
    }
}

fn totals(rows: &[Spend], from: Date, to: Date) -> Totals {
    let (from, to) = (from.to_string(), to.to_string());
    let mut t = Totals::default();
    for r in rows.iter().filter(|r| r.on >= from && r.on <= to) {
        if r.eating_out {
            t.eating_out += r.paise;
            t.orders += i64::from(r.delivery);
        } else {
            t.groceries += r.paise;
        }
    }
    t
}

/// The first day of this month and of the five before it, oldest first.
fn six_months(today: Date) -> Vec<Date> {
    let mut starts = vec![first_of_month(today)];
    for _ in 0..5 {
        let last = starts[starts.len() - 1];
        starts.push(first_of_month(last - Duration::days(1)));
    }
    starts.reverse();
    starts
}

async fn food_money(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<FoodMoney, AppError> {
    let starts = six_months(today);
    let rows = spend_rows(db, user_id, starts[0], today).await?;
    let months = starts
        .iter()
        .map(|&s| {
            let end = if first_of_month(today) == s { today } else { s + Duration::days(i64::from(s.month().length(s.year())) - 1) };
            let t = totals(&rows, s, end);
            MonthMoney { month: month_key(s), eating_out_paise: t.eating_out, groceries_paise: t.groceries, delivery_orders: t.orders }
        })
        .collect();
    let t90 = totals(&rows, today - Duration::days(89), today);
    Ok(FoodMoney {
        months,
        eating_out_share_bps: share_bps(t90.eating_out, t90.groceries),
        delivery_per_week_x100: per_week_x100(t90.orders, 90),
        cap_paise: cap_of(db, user_id).await?,
    })
}

// ---------------------------------------------------------------------------------------------
// Body

async fn latest_mark(db: &sqlx::SqlitePool, user_id: &str, kind: &str) -> Result<Option<(String, Option<i64>)>, AppError> {
    Ok(sqlx::query_as(
        "SELECT measured_on, value_x10 FROM health_marks WHERE user_id = ? AND kind = ?
         ORDER BY measured_on DESC, created_at DESC, mark_id LIMIT 1",
    )
    .bind(user_id)
    .bind(kind)
    .fetch_optional(db)
    .await?)
}

async fn last_checkup(db: &sqlx::SqlitePool, user_id: &str) -> Result<Option<Date>, AppError> {
    Ok(latest_mark(db, user_id, "checkup").await?.and_then(|(on, _)| parse_day(&on)))
}

async fn body(db: &sqlx::SqlitePool, user_id: &str) -> Result<Body, AppError> {
    let weight = latest_mark(db, user_id, "weight").await?;
    let prev_weight_x10 = match weight.as_ref().and_then(|(on, _)| parse_day(on)) {
        Some(on) => {
            let cutoff = (on - Duration::days(60)).to_string();
            sqlx::query_scalar(
                "SELECT value_x10 FROM health_marks WHERE user_id = ? AND kind = 'weight' AND measured_on <= ?
                 ORDER BY measured_on DESC, created_at DESC, mark_id LIMIT 1",
            )
            .bind(user_id)
            .bind(cutoff)
            .fetch_optional(db)
            .await?
            .flatten()
        }
        None => None,
    };
    let checkup = last_checkup(db, user_id).await?;
    Ok(Body {
        weight_x10: weight.as_ref().and_then(|(_, v)| *v),
        weight_on: weight.map(|(on, _)| on),
        prev_weight_x10,
        waist_x10: latest_mark(db, user_id, "waist").await?.and_then(|(_, v)| v),
        last_checkup_on: checkup.map(|d| d.to_string()),
        checkup_due_on: checkup.map(|d| (d + Duration::days(365)).to_string()),
    })
}

// ---------------------------------------------------------------------------------------------
// Overview

pub async fn overview(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
) -> Result<Json<DietOverview>, AppError> {
    Ok(Json(overview_at(&state.db, &user_id, today_local()).await?))
}

async fn overview_at(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<DietOverview, AppError> {
    let ws = week_start_setting(db, user_id).await?;
    let current = week_start_of(today, ws);
    let habits = load_habits(db, user_id, true).await?;

    let weeks: Vec<Date> = (0..12).map(|i| current - Duration::days(7 * i)).collect();
    let oldest = weeks[11].to_string();
    let logged: HashSet<String> = sqlx::query_scalar("SELECT week_start FROM diet_weeks WHERE user_id = ? AND week_start >= ?")
        .bind(user_id)
        .bind(&oldest)
        .fetch_all(db)
        .await?
        .into_iter()
        .collect();
    let entries: Vec<(String, String, i64)> =
        sqlx::query_as("SELECT week_start, habit_id, days_hit FROM diet_week_habits WHERE user_id = ? AND week_start >= ?")
            .bind(user_id)
            .bind(&oldest)
            .fetch_all(db)
            .await?;
    let mut hits: HashMap<(String, String), i64> = HashMap::new();
    for (w, h, d) in entries {
        hits.insert((w, h), d);
    }
    let recent = weeks
        .iter()
        .map(|w| {
            let key = w.to_string();
            let is_logged = logged.contains(&key);
            let hit_bps = (is_logged && !habits.is_empty()).then(|| {
                let sum: i64 = habits
                    .iter()
                    .map(|h| {
                        let d = hits.get(&(key.clone(), h.id.clone())).copied().unwrap_or(0);
                        d.min(h.target_days) * 10_000 / h.target_days
                    })
                    .sum();
                sum / habits.len() as i64
            });
            RecentWeek { week_start: key, logged: is_logged, hit_bps }
        })
        .collect();

    Ok(DietOverview {
        presets: PRESETS.iter().map(|&(key, label, target_days)| Preset { key, label, target_days }).collect(),
        habits,
        week: week_out(db, user_id, &current.to_string()).await?,
        recent,
        money: food_money(db, user_id, today).await?,
        body: body(db, user_id).await?,
    })
}

// ---------------------------------------------------------------------------------------------
// Habit rules input

/// The habit rules' input for one user.
pub async fn habits_input(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<DietIn, AppError> {
    let ws = week_start_setting(db, user_id).await?;
    let current = week_start_of(today, ws);
    let habits: Vec<HabitIn> = load_habits(db, user_id, true)
        .await?
        .into_iter()
        .map(|h| HabitIn { id: h.id, label: h.label, target_days: h.target_days })
        .collect();

    let logged_weeks: Vec<String> =
        sqlx::query_scalar("SELECT week_start FROM diet_weeks WHERE user_id = ? ORDER BY week_start DESC LIMIT 8")
            .bind(user_id)
            .fetch_all(db)
            .await?;
    let prev = (current - Duration::days(7)).to_string();
    let week_logged = logged_weeks.contains(&current.to_string());
    let prev_week_logged = logged_weeks.contains(&prev);
    let mut recent_weeks = Vec::new();
    for w in logged_weeks.iter().take(4) {
        let rows: Vec<(String, i64)> =
            sqlx::query_as("SELECT habit_id, days_hit FROM diet_week_habits WHERE user_id = ? AND week_start = ?")
                .bind(user_id)
                .bind(w)
                .fetch_all(db)
                .await?;
        recent_weeks.push(rows);
    }

    let starts = six_months(today);
    let rows = spend_rows(db, user_id, starts[0], today).await?;
    let t90 = totals(&rows, today - Duration::days(89), today);
    let month_start = first_of_month(today);
    let this_month = totals(&rows, month_start, today);
    let last_month = totals(&rows, first_of_month(month_start - Duration::days(1)), month_start - Duration::days(1));

    Ok(DietIn {
        habits,
        week_start: Some(current),
        week_logged,
        prev_week_logged,
        recent_weeks,
        eating_out_90d_paise: t90.eating_out,
        groceries_90d_paise: t90.groceries,
        delivery_90d_orders: t90.orders,
        eating_out_month_paise: this_month.eating_out,
        eating_out_last_month_paise: last_month.eating_out,
        day: today.day(),
        days_in_month: today.month().length(today.year()),
        cap_paise: cap_of(db, user_id).await?,
        last_checkup: last_checkup(db, user_id).await?,
    })
}

// ---------------------------------------------------------------------------------------------
// Report

/// The diet section for a check-in period, or None when there is nothing to say.
pub async fn report(
    db: &sqlx::SqlitePool,
    user_id: &str,
    period: Period,
    today: Date,
) -> Result<Option<DietReport>, AppError> {
    let prev = period.previous();
    let rows = spend_rows(db, user_id, prev.start, period.end).await?;
    let now = totals(&rows, period.start, period.end);
    let before = totals(&rows, prev.start, prev.end);

    let habits = load_habits(db, user_id, true).await?;
    let marks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM health_marks WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(db)
        .await?;
    if !now.any() && !before.any() && habits.is_empty() && marks == 0 {
        return Ok(None);
    }

    let ws = week_start_setting(db, user_id).await?;
    let weeks_in_period = (0..period.days()).filter(|i| weekday_num(period.start + Duration::days(*i)) == ws).count() as i64;
    let (from, to) = (period.start.to_string(), period.end.to_string());
    let weeks_logged: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM diet_weeks WHERE user_id = ? AND week_start >= ? AND week_start <= ?")
            .bind(user_id)
            .bind(&from)
            .bind(&to)
            .fetch_one(db)
            .await?;
    let entries: Vec<(String, i64)> = sqlx::query_as(
        "SELECT habit_id, days_hit FROM diet_week_habits WHERE user_id = ? AND week_start >= ? AND week_start <= ?",
    )
    .bind(user_id)
    .bind(&from)
    .bind(&to)
    .fetch_all(db)
    .await?;
    let mut by_habit: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for (id, d) in entries {
        by_habit.entry(id).or_default().push(d);
    }
    let rates = habits
        .iter()
        .map(|h| {
            let days = by_habit.get(&h.id).map(Vec::as_slice).unwrap_or(&[]);
            let logged = days.len() as i64;
            let hit_bps = (logged > 0)
                .then(|| days.iter().map(|d| (*d).min(h.target_days) * 10_000 / h.target_days).sum::<i64>() / logged);
            DietHabitRate {
                label: h.label.clone(),
                target_days: h.target_days,
                hit_bps,
                weeks_on_target: days.iter().filter(|d| **d >= h.target_days).count() as i64,
                weeks_logged: logged,
            }
        })
        .collect();

    let weight: Option<i64> = sqlx::query_scalar(
        "SELECT value_x10 FROM health_marks WHERE user_id = ? AND kind = 'weight' AND measured_on >= ? AND measured_on <= ?
         ORDER BY measured_on DESC, created_at DESC, mark_id LIMIT 1",
    )
    .bind(user_id)
    .bind(&from)
    .bind(&to)
    .fetch_optional(db)
    .await?
    .flatten();
    let before_weight: Option<i64> = sqlx::query_scalar(
        "SELECT value_x10 FROM health_marks WHERE user_id = ? AND kind = 'weight' AND measured_on < ?
         ORDER BY measured_on DESC, created_at DESC, mark_id LIMIT 1",
    )
    .bind(user_id)
    .bind(&from)
    .fetch_optional(db)
    .await?
    .flatten();

    let mut report = DietReport {
        eating_out_paise: now.eating_out,
        groceries_paise: now.groceries,
        prev_eating_out_paise: before.any().then_some(before.eating_out),
        prev_groceries_paise: before.any().then_some(before.groceries),
        eating_out_share_bps: share_bps(now.eating_out, now.groceries),
        delivery_per_week_x100: per_week_x100(now.orders, period.days()),
        weeks_logged,
        weeks_in_period,
        habits: rates,
        weight_x10: weight,
        weight_change_x10: weight.zip(before_weight).map(|(w, b)| w - b),
        checkup_due_on: last_checkup(db, user_id).await?.map(|d| (d + Duration::days(365)).to_string()),
        wins: Vec::new(),
        watch: Vec::new(),
    };
    let prev_delivery = before.any().then(|| per_week_x100(before.orders, prev.days()));
    add_notes(&mut report, prev_delivery, today);
    Ok(Some(report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use time::macros::date;

    /// The in-memory DB pattern from `handlers::checkin`.
    async fn test_state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('other', 'other', 'x', 'user')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32][..]),
        );
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("handler failed: {} {}", e.0, e.1),
        }
    }

    fn code<T>(r: Result<T, AppError>) -> StatusCode {
        match r {
            Ok(_) => panic!("expected an error"),
            Err(e) => e.0,
        }
    }

    fn who(id: &str) -> User {
        Extension((id.to_string(), String::new()))
    }

    // Monday 9 Nov 2026.
    fn today() -> Date {
        date!(2026 - 11 - 09)
    }

    async fn add_habit(s: &AppState, user: &str, label: &str, target: i64) -> DietHabit {
        let Json(h) = ok(create_habit(
            State(s.clone()),
            who(user),
            Json(NewHabit { label: label.into(), preset_key: None, target_days: target }),
        )
        .await);
        h
    }

    fn week(feel: Option<i64>, entries: &[(&str, i64)]) -> WeekPayload {
        WeekPayload {
            feel,
            note: None,
            entries: entries.iter().map(|(h, d)| EntryIn { habit_id: (*h).into(), days_hit: *d }).collect(),
        }
    }

    fn mark(kind: &str, on: &str, v: Option<i64>) -> NewMark {
        NewMark { kind: kind.into(), measured_on: on.into(), value_x10: v, note: None }
    }

    async fn seed(s: &AppState, statements: &[&str]) {
        for q in statements {
            sqlx::query(q).execute(&s.db).await.unwrap_or_else(|e| panic!("{q}: {e}"));
        }
    }

    fn txn(id: &str, user: &str, on: &str, paise: i64, merchant: &str, category: &str) -> String {
        format!(
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key)
             VALUES ('{id}', '{user}', '{on}', 'debit', {paise}, '{merchant}', '{category}', 'UPI', 'manual', '{id}')"
        )
    }

    #[tokio::test]
    async fn habits_validate_limit_and_reactivate() {
        let s = test_state().await;
        let bad = |label: &str, target: i64| NewHabit { label: label.into(), preset_key: None, target_days: target };
        assert_eq!(code(create_habit(State(s.clone()), who("u"), Json(bad("   ", 3))).await), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(code(create_habit(State(s.clone()), who("u"), Json(bad(&"x".repeat(61), 3))).await), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(code(create_habit(State(s.clone()), who("u"), Json(bad("Walk", 0))).await), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(code(create_habit(State(s.clone()), who("u"), Json(bad("Walk", 8))).await), StatusCode::UNPROCESSABLE_ENTITY);
        let bad_preset = NewHabit { label: "Walk".into(), preset_key: Some("nope".into()), target_days: 3 };
        assert_eq!(code(create_habit(State(s.clone()), who("u"), Json(bad_preset)).await), StatusCode::UNPROCESSABLE_ENTITY);

        let a = add_habit(&s, "u", "  Home-cooked dinner ", 5).await;
        assert_eq!((a.label.as_str(), a.target_days, a.active), ("Home-cooked dinner", 5, true));
        let b = add_habit(&s, "u", "Walk 30 minutes", 4).await;
        add_habit(&s, "u", "Water", 5).await;
        // A fourth is refused with the plain message; another user is unaffected.
        let Err(e) = create_habit(State(s.clone()), who("u"), Json(bad("Fourth", 3))).await else { panic!("expected 409") };
        assert_eq!((e.0, e.1.as_str()), (StatusCode::CONFLICT, "You can track up to three at a time."));
        add_habit(&s, "other", "Their habit", 2).await;

        // Remove one, add another, then reactivating the removed one hits the limit.
        assert_eq!(ok(delete_habit(State(s.clone()), who("u"), Path(b.id.clone())).await), StatusCode::NO_CONTENT);
        add_habit(&s, "u", "Fourth", 3).await;
        let react = HabitPatch { label: None, target_days: None, active: Some(true), sort: None };
        assert_eq!(code(update_habit(State(s.clone()), who("u"), Path(b.id.clone()), Json(react)).await), StatusCode::CONFLICT);

        // Editing an active habit never trips the limit.
        let edit = HabitPatch { label: Some("Dinner at home".into()), target_days: Some(6), active: Some(true), sort: Some(9) };
        let Json(h) = ok(update_habit(State(s.clone()), who("u"), Path(a.id.clone()), Json(edit)).await);
        assert_eq!((h.label.as_str(), h.target_days), ("Dinner at home", 6));
        let none = HabitPatch { label: Some(String::new()), target_days: None, active: None, sort: None };
        assert_eq!(code(update_habit(State(s.clone()), who("u"), Path(a.id.clone()), Json(none)).await), StatusCode::UNPROCESSABLE_ENTITY);

        // Deactivated rows are kept, and the overview lists only active ones.
        let ov = ok(overview_at(&s.db, "u", today()).await);
        assert_eq!(ov.habits.len(), 3);
        assert!(ov.habits.iter().all(|h| h.id != b.id));
        assert_eq!(ov.presets.len(), 6);
        let kept: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM diet_habits WHERE habit_id = ?")
            .bind(&b.id)
            .fetch_one(&s.db)
            .await
            .unwrap();
        assert_eq!(kept, 1);
    }

    #[tokio::test]
    async fn another_users_habit_is_a_404() {
        let s = test_state().await;
        let theirs = add_habit(&s, "other", "Their habit", 3).await;
        let patch = HabitPatch { label: Some("Mine now".into()), target_days: None, active: None, sort: None };
        assert_eq!(code(update_habit(State(s.clone()), who("u"), Path(theirs.id.clone()), Json(patch)).await), StatusCode::NOT_FOUND);
        assert_eq!(code(delete_habit(State(s.clone()), who("u"), Path(theirs.id.clone())).await), StatusCode::NOT_FOUND);
        assert_eq!(code(delete_habit(State(s.clone()), who("u"), Path("missing".into())).await), StatusCode::NOT_FOUND);
        // Logging against it is a 404 too, and nothing is written.
        let r = put_week_at(&s.db, "u", "2026-11-09", week(None, &[(&theirs.id, 3)]), today()).await;
        assert_eq!(code(r), StatusCode::NOT_FOUND);
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM diet_weeks WHERE user_id = 'u'").fetch_one(&s.db).await.unwrap();
        assert_eq!(rows, 0);
        // Still active for its owner.
        assert!(ok(overview_at(&s.db, "other", today()).await).habits[0].active);
    }

    #[tokio::test]
    async fn weeks_validate_and_replace() {
        let s = test_state().await;
        let a = add_habit(&s, "u", "Home-cooked dinner", 5).await;
        let b = add_habit(&s, "u", "Walk 30 minutes", 4).await;
        let put = |ws: &str, p: WeekPayload| {
            let db = s.db.clone();
            let ws = ws.to_string();
            async move { put_week_at(&db, "u", &ws, p, today()).await }
        };
        let unp = StatusCode::UNPROCESSABLE_ENTITY;
        assert_eq!(code(put("2026-11-10", week(None, &[])).await), unp); // a Tuesday
        assert_eq!(code(put("nonsense", week(None, &[])).await), unp);
        assert_eq!(code(put("2026-11-16", week(None, &[])).await), unp); // next week
        assert_eq!(code(put("2026-09-07", week(None, &[])).await), unp); // 9 weeks back
        assert_eq!(code(put("2026-11-09", week(Some(0), &[])).await), unp);
        assert_eq!(code(put("2026-11-09", week(Some(6), &[])).await), unp);
        assert_eq!(code(put("2026-11-09", week(None, &[(&a.id, 8)])).await), unp);
        assert_eq!(code(put("2026-11-09", week(None, &[(&a.id, -1)])).await), unp);
        assert_eq!(code(put("2026-11-09", week(None, &[(&a.id, 1), (&a.id, 2)])).await), unp);
        let long = WeekPayload { feel: None, note: Some("n".repeat(281)), entries: vec![] };
        assert_eq!(code(put("2026-11-09", long).await), unp);
        // Eight weeks back is the edge and is allowed.
        ok(put("2026-09-14", week(None, &[])).await);

        let w = ok(put("2026-11-09", week(Some(4), &[(&a.id, 3), (&b.id, 4)])).await);
        assert_eq!((w.feel, w.entries.len()), (Some(4), 2));
        // Replace: one entry, new feel, and a note.
        let p = WeekPayload { feel: None, note: Some("Busy week".into()), entries: vec![EntryIn { habit_id: a.id.clone(), days_hit: 5 }] };
        let w = ok(put("2026-11-09", p).await);
        assert_eq!((w.feel, w.note.as_str(), w.entries.len(), w.entries[0].days_hit), (None, "Busy week", 1, 5));

        let ov = ok(overview_at(&s.db, "u", today()).await);
        assert_eq!(ov.week.week_start, "2026-11-09");
        assert_eq!(ov.week.entries.len(), 1);
        assert_eq!(ov.recent.len(), 12);
        assert_eq!(ov.recent[0].week_start, "2026-11-09");
        // 5 of 5 for a, nothing for b: half.
        assert_eq!((ov.recent[0].logged, ov.recent[0].hit_bps), (true, Some(5_000)));
        assert_eq!((ov.recent[1].logged, ov.recent[1].hit_bps), (false, None));
        assert!(ov.recent[8].logged && ov.recent[8].week_start == "2026-09-14");

        // A past week reads back for filling in; another user's read of the same week is empty.
        let got = |user: &'static str| {
            let st = s.clone();
            async move { ok(get_week(State(st), Extension((user.to_string(), String::new())), Path("2026-11-09".into())).await).0 }
        };
        let mine = got("u").await;
        assert_eq!((mine.note.as_str(), mine.entries.len()), ("Busy week", 1));
        let theirs = got("other").await;
        assert!(theirs.entries.is_empty() && theirs.note.is_empty());
    }

    #[tokio::test]
    async fn weeks_follow_the_users_week_start() {
        let s = test_state().await;
        seed(&s, &["INSERT INTO user_settings (user_id, week_start) VALUES ('u', 0)"]).await;
        let a = add_habit(&s, "u", "Walk", 4).await;
        // Sunday 8 Nov starts the current week, so Monday is no longer a week start.
        assert_eq!(code(put_week_at(&s.db, "u", "2026-11-09", week(None, &[]), today()).await), StatusCode::UNPROCESSABLE_ENTITY);
        ok(put_week_at(&s.db, "u", "2026-11-08", week(None, &[(&a.id, 2)]), today()).await);
        assert_eq!(ok(overview_at(&s.db, "u", today()).await).week.week_start, "2026-11-08");
        // Another user with no settings row defaults to Monday.
        assert_eq!(ok(overview_at(&s.db, "other", today()).await).week.week_start, "2026-11-09");
    }

    #[tokio::test]
    async fn marks_validate_list_delete_and_isolate() {
        let s = test_state().await;
        let unp = StatusCode::UNPROCESSABLE_ENTITY;
        let c = |p: NewMark| {
            let db = s.db.clone();
            async move { create_mark_at(&db, "u", p, today()).await }
        };
        assert_eq!(code(c(mark("mood", "2026-11-01", Some(5))).await), unp);
        assert_eq!(code(c(mark("weight", "2026-11-01", None)).await), unp);
        assert_eq!(code(c(mark("weight", "2026-11-01", Some(199))).await), unp);
        assert_eq!(code(c(mark("weight", "2026-11-01", Some(3001))).await), unp);
        assert_eq!(code(c(mark("waist", "2026-11-01", Some(299))).await), unp);
        assert_eq!(code(c(mark("waist", "2026-11-01", Some(2001))).await), unp);
        assert_eq!(code(c(mark("checkup", "2026-11-01", Some(1))).await), unp);
        assert_eq!(code(c(mark("weight", "2026-11-10", Some(700))).await), unp);
        assert_eq!(code(c(mark("weight", "11/01/2026", Some(700))).await), unp);
        let long = NewMark { note: Some("n".repeat(141)), ..mark("weight", "2026-11-01", Some(700)) };
        assert_eq!(code(c(long).await), unp);

        let w1 = ok(c(mark("weight", "2026-08-01", Some(712))).await);
        ok(c(mark("weight", "2026-10-12", Some(705))).await);
        ok(c(mark("waist", "2026-10-12", Some(850))).await);
        let chk = ok(c(mark("checkup", "2025-11-20", None)).await);
        assert_eq!(chk.value_x10, None);
        ok(create_mark_at(&s.db, "other", mark("weight", "2026-11-01", Some(900)), today()).await);

        let Json(all) = ok(list_marks(State(s.clone()), who("u"), Query(MarksQuery { kind: None, limit: None })).await);
        assert_eq!(all.marks.len(), 4);
        let Json(ws) = ok(list_marks(State(s.clone()), who("u"), Query(MarksQuery { kind: Some("weight".into()), limit: Some(1) })).await);
        assert_eq!((ws.marks.len(), ws.marks[0].value_x10, ws.marks[0].measured_on.as_str()), (1, Some(705), "2026-10-12"));
        for limit in [0, 51] {
            assert_eq!(code(list_marks(State(s.clone()), who("u"), Query(MarksQuery { kind: None, limit: Some(limit) })).await), unp);
        }
        assert_eq!(code(list_marks(State(s.clone()), who("u"), Query(MarksQuery { kind: Some("x".into()), limit: None })).await), unp);

        let ov = ok(overview_at(&s.db, "u", today()).await);
        let b = &ov.body;
        assert_eq!((b.weight_x10, b.weight_on.as_deref(), b.prev_weight_x10, b.waist_x10), (Some(705), Some("2026-10-12"), Some(712), Some(850)));
        assert_eq!((b.last_checkup_on.as_deref(), b.checkup_due_on.as_deref()), (Some("2025-11-20"), Some("2026-11-20")));

        // Another user's mark id is a 404 and survives.
        let theirs: String = sqlx::query_scalar("SELECT mark_id FROM health_marks WHERE user_id = 'other'").fetch_one(&s.db).await.unwrap();
        assert_eq!(code(delete_mark(State(s.clone()), who("u"), Path(theirs.clone())).await), StatusCode::NOT_FOUND);
        assert_eq!(ok(delete_mark(State(s.clone()), who("other"), Path(theirs)).await), StatusCode::NO_CONTENT);
        assert_eq!(ok(delete_mark(State(s.clone()), who("u"), Path(w1.id.clone())).await), StatusCode::NO_CONTENT);
        assert_eq!(code(delete_mark(State(s.clone()), who("u"), Path(w1.id)).await), StatusCode::NOT_FOUND);
        // With the old weight gone there is nothing 60 days earlier.
        assert_eq!(ok(overview_at(&s.db, "u", today()).await).body.prev_weight_x10, None);
        let none = ok(overview_at(&s.db, "other", today()).await);
        assert_eq!(none.body.weight_x10, None);
    }

    #[tokio::test]
    async fn settings_cap_round_trip() {
        let s = test_state().await;
        let put = |cap: Option<i64>| {
            let s = s.clone();
            async move { put_settings(State(s), who("u"), Json(SettingsPayload { eating_out_cap_paise: cap })).await }
        };
        assert_eq!(code(put(Some(0)).await), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(code(put(Some(-5)).await), StatusCode::UNPROCESSABLE_ENTITY);
        let Json(r) = ok(put(Some(300_000)).await);
        assert_eq!(r.eating_out_cap_paise, Some(300_000));
        assert_eq!(ok(overview_at(&s.db, "u", today()).await).money.cap_paise, Some(300_000));
        assert_eq!(ok(overview_at(&s.db, "other", today()).await).money.cap_paise, None);
        // Saving a cap keeps the other settings row intact, and null clears it.
        let _ = ok(put(None).await);
        assert_eq!(ok(overview_at(&s.db, "u", today()).await).money.cap_paise, None);
    }

    #[tokio::test]
    async fn food_money_counts_months_share_and_delivery() {
        let s = test_state().await;
        let stmts = [
            // Nov: eating out 1,000 (one Swiggy order), groceries 500.
            txn("t1", "u", "2026-11-03", 60_000, "Swiggy", "food"),
            txn("t2", "u", "2026-11-05", 40_000, "Corner Cafe", "food"),
            txn("t3", "u", "2026-11-06", 50_000, "Grocer", "groceries"),
            // Oct: a Zomato and a Dominos order, groceries 1,500.
            txn("t4", "u", "2026-10-10", 30_000, "ZOMATO", "food"),
            txn("t5", "u", "2026-10-12", 20_000, "Dominos Pizza", "food"),
            txn("t6", "u", "2026-10-20", 150_000, "Grocer", "groceries"),
            // A swiggy payment tagged groceries is not a delivery order; credits and other users are ignored.
            txn("t7", "u", "2026-10-21", 10_000, "Swiggy Instamart", "groceries"),
            txn("t8", "other", "2026-11-03", 999_999, "Swiggy", "food"),
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key)
             VALUES ('t9', 'u', '2026-11-04', 'credit', 777777, 'Swiggy', 'food', 'UPI', 'manual', 't9')".to_string(),
            // Too old for any of the six months.
            txn("t10", "u", "2026-05-31", 88_888, "Swiggy", "food"),
        ];
        let refs: Vec<&str> = stmts.iter().map(String::as_str).collect();
        seed(&s, &refs).await;
        let m = ok(overview_at(&s.db, "u", today()).await).money;
        let months: Vec<&str> = m.months.iter().map(|x| x.month.as_str()).collect();
        assert_eq!(months, ["2026-06", "2026-07", "2026-08", "2026-09", "2026-10", "2026-11"]);
        let nov = &m.months[5];
        assert_eq!((nov.eating_out_paise, nov.groceries_paise, nov.delivery_orders), (100_000, 50_000, 1));
        let oct = &m.months[4];
        assert_eq!((oct.eating_out_paise, oct.groceries_paise, oct.delivery_orders), (50_000, 160_000, 2));
        assert_eq!(m.months[0].eating_out_paise, 0);
        // 90 days back from 9 Nov reaches 12 Aug: eating out 150,000 of 360,000 = 41.67%; 3 orders.
        assert_eq!(m.eating_out_share_bps, Some(4_167));
        assert_eq!(m.delivery_per_week_x100, 23);
        assert_eq!(ok(overview_at(&s.db, "other", today()).await).money.delivery_per_week_x100, 8);
        assert_eq!(ok(overview_at(&s.db, "nobody", today()).await).money.eating_out_share_bps, None);

        // The habit rules' input sees the same figures: heavy eating out raises the habit.
        let input = ok(habits_input(&s.db, "u", today()).await);
        assert_eq!((input.eating_out_90d_paise, input.groceries_90d_paise, input.delivery_90d_orders), (150_000, 210_000, 3));
        assert_eq!((input.eating_out_month_paise, input.eating_out_last_month_paise), (100_000, 50_000));
        let h = crate::planner::diet::habits(&input, today());
        assert!(h.iter().any(|x| x.key == "eating_out"));
        assert!(h.iter().any(|x| x.key == "health_checkup"));
    }

    #[tokio::test]
    async fn habit_rules_input_tracks_logged_weeks() {
        let s = test_state().await;
        let a = add_habit(&s, "u", "Home-cooked dinner", 5).await;
        ok(put_week_at(&s.db, "u", "2026-11-02", week(None, &[(&a.id, 3)]), today()).await);
        let input = ok(habits_input(&s.db, "u", today()).await);
        assert_eq!(input.week_start, Some(today()));
        assert_eq!((input.week_logged, input.prev_week_logged), (false, true));
        assert_eq!(input.recent_weeks.len(), 1);
        let h = crate::planner::diet::habits(&input, today());
        let w = h.iter().find(|x| x.key == "diet_week").unwrap();
        assert_eq!(w.why, "Home-cooked dinner is at 3 of 5 days lately.");
        // Nobody else's habits show up.
        assert!(ok(habits_input(&s.db, "other", today()).await).habits.is_empty());
    }

    #[tokio::test]
    async fn report_numbers_for_a_quarter() {
        let s = test_state().await;
        let a = add_habit(&s, "u", "Home-cooked dinner", 5).await;
        let b = add_habit(&s, "u", "Walk 30 minutes", 4).await;
        // Period 1 Jul..30 Sep 2026 (92 days), previous period 1 Apr..30 Jun.
        let period = Period { start: date!(2026 - 07 - 01), end: date!(2026 - 09 - 30) };
        let stmts = [
            txn("t1", "u", "2026-07-10", 310_000, "Swiggy", "food"),
            txn("t2", "u", "2026-08-10", 100_000, "Cafe", "food"),
            txn("t3", "u", "2026-08-11", 690_000, "Grocer", "groceries"),
            txn("p1", "u", "2026-05-10", 440_000, "Cafe", "food"),
            txn("p2", "u", "2026-05-11", 560_000, "Grocer", "groceries"),
            txn("x1", "other", "2026-08-10", 999_999, "Swiggy", "food"),
        ];
        let refs: Vec<&str> = stmts.iter().map(String::as_str).collect();
        seed(&s, &refs).await;
        // Mondays in the period: 6 Jul .. 28 Sep = 13 weeks. Log four weeks (put_week_at is relative to "today").
        let t = date!(2026 - 09 - 30);
        for (w, ad, bd) in [("2026-07-06", 5, 4), ("2026-07-13", 6, 4), ("2026-07-20", 3, 1), ("2026-09-28", 5, 4)] {
            let on = parse_day(w).unwrap() + Duration::days(2);
            ok(put_week_at(&s.db, "u", w, week(None, &[(&a.id, ad), (&b.id, bd)]), on).await);
        }
        for (kind, on, v) in [("weight", "2026-06-20", Some(712)), ("weight", "2026-09-20", Some(700)), ("checkup", "2025-09-01", None)] {
            ok(create_mark_at(&s.db, "u", mark(kind, on, v), t).await);
        }
        let r = ok(report(&s.db, "u", period, date!(2026 - 10 - 01)).await).expect("a report");
        assert_eq!((r.eating_out_paise, r.groceries_paise), (410_000, 690_000));
        assert_eq!((r.prev_eating_out_paise, r.prev_groceries_paise), (Some(440_000), Some(560_000)));
        assert_eq!(r.eating_out_share_bps, Some(3_727));
        assert_eq!(r.delivery_per_week_x100, 8); // 1 order over 92 days
        assert_eq!((r.weeks_in_period, r.weeks_logged), (13, 4));
        assert_eq!(r.habits.len(), 2);
        let h = &r.habits[0];
        assert_eq!((h.label.as_str(), h.weeks_logged, h.weeks_on_target), ("Home-cooked dinner", 4, 3));
        // min(5,5) 10000 + 10000 + 6000 + 10000 = 36000 / 4
        assert_eq!(h.hit_bps, Some(9_000));
        assert_eq!(r.habits[1].weeks_on_target, 3);
        assert_eq!((r.weight_x10, r.weight_change_x10), (Some(700), Some(-12)));
        assert_eq!(r.checkup_due_on.as_deref(), Some("2026-09-01"));
        assert!(r.watch.iter().any(|w| w == "Your yearly check-up is overdue since 1 Sep."), "{:?}", r.watch);
        assert!(r.wins.iter().any(|w| w == "Eating out fell to 37% of food, from 44%."), "{:?}", r.wins);
        assert!(r.wins.len() <= 2 && r.watch.len() <= 2);
        for line in r.wins.iter().chain(&r.watch) {
            assert!(!line.contains("kg") && !line.contains("70"), "{line}");
        }
    }

    #[tokio::test]
    async fn report_is_none_without_anything_and_private() {
        let s = test_state().await;
        let period = Period { start: date!(2026 - 07 - 01), end: date!(2026 - 09 - 30) };
        assert!(ok(report(&s.db, "u", period, today()).await).is_none());
        // Another user's spend, habits and marks do not make a report for this user.
        let a = add_habit(&s, "other", "Walk", 3).await;
        ok(put_week_at(&s.db, "other", "2026-09-07", week(None, &[(&a.id, 3)]), date!(2026 - 09 - 30)).await);
        ok(create_mark_at(&s.db, "other", mark("weight", "2026-08-01", Some(700)), today()).await);
        seed(&s, &[&txn("x1", "other", "2026-08-10", 100_000, "Cafe", "food")]).await;
        assert!(ok(report(&s.db, "u", period, today()).await).is_none());
        assert!(ok(report(&s.db, "other", period, today()).await).is_some());

        // Only spend in the previous period still gives a report, with no previous figures for this one.
        seed(&s, &[&txn("p1", "u", "2026-05-10", 100_000, "Cafe", "food")]).await;
        let r = ok(report(&s.db, "u", period, today()).await).expect("report from prior spend");
        assert_eq!((r.eating_out_paise, r.eating_out_share_bps, r.prev_eating_out_paise), (0, None, Some(100_000)));
        assert_eq!(r.weeks_logged, 0);
    }
}
