//! Goals a couple agrees on and both pay into (a wedding, a child). Pure: the handler gathers the
//! rows. A person's share is just what they have put in; nothing here splits a bill.

use serde::Serialize;
use time::Date;

use super::goals::months_until;

/// What to put aside each month, from this month to the deadline, to reach `target`. `None` when the
/// goal has no date; 0 once it is met. A deadline that has passed asks for all of the rest at once.
pub fn monthly_needed(target: i64, saved: i64, today: Date, by: Option<Date>) -> Option<i64> {
    let by = by?;
    let remaining = (target - saved).max(0);
    if remaining == 0 {
        return Some(0);
    }
    let months = if by < today { 1 } else { months_until(today, by) };
    Some((remaining - 1) / months + 1)
}

/// How much of the target is covered, 0 to 100.
pub fn percent(target: i64, saved: i64) -> i64 {
    if target <= 0 {
        return 0;
    }
    (saved.max(0) * 100 / target).clamp(0, 100)
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Share {
    pub name: String,
    pub paise: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    const TODAY: Date = date!(2026 - 11 - 15);

    #[test]
    fn monthly_amount_rounds_up_and_counts_this_month() {
        // Nov 2026 to Dec 2026 is two months: ₹1,001 left needs ₹501 a month, never less.
        assert_eq!(monthly_needed(100_100, 0, TODAY, Some(date!(2026 - 12 - 31))), Some(50_050));
        assert_eq!(monthly_needed(100_101, 0, TODAY, Some(date!(2026 - 12 - 31))), Some(50_051));
        assert_eq!(monthly_needed(100_000, 40_000, TODAY, Some(date!(2026 - 11 - 30))), Some(60_000), "due this month: all of it now");
    }

    #[test]
    fn no_date_means_no_monthly_figure_and_a_met_goal_needs_nothing() {
        assert_eq!(monthly_needed(100_000, 0, TODAY, None), None);
        assert_eq!(monthly_needed(100_000, 100_000, TODAY, Some(date!(2027 - 6 - 1))), Some(0));
        assert_eq!(monthly_needed(100_000, 250_000, TODAY, Some(date!(2027 - 6 - 1))), Some(0), "over-funded is fine");
    }

    #[test]
    fn a_passed_deadline_asks_for_the_rest_now() {
        assert_eq!(monthly_needed(100_000, 30_000, TODAY, Some(date!(2026 - 1 - 1))), Some(70_000));
    }

    #[test]
    fn percent_stays_between_zero_and_a_hundred() {
        assert_eq!(percent(200, 50), 25);
        assert_eq!(percent(200, 500), 100);
        assert_eq!(percent(200, -5), 0);
        assert_eq!(percent(0, 5), 0);
    }
}
