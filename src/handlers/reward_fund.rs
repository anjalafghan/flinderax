//! The reward fund (`GET /api/rewards/fund`): what each month paid you for looking after money,
//! health, relationships and the future, the wallet those payouts build up, and how this month is
//! going so far. The rules are pure in `planner::reward_fund`; this file counts the signals.
//!
//! A month is settled once, the first time the fund is looked at after it ends, and the row in
//! `reward_months` never changes after, so data added later can't rewrite history. Every signal
//! is a count of something done in the month (activity is by the day you did it, never the day
//! of the data), so importing old history can't be farmed.

use std::collections::HashMap;

use axum::{extract::State, Extension, Json};
use serde::Serialize;
use time::Date;

use crate::{
    alerts::is_spending,
    handlers::{
        common::{today_local, AppError},
        couple::couple_of,
        dashboard::load_input,
    },
    models::AppState,
    planner::{
        self, month_index,
        reward_fund::{score, wallet, MonthScore, MonthSignals, AREAS, HISTORY_MONTHS, MAX_PAYOUT_PAISE},
    },
};

type User = Extension<(String, String)>;

/// How many past months a first look (or a long gap) settles at most.
const MAX_CATCH_UP: i32 = 12;

pub fn month_key(idx: i32) -> String {
    crate::planner::dates::month_key(idx.div_euclid(12), (idx.rem_euclid(12) + 1) as u8)
}

#[derive(Serialize)]
pub struct SettledMonth {
    pub month: String,
    pub score_bps: i64,
    pub earned_paise: i64,
    pub saved_paise: i64,
    pub payout_paise: i64,
    /// The area scores as they were when the month settled.
    pub detail: serde_json::Value,
}

#[derive(Serialize)]
pub struct ThisMonth {
    pub month: String,
    pub days_left: i64,
    /// The score so far; it settles when the month ends.
    pub score: MonthScore,
}

#[derive(Serialize)]
pub struct FundOut {
    /// What the wallet holds to spend on treats.
    pub wallet_paise: i64,
    pub paid_in_paise: i64,
    pub spent_paise: i64,
    pub max_payout_paise: i64,
    pub fund_since: String,
    pub this_month: ThisMonth,
    /// Settled months, newest first.
    pub months: Vec<SettledMonth>,
}

/// Counts every signal for the months `from..=to` (month indexes), keyed by index.
async fn signals(db: &sqlx::SqlitePool, user_id: &str, from: i32, to: i32, today: Date) -> Result<HashMap<i32, MonthSignals>, AppError> {
    let start = format!("{}-01", month_key(from));
    let end = format!("{}-01", month_key(to + 1));
    let mut out: HashMap<i32, MonthSignals> = (from..=to).map(|m| (m, MonthSignals::default())).collect();
    let idx = |m: &str| planner::parse_year_month(m);
    macro_rules! each {
        ($rows:expr, |$s:ident, $r:ident| $body:block) => {
            for $r in $rows {
                if let Some($s) = idx(&$r.0).and_then(|m| out.get_mut(&m)) $body
            }
        };
    }

    // money
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT substr(day, 1, 7), COALESCE(SUM(points), 0), COUNT(DISTINCT day) FROM activity_log
          WHERE user_id = ? AND day >= ? AND day < ? GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { s.activity_points = r.1; s.active_days = r.2; });

    let rows: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT substr(occurred_on, 1, 7), direction, category, COALESCE(SUM(amount_paise), 0) FROM transactions
          WHERE user_id = ? AND occurred_on >= ? AND occurred_on < ? GROUP BY 1, 2, 3",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| {
        s.has_transactions = true;
        if r.1 == "debit" && is_spending(&r.2) {
            s.spent_paise = s.spent_paise.saturating_add(r.3);
        }
    });
    // The allowance is today's plan: the plan keeps no history, so every month uses it.
    let allowance = planner::compute(&load_input(db, user_id, today).await?).budget.spending_paise;
    for s in out.values_mut() {
        s.allowance_paise = allowance;
    }

    // health
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT substr(week_start, 1, 7), COUNT(*) FROM diet_weeks WHERE user_id = ? AND week_start >= ? AND week_start < ? GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { s.diet_weeks = r.1; });
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT substr(w.week_start, 1, 7), COALESCE(SUM(MIN(w.days_hit, h.target_days)), 0), COALESCE(SUM(h.target_days), 0)
           FROM diet_week_habits w JOIN diet_habits h ON h.habit_id = w.habit_id
          WHERE w.user_id = ? AND w.week_start >= ? AND w.week_start < ? GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { if r.2 > 0 { s.habit_hit_bps = Some(r.1 * 10_000 / r.2); } });
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT substr(measured_on, 1, 7), COALESCE(SUM(kind IN ('weight', 'waist')), 0), COALESCE(SUM(kind = 'checkup'), 0)
           FROM health_marks WHERE user_id = ? AND measured_on >= ? AND measured_on < ? GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { s.health_marks = r.1; s.checkups = r.2; });

    // relationships
    if let Some((couple, who)) = couple_of(db, user_id).await? {
        // A kept date counts in the month its day falls in, once that day has passed.
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT strftime('%Y-%m', COALESCE(planned_at, picked_at), 'unixepoch'), COUNT(*) FROM joy_history
              WHERE couple_id = ? AND outcome = 'did_it' AND COALESCE(planned_at, picked_at) <= unixepoch()
                AND COALESCE(planned_at, picked_at) >= unixepoch(?) AND COALESCE(planned_at, picked_at) < unixepoch(?) GROUP BY 1",
        )
        .bind(&couple).bind(&start).bind(&end).fetch_all(db).await?;
        each!(rows, |s, r| { s.dates_kept = r.1; });
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT strftime('%Y-%m', at, 'unixepoch'), COUNT(*) FROM joy_draws
              WHERE couple_id = ? AND COALESCE(who, ?) = ? AND at >= unixepoch(?) AND at < unixepoch(?) GROUP BY 1",
        )
        .bind(&couple).bind(who as i64).bind(who as i64).bind(&start).bind(&end).fetch_all(db).await?;
        each!(rows, |s, r| { s.jar_draws = r.1; });
    }
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT substr(on_day, 1, 7), COUNT(*) FROM touches WHERE user_id = ? AND on_day >= ? AND on_day < ? GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { s.touches = r.1; });
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT substr(done_on, 1, 7), COUNT(*) FROM followups WHERE user_id = ? AND done_on >= ? AND done_on < ? GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { s.followups_done = r.1; });
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT substr(on_day, 1, 7), COUNT(*) FROM gifts WHERE user_id = ? AND status = 'given' AND on_day >= ? AND on_day < ? GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { s.gifts_given = r.1; });

    // future
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT substr(c.on_day, 1, 7), COUNT(DISTINCT CASE WHEN c.amount_paise > 0 THEN c.on_day END), COALESCE(SUM(c.amount_paise), 0)
           FROM goal_contributions c JOIN goals g ON g.goal_id = c.goal_id
          WHERE g.user_id = ? AND c.on_day >= ? AND c.on_day < ? GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { s.goal_days = r.1; s.goal_net_paise = r.2; });
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT substr(done_on, 1, 7), COUNT(*) FROM checkins WHERE user_id = ? AND done_on >= ? AND done_on < ? GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { s.checkins = r.1; });
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT substr(t, 1, 7), COUNT(*) FROM (
            SELECT created_at AS t FROM goals WHERE user_id = ?1
            UNION ALL SELECT created_at FROM life_choices WHERE user_id = ?1
            UNION ALL SELECT created_at FROM planned_outflows WHERE user_id = ?1
            UNION ALL SELECT updated_at FROM whatif_scenarios WHERE user_id = ?1
         ) WHERE t >= ?2 AND t < ?3 GROUP BY 1",
    )
    .bind(user_id).bind(&start).bind(&end).fetch_all(db).await?;
    each!(rows, |s, r| { s.plans_touched = r.1; });

    Ok(out)
}

/// Scores month `m`, learning from up to six months before it.
fn score_month(sig: &HashMap<i32, MonthSignals>, m: i32) -> MonthScore {
    let empty = MonthSignals::default();
    let history: [Vec<i64>; 4] = AREAS.map(|a| {
        (m - HISTORY_MONTHS as i32..m).map(|p| planner::reward_fund::raw(a, sig.get(&p).unwrap_or(&empty))).collect()
    });
    score(sig.get(&m).unwrap_or(&empty), &history)
}

/// The first month the fund pays for. The first look starts it at last month, so a new fund
/// already has one month to show.
async fn fund_since(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<i32, AppError> {
    let have: Option<Option<String>> = sqlx::query_scalar("SELECT fund_since FROM reward_settings WHERE user_id = ?").bind(user_id).fetch_optional(db).await?;
    if let Some(m) = have.flatten().as_deref().and_then(planner::parse_year_month) {
        return Ok(m);
    }
    let start = month_index(today) - 1;
    sqlx::query(
        "INSERT INTO reward_settings (user_id, fund_since) VALUES (?, ?)
         ON CONFLICT(user_id) DO UPDATE SET fund_since = COALESCE(reward_settings.fund_since, excluded.fund_since)",
    )
    .bind(user_id)
    .bind(month_key(start))
    .execute(db)
    .await?;
    let set: Option<String> = sqlx::query_scalar("SELECT fund_since FROM reward_settings WHERE user_id = ?").bind(user_id).fetch_one(db).await?;
    Ok(set.as_deref().and_then(planner::parse_year_month).unwrap_or(start))
}

/// Settles every finished month not yet settled and returns the fund as of `today`.
pub async fn compute(state: &AppState, user_id: &str, today: Date) -> Result<FundOut, AppError> {
    let db = &state.db;
    let now = month_index(today);
    let since = fund_since(db, user_id, today).await?.max(now - MAX_CATCH_UP);
    let settled: Vec<(String,)> = sqlx::query_as("SELECT month FROM reward_months WHERE user_id = ? AND month >= ?")
        .bind(user_id)
        .bind(month_key(since))
        .fetch_all(db)
        .await?;
    let settled: Vec<i32> = settled.iter().filter_map(|(m,)| planner::parse_year_month(m)).collect();
    let open: Vec<i32> = (since..now).filter(|m| !settled.contains(m)).collect();
    let first = open.first().copied().unwrap_or(now);
    let sig = signals(db, user_id, first - HISTORY_MONTHS as i32, now, today).await?;

    for &m in &open {
        let s = score_month(&sig, m);
        let detail = serde_json::json!({ "areas": s.areas, "held_back_paise": s.held_back_paise, "focus": s.focus });
        sqlx::query(
            "INSERT OR IGNORE INTO reward_months (user_id, month, score_bps, earned_paise, saved_paise, payout_paise, detail_json, settled_on)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(month_key(m))
        .bind(s.score_bps)
        .bind(s.earned_paise)
        .bind(s.saved_paise)
        .bind(s.payout_paise)
        .bind(detail.to_string())
        .bind(today.to_string())
        .execute(db)
        .await?;
    }

    let (paid_in, spent) = totals(db, user_id).await?;
    let rows: Vec<(String, i64, i64, i64, i64, String)> = sqlx::query_as(
        "SELECT month, score_bps, earned_paise, saved_paise, payout_paise, detail_json FROM reward_months
          WHERE user_id = ? ORDER BY month DESC LIMIT 24",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    let last_day = planner::cycle::add_months(today.year(), today.month(), 1);
    let next_first = Date::from_calendar_date(last_day.0, last_day.1, 1).unwrap_or(today);
    Ok(FundOut {
        wallet_paise: wallet(paid_in, spent),
        paid_in_paise: paid_in,
        spent_paise: spent,
        max_payout_paise: MAX_PAYOUT_PAISE,
        fund_since: month_key(since),
        this_month: ThisMonth { month: month_key(now), days_left: (next_first - today).whole_days().max(0), score: score_month(&sig, now) },
        months: rows
            .into_iter()
            .map(|(month, score_bps, earned_paise, saved_paise, payout_paise, detail)| SettledMonth {
                month,
                score_bps,
                earned_paise,
                saved_paise,
                payout_paise,
                detail: serde_json::from_str(&detail).unwrap_or(serde_json::Value::Null),
            })
            .collect(),
    })
}

/// (everything paid in, everything treats took out of the wallet).
pub async fn totals(db: &sqlx::SqlitePool, user_id: &str) -> Result<(i64, i64), AppError> {
    let paid_in: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(payout_paise), 0) FROM reward_months WHERE user_id = ?").bind(user_id).fetch_one(db).await?;
    let spent: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(cost_paise), 0) FROM reward_claims WHERE user_id = ? AND from_wallet = 1")
        .bind(user_id)
        .fetch_one(db)
        .await?;
    Ok((paid_in, spent))
}

pub async fn fund(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<FundOut>, AppError> {
    Ok(Json(compute(&state, &user_id, today_local()).await?))
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
            "INSERT INTO user_settings (user_id, spending_paise) VALUES ('u', 2000000)",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32][..]),
        );
        AppState { db, paseto_key: std::sync::Arc::new(key) }
    }

    fn day(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), d).unwrap()
    }

    async fn exec(state: &AppState, q: &str) {
        sqlx::query(q).execute(&state.db).await.unwrap();
    }

    async fn ok(state: &AppState, today: Date) -> FundOut {
        match compute(state, "u", today).await {
            Ok(v) => v,
            Err(e) => panic!("{} {}", e.0, e.1),
        }
    }

    #[test]
    fn month_keys_round_trip() {
        for k in ["2026-01", "2026-12", "1999-07"] {
            assert_eq!(month_key(planner::parse_year_month(k).unwrap()), k);
        }
    }

    #[tokio::test]
    async fn a_first_look_starts_last_month_and_a_quiet_month_pays_nothing() {
        let state = test_state().await;
        let out = ok(&state, day(2026, 11, 21)).await;
        assert_eq!(out.fund_since, "2026-10");
        assert_eq!(out.months.len(), 1);
        assert_eq!((out.months[0].month.as_str(), out.months[0].payout_paise), ("2026-10", 0));
        assert_eq!(out.wallet_paise, 0);
        assert_eq!(out.this_month.month, "2026-11");
        assert_eq!(out.this_month.days_left, 10);
    }

    #[tokio::test]
    async fn a_busy_month_under_budget_pays_and_is_settled_once() {
        let state = test_state().await;
        // October: logged most days, stayed under the ₹20,000 allowance, a diet week, a call, a goal top-up.
        for d in 1..=25 {
            exec(&state, &format!("INSERT INTO activity_log (user_id, kind, day, ref_id, points) VALUES ('u', 'txn', '2026-10-{d:02}', 'r{d}', 3)")).await;
        }
        exec(&state, "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key) VALUES ('t1', 'u', '2026-10-05', 'debit', 1500000, 'x', 'food', 'x', 'manual', 'k1')").await;
        exec(&state, "INSERT INTO diet_weeks (user_id, week_start) VALUES ('u', '2026-10-05'), ('u', '2026-10-12'), ('u', '2026-10-19')").await;
        exec(&state, "INSERT INTO health_marks (mark_id, user_id, kind, measured_on, value_x10) VALUES ('h1', 'u', 'weight', '2026-10-03', 700)").await;
        exec(&state, "INSERT INTO people (person_id, user_id, name, circle) VALUES ('p', 'u', 'Ma', 'family')").await;
        for (i, d) in ["03", "10", "17", "24"].iter().enumerate() {
            exec(&state, &format!("INSERT INTO touches (touch_id, user_id, person_id, on_day, kind) VALUES ('t{i}', 'u', 'p', '2026-10-{d}', 'call')")).await;
        }
        exec(&state, "INSERT INTO goals (goal_id, user_id, name, target_paise) VALUES ('g', 'u', 'Trip', 100000)").await;
        exec(&state, "INSERT INTO goal_contributions (goal_id, amount_paise, on_day) VALUES ('g', 50000, '2026-10-02'), ('g', 50000, '2026-10-20')").await;

        let out = ok(&state, day(2026, 11, 21)).await;
        let oct = &out.months[0];
        assert_eq!(oct.month, "2026-10");
        assert!(oct.score_bps > 5_000, "{}", oct.score_bps);
        assert!(oct.payout_paise > 0 && oct.payout_paise <= MAX_PAYOUT_PAISE);
        assert!(oct.payout_paise <= oct.saved_paise);
        assert_eq!(oct.saved_paise, 500_000);
        assert_eq!(out.wallet_paise, oct.payout_paise);

        // More October data later can't rewrite a settled month.
        exec(&state, "INSERT INTO checkins (checkin_id, user_id, done_on, period_start, period_end, summary_json) VALUES ('c', 'u', '2026-10-30', '2026-07-01', '2026-09-30', '{}')").await;
        let again = ok(&state, day(2026, 11, 22)).await;
        assert_eq!(again.months[0].score_bps, oct.score_bps);
        assert_eq!(again.months.len(), 1);

        // A month later November settles too and the wallet keeps adding up.
        let dec = ok(&state, day(2026, 12, 2)).await;
        assert_eq!(dec.months.len(), 2);
        assert_eq!(dec.paid_in_paise, dec.months.iter().map(|m| m.payout_paise).sum::<i64>());
    }

    #[tokio::test]
    async fn over_budget_pays_nothing_however_busy() {
        let state = test_state().await;
        for d in 1..=28 {
            exec(&state, &format!("INSERT INTO activity_log (user_id, kind, day, ref_id, points) VALUES ('u', 'statement', '2026-10-{d:02}', 'r{d}', 15)")).await;
        }
        exec(&state, "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key) VALUES ('t1', 'u', '2026-10-05', 'debit', 2500000, 'x', 'food', 'x', 'manual', 'k1')").await;
        let out = ok(&state, day(2026, 11, 21)).await;
        assert!(out.months[0].earned_paise > 0);
        assert_eq!(out.months[0].payout_paise, 0);
    }

    #[tokio::test]
    async fn the_fund_is_per_person() {
        let state = test_state().await;
        exec(&state, "INSERT INTO activity_log (user_id, kind, day, ref_id, points) VALUES ('other', 'statement', '2026-10-03', 'r', 15)").await;
        let out = ok(&state, day(2026, 11, 21)).await;
        assert_eq!(out.months[0].score_bps, 0);
    }

    #[tokio::test]
    async fn a_long_gap_settles_at_most_a_year() {
        let state = test_state().await;
        exec(&state, "INSERT INTO reward_settings (user_id, fund_since) VALUES ('u', '2020-01')").await;
        let out = ok(&state, day(2026, 11, 21)).await;
        assert_eq!(out.months.len(), 12);
        assert_eq!(out.fund_since, "2025-11");
    }
}
