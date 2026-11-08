//! Card billing-cycle maths. Pure functions, no DB access.
//!
//! A statement date is "passed" on the day itself: on the 23rd the bank app already shows the
//! new statement, so `next_statement_date` is strictly after `today`.

use time::{Date, Month};

pub fn days_in_month(year: i32, month: Month) -> u8 {
    time::util::days_in_month(month, year)
}

/// The years the `time` crate can represent. Anything outside is clamped into this range, so the
/// helpers below can never panic, whatever a caller passes in.
const MIN_YEAR: i32 = -9999;
const MAX_YEAR: i32 = 9999;

/// `day` clamped to the length of the month (31 -> 30/28). A year beyond `time`'s range is pulled
/// back to the nearest representable year rather than panicking.
pub fn clamped(year: i32, month: Month, day: u8) -> Date {
    let year = year.clamp(MIN_YEAR, MAX_YEAR);
    let d = day.clamp(1, days_in_month(year, month));
    // The year is inside the representable range and the day is inside the month, so this is valid.
    Date::from_calendar_date(year, month, d).expect("year and day are clamped into range")
}

/// `month` moved `delta` months (either way) from `year`, with the year clamped to `time`'s range.
pub fn add_months(year: i32, month: Month, delta: i32) -> (i32, Month) {
    // i64 so that extreme inputs cannot overflow the intermediate month index.
    let idx = i64::from(year) * 12 + (month as i64 - 1) + i64::from(delta);
    let y = idx.div_euclid(12).clamp(i64::from(MIN_YEAR), i64::from(MAX_YEAR)) as i32;
    let m = Month::try_from((idx.rem_euclid(12) + 1) as u8).expect("rem_euclid(12) + 1 is 1..=12");
    (y, m)
}

/// The `day`-of-month date in the month `delta` months away from `anchor`'s month.
pub fn day_in_month_offset(anchor: Date, delta: i32, day: u8) -> Date {
    let (y, m) = add_months(anchor.year(), anchor.month(), delta);
    clamped(y, m, day)
}

/// First statement date strictly after `today`.
pub fn next_statement_date(today: Date, statement_day: u8) -> Date {
    let this_month = clamped(today.year(), today.month(), statement_day);
    if this_month > today {
        this_month
    } else {
        day_in_month_offset(today, 1, statement_day)
    }
}

/// Latest statement date on or before `today`.
pub fn last_statement_date(today: Date, statement_day: u8) -> Date {
    let this_month = clamped(today.year(), today.month(), statement_day);
    if this_month <= today {
        this_month
    } else {
        day_in_month_offset(today, -1, statement_day)
    }
}

/// The first `due_day` strictly after the statement date.
pub fn due_date_for(statement_date: Date, due_day: u8) -> Date {
    let same_month = clamped(statement_date.year(), statement_date.month(), due_day);
    if same_month > statement_date {
        same_month
    } else {
        day_in_month_offset(statement_date, 1, due_day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn statement_day_already_passed_rolls_to_next_month() {
        assert_eq!(next_statement_date(date!(2026 - 10 - 25), 23), date!(2026 - 11 - 23));
        assert_eq!(next_statement_date(date!(2026 - 10 - 23), 23), date!(2026 - 11 - 23));
        assert_eq!(next_statement_date(date!(2026 - 10 - 05), 23), date!(2026 - 10 - 23));
    }

    #[test]
    fn year_rollover() {
        assert_eq!(next_statement_date(date!(2026 - 12 - 30), 23), date!(2027 - 01 - 23));
        assert_eq!(last_statement_date(date!(2027 - 01 - 05), 23), date!(2026 - 12 - 23));
    }

    #[test]
    fn day_31_is_clamped_in_short_months() {
        assert_eq!(next_statement_date(date!(2026 - 02 - 10), 31), date!(2026 - 02 - 28));
        assert_eq!(next_statement_date(date!(2028 - 02 - 10), 31), date!(2028 - 02 - 29));
        assert_eq!(next_statement_date(date!(2026 - 04 - 30), 31), date!(2026 - 05 - 31));
    }

    #[test]
    fn due_day_31_in_february() {
        // statement 20 Jan, due day 31 -> 31 Jan; statement 5 Feb, due day 31 -> 28 Feb
        assert_eq!(due_date_for(date!(2026 - 01 - 20), 31), date!(2026 - 01 - 31));
        assert_eq!(due_date_for(date!(2026 - 02 - 05), 31), date!(2026 - 02 - 28));
        // statement on the 28th (clamped), due day 31 -> same clamped day is not "after"
        assert_eq!(due_date_for(date!(2026 - 02 - 28), 31), date!(2026 - 03 - 31));
    }

    #[test]
    fn due_date_after_statement_for_7420() {
        // statement 23rd, due 13th -> due falls in the following month
        assert_eq!(due_date_for(date!(2026 - 10 - 23), 13), date!(2026 - 11 - 13));
        assert_eq!(due_date_for(date!(2026 - 09 - 23), 13), date!(2026 - 10 - 13));
    }

    #[test]
    fn due_day_equal_to_statement_day_goes_next_month() {
        assert_eq!(due_date_for(date!(2026 - 10 - 15), 15), date!(2026 - 11 - 15));
    }

    #[test]
    fn huge_offsets_never_panic() {
        // Far past year 9999 and before year -9999: clamped into range, no panic.
        assert_eq!(day_in_month_offset(date!(2026 - 10 - 05), 1_000_000_000, 15).year(), 9999);
        assert_eq!(day_in_month_offset(date!(2026 - 10 - 05), -1_000_000_000, 15).year(), -9999);
        assert_eq!(add_months(i32::MAX, Month::December, i32::MAX).0, 9999);
        assert_eq!(clamped(i32::MAX, Month::February, 31).year(), 9999);
        assert_eq!(clamped(i32::MIN, Month::December, 1).year(), -9999);
    }
}
