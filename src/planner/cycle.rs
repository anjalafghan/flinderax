//! Card billing-cycle maths. Pure functions, no DB access.
//!
//! A statement date is "passed" on the day itself: on the 23rd the bank app already shows the
//! new statement, so `next_statement_date` is strictly after `today`.

use time::{Date, Month};

pub fn days_in_month(year: i32, month: Month) -> u8 {
    time::util::days_in_month(month, year)
}

/// `day` clamped to the length of the month (31 -> 30/28).
pub fn clamped(year: i32, month: Month, day: u8) -> Date {
    let d = day.clamp(1, days_in_month(year, month));
    Date::from_calendar_date(year, month, d).expect("clamped date is valid")
}

pub fn add_months(year: i32, month: Month, delta: i32) -> (i32, Month) {
    let idx = year * 12 + (month as i32 - 1) + delta;
    let y = idx.div_euclid(12);
    let m = Month::try_from((idx.rem_euclid(12) + 1) as u8).expect("1..=12");
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
}
