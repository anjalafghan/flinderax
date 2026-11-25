//! Date text helpers shared by the planner and the handlers. Pure.
//!
//! Two reading rules exist and both are kept on purpose:
//! * [`parse_ymd`] is strict: the whole string must be `YYYY-MM-DD`.
//! * [`parse_ymd_prefix`] reads the first ten characters, so a stored timestamp such as
//!   `2026-03-04 10:11:12` or `2026-03-04T10:11:12Z` yields its day.

use time::{macros::format_description, Date};

/// Strict `YYYY-MM-DD`. `None` for anything else, including a trailing timestamp.
pub fn parse_ymd(s: &str) -> Option<Date> {
    Date::parse(s, format_description!("[year]-[month]-[day]")).ok()
}

/// The day of a stored date or timestamp: the first ten characters read as `YYYY-MM-DD`.
pub fn parse_ymd_prefix(s: &str) -> Option<Date> {
    parse_ymd(s.get(..10)?)
}

/// `YYYY-MM-DD`.
pub fn fmt_ymd(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
}

/// `YYYY-MM` month key (`2026-03`), as stored in month columns.
pub fn month_key(year: i32, month: u8) -> String {
    format!("{:04}-{:02}", year, month)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    #[test]
    fn strict_parse_accepts_only_a_bare_date() {
        assert_eq!(parse_ymd("2026-03-04"), Some(Date::from_calendar_date(2026, Month::March, 4).unwrap()));
        assert_eq!(parse_ymd("2026-03-04 10:11:12"), None);
        assert_eq!(parse_ymd(" 2026-03-04"), None);
        assert_eq!(parse_ymd("2026-02-30"), None);
        assert_eq!(parse_ymd("2026-3-4"), None);
        assert_eq!(parse_ymd(""), None);
    }

    #[test]
    fn prefix_parse_reads_timestamps_and_survives_short_or_multibyte_input() {
        let want = Date::from_calendar_date(2026, Month::March, 4).unwrap();
        assert_eq!(parse_ymd_prefix("2026-03-04"), Some(want));
        assert_eq!(parse_ymd_prefix("2026-03-04 10:11:12"), Some(want));
        assert_eq!(parse_ymd_prefix("2026-03-04T10:11:12Z"), Some(want));
        assert_eq!(parse_ymd_prefix("2026-03-0"), None);
        assert_eq!(parse_ymd_prefix(""), None);
        // Byte 10 falls inside a multi-byte character: no panic, just None.
        assert_eq!(parse_ymd_prefix("2026-03-0₹"), None);
        assert_eq!(parse_ymd_prefix("2026-13-04"), None);
    }

    #[test]
    fn formats_round_trip() {
        let d = Date::from_calendar_date(2026, Month::March, 4).unwrap();
        assert_eq!(fmt_ymd(d), "2026-03-04");
        assert_eq!(parse_ymd(&fmt_ymd(d)), Some(d));
        assert_eq!(month_key(2026, 3), "2026-03");
        assert_eq!(month_key(2026, 12), "2026-12");
        assert_eq!(month_key(5, 1), "0005-01");
    }
}
