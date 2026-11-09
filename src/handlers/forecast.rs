//! Forecast API. `GET /api/plan/forecast` loads the saved profile, the statement analytics and the
//! resolved expenses, works out the basis (the statements once there are three full months, else
//! the plan's own numbers), runs the pure engine in `planner::forecast`, and adds the cut-spend
//! lever, which needs the category spend that only the handler has.

use axum::{extract::State, Extension, Json};
use time::{macros::format_description, Date, Month};

use crate::{
    handlers::{
        analytics,
        common::{today_local, AppError},
        life,
    },
    models::AppState,
    planner::{
        analytics::AnalyticsOut,
        cycle::add_months,
        forecast::{self, Basis, Forecast, ForecastInput, GoalIn, Lever, LeverKind},
        life::{inr, parse_date, plan, LifePlan, LifeProfile},
    },
};

/// Most full months the averages use.
const MAX_MONTHS_USED: usize = 12;
/// Fewest full months before the statements are trusted over the plan's numbers.
const MIN_MONTHS_USED: usize = 3;
/// How far back to look for full months, in calendar months.
const LOOKBACK_MONTHS: usize = 36;
/// Without statements, the spread of the surplus is this share of it.
const PROFILE_SD_PCT: i64 = 15;
/// Statements and the plan may differ by this much (in percent) before the note appears.
const NOTE_TOLERANCE_PCT: i64 = 10;
/// The analytics category window is 90 days: three months of spend.
const CATEGORY_WINDOW_MONTHS: i64 = 3;
/// A cut is realistic when the save-more amount is at most this share of the top category.
const CUT_SHARE_PCT: i64 = 25;
/// A goal with no date (the emergency fund) is priced over the plan's 12-month horizon.
const UNDATED_HORIZON_MONTHS: i64 = 12;
const MAX_LEVERS: usize = 3;

pub async fn get(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<Forecast>, AppError> {
    Ok(Json(compute(&state, &user_id, today_local()).await?))
}

/// The forecast for a user as of `today`.
pub async fn compute(state: &AppState, user_id: &str, today: Date) -> Result<Forecast, AppError> {
    let db = &state.db;
    let saved: Option<String> = sqlx::query_scalar("SELECT profile_json FROM life_profiles WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await?;
    let profile: LifeProfile = saved.as_deref().and_then(|j| serde_json::from_str(j).ok()).unwrap_or_default();
    let expenses = life::resolve_expenses(db, user_id, today, &profile).await?;
    let analytics = analytics::compute(db, user_id, today).await?;
    let basis = basis_for(&analytics, &profile, expenses, today);
    // resolve_expenses already prefers the statements' spend over the budget estimate.
    let life_plan = plan(&profile, expenses, today);
    let input = ForecastInput {
        today,
        surplus_paise: basis.surplus_paise,
        surplus_sd_paise: basis.surplus_sd_paise,
        goals: goals_in(&life_plan, today),
        paths: forecast::DEFAULT_PATHS,
    };
    let mut out = forecast::run(&input);
    let top = top_category_monthly(&analytics);
    for g in &mut out.goals {
        add_cut_spend(&mut g.levers, top.as_ref());
    }
    Ok(Forecast {
        basis,
        goals: out.goals,
        path: out.path,
        summary: out.summary,
        saved_profile: saved.is_some(),
    })
}

/// The plan's goals as engine inputs. The horizon is months to the goal's date.
fn goals_in(life_plan: &LifePlan, today: Date) -> Vec<GoalIn> {
    life_plan
        .goals
        .iter()
        .map(|g| {
            let when = g.when.as_deref().and_then(parse_date);
            let months = match when {
                Some(w) => months_between(today, w).max(1),
                None => UNDATED_HORIZON_MONTHS,
            };
            GoalIn {
                key: g.key,
                label: g.label,
                when,
                months,
                target_paise: g.future_cost_paise,
                saved_paise: g.saved_paise,
                needed_monthly_paise: g.monthly_sip_paise,
                mix: g.mix,
            }
        })
        .collect()
}

fn months_between(from: Date, to: Date) -> i64 {
    (to.year() as i64 - from.year() as i64) * 12 + (to.month() as i64 - from.month() as i64)
}

/// The basis: the statements' averages when there are enough full months, else the plan's numbers.
fn basis_for(analytics: &AnalyticsOut, profile: &LifeProfile, expenses: i64, today: Date) -> Basis {
    let entered = profile.my_take_home_paise.max(0).saturating_add(profile.partner_take_home_paise.max(0));
    if let Some(f) = statement_figures(analytics, today).filter(|f| f.months >= MIN_MONTHS_USED) {
        let note = (entered > 0 && (f.income - entered).abs() * 100 > entered * NOTE_TOLERANCE_PCT).then(|| {
            format!(
                "Your statements show {} a month coming in; your plan says {}.",
                inr(f.income),
                inr(entered)
            )
        });
        return Basis {
            source: "statements",
            months_used: f.months as i64,
            income_paise: f.income,
            spend_paise: f.spend,
            invested_paise: f.invested,
            surplus_paise: f.income - f.spend,
            surplus_sd_paise: f.surplus_sd,
            entered_income_paise: entered,
            note,
        };
    }
    let surplus = entered.saturating_sub(expenses);
    Basis {
        source: "profile",
        months_used: 0,
        income_paise: entered,
        spend_paise: expenses,
        invested_paise: 0,
        surplus_paise: surplus,
        surplus_sd_paise: surplus.max(0) * PROFILE_SD_PCT / 100,
        entered_income_paise: entered,
        note: None,
    }
}

/// One month's figures from the analytics, in paise.
#[derive(Clone, Copy, Debug, PartialEq)]
struct MonthFigures {
    income: i64,
    spend: i64,
    invested: i64,
}

/// Averages over full months, with the sample standard deviation of the monthly surplus.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Figures {
    months: usize,
    income: i64,
    spend: i64,
    invested: i64,
    surplus_sd: i64,
}

fn figures(rows: &[MonthFigures]) -> Figures {
    let n = rows.len();
    if n == 0 {
        return Figures { months: 0, income: 0, spend: 0, invested: 0, surplus_sd: 0 };
    }
    let count = n as i64;
    let avg = |f: fn(&MonthFigures) -> i64| rows.iter().map(f).sum::<i64>() / count;
    let surplus: Vec<f64> = rows.iter().map(|r| (r.income - r.spend) as f64).collect();
    let surplus_sd = if n < 2 {
        0
    } else {
        let mean = surplus.iter().sum::<f64>() / n as f64;
        let var = surplus.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
        var.sqrt().round() as i64
    };
    Figures {
        months: n,
        income: avg(|r| r.income),
        spend: avg(|r| r.spend),
        invested: avg(|r| r.invested),
        surplus_sd,
    }
}

/// The statements' figures over the latest full months, or `None` without coverage.
/// Average monthly spend from the statements, when there are enough full months to trust it.
pub(crate) fn statement_spend(analytics: &AnalyticsOut, today: Date) -> Option<i64> {
    statement_figures(analytics, today).filter(|f| f.months >= MIN_MONTHS_USED && f.spend > 0).map(|f| f.spend)
}

fn statement_figures(analytics: &AnalyticsOut, today: Date) -> Option<Figures> {
    let cov = analytics.coverage.as_ref()?;
    let fmt = format_description!("[year]-[month]-[day]");
    let first = Date::parse(&cov.first_date, fmt).ok()?;
    let last = Date::parse(&cov.last_date, fmt).ok()?;
    let rows: Vec<MonthFigures> = full_months(first, last, today)
        .into_iter()
        .map(|(y, m)| {
            let key = format!("{y:04}-{:02}", m as u8);
            analytics
                .months
                .iter()
                .find(|r| r.month == key)
                .map(|r| MonthFigures { income: r.income_paise, spend: r.spend_paise, invested: r.invested_paise })
                .unwrap_or(MonthFigures { income: 0, spend: 0, invested: 0 })
        })
        .collect();
    Some(figures(&rows))
}

/// Full months inside coverage, before `today`'s month, oldest first, at most `MAX_MONTHS_USED`.
/// A month is full when coverage starts on or before its 1st and reaches at least its 28th.
fn full_months(first: Date, last: Date, today: Date) -> Vec<(i32, Month)> {
    let mut out = Vec::new();
    let (mut y, mut m) = (today.year(), today.month());
    for _ in 0..LOOKBACK_MONTHS {
        (y, m) = add_months(y, m, -1);
        let Ok(start) = Date::from_calendar_date(y, m, 1) else { break };
        if start < first {
            break;
        }
        let Ok(end) = Date::from_calendar_date(y, m, 28) else { break };
        if last >= end {
            out.push((y, m));
            if out.len() == MAX_MONTHS_USED {
                break;
            }
        }
    }
    out.reverse();
    out
}

/// The top spending category's monthly spend, as (name, paise a month).
fn top_category_monthly(analytics: &AnalyticsOut) -> Option<(String, i64)> {
    // Uncategorised spend isn't a habit you can cut; name the biggest real category.
    let top = analytics.categories.iter().find(|c| c.category != "uncategorized")?;
    let monthly = top.paise / CATEGORY_WINDOW_MONTHS;
    (monthly > 0).then(|| (top.category.clone(), monthly))
}

/// Adds "cut spend" right after the save-more lever, when that amount is a small enough share of
/// the top category. Keeps at most `MAX_LEVERS`.
fn add_cut_spend(levers: &mut Vec<Lever>, top: Option<&(String, i64)>) {
    let Some(pos) = levers.iter().position(|l| l.kind == LeverKind::SaveMore) else { return };
    let (Some(amount), Some((category, monthly))) = (levers[pos].monthly_paise, top) else { return };
    if amount * 100 > monthly * CUT_SHARE_PCT {
        return;
    }
    let pct = amount * 100 / monthly;
    levers.insert(
        pos + 1,
        Lever {
            kind: LeverKind::CutSpend,
            text: format!("That's about {pct}% of your monthly {category} spend."),
            monthly_paise: Some(amount),
            months: None,
            target_paise: None,
        },
    );
    levers.truncate(MAX_LEVERS);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::life as life_handler;
    use sqlx::sqlite::SqlitePoolOptions;
    use time::macros::date;

    async fn state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')")
            .execute(&db)
            .await
            .unwrap();
        let key = rusty_paseto::prelude::PasetoSymmetricKey::from(rusty_paseto::prelude::Key::from(&[7u8; 32]));
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    fn user() -> Extension<(String, String)> {
        Extension(("u".to_string(), "user".to_string()))
    }

    fn today() -> Date {
        date!(2026 - 11 - 09)
    }

    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("unexpected error {}: {}", e.0, e.1),
        }
    }

    fn assert_well_formed(f: &Forecast) {
        let years: Vec<i64> = f.path.iter().map(|b| b.year).collect();
        assert!(years.windows(2).all(|w| w[0] < w[1]), "years ascend: {years:?}");
        for g in &f.goals {
            assert!(g.p10_paise <= g.p50_paise && g.p50_paise <= g.p90_paise, "{}", g.key);
            assert!((0..=10_000).contains(&g.likelihood_bps), "{}", g.key);
            assert!(g.levers.len() <= MAX_LEVERS, "{}", g.key);
        }
        assert!(!f.summary.is_empty());
    }

    #[test]
    fn full_months_are_the_covered_months_before_this_one() {
        let got = full_months(date!(2026 - 07 - 01), date!(2026 - 10 - 31), today());
        assert_eq!(got, vec![(2026, Month::July), (2026, Month::August), (2026, Month::September), (2026, Month::October)]);
    }

    #[test]
    fn a_month_that_coverage_starts_into_or_stops_short_of_is_not_full() {
        let late_start = full_months(date!(2026 - 07 - 15), date!(2026 - 10 - 31), today());
        assert_eq!(late_start.first(), Some(&(2026, Month::August)));
        let early_stop = full_months(date!(2026 - 07 - 01), date!(2026 - 10 - 20), today());
        assert_eq!(early_stop.last(), Some(&(2026, Month::September)));
        assert_eq!(early_stop.len(), 3);
    }

    #[test]
    fn full_months_stop_at_twelve() {
        let got = full_months(date!(2025 - 01 - 01), date!(2026 - 10 - 31), today());
        assert_eq!(got.len(), MAX_MONTHS_USED);
        assert_eq!(got.first(), Some(&(2025, Month::November)));
        assert_eq!(got.last(), Some(&(2026, Month::October)));
    }

    #[test]
    fn figures_average_the_months_and_take_the_sample_spread() {
        let flat = figures(&[
            MonthFigures { income: 10_000_000, spend: 3_000_000, invested: 500_000 },
            MonthFigures { income: 10_000_000, spend: 3_000_000, invested: 500_000 },
            MonthFigures { income: 10_000_000, spend: 3_000_000, invested: 500_000 },
        ]);
        assert_eq!(flat.months, 3);
        assert_eq!((flat.income, flat.spend, flat.invested), (10_000_000, 3_000_000, 500_000));
        assert_eq!(flat.surplus_sd, 0);

        // Surpluses of 60,000 and 80,000 rupees: mean 70,000, sample sd about 14,142.
        let wobbly = figures(&[
            MonthFigures { income: 9_000_000, spend: 3_000_000, invested: 0 },
            MonthFigures { income: 11_000_000, spend: 3_000_000, invested: 0 },
        ]);
        assert!((wobbly.surplus_sd - 1_414_214).abs() <= 1, "{}", wobbly.surplus_sd);
        assert_eq!(figures(&[]).months, 0);
    }

    #[test]
    fn cut_spend_is_added_only_for_a_small_share_of_the_top_category() {
        let save = |amount| Lever {
            kind: LeverKind::SaveMore,
            text: String::new(),
            monthly_paise: Some(amount),
            months: None,
            target_paise: None,
        };
        let top = ("food".to_string(), 1_000_000);

        let mut small = vec![save(200_000)];
        add_cut_spend(&mut small, Some(&top));
        assert_eq!(small.len(), 2);
        assert_eq!(small[1].kind, LeverKind::CutSpend);
        assert_eq!(small[1].text, "That's about 20% of your monthly food spend.");

        let mut big = vec![save(300_000)];
        add_cut_spend(&mut big, Some(&top));
        assert_eq!(big.len(), 1, "30% of the category is too much to ask");

        let mut none = vec![save(200_000)];
        add_cut_spend(&mut none, None);
        assert_eq!(none.len(), 1);
    }

    #[tokio::test]
    async fn with_no_statements_the_forecast_runs_on_the_profile() {
        let st = state().await;
        let profile = LifeProfile {
            my_take_home_paise: 15_000_000,
            partner_take_home_paise: 10_000_000,
            monthly_expenses_paise: Some(6_000_000),
            wedding_date: Some("2029-02-14".into()),
            child_birth_year: Some(2030),
            retirement_age: 58,
            ..Default::default()
        };
        assert!(ok(life_handler::save(State(st.clone()), user(), Json(profile)).await).0.saved);

        let f = ok(compute(&st, "u", today()).await);
        assert_eq!(f.basis.source, "profile");
        assert_eq!(f.basis.months_used, 0);
        assert_eq!(f.basis.income_paise, 25_000_000);
        assert_eq!(f.basis.spend_paise, 6_000_000);
        assert_eq!(f.basis.surplus_paise, 19_000_000);
        assert_eq!(f.basis.surplus_sd_paise, 2_850_000);
        assert_eq!(f.basis.note, None);
        assert!(f.saved_profile);
        let keys: Vec<&str> = f.goals.iter().map(|g| g.key).collect();
        assert_eq!(keys, vec!["emergency", "wedding", "child_birth", "child_education", "retirement"]);
        assert_well_formed(&f);
    }

    #[tokio::test]
    async fn without_a_saved_profile_the_emergency_fund_and_retirement_remain() {
        let st = state().await;
        let f = ok(compute(&st, "u", today()).await);
        assert!(!f.saved_profile);
        assert_eq!(f.basis.source, "profile");
        let keys: Vec<&str> = f.goals.iter().map(|g| g.key).collect();
        assert_eq!(keys, vec!["emergency", "retirement"]);
        assert_well_formed(&f);
    }

    #[tokio::test]
    async fn three_or_more_full_months_of_statements_become_the_basis() {
        let st = state().await;
        for q in [
            "INSERT INTO statement_imports (import_id, user_id, first_date, last_date, rows) VALUES ('imp1', 'u', '2026-07-01', '2026-10-31', 8)",
        ] {
            sqlx::query(q).execute(&st.db).await.unwrap();
        }
        for (i, month) in ["07", "08", "09", "10"].iter().enumerate() {
            let rows = [
                (format!("inc-{month}"), "credit", 10_000_000i64, "income"),
                (format!("food-{month}"), "debit", 3_000_000i64, "food"),
            ];
            for (k, (id, direction, amount, category)) in rows.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key)
                     VALUES (?, 'u', ?, ?, ?, 'Shop', ?, 'UPI payment', 'manual', ?)",
                )
                .bind(format!("t{i}{k}"))
                .bind(format!("2026-{month}-05"))
                .bind(*direction)
                .bind(*amount)
                .bind(*category)
                .bind(id)
                .execute(&st.db)
                .await
                .unwrap();
            }
        }
        // The plan says ₹1,40,000 coming in; the statements show ₹1,00,000, which is more than 10% off.
        let profile = LifeProfile { my_take_home_paise: 14_000_000, monthly_expenses_paise: Some(3_000_000), ..Default::default() };
        assert!(ok(life_handler::save(State(st.clone()), user(), Json(profile)).await).0.saved);

        let f = ok(compute(&st, "u", today()).await);
        assert_eq!(f.basis.source, "statements");
        assert_eq!(f.basis.months_used, 4);
        assert_eq!(f.basis.income_paise, 10_000_000);
        assert_eq!(f.basis.spend_paise, 3_000_000);
        assert_eq!(f.basis.surplus_paise, 7_000_000);
        assert_eq!(f.basis.surplus_sd_paise, 0);
        assert_eq!(f.basis.entered_income_paise, 14_000_000);
        let note = f.basis.note.as_deref().expect("a note when the figures differ");
        assert!(note.contains("₹1,00,000") && note.contains("₹1,40,000"), "{note}");
        assert_well_formed(&f);

        // With no typed expenses, the emergency fund is six months of what the statements show.
        let untyped = LifeProfile { my_take_home_paise: 14_000_000, ..Default::default() };
        assert!(ok(life_handler::save(State(st.clone()), user(), Json(untyped)).await).0.saved);
        let f = ok(compute(&st, "u", today()).await);
        let emergency = f.goals.iter().find(|g| g.key == "emergency").unwrap();
        assert_eq!(emergency.target_paise, 6 * 3_000_000);
    }
}
