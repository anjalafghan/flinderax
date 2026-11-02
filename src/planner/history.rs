//! Month-end net worth from balance history and card snapshots. Pure, no DB.

use serde::Serialize;
use time::{Date, Duration};

use super::cycle::add_months;

/// A balance an account (or a card's outstanding) had from `at` on.
#[derive(Debug, Clone)]
pub struct Point {
    pub id: String,
    pub at: Date,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MonthPoint {
    /// `YYYY-MM`
    pub month: String,
    /// Unlocked bank and cash balances.
    pub liquid_paise: i64,
    /// FDs, RDs and anything marked locked.
    pub locked_paise: i64,
    /// What all cards owe (outstanding, including EMI balances).
    pub card_debt_paise: i64,
    pub net_worth_paise: i64,
}

/// The value of each id as of `cutoff`: its last point on or before it. `points` must be sorted
/// by `at` ascending (later points win on the same day).
fn sum_as_of(points: &[Point], cutoff: Date, include: impl Fn(&str) -> bool) -> (i64, bool) {
    let mut latest: std::collections::HashMap<&str, i64> = std::collections::HashMap::new();
    for p in points.iter().take_while(|p| p.at <= cutoff) {
        latest.insert(&p.id, p.value);
    }
    let any = !latest.is_empty();
    (latest.iter().filter(|(id, _)| include(id)).map(|(_, v)| v).sum(), any)
}

/// The last `months` months up to and including `today`'s, each valued at its last day (today for
/// the current month). Months before any data exists are left out.
pub fn monthly_net_worth(
    balances: &[Point],
    is_locked: impl Fn(&str) -> bool,
    debts: &[Point],
    today: Date,
    months: u32,
) -> Vec<MonthPoint> {
    let mut out = Vec::new();
    for back in (0..months as i32).rev() {
        let (y, m) = add_months(today.year(), today.month(), -back);
        let (ny, nm) = add_months(y, m, 1);
        let month_end = Date::from_calendar_date(ny, nm, 1).expect("valid") - Duration::days(1);
        let cutoff = month_end.min(today);
        let (liquid, any_bal) = sum_as_of(balances, cutoff, |id| !is_locked(id));
        let (locked, _) = sum_as_of(balances, cutoff, &is_locked);
        let (debt, any_debt) = sum_as_of(debts, cutoff, |_| true);
        if !any_bal && !any_debt && out.is_empty() {
            continue;
        }
        let debt = debt.max(0);
        out.push(MonthPoint {
            month: format!("{y:04}-{:02}", m as u8),
            liquid_paise: liquid,
            locked_paise: locked,
            card_debt_paise: debt,
            net_worth_paise: liquid + locked - debt,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn pt(id: &str, at: Date, rupees: i64) -> Point {
        Point { id: id.into(), at, value: rupees * 100 }
    }

    #[test]
    fn values_each_month_at_its_last_point() {
        let bal = vec![
            pt("hdfc", date!(2026 - 09 - 05), 10_000),
            pt("fd", date!(2026 - 09 - 05), 50_000),
            pt("hdfc", date!(2026 - 10 - 01), 40_000),
            pt("hdfc", date!(2026 - 10 - 20), 30_000),
        ];
        let debt = vec![pt("card", date!(2026 - 10 - 10), 5_000)];
        let s = monthly_net_worth(&bal, |id| id == "fd", &debt, date!(2026 - 11 - 02), 4);
        let months: Vec<&str> = s.iter().map(|m| m.month.as_str()).collect();
        assert_eq!(months, ["2026-09", "2026-10", "2026-11"], "August has no data and is skipped");
        assert_eq!(s[0].net_worth_paise, 60_000_00);
        assert_eq!(s[1].liquid_paise, 30_000_00);
        assert_eq!(s[1].card_debt_paise, 5_000_00);
        assert_eq!(s[1].net_worth_paise, 75_000_00);
        assert_eq!(s[2], MonthPoint { month: "2026-11".into(), ..s[1].clone() });
    }

    #[test]
    fn current_month_is_cut_at_today() {
        let bal = vec![pt("hdfc", date!(2026 - 11 - 01), 1), pt("hdfc", date!(2026 - 11 - 20), 2)];
        let s = monthly_net_worth(&bal, |_| false, &[], date!(2026 - 11 - 02), 1);
        assert_eq!(s[0].liquid_paise, 100);
    }

    #[test]
    fn empty_history_is_empty() {
        assert!(monthly_net_worth(&[], |_| false, &[], date!(2026 - 11 - 02), 12).is_empty());
    }
}
