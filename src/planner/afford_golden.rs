//! Characterization tests for [`afford`]: the whole Verdict, pinned across scenarios that reach each
//! branch. A refactor must leave `afford_golden.json` byte-for-byte alone. Regenerate on purpose with
//! `UPDATE_GOLDEN=1 cargo test afford_golden`.

use super::afford::*;
use time::{macros::date, Date};

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/planner/afford_golden.json");
const TODAY: Date = date!(2026 - 11 - 08);

fn money(p: i64) -> String {
    format!("₹{}", p / 100)
}

/// A projection that runs short for any spend before `until`, by `by` paise.
fn short_until(until: Option<Date>, by: i64) -> impl Fn(Date) -> Option<i64> {
    move |d| until.filter(|u| d < *u).map(|_| by)
}

fn snapshot() -> String {
    let now = Pay::Now;
    let emi = |months, rate_bps, fee_paise| Pay::Emi { months, rate_bps, fee_paise };
    let mut out = Vec::new();
    let mut run = |name: &str, amount: i64, pay: Pay, safe: i64, rest: i64, horizon: i64, short: &dyn Fn(Date) -> Option<i64>| {
        let v = afford(amount, pay, TODAY, safe, rest, horizon, short, money);
        out.push(serde_json::json!({ "name": name, "verdict": v }));
    };
    run("now_fits", 2_000_000, now, 5_000_000, 0, 6, &short_until(None, 0));
    run("now_exactly_safe", 5_000_000, now, 5_000_000, 0, 6, &short_until(None, 0));
    run("now_beyond_safe_but_nothing_runs_short", 8_000_000, now, 5_000_000, 0, 6, &short_until(None, 0));
    run("now_wait_two_months", 8_000_000, now, 5_000_000, 0, 6, &short_until(Some(date!(2027 - 01 - 01)), 1_200_000));
    run("now_wait_next_month", 8_000_000, now, 5_000_000, 0, 6, &short_until(Some(date!(2026 - 12 - 01)), 300_000));
    run("now_never_recovers_within_horizon", 8_000_000, now, 5_000_000, 0, 4, &short_until(Some(date!(2030 - 01 - 01)), 700_000));
    run("now_horizon_of_one_cannot_wait", 8_000_000, now, 5_000_000, 0, 1, &short_until(Some(date!(2027 - 01 - 01)), 700_000));
    run("now_wait_crosses_the_year", 8_000_000, now, 5_000_000, 0, 12, &short_until(Some(date!(2027 - 03 - 01)), 700_000));
    run("now_clear_month_exactly_at_the_horizon_is_out_of_reach", 8_000_000, now, 5_000_000, 0, 3, &short_until(Some(date!(2027 - 02 - 01)), 700_000));
    run("now_clear_month_one_inside_the_horizon", 8_000_000, now, 5_000_000, 0, 4, &short_until(Some(date!(2027 - 02 - 01)), 700_000));
    run("now_negative_safe_to_spend", 100_000, now, -500_000, 0, 6, &short_until(None, 0));
    run("emi_fits_with_interest", 10_000_000, emi(12, 1_200, 0), 0, 1_000_000, 6, &short_until(None, 0));
    run("emi_no_cost_with_no_fee", 12_000_000, emi(12, 0, 0), 0, 1_000_000, 6, &short_until(None, 0));
    run("emi_no_cost_with_fee_and_gst", 12_000_000, emi(6, 0, 19_900), 0, 5_000_000, 6, &short_until(None, 0));
    run("emi_too_big_for_savings", 10_000_000, emi(6, 1_500, 0), 0, 500_000, 6, &short_until(None, 0));
    run("emi_exactly_equals_savings", 1_200_000, emi(12, 0, 0), 0, 100_000, 6, &short_until(None, 0));
    run("emi_with_negative_savings", 1_200_000, emi(12, 1_000, 0), 300_000, -400_000, 6, &short_until(None, 0));
    run("emi_zero_months", 500_000, emi(0, 1_200, 10_000), 0, 100_000, 6, &short_until(None, 0));
    run("emi_negative_rate_treated_as_zero", 700_000, emi(7, -300, 0), 0, 200_000, 6, &short_until(None, 0));
    serde_json::to_string_pretty(&out).unwrap() + "\n"
}

#[test]
fn afford_matches_the_pinned_output() {
    let now = snapshot();
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(FILE, &now).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
    assert!(now == pinned, "afford output changed; if on purpose, rerun with UPDATE_GOLDEN=1");
}
