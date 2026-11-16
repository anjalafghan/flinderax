//! A child's education fund: what each stage will cost by then, and the monthly top-up that
//! reaches it. Pure: no DB, no clock (`today` comes in).
//!
//! Money is integer paise and rates are basis points. The compounding below is i128 fixed point
//! (no floats anywhere), and every figure that is shown is rounded in the cautious direction:
//! costs up, top-ups up.
//!
//! The maths, with `r` the monthly return (annual return / 12) and `n_k` the months to stage `k`:
//! money paid in at the end of each month for `n` months grows to `P * s(n)`, where
//! `s(n) = sum of (1 + r)^t for t < n`, and the corpus grows to `C * (1 + r)^n`. Fees come out
//! of the pot as they fall due, so the top-up must keep the pot from running dry at every fee, not
//! only at the last: `P` is the largest, over the fees, of
//! `(fees paid so far, each carried forward to that date, less the corpus carried forward) / s(n_k)`.
//!
//! Return and inflation are guesses. [`summarise`] therefore gives three answers: a base case and
//! a low and high case that shift both assumptions the helpful and the unhelpful way. This is
//! planning arithmetic, not investment advice.

use serde::{Deserialize, Serialize};
use time::Date;

use super::life::{EDUCATION_INFLATION_BPS, EQUITY_RETURN_BPS};
use super::lifeline::Earmark;
use crate::validate::MAX_AMOUNT_PAISE;

/// Education costs usually rise faster than prices in general.
pub const DEFAULT_INFLATION_BPS: i64 = EDUCATION_INFLATION_BPS;
/// Money set aside for a goal this far off is assumed to be mostly in equity-linked funds.
pub const DEFAULT_RETURN_BPS: i64 = EQUITY_RETURN_BPS;
/// The low and high cases move inflation and return this far each way.
pub const RANGE_SHIFT_BPS: i64 = 200;
pub const MAX_STAGES: usize = 6;
pub const MAX_INFLATION_BPS: i64 = 2_000;
pub const MAX_RETURN_BPS: i64 = 3_000;
/// Fees fall due in June, when the school year starts.
const FEE_MONTH: i64 = 6;
const MAX_NAME: usize = 40;
const SCALE: i128 = 1_000_000_000_000;

/// One stage of education: when the child starts it and what it costs in today's money.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Stage {
    pub name: String,
    pub start_age: i64,
    pub today_cost_paise: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Plan {
    pub child_birth_year: i64,
    pub stages: Vec<Stage>,
    pub inflation_bps: i64,
    pub return_bps: i64,
    /// Already set aside for this, in paise.
    pub corpus_paise: i64,
}

impl Plan {
    /// A starting point: school at 5, college at 18, on the default assumptions.
    pub fn starter(child_birth_year: i64) -> Plan {
        Plan {
            child_birth_year,
            stages: vec![
                Stage { name: "School".into(), start_age: 5, today_cost_paise: 30_000_000 },
                Stage { name: "College".into(), start_age: 18, today_cost_paise: 150_000_000 },
            ],
            inflation_bps: DEFAULT_INFLATION_BPS,
            return_bps: DEFAULT_RETURN_BPS,
            corpus_paise: 0,
        }
    }
}

/// Checks a plan. The text is shown to the user.
pub fn validate(plan: &Plan, today: Date) -> Result<(), String> {
    let ty = today.year() as i64;
    if !(ty - 30..=ty + 25).contains(&plan.child_birth_year) {
        return Err(format!("Birth year must be between {} and {}.", ty - 30, ty + 25));
    }
    if plan.stages.is_empty() || plan.stages.len() > MAX_STAGES {
        return Err(format!("Add between 1 and {MAX_STAGES} stages."));
    }
    for s in &plan.stages {
        let n = s.name.trim().chars().count();
        if n == 0 || n > MAX_NAME {
            return Err(format!("Each stage needs a name of 1 to {MAX_NAME} characters."));
        }
        if !(0..=40).contains(&s.start_age) {
            return Err(format!("{}: the starting age must be between 0 and 40.", s.name.trim()));
        }
        if !(0..=MAX_AMOUNT_PAISE).contains(&s.today_cost_paise) {
            return Err(format!("{}: the cost is out of range.", s.name.trim()));
        }
    }
    if !(0..=MAX_INFLATION_BPS).contains(&plan.inflation_bps) {
        return Err("Fee inflation must be between 0% and 20%.".into());
    }
    if !(0..=MAX_RETURN_BPS).contains(&plan.return_bps) {
        return Err("Expected return must be between 0% and 30%.".into());
    }
    if !(0..=MAX_AMOUNT_PAISE).contains(&plan.corpus_paise) {
        return Err("The amount already set aside is out of range.".into());
    }
    Ok(())
}

/// Whole months from this month to June of `year`. Zero or less: it has begun.
fn months_to(today: Date, year: i64) -> i64 {
    (year - today.year() as i64) * 12 + (FEE_MONTH - today.month() as i64)
}

/// The monthly step of an annual rate in basis points, as fixed point. `up` rounds it up.
fn step(bps: i64, up: bool) -> i128 {
    let num = bps.max(0) as i128 * SCALE;
    SCALE + if up { (num + 119_999) / 120_000 } else { num / 120_000 }
}

/// `(1 + monthly step)^t` for t = 0..=months, fixed point, each multiplication rounded up.
fn powers(step: i128, months: i64) -> Vec<i128> {
    let mut out = Vec::with_capacity(months.max(0) as usize + 1);
    let mut f = SCALE;
    out.push(f);
    for _ in 0..months.max(0) {
        f = (f * step + SCALE - 1) / SCALE;
        out.push(f);
    }
    out
}

/// What `today_paise` costs after `months` of `bps` yearly inflation, rounded up.
pub fn inflate(today_paise: i64, bps: i64, months: i64) -> i64 {
    if months <= 0 {
        return today_paise;
    }
    let f = powers(step(bps, true), months)[months as usize];
    let v = (today_paise.max(0) as i128 * f + SCALE - 1) / SCALE;
    v.min(MAX_AMOUNT_PAISE as i128) as i64
}

/// `x * f / SCALE` rounded up, split so a large `x` cannot overflow; saturates at the top.
fn carry(x: i128, f: i128) -> i128 {
    let (q, r) = (x / SCALE, x % SCALE);
    q.saturating_mul(f).saturating_add((r * f + SCALE - 1) / SCALE)
}

/// The level monthly top-up, paid at the end of each month, that keeps the pot from running dry
/// at any of the `fees` (months from now, amount then), given `corpus_paise` already in it and a
/// yearly return of `return_bps`. Rounded up. Fees that are not in the future are ignored; with
/// nothing left to pay it is zero.
pub fn monthly_top_up(fees: &[(i64, i64)], corpus_paise: i64, return_bps: i64) -> i64 {
    let mut fees: Vec<(i64, i64)> = fees.iter().copied().filter(|(m, a)| *m >= 1 && *a > 0).collect();
    fees.sort_unstable();
    let Some(&(last, _)) = fees.last() else { return 0 };
    let pw = powers(step(return_bps, false), last);
    // s[n]: what one rupee a month for n months is worth at the end, fixed point.
    let mut s = vec![0i128; last as usize + 1];
    for n in 1..=last as usize {
        s[n] = s[n - 1] + pw[n - 1];
    }
    let mut best: i128 = 0;
    let mut paid: i128 = 0; // fees so far carried forward to the current fee's date, fixed point
    let mut prev = 0usize;
    for &(m, amount) in &fees {
        let m = m as usize;
        paid = if prev == 0 { 0 } else { carry(paid, pw[m - prev]) };
        paid = paid.saturating_add(amount as i128 * SCALE);
        prev = m;
        let need = paid.saturating_sub((corpus_paise.max(0) as i128).saturating_mul(pw[m]));
        if need > 0 {
            best = best.max((need + s[m] - 1) / s[m]);
        }
    }
    best.min(MAX_AMOUNT_PAISE as i128) as i64
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct StageCost {
    pub name: String,
    pub start_age: i64,
    /// The calendar year the first fee falls due.
    pub year: i64,
    /// Months until then. Zero or less: the stage has begun.
    pub months_away: i64,
    pub today_cost_paise: i64,
    /// What it costs then, on the base assumptions. Equal to today's cost once it has begun.
    pub future_cost_paise: i64,
    /// A stage that has begun is not saved for: its fees are already being paid.
    pub started: bool,
}

/// The same plan on one set of assumptions.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Scenario {
    pub label: String,
    pub inflation_bps: i64,
    pub return_bps: i64,
    /// Each stage's cost then, in the order of [`Summary::stages`]. Zero for a stage that has begun.
    pub future_costs_paise: Vec<i64>,
    pub total_future_cost_paise: i64,
    /// Level monthly top-up needed, rounded up.
    pub monthly_top_up_paise: i64,
    /// How many months the top-up runs: until the last fee.
    pub months_to_save: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Summary {
    pub child_birth_year: i64,
    pub child_age: i64,
    pub corpus_paise: i64,
    pub stages: Vec<StageCost>,
    /// Costs rise least and savings earn most: the smallest top-up.
    pub low: Scenario,
    pub base: Scenario,
    /// Costs rise most and savings earn least: the largest top-up.
    pub high: Scenario,
    /// Every stage has begun or passed, so there is nothing left to save for.
    pub nothing_left: bool,
    pub notes: Vec<String>,
}

fn scenario(plan: &Plan, today: Date, label: &str, inflation_bps: i64, return_bps: i64) -> Scenario {
    let mut costs = Vec::with_capacity(plan.stages.len());
    let mut fees = Vec::new();
    for s in &plan.stages {
        let months = months_to(today, plan.child_birth_year + s.start_age);
        if months >= 1 {
            let cost = inflate(s.today_cost_paise, inflation_bps, months);
            fees.push((months, cost));
            costs.push(cost);
        } else {
            costs.push(0);
        }
    }
    Scenario {
        label: label.into(),
        inflation_bps,
        return_bps,
        total_future_cost_paise: costs.iter().fold(0i64, |a, c| a.saturating_add(*c)).min(MAX_AMOUNT_PAISE),
        future_costs_paise: costs,
        monthly_top_up_paise: monthly_top_up(&fees, plan.corpus_paise, return_bps),
        months_to_save: fees.iter().map(|(m, _)| *m).max().unwrap_or(0),
    }
}

/// Costs at each stage, the top-up, and the low and high cases around it.
pub fn summarise(plan: &Plan, today: Date) -> Summary {
    let (infl, ret) = (plan.inflation_bps.clamp(0, MAX_INFLATION_BPS), plan.return_bps.clamp(0, MAX_RETURN_BPS));
    let base = scenario(plan, today, "Base", infl, ret);
    let low = scenario(plan, today, "Low", (infl - RANGE_SHIFT_BPS).max(0), (ret + RANGE_SHIFT_BPS).min(MAX_RETURN_BPS));
    let high = scenario(plan, today, "High", infl + RANGE_SHIFT_BPS, (ret - RANGE_SHIFT_BPS).max(0));
    let stages: Vec<StageCost> = plan
        .stages
        .iter()
        .zip(&base.future_costs_paise)
        .map(|(s, cost)| {
            let year = plan.child_birth_year + s.start_age;
            let months = months_to(today, year);
            StageCost {
                name: s.name.trim().to_string(),
                start_age: s.start_age,
                year,
                months_away: months,
                today_cost_paise: s.today_cost_paise,
                future_cost_paise: if months >= 1 { *cost } else { s.today_cost_paise },
                started: months < 1,
            }
        })
        .collect();
    let nothing_left = base.months_to_save == 0;
    let mut notes = vec![
        "Fee inflation and the return on savings are guesses, not forecasts. The low and high cases show how far the answer moves if they are off by 2 percentage points.".to_string(),
        "This is planning arithmetic, not investment advice. Past returns do not promise future ones.".to_string(),
    ];
    if stages.iter().any(|s| s.started) && !nothing_left {
        notes.push("Stages that have already begun are not saved for here: their fees are already being paid.".to_string());
    }
    if nothing_left {
        notes.push("Every stage has begun or passed, so there is nothing left to save for.".to_string());
    }
    Summary {
        child_birth_year: plan.child_birth_year,
        child_age: today.year() as i64 - plan.child_birth_year,
        corpus_paise: plan.corpus_paise,
        stages,
        low,
        base,
        high,
        nothing_left,
        notes,
    }
}

/// The plan as the life line takes it: the top-up, the corpus and each coming fee on the base
/// assumptions.
pub fn earmark(plan: &Plan, today: Date) -> Earmark {
    let base = summarise(plan, today).base;
    let payouts = plan
        .stages
        .iter()
        .zip(&base.future_costs_paise)
        .map(|(s, cost)| (months_to(today, plan.child_birth_year + s.start_age), *cost))
        .filter(|(m, c)| *m >= 1 && *c > 0)
        .collect();
    Earmark {
        corpus_paise: plan.corpus_paise,
        monthly_paise: base.monthly_top_up_paise,
        return_bps: base.return_bps,
        payouts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn today() -> Date {
        date!(2026 - 06 - 15)
    }

    fn plan(stages: Vec<(&str, i64, i64)>) -> Plan {
        Plan {
            child_birth_year: 2026,
            stages: stages.into_iter().map(|(n, a, c)| Stage { name: n.into(), start_age: a, today_cost_paise: c }).collect(),
            inflation_bps: 1_000,
            return_bps: 1_200,
            corpus_paise: 0,
        }
    }

    #[test]
    fn inflating_with_no_inflation_or_no_time_changes_nothing() {
        assert_eq!(inflate(1_000_000, 0, 120), 1_000_000);
        assert_eq!(inflate(1_000_000, 1_000, 0), 1_000_000);
        assert_eq!(inflate(1_000_000, 1_000, -5), 1_000_000);
    }

    #[test]
    fn inflating_compounds_monthly_and_rounds_up() {
        // 12 months at 12% a year, 1% a month: 1.01^12 = 1.126825030...
        assert_eq!(inflate(1_000_000, 1_200, 12), 1_126_826);
        assert!(inflate(1, 1_000, 1) >= 1);
        // Never above the cap, however long and however steep.
        assert!(inflate(MAX_AMOUNT_PAISE, 2_000, 1_000) <= MAX_AMOUNT_PAISE);
    }

    #[test]
    fn with_no_return_the_top_up_is_the_shortfall_spread_evenly_and_rounded_up() {
        // 10 months, fee 1,000,001, nothing saved: ceil(1,000,001 / 10).
        assert_eq!(monthly_top_up(&[(10, 1_000_001)], 0, 0), 100_001);
        // The corpus counts in full.
        assert_eq!(monthly_top_up(&[(10, 1_000_000)], 400_000, 0), 60_000);
    }

    #[test]
    fn a_return_lowers_the_top_up_and_paying_it_in_reaches_the_fee() {
        let none = monthly_top_up(&[(120, 10_000_000)], 0, 0);
        let some = monthly_top_up(&[(120, 10_000_000)], 0, 1_200);
        assert_eq!(none, 83_334);
        assert!(some < none);
        // Simulate in whole paise at 1% a month, paid at the end of each month: the answer
        // reaches the fee and one paise less would not.
        let grow = |p: i64| {
            let mut pot = 0i128;
            for _ in 0..120 {
                pot = pot + pot / 100 + p as i128;
            }
            pot
        };
        // (Flooring the interest each month is slightly harsh, so allow a few paise either way.)
        assert!(grow(some + 2) >= 10_000_000, "{}", grow(some + 2));
        assert!(grow(some - 2) < 10_000_000, "{}", grow(some - 2));
    }

    #[test]
    fn the_corpus_alone_can_cover_it() {
        // 12 months at 12%: 1,000,000 grows to about 1,126,825.
        assert_eq!(monthly_top_up(&[(12, 1_126_000)], 1_000_000, 1_200), 0);
        assert_eq!(monthly_top_up(&[(12, 1_000_000)], 5_000_000, 1_200), 0);
    }

    #[test]
    fn the_pot_never_runs_dry_at_an_early_fee() {
        // A big fee soon and a small one later: the first sets the pace.
        let fees = [(12, 1_200_000), (120, 100_000)];
        let both = monthly_top_up(&fees, 0, 0);
        assert_eq!(both, 100_000, "1,200,000 / 12 beats 1,300,000 / 120");
        assert_eq!(monthly_top_up(&[(120, 1_300_000)], 0, 0), 10_834);
        // Simulating the pot with the answer stays at or above zero at each fee.
        let mut pot = 0i64;
        for month in 1..=120 {
            pot += both;
            if month == 12 {
                pot -= 1_200_000;
            }
            if month == 120 {
                pot -= 100_000;
            }
            assert!(pot >= 0, "month {month}: {pot}");
        }
    }

    #[test]
    fn fees_in_any_order_and_on_the_same_month_give_the_same_answer() {
        let a = monthly_top_up(&[(60, 500_000), (24, 300_000), (24, 200_000)], 50_000, 900);
        let b = monthly_top_up(&[(24, 200_000), (24, 300_000), (60, 500_000)], 50_000, 900);
        assert_eq!(a, b);
        assert!(a > 0);
    }

    #[test]
    fn fees_that_are_not_ahead_are_ignored() {
        assert_eq!(monthly_top_up(&[], 100, 1_000), 0);
        assert_eq!(monthly_top_up(&[(0, 5_000_000), (-12, 5_000_000)], 0, 1_000), 0);
        assert_eq!(monthly_top_up(&[(0, 5_000_000), (10, 0)], 0, 1_000), 0);
    }

    #[test]
    fn huge_inputs_do_not_overflow() {
        let t = monthly_top_up(&[(480, MAX_AMOUNT_PAISE), (240, MAX_AMOUNT_PAISE)], 0, MAX_RETURN_BPS);
        assert!(t > 0 && t <= MAX_AMOUNT_PAISE);
        assert_eq!(monthly_top_up(&[(1, MAX_AMOUNT_PAISE)], 0, 0), MAX_AMOUNT_PAISE);
    }

    #[test]
    fn each_stage_gets_its_cost_then() {
        // Born 2026, today June 2026: school at 5 is June 2031, college at 18 is June 2044.
        let s = summarise(&plan(vec![("School", 5, 3_000_000), ("College", 18, 15_000_000)]), today());
        assert_eq!(s.child_age, 0);
        assert_eq!(s.stages[0].year, 2031);
        assert_eq!(s.stages[0].months_away, 60);
        assert_eq!(s.stages[1].months_away, 216);
        assert!(s.stages[0].future_cost_paise > 3_000_000);
        assert!(s.stages[1].future_cost_paise > 15_000_000 * 5, "fees compound for longer");
        assert_eq!(s.stages[1].future_cost_paise, s.base.future_costs_paise[1]);
        assert_eq!(s.base.total_future_cost_paise, s.base.future_costs_paise.iter().sum::<i64>());
        assert_eq!(s.base.months_to_save, 216);
        assert!(!s.nothing_left);
    }

    #[test]
    fn the_range_runs_from_a_light_top_up_to_a_heavy_one() {
        let s = summarise(&plan(vec![("College", 18, 15_000_000)]), today());
        assert!(s.low.monthly_top_up_paise < s.base.monthly_top_up_paise);
        assert!(s.base.monthly_top_up_paise < s.high.monthly_top_up_paise);
        assert_eq!((s.low.inflation_bps, s.low.return_bps), (800, 1_400));
        assert_eq!((s.high.inflation_bps, s.high.return_bps), (1_200, 1_000));
        assert!(s.low.total_future_cost_paise < s.high.total_future_cost_paise);
    }

    #[test]
    fn the_range_stays_inside_the_limits_when_assumptions_are_at_the_edge() {
        let mut p = plan(vec![("College", 18, 15_000_000)]);
        p.inflation_bps = 100;
        p.return_bps = 0;
        let s = summarise(&p, today());
        assert_eq!((s.low.inflation_bps, s.low.return_bps), (0, 200));
        assert_eq!((s.high.inflation_bps, s.high.return_bps), (300, 0));
        p.return_bps = MAX_RETURN_BPS;
        assert_eq!(summarise(&p, today()).low.return_bps, MAX_RETURN_BPS);
    }

    #[test]
    fn a_stage_that_has_begun_is_not_saved_for() {
        // Born 2018: school began 2023, college is June 2036.
        let mut p = plan(vec![("School", 5, 3_000_000), ("College", 18, 15_000_000)]);
        p.child_birth_year = 2018;
        let s = summarise(&p, today());
        assert!(s.stages[0].started && !s.stages[1].started);
        assert_eq!(s.stages[0].future_cost_paise, 3_000_000, "shown at today's cost");
        assert_eq!(s.base.future_costs_paise[0], 0);
        assert_eq!(s.base.months_to_save, 120);
        let college_only = summarise(&Plan { stages: p.stages[1..].to_vec(), ..p.clone() }, today());
        assert_eq!(s.base.monthly_top_up_paise, college_only.base.monthly_top_up_paise);
    }

    #[test]
    fn a_stage_due_this_month_has_begun_and_one_due_next_month_has_not() {
        // June is the fee month: due now, not ahead.
        let mut p = plan(vec![("School", 5, 3_000_000)]);
        p.child_birth_year = 2021;
        assert!(summarise(&p, today()).stages[0].started);
        // A month before, it is one month ahead.
        let s = summarise(&p, date!(2026 - 05 - 31));
        assert_eq!(s.stages[0].months_away, 1);
        assert!(!s.stages[0].started);
    }

    #[test]
    fn with_no_time_left_there_is_nothing_to_save() {
        let mut p = plan(vec![("School", 5, 3_000_000), ("College", 18, 15_000_000)]);
        p.child_birth_year = 2000;
        let s = summarise(&p, today());
        assert!(s.nothing_left);
        assert_eq!(s.base.monthly_top_up_paise, 0);
        assert_eq!(s.high.monthly_top_up_paise, 0);
        assert_eq!(s.base.total_future_cost_paise, 0);
        assert!(s.notes.iter().any(|n| n.contains("nothing left")));
        assert!(earmark(&p, today()).payouts.is_empty());
    }

    #[test]
    fn zero_return_and_zero_inflation_are_plain_division() {
        let mut p = plan(vec![("College", 18, 21_600_000)]);
        p.inflation_bps = 0;
        p.return_bps = 0;
        let s = summarise(&p, today());
        assert_eq!(s.base.future_costs_paise, vec![21_600_000]);
        assert_eq!(s.base.monthly_top_up_paise, 100_000, "21,600,000 over 216 months");
    }

    #[test]
    fn a_corpus_reduces_the_top_up() {
        let mut p = plan(vec![("College", 18, 15_000_000)]);
        let without = summarise(&p, today()).base.monthly_top_up_paise;
        p.corpus_paise = 2_000_000;
        let with = summarise(&p, today()).base.monthly_top_up_paise;
        assert!(with < without);
        p.corpus_paise = MAX_AMOUNT_PAISE;
        assert_eq!(summarise(&p, today()).base.monthly_top_up_paise, 0);
    }

    #[test]
    fn the_summary_says_the_numbers_are_guesses_and_not_advice() {
        let s = summarise(&plan(vec![("College", 18, 1)]), today());
        assert!(s.notes[0].contains("guesses"));
        assert!(s.notes[1].contains("not investment advice"));
    }

    #[test]
    fn the_life_line_gets_the_base_case() {
        let p = plan(vec![("School", 5, 3_000_000), ("College", 18, 15_000_000)]);
        let e = earmark(&p, today());
        let base = summarise(&p, today()).base;
        assert_eq!(e.monthly_paise, base.monthly_top_up_paise);
        assert_eq!(e.return_bps, 1_200);
        assert_eq!(e.payouts, vec![(60, base.future_costs_paise[0]), (216, base.future_costs_paise[1])]);
    }

    #[test]
    fn validation_names_what_is_wrong() {
        let t = today();
        let ok = plan(vec![("College", 18, 1_000)]);
        assert!(validate(&ok, t).is_ok());
        assert!(validate(&Plan { child_birth_year: 1990, ..ok.clone() }, t).is_err());
        assert!(validate(&Plan { child_birth_year: 2060, ..ok.clone() }, t).is_err());
        assert!(validate(&Plan { stages: vec![], ..ok.clone() }, t).is_err());
        assert!(validate(&Plan { stages: vec![ok.stages[0].clone(); MAX_STAGES + 1], ..ok.clone() }, t).is_err());
        assert!(validate(&plan(vec![(" ", 18, 1)]), t).is_err());
        assert!(validate(&plan(vec![("x", 41, 1)]), t).is_err());
        assert!(validate(&plan(vec![("x", 18, -1)]), t).is_err());
        assert!(validate(&Plan { inflation_bps: 2_001, ..ok.clone() }, t).is_err());
        assert!(validate(&Plan { return_bps: -1, ..ok.clone() }, t).is_err());
        assert!(validate(&Plan { corpus_paise: -1, ..ok }, t).is_err());
    }

    #[test]
    fn the_starter_plan_is_valid() {
        assert!(validate(&Plan::starter(2026), today()).is_ok());
    }
}
