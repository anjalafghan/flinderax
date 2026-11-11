//! Streaks: runs of good days or months, for the encouraging card on Today. Pure: the handler loads
//! the days and months, this counts the runs.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use time::{Date, Duration};

use super::month_index;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Streak {
    /// The run that is still alive.
    pub current: u32,
    /// The longest run in the data.
    pub best: u32,
}

/// Runs of consecutive days. The current run may end today or yesterday: a day that has not
/// finished yet must not break a streak.
pub fn day_streak(days: &BTreeSet<Date>, today: Date) -> Streak {
    let days: Vec<Date> = days.iter().copied().filter(|d| *d <= today).collect();
    let mut best = 0u32;
    let mut run = 0u32;
    let mut prev: Option<Date> = None;
    for d in &days {
        run = match prev {
            Some(p) if *d - p == Duration::days(1) => run + 1,
            _ => 1,
        };
        best = best.max(run);
        prev = Some(*d);
    }
    let set: BTreeSet<Date> = days.into_iter().collect();
    let mut cursor = if set.contains(&today) { Some(today) } else { Some(today - Duration::days(1)).filter(|y| set.contains(y)) };
    let mut current = 0u32;
    while let Some(d) = cursor.filter(|d| set.contains(d)) {
        current += 1;
        cursor = d.checked_sub(Duration::days(1));
    }
    Streak { current, best }
}

/// Runs of consecutive months (by `month_index`). The current run ends at `anchor`, the latest
/// month that counts (last month for things judged at month end; this month when it already has one).
pub fn month_streak(good: &BTreeSet<i32>, anchor: i32) -> Streak {
    let mut best = 0u32;
    let mut run = 0u32;
    let mut prev: Option<i32> = None;
    for m in good.iter().filter(|m| **m <= anchor) {
        run = match prev {
            Some(p) if *m - p == 1 => run + 1,
            _ => 1,
        };
        best = best.max(run);
        prev = Some(*m);
    }
    let mut current = 0u32;
    let mut m = anchor;
    while good.contains(&m) {
        current += 1;
        m -= 1;
    }
    Streak { current, best }
}

/// Months in which you earned something and spent no more than that. `rows` are
/// `(month index, income, spent)` totals in paise.
pub fn saved_months(rows: &BTreeMap<i32, (i64, i64)>) -> BTreeSet<i32> {
    rows.iter().filter(|(_, (income, spent))| *income > 0 && spent <= income).map(|(m, _)| *m).collect()
}

/// For a date-based streak: this month counts if it already has one, otherwise the run is judged
/// up to last month (so a month that is not over yet does not break it).
pub fn dates_anchor(good: &BTreeSet<i32>, today: Date) -> i32 {
    let this = month_index(today);
    if good.contains(&this) { this } else { this - 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn days(list: &[Date]) -> BTreeSet<Date> {
        list.iter().copied().collect()
    }

    #[test]
    fn a_run_ending_today_counts_and_the_best_run_is_remembered() {
        let d = days(&[date!(2026 - 11 - 01), date!(2026 - 11 - 02), date!(2026 - 11 - 03), date!(2026 - 11 - 04), date!(2026 - 11 - 09), date!(2026 - 11 - 10), date!(2026 - 11 - 11)]);
        assert_eq!(day_streak(&d, date!(2026 - 11 - 11)), Streak { current: 3, best: 4 });
    }

    #[test]
    fn today_not_over_yet_does_not_break_a_streak_but_two_missed_days_do() {
        let d = days(&[date!(2026 - 11 - 08), date!(2026 - 11 - 09), date!(2026 - 11 - 10)]);
        assert_eq!(day_streak(&d, date!(2026 - 11 - 11)).current, 3);
        assert_eq!(day_streak(&d, date!(2026 - 11 - 12)).current, 0);
    }

    #[test]
    fn future_days_and_an_empty_set_are_ignored() {
        assert_eq!(day_streak(&BTreeSet::new(), date!(2026 - 11 - 11)), Streak::default());
        let d = days(&[date!(2026 - 11 - 11), date!(2026 - 11 - 12)]);
        assert_eq!(day_streak(&d, date!(2026 - 11 - 11)), Streak { current: 1, best: 1 });
    }

    #[test]
    fn months_in_a_row_and_a_gap() {
        let good: BTreeSet<i32> = [10, 11, 12, 14, 15, 16, 17].into_iter().collect();
        assert_eq!(month_streak(&good, 17), Streak { current: 4, best: 4 });
        assert_eq!(month_streak(&good, 13), Streak { current: 0, best: 3 });
        assert_eq!(month_streak(&good, 12), Streak { current: 3, best: 3 });
    }

    #[test]
    fn saved_months_need_income_and_spending_within_it() {
        let rows: BTreeMap<i32, (i64, i64)> = [(1, (100, 80)), (2, (100, 100)), (3, (100, 101)), (4, (0, 0)), (5, (50, 10))].into_iter().collect();
        assert_eq!(saved_months(&rows).into_iter().collect::<Vec<_>>(), [1, 2, 5]);
    }

    #[test]
    fn a_date_streak_is_judged_up_to_last_month_until_this_month_has_one() {
        let today = date!(2026 - 11 - 11);
        let this = month_index(today);
        let without: BTreeSet<i32> = [this - 2, this - 1].into_iter().collect();
        assert_eq!(dates_anchor(&without, today), this - 1);
        assert_eq!(month_streak(&without, dates_anchor(&without, today)).current, 2);
        let with: BTreeSet<i32> = [this - 1, this].into_iter().collect();
        assert_eq!(dates_anchor(&with, today), this);
        assert_eq!(month_streak(&with, dates_anchor(&with, today)).current, 2);
    }
}
