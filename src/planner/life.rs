//! Life plan: the couple's goals (emergency fund, wedding, child, retirement) priced in future
//! rupees, the monthly SIP each one needs, how the SIP splits between the two incomes, and whether
//! the surplus covers it all. Pure: no DB, no clock. `today` comes in as a parameter.
//! Money is integer paise; rates are basis points (100 bps = 1%).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date, Month};

use super::format_inr;

/// Money in advice text: whole rupees (planning figures, not ledger entries).
fn inr(paise: i64) -> String {
    format_inr(paise.saturating_add(50).div_euclid(100).saturating_mul(100))
}
use crate::validate::{MAX_AMOUNT_PAISE, MAX_MONTHS};

pub const GENERAL_INFLATION_BPS: i64 = 600;
pub const EDUCATION_INFLATION_BPS: i64 = 1000;
pub const MEDICAL_INFLATION_BPS: i64 = 1000;
pub const WEDDING_INFLATION_BPS: i64 = 800;
pub const EQUITY_RETURN_BPS: i64 = 1150;
pub const HYBRID_RETURN_BPS: i64 = 900;
pub const DEBT_RETURN_BPS: i64 = 700;
pub const STEP_UP_BPS: i64 = 1000;
pub const SWR_BPS: i64 = 325;

/// ₹25,00,000 wedding budget in today's money, when a wedding date is set without one.
const DEFAULT_WEDDING_BUDGET: i64 = 250_000_000;
/// ₹30,00,000 UG+PG cost in today's money.
const DEFAULT_CHILD_EDUCATION: i64 = 300_000_000;
/// ₹3,00,000 for delivery and the first year, in today's money.
const CHILD_BIRTH_TODAY: i64 = 30_000_000;
const EMERGENCY_MONTHS: i64 = 6;
const EMERGENCY_HORIZON_MONTHS: i64 = 12;
const TERM_YEARS_OF_INCOME: i64 = 15;
/// ₹10,00,000 base family floater; ₹15,00,000 when a child is planned or someone is 45+.
const BASE_HEALTH_COVER: i64 = 100_000_000;
const FAMILY_HEALTH_COVER: i64 = 150_000_000;
const HEALTH_AGE_TRIGGER: i64 = 45;
const DEFAULT_MY_AGE: i64 = 28;
const DEFAULT_RETIREMENT_AGE: i64 = 60;
const MONTHS_PER_YEAR: i64 = 12;
/// Under this many months a goal is held in debt; up to `HYBRID_MAX_MONTHS` it is hybrid.
const DEBT_MAX_MONTHS: i64 = 36;
const HYBRID_MAX_MONTHS: i64 = 84;

/// What the user enters. Every field has a default, so a partial body still parses.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct LifeProfile {
    pub my_age: i64,
    pub partner_age: Option<i64>,
    pub my_take_home_paise: i64,
    pub partner_take_home_paise: i64,
    pub monthly_expenses_paise: Option<i64>,
    pub wedding_date: Option<String>,
    pub wedding_budget_paise: Option<i64>,
    pub wedding_saved_paise: i64,
    pub child_birth_year: Option<i64>,
    pub child_education_today_paise: i64,
    pub child_saved_paise: i64,
    pub retirement_age: i64,
    pub retirement_saved_paise: i64,
    pub emergency_saved_paise: i64,
    pub term_cover_paise: i64,
    pub health_cover_paise: i64,
    pub loans_paise: i64,
}

impl Default for LifeProfile {
    fn default() -> Self {
        Self {
            my_age: DEFAULT_MY_AGE,
            partner_age: None,
            my_take_home_paise: 0,
            partner_take_home_paise: 0,
            monthly_expenses_paise: None,
            wedding_date: None,
            wedding_budget_paise: None,
            wedding_saved_paise: 0,
            child_birth_year: None,
            child_education_today_paise: DEFAULT_CHILD_EDUCATION,
            child_saved_paise: 0,
            retirement_age: DEFAULT_RETIREMENT_AGE,
            retirement_saved_paise: 0,
            emergency_saved_paise: 0,
            term_cover_paise: 0,
            health_cover_paise: 0,
            loans_paise: 0,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Assumptions {
    pub general_inflation_bps: i64,
    pub education_inflation_bps: i64,
    pub medical_inflation_bps: i64,
    pub wedding_inflation_bps: i64,
    pub equity_return_bps: i64,
    pub hybrid_return_bps: i64,
    pub debt_return_bps: i64,
    pub step_up_bps: i64,
    pub swr_bps: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct LifeGoal {
    pub key: &'static str,
    pub label: &'static str,
    pub when: Option<String>,
    pub years_away: i64,
    pub today_cost_paise: i64,
    pub future_cost_paise: i64,
    pub saved_paise: i64,
    pub gap_paise: i64,
    pub monthly_sip_paise: i64,
    pub step_up_sip_paise: i64,
    pub mix: &'static str,
    pub mix_note: &'static str,
    pub my_share_paise: i64,
    pub partner_share_paise: i64,
    pub status: &'static str,
    pub note: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Insurance {
    pub term_needed_paise: i64,
    pub term_have_paise: i64,
    pub term_gap_paise: i64,
    pub health_needed_paise: i64,
    pub health_have_paise: i64,
    pub health_gap_paise: i64,
    pub notes: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Monthly {
    pub total_sip_paise: i64,
    pub total_step_up_sip_paise: i64,
    pub surplus_paise: i64,
    pub fits: bool,
    pub shortfall_paise: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct TimelineRow {
    pub year: i64,
    pub age: i64,
    pub events: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct LifePlan {
    pub assumptions: Assumptions,
    pub goals: Vec<LifeGoal>,
    pub insurance: Insurance,
    pub monthly: Monthly,
    pub timeline: Vec<TimelineRow>,
}

/// Which return and wording a goal gets, from how far away it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mix {
    Debt,
    Hybrid,
    Equity,
}

impl Mix {
    fn return_bps(self) -> i64 {
        match self {
            Mix::Debt => DEBT_RETURN_BPS,
            Mix::Hybrid => HYBRID_RETURN_BPS,
            Mix::Equity => EQUITY_RETURN_BPS,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Mix::Debt => "debt",
            Mix::Hybrid => "hybrid",
            Mix::Equity => "equity",
        }
    }

    fn note(self) -> &'static str {
        match self {
            Mix::Debt => "Under 3 years: RD/FD or a liquid fund, not equity.",
            Mix::Hybrid => "3 to 7 years: a balanced or hybrid fund; move to RD/FD in the last 2 to 3 years.",
            Mix::Equity => "Over 7 years: equity index funds; move to RD/FD in the last 2 to 3 years.",
        }
    }
}

/// Horizon in months decides the mix: under 3 years debt, 3 to 7 hybrid, over 7 equity.
pub fn mix_for(months: i64) -> Mix {
    if months < DEBT_MAX_MONTHS {
        Mix::Debt
    } else if months <= HYBRID_MAX_MONTHS {
        Mix::Hybrid
    } else {
        Mix::Equity
    }
}

/// Parses a `YYYY-MM-DD` date. `None` when it isn't one.
pub fn parse_date(s: &str) -> Option<Date> {
    Date::parse(s.trim(), format_description!("[year]-[month]-[day]")).ok()
}

/// Keeps an amount inside `0..=MAX_AMOUNT_PAISE`.
fn cap(v: i64) -> i64 {
    v.clamp(0, MAX_AMOUNT_PAISE)
}

/// Rounds a float amount to paise, clamped to `0..=MAX_AMOUNT_PAISE`. NaN becomes 0.
fn to_paise(x: f64) -> i64 {
    if x.is_nan() {
        0
    } else {
        x.round().clamp(0.0, MAX_AMOUNT_PAISE as f64) as i64
    }
}

fn rate(bps: i64) -> f64 {
    bps as f64 / 10_000.0
}

/// `pv` grown for `months` at an annual rate in bps: `pv * (1 + i)^(months / 12)`.
/// Also used for inflating today's cost to a future date.
pub fn future_value(pv: i64, annual_bps: i64, months: i64) -> i64 {
    let years = months.clamp(0, MAX_MONTHS) as f64 / MONTHS_PER_YEAR as f64;
    to_paise(pv as f64 * (1.0 + rate(annual_bps)).powf(years))
}

/// Flat monthly SIP (paid at the start of each month) that grows `gap` to the target in `months`.
/// `months` of 0 means the whole gap is due now; a zero rate just splits the gap evenly.
pub fn flat_sip(gap: i64, annual_bps: i64, months: i64) -> i64 {
    if gap <= 0 {
        return 0;
    }
    let m = months.clamp(0, MAX_MONTHS);
    if m == 0 {
        return gap;
    }
    let r = rate(annual_bps) / MONTHS_PER_YEAR as f64;
    if r == 0.0 {
        return to_paise(gap as f64 / m as f64);
    }
    let growth = (1.0 + r).powi(m as i32);
    to_paise(gap as f64 * r / ((growth - 1.0) * (1.0 + r)))
}

/// First-year SIP when the SIP steps up by `STEP_UP_BPS` every 12 months and still closes `gap`.
/// Month `t` pays `(1+step)^(t/12)` times the first-year amount, compounded for the months left.
pub fn step_up_sip(gap: i64, annual_bps: i64, months: i64) -> i64 {
    if gap <= 0 {
        return 0;
    }
    let m = months.clamp(0, MAX_MONTHS);
    if m == 0 {
        return gap;
    }
    let r = rate(annual_bps) / MONTHS_PER_YEAR as f64;
    let step = 1.0 + rate(STEP_UP_BPS);
    let units: f64 = (0..m)
        .map(|t| step.powi((t / MONTHS_PER_YEAR) as i32) * (1.0 + r).powi((m - t) as i32))
        .sum();
    if units <= 0.0 {
        return gap;
    }
    to_paise(gap as f64 / units)
}

/// Whole months from `today` to `when`, by calendar month, clamped to `0..=MAX_MONTHS`.
fn months_until(today: Date, when: Date) -> i64 {
    let m = (when.year() as i64 - today.year() as i64) * MONTHS_PER_YEAR
        + (when.month() as i64 - today.month() as i64);
    m.clamp(0, MAX_MONTHS)
}

/// `d` moved by whole years. Feb 29 falls back to the 28th in a non-leap year.
fn shift_years(d: Date, years: i64) -> Date {
    let y = (d.year() as i64 + years).clamp(1, 9999) as i32;
    Date::from_calendar_date(y, d.month(), d.day())
        .or_else(|_| Date::from_calendar_date(y, d.month(), 28))
        .unwrap_or(d)
}

/// A fixed calendar date in `year` (clamped to a sane range), or `fallback` if it can't be built.
fn on(year: i64, month: Month, day: u8, fallback: Date) -> Date {
    let y = year.clamp(1, 9999) as i32;
    Date::from_calendar_date(y, month, day).unwrap_or(fallback)
}

/// The annual expenses at retirement, turned into a corpus at the safe withdrawal rate.
fn corpus(annual_paise: i64) -> i64 {
    to_paise(annual_paise as f64 * 10_000.0 / SWR_BPS as f64)
}

/// A goal before its share and status are known.
struct Spec {
    key: &'static str,
    label: &'static str,
    when: Option<Date>,
    months: i64,
    today_cost: i64,
    future_cost: i64,
    saved: i64,
    mix: Mix,
    mix_note: &'static str,
}

fn build(spec: Spec) -> LifeGoal {
    let ret = spec.mix.return_bps();
    let saved = cap(spec.saved);
    let future_cost = cap(spec.future_cost);
    let fv_saved = future_value(saved, ret, spec.months);
    let gap = future_cost.saturating_sub(fv_saved).max(0);
    LifeGoal {
        key: spec.key,
        label: spec.label,
        when: spec.when.map(|d| d.to_string()),
        years_away: if spec.when.is_some() { spec.months / MONTHS_PER_YEAR } else { 0 },
        today_cost_paise: cap(spec.today_cost),
        future_cost_paise: future_cost,
        saved_paise: saved,
        gap_paise: gap,
        monthly_sip_paise: flat_sip(gap, ret, spec.months),
        step_up_sip_paise: step_up_sip(gap, ret, spec.months),
        mix: spec.mix.name(),
        mix_note: spec.mix_note,
        my_share_paise: 0,
        partner_share_paise: 0,
        status: "needs_more",
        note: String::new(),
    }
}

/// Builds the whole life plan. `expenses_paise` is the monthly household spend the caller resolved.
pub fn plan(profile: &LifeProfile, expenses_paise: i64, today: Date) -> LifePlan {
    let ty = today.year() as i64;
    let my_age = profile.my_age.clamp(0, 100);
    let retirement_age = profile.retirement_age.clamp(0, 100);
    let years_to_retire = (retirement_age - my_age).max(0);
    let expenses = cap(expenses_paise);
    let my_take = cap(profile.my_take_home_paise);
    let partner_take = cap(profile.partner_take_home_paise);
    let combined = my_take.saturating_add(partner_take);
    let child_birth = profile.child_birth_year.map(|y| y.clamp(ty - 100, ty + 100));

    let mut goals: Vec<(i64, LifeGoal)> = Vec::new();

    // Emergency fund: 6 months of expenses, now, held in debt.
    let emergency_target = cap(expenses.saturating_mul(EMERGENCY_MONTHS));
    goals.push((
        -1,
        build(Spec {
            key: "emergency",
            label: "Emergency fund",
            when: None,
            months: EMERGENCY_HORIZON_MONTHS,
            today_cost: emergency_target,
            future_cost: emergency_target,
            saved: profile.emergency_saved_paise,
            mix: Mix::Debt,
            mix_note: "Keep it liquid: savings account, liquid fund or short FD.",
        }),
    ));

    // Wedding: budget in today's money, inflated to the wedding date.
    if let Some(when) = profile.wedding_date.as_deref().and_then(parse_date) {
        let months = months_until(today, when);
        let budget = cap(profile.wedding_budget_paise.unwrap_or(DEFAULT_WEDDING_BUDGET));
        let mix = mix_for(months);
        goals.push((
            months,
            build(Spec {
                key: "wedding",
                label: "Wedding",
                when: Some(when),
                months,
                today_cost: budget,
                future_cost: future_value(budget, WEDDING_INFLATION_BPS, months),
                saved: profile.wedding_saved_paise,
                mix,
                mix_note: mix.note(),
            }),
        ));
    }

    if let Some(birth_year) = child_birth {
        // Baby's first year: January of the birth year.
        let when = on(birth_year, Month::January, 1, today);
        let months = months_until(today, when);
        let mix = mix_for(months);
        goals.push((
            months,
            build(Spec {
                key: "child_birth",
                label: "Baby's first year",
                when: Some(when),
                months,
                today_cost: CHILD_BIRTH_TODAY,
                future_cost: future_value(CHILD_BIRTH_TODAY, MEDICAL_INFLATION_BPS, months),
                saved: profile.child_saved_paise,
                mix,
                mix_note: mix.note(),
            }),
        ));

        // Education: June of the child's 18th year.
        let when = on(birth_year + 18, Month::June, 1, today);
        let months = months_until(today, when);
        let mix = mix_for(months);
        let education_today = cap(profile.child_education_today_paise);
        goals.push((
            months,
            build(Spec {
                key: "child_education",
                label: "Child's education",
                when: Some(when),
                months,
                today_cost: education_today,
                future_cost: future_value(education_today, EDUCATION_INFLATION_BPS, months),
                saved: profile.child_saved_paise,
                mix,
                mix_note: mix.note(),
            }),
        ));
    }

    // Retirement: the corpus that pays annual expenses at the safe withdrawal rate.
    let retire_on = shift_years(today, years_to_retire);
    let months = years_to_retire * MONTHS_PER_YEAR;
    let annual_today = expenses.saturating_mul(MONTHS_PER_YEAR);
    let annual_then = future_value(annual_today, GENERAL_INFLATION_BPS, months);
    let mix = mix_for(months);
    goals.push((
        months,
        build(Spec {
            key: "retirement",
            label: "Retirement",
            when: Some(retire_on),
            months,
            today_cost: corpus(annual_today),
            future_cost: corpus(annual_then),
            saved: profile.retirement_saved_paise,
            mix,
            mix_note: mix.note(),
        }),
    ));

    goals.sort_by_key(|(order, _)| *order);
    let mut goals: Vec<LifeGoal> = goals.into_iter().map(|(_, g)| g).collect();

    // Split each step-up SIP by take-home share, then give each goal a status in date order:
    // each one draws from the surplus left after the goals before it.
    let total_step_up: i64 = goals.iter().map(|g| g.step_up_sip_paise).sum();
    let total_sip: i64 = goals.iter().map(|g| g.monthly_sip_paise).sum();
    let surplus = combined.saturating_sub(expenses).clamp(-MAX_AMOUNT_PAISE, MAX_AMOUNT_PAISE);
    let mut remaining = surplus.max(0);
    for g in goals.iter_mut() {
        let step = g.step_up_sip_paise;
        g.partner_share_paise = if combined == 0 || partner_take == 0 {
            0
        } else {
            (step as f64 * partner_take as f64 / combined as f64).round() as i64
        };
        g.my_share_paise = step - g.partner_share_paise;
        if g.gap_paise == 0 {
            g.status = "done";
            g.note = "Already covered by what is saved.".to_string();
        } else if step <= remaining {
            g.status = "on_track";
            g.note = format!("About {} a month at step-up, which fits the surplus.", inr(step));
            remaining -= step;
        } else {
            g.status = "needs_more";
            g.note = format!(
                "Needs {} a month at step-up, about {} more than the surplus left for it.",
                inr(step),
                inr(step - remaining)
            );
            remaining = 0;
        }
    }

    let child_education_future = goals.iter().find(|g| g.key == "child_education").map(|g| g.future_cost_paise);
    let loans = cap(profile.loans_paise);
    let term_needed = cap(
        cap(my_take.saturating_mul(MONTHS_PER_YEAR).saturating_mul(TERM_YEARS_OF_INCOME))
            .saturating_add(cap(partner_take.saturating_mul(MONTHS_PER_YEAR).saturating_mul(TERM_YEARS_OF_INCOME)))
            .saturating_add(loans)
            .saturating_add(child_education_future.unwrap_or(0)),
    );
    let term_have = cap(profile.term_cover_paise);
    let term_gap = term_needed.saturating_sub(term_have).max(0);

    let partner_age = profile.partner_age.map(|a| a.clamp(0, 100));
    let wider_health_cover = child_birth.is_some() || my_age >= HEALTH_AGE_TRIGGER || partner_age.is_some_and(|a| a >= HEALTH_AGE_TRIGGER);
    let health_needed = if wider_health_cover { FAMILY_HEALTH_COVER } else { BASE_HEALTH_COVER };
    let health_have = cap(profile.health_cover_paise);
    let health_gap = health_needed.saturating_sub(health_have).max(0);

    let mut notes = Vec::new();
    let child_part = if child_education_future.is_some() { " and the child's education" } else { "" };
    if combined == 0 {
        notes.push("Enter take-home pay to size term cover.".to_string());
    }
    if term_gap > 0 {
        notes.push(format!(
            "Term cover is short by {}. It should cover {} years of take-home for each of you, plus loans{}.",
            inr(term_gap),
            TERM_YEARS_OF_INCOME,
            child_part
        ));
    } else {
        notes.push(format!(
            "Term cover covers {} years of take-home, loans{}.",
            TERM_YEARS_OF_INCOME,
            child_part
        ));
    }
    let reason = if child_birth.is_some() {
        "a child is planned"
    } else if wider_health_cover {
        "someone in the family is 45 or older"
    } else {
        "a base family floater"
    };
    if health_gap > 0 {
        notes.push(format!(
            "Health cover is short by {}. Target is {} because {}.",
            inr(health_gap),
            inr(health_needed),
            reason
        ));
    } else {
        notes.push(format!("Health cover meets the {} target because {}.", inr(health_needed), reason));
    }

    // Timeline: one row per year with an event, from this year to the retirement year.
    let retire_year = ty + years_to_retire;
    let mut events: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    events.entry(ty).or_default().push("Emergency fund target".to_string());
    for g in &goals {
        let Some(when) = &g.when else { continue };
        let Some(year) = when.get(..4).and_then(|y| y.parse::<i64>().ok()) else { continue };
        let text = if g.key == "retirement" { format!("Retirement at {retirement_age}") } else { g.label.to_string() };
        events.entry(year).or_default().push(text);
    }
    let timeline = events
        .into_iter()
        .filter(|(year, _)| *year >= ty && *year <= retire_year)
        .map(|(year, events)| TimelineRow { year, age: my_age + (year - ty), events })
        .collect();

    LifePlan {
        assumptions: Assumptions {
            general_inflation_bps: GENERAL_INFLATION_BPS,
            education_inflation_bps: EDUCATION_INFLATION_BPS,
            medical_inflation_bps: MEDICAL_INFLATION_BPS,
            wedding_inflation_bps: WEDDING_INFLATION_BPS,
            equity_return_bps: EQUITY_RETURN_BPS,
            hybrid_return_bps: HYBRID_RETURN_BPS,
            debt_return_bps: DEBT_RETURN_BPS,
            step_up_bps: STEP_UP_BPS,
            swr_bps: SWR_BPS,
        },
        goals,
        insurance: Insurance {
            term_needed_paise: term_needed,
            term_have_paise: term_have,
            term_gap_paise: term_gap,
            health_needed_paise: health_needed,
            health_have_paise: health_have,
            health_gap_paise: health_gap,
            notes,
        },
        monthly: Monthly {
            total_sip_paise: total_sip,
            total_step_up_sip_paise: total_step_up,
            surplus_paise: surplus,
            fits: total_step_up <= surplus,
            shortfall_paise: total_step_up.saturating_sub(surplus).max(0),
        },
        timeline,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn today() -> Date {
        date!(2026 - 11 - 09)
    }

    fn goal<'a>(plan: &'a LifePlan, key: &str) -> &'a LifeGoal {
        plan.goals.iter().find(|g| g.key == key).unwrap()
    }

    /// Simulates start-of-month payments: `pay(t)` in month t, each compounding monthly.
    fn simulate(months: i64, annual_bps: i64, pay: impl Fn(i64) -> f64) -> f64 {
        let r = rate(annual_bps) / 12.0;
        (0..months).fold(0.0, |fv, t| (fv + pay(t)) * (1.0 + r))
    }

    #[test]
    fn future_value_matches_a_known_case() {
        // ₹25L at 8% for 2 years is ₹29.16L.
        let fv = future_value(250_000_000, 800, 24);
        assert!((fv - 291_600_000).abs() < 100, "got {fv}");
    }

    #[test]
    fn flat_sip_closes_the_target() {
        let gap = 100_000_000; // ₹10L
        let sip = flat_sip(gap, DEBT_RETURN_BPS, 36);
        let fv = simulate(36, DEBT_RETURN_BPS, |_| sip as f64);
        assert!((fv - gap as f64).abs() < 1_000.0, "fv {fv} vs gap {gap}");
    }

    #[test]
    fn step_up_first_year_sip_is_below_flat_and_closes_the_target() {
        let gap = 100_000_000;
        let (months, bps) = (120, EQUITY_RETURN_BPS);
        let flat = flat_sip(gap, bps, months);
        let first = step_up_sip(gap, bps, months);
        assert!(first < flat, "step-up {first} should be below flat {flat}");
        let growth = 1.0 + rate(STEP_UP_BPS);
        let fv = simulate(months, bps, |t| first as f64 * growth.powi((t / 12) as i32));
        assert!((fv - gap as f64).abs() < 1_000.0, "fv {fv} vs gap {gap}");
    }

    #[test]
    fn zero_rate_and_zero_months_do_not_divide_by_zero() {
        assert_eq!(flat_sip(1_200, 0, 12), 100);
        assert_eq!(step_up_sip(1_200, 0, 0), 1_200);
        assert_eq!(flat_sip(1_200, 700, 0), 1_200);
        assert_eq!(flat_sip(0, 700, 12), 0);
    }

    #[test]
    fn a_wedding_under_three_years_is_held_in_debt() {
        let profile = LifeProfile { wedding_date: Some("2028-06-01".into()), ..Default::default() };
        let plan = plan(&profile, 5_000_000, today());
        let wedding = goal(&plan, "wedding");
        assert_eq!(wedding.mix, "debt");
        assert!(wedding.mix_note.contains("Under 3 years"));
        assert_eq!(wedding.years_away, 1);
    }

    #[test]
    fn child_education_eighteen_years_out_is_equity() {
        let profile = LifeProfile { child_birth_year: Some(2026), ..Default::default() };
        let plan = plan(&profile, 5_000_000, today());
        let edu = goal(&plan, "child_education");
        assert_eq!(edu.when.as_deref(), Some("2044-06-01"));
        assert_eq!(edu.mix, "equity");
        assert_eq!(edu.years_away, 17);
        let expected = future_value(DEFAULT_CHILD_EDUCATION, EDUCATION_INFLATION_BPS, 211);
        assert_eq!(edu.future_cost_paise, expected);
    }

    #[test]
    fn retirement_corpus_is_about_thirty_point_eight_times_annual_expenses() {
        let expenses = 10_000_000; // ₹1L a month
        let profile = LifeProfile { my_age: 28, retirement_age: 60, ..Default::default() };
        let plan = plan(&profile, expenses, today());
        let retirement = goal(&plan, "retirement");
        let annual_then = future_value(expenses * 12, GENERAL_INFLATION_BPS, 32 * 12) as f64;
        let ratio = retirement.future_cost_paise as f64 / annual_then;
        assert!((ratio - 10_000.0 / 325.0).abs() < 0.001, "ratio {ratio}");
        assert_eq!(retirement.when.as_deref(), Some("2058-11-09"));
    }

    #[test]
    fn no_partner_income_means_no_partner_share() {
        let profile = LifeProfile { my_take_home_paise: 15_000_000, ..Default::default() };
        let plan = plan(&profile, 6_000_000, today());
        for g in &plan.goals {
            assert_eq!(g.partner_share_paise, 0, "{}", g.key);
            assert_eq!(g.my_share_paise, g.step_up_sip_paise, "{}", g.key);
        }
    }

    #[test]
    fn insurance_term_need_includes_loans_and_child_education() {
        let profile = LifeProfile {
            my_take_home_paise: 10_000_000,
            partner_take_home_paise: 5_000_000,
            loans_paise: 500_000_000,
            child_birth_year: Some(2029),
            ..Default::default()
        };
        let plan = plan(&profile, 6_000_000, today());
        let edu = goal(&plan, "child_education").future_cost_paise;
        let income_cover = 15 * 12 * (10_000_000 + 5_000_000);
        assert_eq!(plan.insurance.term_needed_paise, income_cover + 500_000_000 + edu);
        assert_eq!(plan.insurance.term_gap_paise, plan.insurance.term_needed_paise);
        assert_eq!(plan.insurance.health_needed_paise, FAMILY_HEALTH_COVER);
    }

    #[test]
    fn fits_and_shortfall_follow_the_surplus() {
        let profile = LifeProfile {
            my_take_home_paise: 10_000_000,
            partner_take_home_paise: 5_000_000,
            wedding_date: Some("2027-11-01".into()),
            wedding_budget_paise: Some(250_000_000),
            ..Default::default()
        };
        let plan = plan(&profile, 8_000_000, today());
        let m = &plan.monthly;
        assert_eq!(m.surplus_paise, 7_000_000);
        assert_eq!(m.fits, m.total_step_up_sip_paise <= m.surplus_paise);
        assert_eq!(m.shortfall_paise, (m.total_step_up_sip_paise - m.surplus_paise).max(0));
        assert!(!m.fits, "a ₹25L wedding in a year should not fit on ₹7L a month");
        assert!(m.shortfall_paise > 0);
    }

    #[test]
    fn a_small_goal_fits_the_surplus() {
        let profile = LifeProfile { my_take_home_paise: 50_000_000, emergency_saved_paise: 100_000_000, ..Default::default() };
        let plan = plan(&profile, 2_000_000, today());
        assert!(plan.monthly.fits);
        assert_eq!(plan.monthly.shortfall_paise, 0);
        assert_eq!(goal(&plan, "emergency").status, "done");
    }

    #[test]
    fn without_wedding_or_child_only_emergency_and_retirement_remain() {
        let plan = plan(&LifeProfile::default(), 5_000_000, today());
        let keys: Vec<&str> = plan.goals.iter().map(|g| g.key).collect();
        assert_eq!(keys, vec!["emergency", "retirement"]);
        assert_eq!(goal(&plan, "emergency").future_cost_paise, 30_000_000);
    }

    #[test]
    fn extreme_inputs_do_not_panic() {
        let profile = LifeProfile {
            my_age: i64::MIN,
            partner_age: Some(i64::MAX),
            my_take_home_paise: i64::MAX,
            partner_take_home_paise: i64::MIN,
            monthly_expenses_paise: Some(i64::MAX),
            wedding_date: Some("9999-12-31".into()),
            wedding_budget_paise: Some(i64::MAX),
            wedding_saved_paise: i64::MAX,
            child_birth_year: Some(i64::MAX),
            child_education_today_paise: i64::MAX,
            child_saved_paise: i64::MIN,
            retirement_age: i64::MAX,
            retirement_saved_paise: i64::MAX,
            emergency_saved_paise: i64::MAX,
            term_cover_paise: i64::MIN,
            health_cover_paise: i64::MAX,
            loans_paise: i64::MAX,
        };
        let plan = plan(&profile, i64::MAX, today());
        for g in &plan.goals {
            assert!(g.future_cost_paise >= 0 && g.future_cost_paise <= MAX_AMOUNT_PAISE);
            assert!(g.monthly_sip_paise >= 0 && g.step_up_sip_paise >= 0);
        }
        let plan = super::plan(&LifeProfile { wedding_date: Some("garbage".into()), ..Default::default() }, -5, today());
        assert!(plan.goals.iter().all(|g| g.key != "wedding"));
    }
}
