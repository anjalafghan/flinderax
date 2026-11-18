//! Activity: the points an action earns and the daily cap on them. Pure: counts in, points out.
//! What counts is putting data in (a statement, a logged transaction, a finished check-in), never
//! how old the data is, and a re-upload of rows already held earns nothing.

use std::collections::BTreeSet;

use time::Date;

use super::streaks::{day_streak, Streak};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Statement,
    Txn,
    Checkin,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Statement => "statement",
            Kind::Txn => "txn",
            Kind::Checkin => "checkin",
        }
    }

    /// Most points this kind can earn in one day, so repeats cannot be farmed.
    pub fn daily_cap(self) -> i64 {
        match self {
            Kind::Statement => 15,
            Kind::Txn => 3,
            Kind::Checkin => 10,
        }
    }
}

/// Points for one action. `new_rows` is how many rows the action really added (after dedupe):
/// nothing new earns nothing. A statement is worth 5 plus 1 per 10 new rows.
pub fn points(kind: Kind, new_rows: usize) -> i64 {
    if new_rows == 0 {
        return 0;
    }
    match kind {
        Kind::Statement => 5 + (new_rows / 10) as i64,
        Kind::Txn => 1,
        Kind::Checkin => 10,
    }
}

/// What may be granted now: `wanted` trimmed to what is left of the kind's cap for the day.
pub fn grant(kind: Kind, wanted: i64, already_today: i64) -> i64 {
    wanted.clamp(0, i64::MAX).min(kind.daily_cap().saturating_sub(already_today).max(0))
}

/// The run of days with at least one action. Same grace as the other streaks: today not yet done
/// does not break a run that reached yesterday.
pub fn active_streak(days: &BTreeSet<Date>, today: Date) -> Streak {
    day_streak(days, today)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn d(day: u8) -> Date {
        Date::from_calendar_date(2026, Month::November, day).unwrap()
    }

    #[test]
    fn nothing_new_earns_nothing() {
        for k in [Kind::Statement, Kind::Txn, Kind::Checkin] {
            assert_eq!(points(k, 0), 0);
        }
    }

    #[test]
    fn a_statement_scales_with_new_rows() {
        assert_eq!(points(Kind::Statement, 1), 5);
        assert_eq!(points(Kind::Statement, 9), 5);
        assert_eq!(points(Kind::Statement, 10), 6);
        assert_eq!(points(Kind::Statement, 500), 55);
    }

    #[test]
    fn the_daily_cap_trims_and_never_goes_negative() {
        assert_eq!(grant(Kind::Statement, 55, 0), 15);
        assert_eq!(grant(Kind::Statement, 5, 12), 3);
        assert_eq!(grant(Kind::Txn, 1, 3), 0);
        assert_eq!(grant(Kind::Txn, 1, 99), 0);
        assert_eq!(grant(Kind::Txn, -4, 0), 0);
    }

    #[test]
    fn streak_survives_a_day_not_yet_done() {
        let days: BTreeSet<Date> = [d(13), d(14), d(15)].into();
        assert_eq!(active_streak(&days, d(16)).current, 3);
        assert_eq!(active_streak(&days, d(18)).current, 0);
        assert_eq!(active_streak(&days, d(18)).best, 3);
    }
}
