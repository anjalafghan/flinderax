//! Characterization tests for [`health`]: the whole Health (score, figures, ranked actions) pinned across
//! scenarios that reach each branch. A refactor must leave `insights_golden.json` byte-for-byte alone.
//! Regenerate on purpose with `UPDATE_GOLDEN=1 cargo test insights_golden`.

use super::insights::*;
use time::{macros::date, Date};

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/planner/insights_golden.json");
const TODAY: Date = date!(2026 - 11 - 10);

fn txn(id: &str, on: Date, debit: bool, rupees: i64, merchant: Option<&str>, category: &str, raw: Option<&str>) -> Txn {
    Txn { txn_id: id.into(), on, debit, amount: rupees * 100, merchant: merchant.map(Into::into), category: category.into(), raw_text: raw.map(Into::into) }
}

/// One salary credit and one spend debit on the 1st/2nd of each listed month (Aug..Oct 2026 are the last 3 completed).
fn months(entries: &[(i32, u8, i64, i64)]) -> Vec<Txn> {
    let mut out = Vec::new();
    for (year, month, income, spend) in entries {
        let m = time::Month::try_from(*month).unwrap();
        out.push(txn(&format!("c{year}{month}"), Date::from_calendar_date(*year, m, 1).unwrap(), false, *income, Some("Employer"), "income", None));
        out.push(txn(&format!("d{year}{month}"), Date::from_calendar_date(*year, m, 2).unwrap(), true, *spend, Some("Rent and food"), "other", None));
    }
    out
}

fn fees(extra: i64) -> Vec<Txn> {
    vec![
        txn("f1", date!(2026 - 06 - 05), true, 1_500 + extra, Some("HDFC"), "fees", Some("Annual fee for card")),
        txn("f2", date!(2026 - 08 - 05), true, 900, Some("HDFC"), "fees", Some("Late payment fee")),
        txn("f3", date!(2026 - 09 - 05), true, 250, Some("Kotak"), "fees", Some("SMS charge")),
    ]
}

fn refund(id: &str, status: RefundStatus, rupees: i64) -> Refund {
    Refund {
        txn_id: id.into(),
        occurred_on: "2026-10-20".into(),
        amount_paise: rupees * 100,
        merchant: Some("Shop".into()),
        rail: Rail::P2m,
        due_by: "2026-10-25".into(),
        days_overdue: if status == RefundStatus::Overdue { 5 } else { 0 },
        compensation_paise: 0,
        status,
    }
}

fn snapshot() -> String {
    let steady = months(&[(2026, 5, 90_000, 50_000), (2026, 6, 90_000, 52_000), (2026, 7, 90_000, 48_000), (2026, 8, 90_000, 50_000), (2026, 9, 90_000, 51_000), (2026, 10, 90_000, 49_000)]);
    let thin = months(&[(2026, 8, 60_000, 58_000), (2026, 9, 60_000, 59_000), (2026, 10, 60_000, 61_000)]);
    let swingy = months(&[(2026, 5, 20_000, 30_000), (2026, 6, 120_000, 40_000), (2026, 7, 30_000, 35_000), (2026, 8, 150_000, 45_000), (2026, 9, 25_000, 40_000), (2026, 10, 110_000, 42_000)]);
    let mut everything_wrong = thin.clone();
    everything_wrong.extend(swingy.iter().filter(|t| t.on < date!(2026 - 08 - 01)).cloned());
    everything_wrong.extend(fees(200_000));
    let mut fee_heavy = steady.clone();
    fee_heavy.extend(fees(250_000));
    let mut no_income = months(&[(2026, 8, 0, 40_000), (2026, 9, 0, 41_000), (2026, 10, 0, 42_000)]);
    no_income.retain(|t| t.debit);
    let transfers_only = vec![txn("t1", date!(2026 - 10 - 03), false, 100_000, None, "transfer", None), txn("t2", date!(2026 - 10 - 04), false, 50_000, None, "refund", None)];

    let waiting = [refund("r1", RefundStatus::Waiting, 450)];
    let one_overdue = [refund("r1", RefundStatus::Overdue, 450), refund("r2", RefundStatus::Waiting, 100)];
    let two_overdue = [refund("r1", RefundStatus::Overdue, 450), refund("r2", RefundStatus::Overdue, 100)];
    let near_target = months(&[(2026, 8, 100_000, 81_000), (2026, 9, 100_000, 81_000), (2026, 10, 100_000, 81_000)]);
    let at_target = months(&[(2026, 8, 100_000, 80_000), (2026, 9, 100_000, 80_000), (2026, 10, 100_000, 80_000)]);
    let huge = months(&[(2026, 8, 10_000_000_000, 1), (2026, 9, 10_000_000_000, 1), (2026, 10, 10_000_000_000, 1)]);
    let ten_percent = months(&[(2026, 8, 100_000, 90_000), (2026, 9, 100_000, 90_000), (2026, 10, 100_000, 90_000)]);
    let cv_at_alert = months(&[(2026, 9, 30_000, 20_000), (2026, 10, 50_000, 20_000)]);
    let cases: Vec<(&str, &[Txn], i64, &[Refund])> = vec![
        ("no_data", &[], 0, &[]),
        ("healthy", &steady, 30_000_000, &[]),
        ("healthy_but_little_liquid", &steady, 5_000_000, &[]),
        ("liquid_negative", &steady, -9_000_000, &[]),
        ("thin_savings", &thin, 6_000_000, &[]),
        ("savings_rate_just_below_target", &near_target, 30_000_000, &[]),
        ("savings_rate_exactly_target", &at_target, 30_000_000, &[]),
        ("emergency_just_under_one_month", &thin, 5_000_000, &[]),
        ("savings_rate_exactly_ten_percent", &ten_percent, 30_000_000, &[]),
        ("income_variation_exactly_at_the_alert", &cv_at_alert, 30_000_000, &[]),
        ("swingy_income", &swingy, 40_000_000, &[]),
        ("fee_heavy", &fee_heavy, 30_000_000, &[]),
        ("everything_wrong_keeps_four_actions", &everything_wrong, 100_000, &one_overdue),
        ("spending_but_no_income", &no_income, 1_000_000, &[]),
        ("transfers_are_not_income", &transfers_only, 1_000_000, &[]),
        ("refund_waiting", &steady, 30_000_000, &waiting),
        ("refund_one_overdue", &steady, 30_000_000, &one_overdue),
        ("refund_two_overdue", &steady, 30_000_000, &two_overdue),
        ("very_large_but_valid_amounts", &huge, 1_000_000_000_000, &[]),
    ];
    let out: Vec<_> = cases.into_iter().map(|(name, txns, liquid, refunds)| serde_json::json!({ "name": name, "health": health(txns, liquid, refunds, TODAY) })).collect();
    serde_json::to_string_pretty(&out).unwrap() + "\n"
}

#[test]
fn health_matches_the_pinned_output() {
    let now = snapshot();
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(FILE, &now).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
    assert!(now == pinned, "health output changed; if on purpose, rerun with UPDATE_GOLDEN=1");
}
