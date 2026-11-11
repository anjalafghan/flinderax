//! Rewards: small treats you pick now and unlock by reaching a goal (or a share of it) or a
//! streak. `GET /api/rewards` lists them with their state (locked / earned / claimed), this month's
//! claimed spend and the cap. "Earned" is always worked out fresh from the goal's saved money and
//! the streaks, so a goal you take money back out of locks its reward again before it is claimed.
//! A monthly cap (optional) stops treats from eating the goals: a claim that would pass it is
//! refused, atomically, so two quick taps can't slip past.

use std::collections::HashMap;

use axum::{extract::State, http::StatusCode, Extension, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use time::Date;

use crate::{
    handlers::{
        common::{bad_request, not_found, today_local, AppError},
        streaks::{self, StreaksOut},
    },
    models::AppState,
    planner::rewards::{cap_left, goal_reached, state as reward_state, streak_reached, RewardState, GOAL_PCTS, MAX_STREAK_TARGET},
    validate,
};

type User = Extension<(String, String)>;

/// Most rewards one person can hold (archived ones count), so the list can't grow without bound.
const MAX_REWARDS: i64 = 200;
const MAX_NAME_CHARS: usize = 80;

#[derive(sqlx::FromRow)]
struct RewardRow {
    reward_id: String,
    name: String,
    cost_paise: i64,
    trigger_kind: String,
    goal_id: Option<String>,
    goal_pct: Option<i64>,
    streak_kind: Option<String>,
    streak_target: Option<i64>,
    claimed_on: Option<String>,
}

#[derive(Serialize)]
pub struct RewardOut {
    pub reward_id: String,
    pub name: String,
    pub cost_paise: i64,
    pub trigger_kind: String,
    pub goal_id: Option<String>,
    pub goal_pct: Option<i64>,
    pub streak_kind: Option<String>,
    pub streak_target: Option<i64>,
    pub state: RewardState,
    pub claimed_on: Option<String>,
}

#[derive(Serialize)]
pub struct HistoryOut {
    pub reward_id: String,
    pub name: String,
    pub cost_paise: i64,
    pub claimed_on: String,
}

#[derive(Serialize)]
pub struct RewardsOut {
    pub rewards: Vec<RewardOut>,
    /// Everything claimed, newest first (removed rewards stay here).
    pub history: Vec<HistoryOut>,
    pub monthly_cap_paise: Option<i64>,
    pub spent_this_month_paise: i64,
    /// What the cap still allows this month; null when there is no cap.
    pub cap_left_paise: Option<i64>,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

fn month_of(day: Date) -> String {
    format!("{:04}-{:02}", day.year(), day.month() as u8)
}

async fn cap_of(db: &sqlx::SqlitePool, user_id: &str) -> Result<Option<i64>, AppError> {
    let cap: Option<Option<i64>> = sqlx::query_scalar("SELECT monthly_cap_paise FROM reward_settings WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await?;
    Ok(cap.flatten())
}

async fn spent_in(db: &sqlx::SqlitePool, user_id: &str, month: &str) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar("SELECT COALESCE(SUM(cost_paise), 0) FROM reward_claims WHERE user_id = ? AND substr(claimed_on, 1, 7) = ?")
        .bind(user_id)
        .bind(month)
        .fetch_one(db)
        .await?)
}

/// Whether one reward's trigger has been met. `goals` maps goal id to (saved, target).
fn reached(r: &RewardRow, goals: &HashMap<String, (i64, i64)>, streaks: Option<&StreaksOut>) -> bool {
    match r.trigger_kind.as_str() {
        "goal" => match (r.goal_id.as_ref().and_then(|g| goals.get(g)), r.goal_pct) {
            (Some(&(saved, target)), Some(pct)) => goal_reached(saved, target, pct),
            _ => false,
        },
        _ => {
            let (Some(s), Some(kind), Some(target)) = (streaks, r.streak_kind.as_deref(), r.streak_target) else { return false };
            let run = match kind {
                "logging" => Some(s.logging),
                "saved" => Some(s.saved),
                _ => s.dates,
            };
            run.is_some_and(|run| streak_reached(run.best, target))
        }
    }
}

async fn goal_totals(db: &sqlx::SqlitePool, user_id: &str) -> Result<HashMap<String, (i64, i64)>, AppError> {
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT g.goal_id, g.target_paise,
                COALESCE((SELECT SUM(amount_paise) FROM goal_contributions c WHERE c.goal_id = g.goal_id), 0)
           FROM goals g WHERE g.user_id = ?",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(|(id, target, saved)| (id, (saved, target))).collect())
}

async fn load_rewards(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<RewardRow>, AppError> {
    Ok(sqlx::query_as(
        "SELECT r.reward_id, r.name, r.cost_paise, r.trigger_kind, r.goal_id, r.goal_pct, r.streak_kind, r.streak_target, c.claimed_on
           FROM rewards r LEFT JOIN reward_claims c ON c.reward_id = r.reward_id
          WHERE r.user_id = ? AND r.archived = 0 ORDER BY r.created_at, r.reward_id",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?)
}

/// The rewards screen for a user as of `today`.
pub async fn compute(state: &AppState, user_id: &str, today: Date) -> Result<RewardsOut, AppError> {
    let db = &state.db;
    let rows = load_rewards(db, user_id).await?;
    let goals = goal_totals(db, user_id).await?;
    let needs_streaks = rows.iter().any(|r| r.trigger_kind == "streak" && r.claimed_on.is_none());
    let streaks = if needs_streaks { Some(streaks::compute(state, user_id, today).await?) } else { None };
    let rewards = rows
        .into_iter()
        .map(|r| {
            let st = reward_state(reached(&r, &goals, streaks.as_ref()), r.claimed_on.is_some());
            RewardOut {
                reward_id: r.reward_id,
                name: r.name,
                cost_paise: r.cost_paise,
                trigger_kind: r.trigger_kind,
                goal_id: r.goal_id,
                goal_pct: r.goal_pct,
                streak_kind: r.streak_kind,
                streak_target: r.streak_target,
                state: st,
                claimed_on: r.claimed_on,
            }
        })
        .collect();
    let history: Vec<(String, String, i64, String)> = sqlx::query_as(
        "SELECT c.reward_id, r.name, c.cost_paise, c.claimed_on FROM reward_claims c JOIN rewards r ON r.reward_id = c.reward_id
          WHERE c.user_id = ? ORDER BY c.claimed_on DESC, c.created_at DESC LIMIT 200",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    let cap = cap_of(db, user_id).await?;
    let spent = spent_in(db, user_id, &month_of(today)).await?;
    Ok(RewardsOut {
        rewards,
        history: history.into_iter().map(|(reward_id, name, cost_paise, claimed_on)| HistoryOut { reward_id, name, cost_paise, claimed_on }).collect(),
        monthly_cap_paise: cap,
        spent_this_month_paise: spent,
        cap_left_paise: cap_left(cap, spent),
    })
}

pub async fn list(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<RewardsOut>, AppError> {
    Ok(Json(compute(&state, &user_id, today_local()).await?))
}

#[derive(Deserialize)]
pub struct CreatePayload {
    pub name: String,
    /// What the treat costs; 0 or missing for a free one.
    pub cost_paise: Option<i64>,
    /// goal | streak
    pub trigger_kind: String,
    pub goal_id: Option<String>,
    /// 25, 50, 75 or 100 (done).
    pub goal_pct: Option<i64>,
    /// logging | saved | dates
    pub streak_kind: Option<String>,
    pub streak_target: Option<i64>,
}

#[derive(Serialize)]
pub struct RewardId {
    pub reward_id: String,
    pub status: bool,
}

pub async fn create(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<CreatePayload>) -> Result<Json<RewardId>, AppError> {
    let name = p.name.trim().to_string();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err(bad_request(format!("Give the reward a name of up to {MAX_NAME_CHARS} characters")));
    }
    let cost = validate::non_negative_amount(p.cost_paise.unwrap_or(0)).map_err(|e| bad_request(format!("The cost: {e}")))?;
    let (goal_id, goal_pct, streak_kind, streak_target) = match p.trigger_kind.as_str() {
        "goal" => {
            let goal_id = p.goal_id.clone().ok_or_else(|| bad_request("Pick a goal"))?;
            let pct = p.goal_pct.unwrap_or(100);
            if !GOAL_PCTS.contains(&pct) {
                return Err(bad_request("The share of the goal must be 25, 50, 75 or 100"));
            }
            let owned: Option<String> = sqlx::query_scalar("SELECT goal_id FROM goals WHERE goal_id = ? AND user_id = ?")
                .bind(&goal_id)
                .bind(&user_id)
                .fetch_optional(&state.db)
                .await?;
            if owned.is_none() {
                return Err(not_found("Goal not found"));
            }
            (Some(goal_id), Some(pct), None, None)
        }
        "streak" => {
            let kind = p.streak_kind.clone().filter(|k| matches!(k.as_str(), "logging" | "saved" | "dates")).ok_or_else(|| bad_request("streak_kind must be logging, saved or dates"))?;
            let target = p.streak_target.filter(|t| (1..=MAX_STREAK_TARGET).contains(t)).ok_or_else(|| bad_request(format!("The streak must be between 1 and {MAX_STREAK_TARGET}")))?;
            (None, None, Some(kind), Some(target))
        }
        _ => return Err(bad_request("trigger_kind must be goal or streak")),
    };
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rewards WHERE user_id = ?").bind(&user_id).fetch_one(&state.db).await?;
    if count >= MAX_REWARDS {
        return Err(bad_request("That's a lot of rewards. Remove some you no longer want first"));
    }
    let reward_id = nanoid!();
    sqlx::query(
        "INSERT INTO rewards (reward_id, user_id, name, cost_paise, trigger_kind, goal_id, goal_pct, streak_kind, streak_target)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&reward_id)
    .bind(&user_id)
    .bind(&name)
    .bind(cost)
    .bind(&p.trigger_kind)
    .bind(goal_id)
    .bind(goal_pct)
    .bind(streak_kind)
    .bind(streak_target)
    .execute(&state.db)
    .await?;
    Ok(Json(RewardId { reward_id, status: true }))
}

#[derive(Deserialize)]
pub struct RewardRef {
    pub reward_id: String,
}

/// Removes a reward from the list. Anything already claimed stays in the history.
pub async fn remove(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<RewardRef>) -> Result<Json<Done>, AppError> {
    let res = sqlx::query("UPDATE rewards SET archived = 1 WHERE reward_id = ? AND user_id = ?").bind(&p.reward_id).bind(&user_id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Reward not found"));
    }
    Ok(Json(Done { status: true }))
}

/// Claims an earned reward, once, if its cost fits what the monthly cap still allows.
pub async fn claim(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<RewardRef>) -> Result<Json<Done>, AppError> {
    claim_on(&state, &user_id, &p.reward_id, today_local()).await?;
    Ok(Json(Done { status: true }))
}

pub async fn claim_on(state: &AppState, user_id: &str, reward_id: &str, today: Date) -> Result<(), AppError> {
    let db = &state.db;
    let row: Option<RewardRow> = sqlx::query_as(
        "SELECT r.reward_id, r.name, r.cost_paise, r.trigger_kind, r.goal_id, r.goal_pct, r.streak_kind, r.streak_target, c.claimed_on
           FROM rewards r LEFT JOIN reward_claims c ON c.reward_id = r.reward_id
          WHERE r.reward_id = ? AND r.user_id = ? AND r.archived = 0",
    )
    .bind(reward_id)
    .bind(user_id)
    .fetch_optional(db)
    .await?;
    let Some(r) = row else { return Err(not_found("Reward not found")) };
    if r.claimed_on.is_some() {
        return Err(AppError(StatusCode::CONFLICT, "You already claimed that one".into()));
    }
    let goals = goal_totals(db, user_id).await?;
    let streaks = if r.trigger_kind == "streak" { Some(streaks::compute(state, user_id, today).await?) } else { None };
    if !reached(&r, &goals, streaks.as_ref()) {
        return Err(bad_request("Not earned yet"));
    }
    let day = today.to_string();
    // One statement checks the cap and inserts, so two taps at once can't both squeeze in.
    let res = sqlx::query(
        "INSERT OR IGNORE INTO reward_claims (reward_id, user_id, cost_paise, claimed_on)
         SELECT r.reward_id, r.user_id, r.cost_paise, ?1 FROM rewards r
          WHERE r.reward_id = ?2 AND r.user_id = ?3
            AND ((SELECT monthly_cap_paise FROM reward_settings WHERE user_id = ?3) IS NULL
                 OR r.cost_paise + COALESCE((SELECT SUM(cost_paise) FROM reward_claims WHERE user_id = ?3 AND substr(claimed_on, 1, 7) = ?4), 0)
                    <= (SELECT monthly_cap_paise FROM reward_settings WHERE user_id = ?3))",
    )
    .bind(&day)
    .bind(reward_id)
    .bind(user_id)
    .bind(month_of(today))
    .execute(db)
    .await?;
    if res.rows_affected() == 0 {
        // Either someone else claimed it a moment ago, or the cap says no.
        let taken: Option<String> = sqlx::query_scalar("SELECT reward_id FROM reward_claims WHERE reward_id = ?").bind(reward_id).fetch_optional(db).await?;
        if taken.is_some() {
            return Err(AppError(StatusCode::CONFLICT, "You already claimed that one".into()));
        }
        return Err(bad_request("That would go over this month's reward budget. Wait for next month, or raise the budget"));
    }
    Ok(())
}

/// Takes a claim back (a treat you didn't get after all), which also frees its share of the cap.
pub async fn unclaim(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<RewardRef>) -> Result<Json<Done>, AppError> {
    let res = sqlx::query("DELETE FROM reward_claims WHERE reward_id = ? AND user_id = ?").bind(&p.reward_id).bind(&user_id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(not_found("That reward isn't claimed"));
    }
    Ok(Json(Done { status: true }))
}

#[derive(Deserialize)]
pub struct CapPayload {
    /// null removes the cap.
    pub monthly_cap_paise: Option<i64>,
}

pub async fn set_cap(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<CapPayload>) -> Result<Json<Done>, AppError> {
    if let Some(c) = p.monthly_cap_paise {
        validate::non_negative_amount(c).map_err(|e| bad_request(format!("The budget: {e}")))?;
    }
    sqlx::query("INSERT INTO reward_settings (user_id, monthly_cap_paise) VALUES (?, ?) ON CONFLICT(user_id) DO UPDATE SET monthly_cap_paise = excluded.monthly_cap_paise")
        .bind(&user_id)
        .bind(p.monthly_cap_paise)
        .execute(&state.db)
        .await?;
    Ok(Json(Done { status: true }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use time::Month;

    async fn test_state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('other', 'other', 'x', 'user')",
            "INSERT INTO goals (goal_id, user_id, name, target_paise) VALUES ('g1', 'u', 'Trip', 100000)",
            "INSERT INTO goals (goal_id, user_id, name, target_paise) VALUES ('go', 'other', 'Theirs', 100000)",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32][..]),
        );
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    fn day(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), d).unwrap()
    }

    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("handler failed: {} {}", e.0, e.1),
        }
    }

    fn err<T>(r: Result<T, AppError>) -> (StatusCode, String) {
        match r {
            Ok(_) => panic!("expected an error"),
            Err(e) => (e.0, e.1),
        }
    }

    fn goal_reward(goal: &str, pct: i64, cost: i64) -> CreatePayload {
        CreatePayload { name: "Dinner out".into(), cost_paise: Some(cost), trigger_kind: "goal".into(), goal_id: Some(goal.into()), goal_pct: Some(pct), streak_kind: None, streak_target: None }
    }

    async fn make(state: &AppState, user: &str, p: CreatePayload) -> Result<String, AppError> {
        create(State(state.clone()), Extension((user.into(), String::new())), Json(p)).await.map(|j| j.0.reward_id)
    }

    async fn save(state: &AppState, goal: &str, paise: i64) {
        sqlx::query("INSERT INTO goal_contributions (goal_id, amount_paise, on_day) VALUES (?, ?, '2026-11-01')").bind(goal).bind(paise).execute(&state.db).await.unwrap();
    }

    async fn state_of(state: &AppState, id: &str, today: Date) -> RewardState {
        ok(compute(state, "u", today).await).rewards.into_iter().find(|r| r.reward_id == id).unwrap().state
    }

    #[tokio::test]
    async fn a_goal_reward_unlocks_at_its_share_and_is_claimed_once() {
        let state = test_state().await;
        let today = day(2026, 11, 11);
        let half = ok(make(&state, "u", goal_reward("g1", 50, 20_000)).await);
        assert_eq!(state_of(&state, &half, today).await, RewardState::Locked);
        assert_eq!(err(claim_on(&state, "u", &half, today).await).0, StatusCode::BAD_REQUEST);
        save(&state, "g1", 50_000).await;
        assert_eq!(state_of(&state, &half, today).await, RewardState::Earned);
        ok(claim_on(&state, "u", &half, today).await);
        assert_eq!(state_of(&state, &half, today).await, RewardState::Claimed);
        assert_eq!(err(claim_on(&state, "u", &half, today).await).0, StatusCode::CONFLICT);
        let out = ok(compute(&state, "u", today).await);
        assert_eq!((out.spent_this_month_paise, out.history.len()), (20_000, 1));
    }

    #[tokio::test]
    async fn taking_money_back_out_locks_an_unclaimed_reward_again() {
        let state = test_state().await;
        let today = day(2026, 11, 11);
        let id = ok(make(&state, "u", goal_reward("g1", 100, 0)).await);
        save(&state, "g1", 100_000).await;
        assert_eq!(state_of(&state, &id, today).await, RewardState::Earned);
        save(&state, "g1", -1).await;
        assert_eq!(state_of(&state, &id, today).await, RewardState::Locked);
    }

    #[tokio::test]
    async fn the_monthly_cap_refuses_a_claim_that_would_pass_it_and_resets_next_month() {
        let state = test_state().await;
        save(&state, "g1", 100_000).await;
        let _ = ok(set_cap(State(state.clone()), Extension(("u".into(), String::new())), Json(CapPayload { monthly_cap_paise: Some(50_000) })).await);
        let a = ok(make(&state, "u", goal_reward("g1", 25, 30_000)).await);
        let b = ok(make(&state, "u", goal_reward("g1", 50, 30_000)).await);
        let nov = day(2026, 11, 11);
        ok(claim_on(&state, "u", &a, nov).await);
        let (code, msg) = err(claim_on(&state, "u", &b, nov).await);
        assert_eq!(code, StatusCode::BAD_REQUEST);
        assert!(msg.contains("budget"), "{msg}");
        let out = ok(compute(&state, "u", nov).await);
        assert_eq!(out.cap_left_paise, Some(20_000));
        // A new month has a fresh budget; undoing a claim frees its share too.
        ok(claim_on(&state, "u", &b, day(2026, 12, 1)).await);
        let _ = ok(unclaim(State(state.clone()), Extension(("u".into(), String::new())), Json(RewardRef { reward_id: a.clone() })).await);
        assert_eq!(ok(compute(&state, "u", nov).await).spent_this_month_paise, 0);
    }

    #[tokio::test]
    async fn a_cap_of_zero_still_allows_free_treats() {
        let state = test_state().await;
        save(&state, "g1", 100_000).await;
        let _ = ok(set_cap(State(state.clone()), Extension(("u".into(), String::new())), Json(CapPayload { monthly_cap_paise: Some(0) })).await);
        let free = ok(make(&state, "u", goal_reward("g1", 100, 0)).await);
        let paid = ok(make(&state, "u", goal_reward("g1", 100, 1)).await);
        ok(claim_on(&state, "u", &free, day(2026, 11, 11)).await);
        assert_eq!(err(claim_on(&state, "u", &paid, day(2026, 11, 11)).await).0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn a_streak_reward_unlocks_from_the_best_run() {
        let state = test_state().await;
        // Days logged 1-3 Nov: a run of three that is still the best later.
        for (i, d) in ["2026-11-01", "2026-11-02", "2026-11-03"].iter().enumerate() {
            sqlx::query("INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key) VALUES (?, 'u', ?, 'debit', 100, 'x', 'food', 'x', 'manual', ?)")
                .bind(format!("t{i}")).bind(d).bind(format!("k{i}")).execute(&state.db).await.unwrap();
        }
        let three = ok(make(&state, "u", CreatePayload { name: "Cake".into(), cost_paise: None, trigger_kind: "streak".into(), goal_id: None, goal_pct: None, streak_kind: Some("logging".into()), streak_target: Some(3) }).await);
        let seven = ok(make(&state, "u", CreatePayload { name: "Trip".into(), cost_paise: None, trigger_kind: "streak".into(), goal_id: None, goal_pct: None, streak_kind: Some("logging".into()), streak_target: Some(7) }).await);
        let later = day(2026, 11, 20);
        assert_eq!(state_of(&state, &three, later).await, RewardState::Earned);
        assert_eq!(state_of(&state, &seven, later).await, RewardState::Locked);
        ok(claim_on(&state, "u", &three, later).await);
    }

    #[tokio::test]
    async fn bad_input_is_refused() {
        let state = test_state().await;
        for cost in [-1, i64::MAX, i64::MIN, validate::MAX_AMOUNT_PAISE + 1] {
            assert_eq!(err(make(&state, "u", goal_reward("g1", 100, cost)).await).0, StatusCode::BAD_REQUEST, "cost {cost}");
        }
        assert!(make(&state, "u", goal_reward("g1", 100, validate::MAX_AMOUNT_PAISE)).await.is_ok());
        assert_eq!(err(make(&state, "u", goal_reward("g1", 33, 0)).await).0, StatusCode::BAD_REQUEST);
        let mut blank = goal_reward("g1", 100, 0);
        blank.name = "   ".into();
        assert_eq!(err(make(&state, "u", blank).await).0, StatusCode::BAD_REQUEST);
        let mut long = goal_reward("g1", 100, 0);
        long.name = "x".repeat(MAX_NAME_CHARS + 1);
        assert_eq!(err(make(&state, "u", long).await).0, StatusCode::BAD_REQUEST);
        for t in [0, -3, MAX_STREAK_TARGET + 1, i64::MAX] {
            let p = CreatePayload { name: "x".into(), cost_paise: None, trigger_kind: "streak".into(), goal_id: None, goal_pct: None, streak_kind: Some("saved".into()), streak_target: Some(t) };
            assert_eq!(err(make(&state, "u", p).await).0, StatusCode::BAD_REQUEST, "target {t}");
        }
        for cap in [-1, i64::MAX, i64::MIN] {
            let r = set_cap(State(state.clone()), Extension(("u".into(), String::new())), Json(CapPayload { monthly_cap_paise: Some(cap) })).await;
            assert_eq!(err(r).0, StatusCode::BAD_REQUEST, "cap {cap}");
        }
    }

    #[tokio::test]
    async fn rewards_are_scoped_to_their_owner() {
        let state = test_state().await;
        let today = day(2026, 11, 11);
        // Another user's goal can't carry my reward.
        assert_eq!(err(make(&state, "u", goal_reward("go", 100, 0)).await).0, StatusCode::NOT_FOUND);
        let mine = ok(make(&state, "u", goal_reward("g1", 100, 0)).await);
        save(&state, "g1", 100_000).await;
        let theirs = Extension(("other".to_string(), String::new()));
        assert!(ok(compute(&state, "other", today).await).rewards.is_empty());
        assert_eq!(err(claim_on(&state, "other", &mine, today).await).0, StatusCode::NOT_FOUND);
        assert_eq!(err(remove(State(state.clone()), theirs, Json(RewardRef { reward_id: mine.clone() })).await).0, StatusCode::NOT_FOUND);
        assert_eq!(state_of(&state, &mine, today).await, RewardState::Earned);
        // Their cap doesn't touch mine either.
        let _ = ok(set_cap(State(state.clone()), Extension(("other".into(), String::new())), Json(CapPayload { monthly_cap_paise: Some(0) })).await);
        assert_eq!(ok(compute(&state, "u", today).await).monthly_cap_paise, None);
    }

    #[tokio::test]
    async fn a_removed_reward_leaves_the_list_but_a_claim_stays_in_history() {
        let state = test_state().await;
        let today = day(2026, 11, 11);
        save(&state, "g1", 100_000).await;
        let id = ok(make(&state, "u", goal_reward("g1", 100, 500)).await);
        ok(claim_on(&state, "u", &id, today).await);
        let _ = ok(remove(State(state.clone()), Extension(("u".into(), String::new())), Json(RewardRef { reward_id: id.clone() })).await);
        let out = ok(compute(&state, "u", today).await);
        assert!(out.rewards.is_empty());
        assert_eq!((out.history.len(), out.spent_this_month_paise), (1, 500));
        assert_eq!(err(claim_on(&state, "u", &id, today).await).0, StatusCode::NOT_FOUND);
    }
}
