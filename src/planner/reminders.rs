//! The reminder calendar feed: one all-day event per habit that has a due date, rendered as an
//! iCalendar file. Pure: the handler loads the habits and the clock, this module only shapes them.

use time::{format_description::FormatItem, macros::format_description, Date, OffsetDateTime};

use crate::joy::ics::{escape, fold};
use crate::planner::habits::Habit;
use crate::planner::people::{day_events, DateIn, SeasonIn};

const DATE: &[FormatItem<'static>] = format_description!("[year][month][day]");
const STAMP: &[FormatItem<'static>] = format_description!("[year][month][day]T[hour][minute][second]Z");
const ISO: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

/// The hour of the morning the reminder fires, on its own day.
const ALARM_HOUR: i64 = 9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reminder {
    pub uid: String,
    pub on: Date,
    pub summary: String,
    pub description: String,
}

/// One reminder per habit with a due date. Past due dates move to today so they still show up.
/// A habit with an action gets the link to it under its why, using `public_url` as the base.
pub fn from_habits(habits: &[Habit], today: Date, public_url: &str) -> Vec<Reminder> {
    habits
        .iter()
        .filter_map(|h| {
            let due = Date::parse(h.due_on.as_deref()?, ISO).ok()?;
            // One-off to-dos already past would land on "today" at every refresh and nag daily; they
            // stay in the app. Recurring ones (statements, SIPs) move to today until done.
            if h.cadence == crate::planner::habits::Cadence::Once && due < today {
                return None;
            }
            let description = match &h.action {
                Some(link) => format!("{}\n{}{}", h.why, public_url, link.to),
                None => h.why.clone(),
            };
            Some(Reminder {
                uid: format!("joy-habit-{}@joy", h.key.replace(':', "-")),
                on: due.max(today),
                summary: h.title.clone(),
                description,
            })
        })
        .collect()
}

/// How far ahead person dates and gift seasons appear in the calendar.
const DATES_AHEAD_DAYS: i64 = 90;

/// All-day events for the person dates and festival seasons in the next 90 days. Titles leave names out
/// unless `feed_names` is on.
pub fn from_dates(dates: &[DateIn], seasons: &[SeasonIn], today: Date, feed_names: bool, public_url: &str) -> Vec<Reminder> {
    day_events(dates, seasons, today, DATES_AHEAD_DAYS, feed_names)
        .into_iter()
        .map(|e| Reminder {
            uid: e.uid,
            on: e.on,
            summary: e.summary,
            description: format!("{}\n{public_url}/people", e.description),
        })
        .collect()
}

/// For a feed that keeps details out (the default): calendar services see that something is due
/// and a link back to joy, never names, account or card names, amounts or health details.
pub fn hide_names(habits: &mut [Habit]) {
    use crate::planner::habits::Area;
    for h in habits {
        h.title = if h.key.starts_with("catch_up:") {
            "Catch up with someone".into()
        } else if h.key.starts_with("gift:") {
            "Gift for an upcoming date".into()
        } else {
            match h.area {
                Area::Food | Area::Health => "A health task in joy".into(),
                Area::People => "Something for the people you care about".into(),
                Area::Work => "A work task in joy".into(),
                _ => "A money task in joy".into(),
            }
        };
        h.why = "Open joy for the details.".into();
    }
}

/// The whole calendar file. `now_unix` is the DTSTAMP of every event.
pub fn render(reminders: &[Reminder], now_unix: i64) -> String {
    let stamp = utc_stamp(now_unix);
    let mut lines: Vec<String> = vec![
        "BEGIN:VCALENDAR".into(),
        "VERSION:2.0".into(),
        "PRODID:-//joy//reminders//EN".into(),
        "X-WR-CALNAME:joy reminders".into(),
        "REFRESH-INTERVAL;VALUE=DURATION:PT12H".into(),
        "X-PUBLISHED-TTL:PT12H".into(),
    ];
    for r in reminders {
        let next = r.on.next_day().unwrap_or(r.on);
        lines.extend([
            "BEGIN:VEVENT".into(),
            format!("UID:{}", escape(&r.uid)),
            format!("DTSTAMP:{stamp}"),
            format!("DTSTART;VALUE=DATE:{}", day(r.on)),
            format!("DTEND;VALUE=DATE:{}", day(next)),
            format!("SUMMARY:{}", escape(&r.summary)),
            format!("DESCRIPTION:{}", escape(&r.description)),
            "BEGIN:VALARM".into(),
            "ACTION:DISPLAY".into(),
            format!("DESCRIPTION:{}", escape(&r.summary)),
            format!("TRIGGER;RELATED=START:PT{ALARM_HOUR}H"),
            "END:VALARM".into(),
            "END:VEVENT".into(),
        ]);
    }
    lines.push("END:VCALENDAR".into());
    lines.iter().map(|l| fold(l)).collect::<Vec<_>>().join("\r\n") + "\r\n"
}

/// The token from a feed file name `<64 lowercase hex>.ics`, or None when the name has any other shape.
pub fn token_from_file(file: &str) -> Option<&str> {
    let token = file.strip_suffix(".ics")?;
    is_token(token).then_some(token)
}

/// Feed tokens are 32 random bytes written as 64 lowercase hex characters.
pub fn is_token(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn day(d: Date) -> String {
    d.format(DATE).unwrap_or_default()
}

fn utc_stamp(unix: i64) -> String {
    OffsetDateTime::from_unix_timestamp(unix)
        .ok()
        .and_then(|t| t.format(STAMP).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::habits::{Area, Cadence, Impact, Link, Status};
    use time::macros::date;

    fn habit(key: &str, title: &str, due_on: Option<&str>, action: Option<&str>) -> Habit {
        Habit {
            key: key.into(),
            area: Area::Data,
            title: title.into(),
            why: "Asha's statements are a month old.".into(),
            action: action.map(|to| Link { label: "Open".into(), to: to.into() }),
            cadence: Cadence::Monthly,
            due_on: due_on.map(str::to_string),
            status: Status::Ok,
            impact: Impact::High,
        }
    }

    fn one(on: Date, summary: &str, description: &str) -> Reminder {
        Reminder { uid: "joy-x@joy".into(), on, summary: summary.into(), description: description.into() }
    }

    #[test]
    fn events_are_all_day_and_crlf() {
        let ics = render(&[one(date!(2026 - 11 - 15), "Upload Acme statement", "Why")], 1_792_300_000);
        assert!(ics.contains("DTSTART;VALUE=DATE:20261115\r\n"));
        assert!(ics.contains("DTEND;VALUE=DATE:20261116\r\n"));
        assert!(ics.contains("TRIGGER;RELATED=START:PT9H\r\n"));
        assert!(ics.contains("X-WR-CALNAME:joy reminders\r\n"));
        assert!(ics.contains("REFRESH-INTERVAL;VALUE=DURATION:PT12H\r\n"));
        assert!(ics.contains("X-PUBLISHED-TTL:PT12H\r\n"));
        assert!(ics.starts_with("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n"));
        assert!(ics.ends_with("END:VCALENDAR\r\n"));
        assert!(!ics.replace("\r\n", "").contains('\n'), "only CRLF line breaks");
    }

    #[test]
    fn long_lines_are_folded_to_75_octets() {
        let title = "Upload ".to_string() + &"ü".repeat(120);
        let ics = render(&[one(date!(2026 - 11 - 15), &title, "d")], 0);
        assert!(ics.split("\r\n").all(|l| l.len() <= 75));
        let unfolded = ics.replace("\r\n ", "");
        assert!(unfolded.contains(&format!("SUMMARY:{title}\r\n")));
    }

    #[test]
    fn text_is_escaped() {
        let ics = render(&[one(date!(2026 - 11 - 15), "Tea, milk; sugar", "line one\nline two\\")], 0);
        assert!(ics.contains("SUMMARY:Tea\\, milk\\; sugar\r\n"));
        assert!(ics.contains("DESCRIPTION:line one\\nline two\\\\\r\n"));
    }

    #[test]
    fn dtstamp_is_utc() {
        // 2026-10-18 13:30 UTC
        let ics = render(&[one(date!(2026 - 11 - 15), "x", "y")], 1_792_330_200);
        assert!(ics.contains("DTSTAMP:20261018T133000Z\r\n"));
    }

    #[test]
    fn habits_become_reminders_with_action_links() {
        let habits = [habit("bank_statement:acctB", "Upload your B statement", Some("2026-11-20"), Some("/statements"))];
        let got = from_habits(&habits, date!(2026 - 11 - 09), "https://joy.example");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].uid, "joy-habit-bank_statement-acctB@joy");
        assert_eq!(got[0].on, date!(2026 - 11 - 20));
        assert_eq!(got[0].summary, "Upload your B statement");
        assert_eq!(got[0].description, "Asha's statements are a month old.\nhttps://joy.example/statements");
    }

    #[test]
    fn past_due_moves_to_today() {
        let habits = [habit("k", "t", Some("2026-11-01"), None)];
        let got = from_habits(&habits, date!(2026 - 11 - 09), "https://joy.example");
        assert_eq!(got[0].on, date!(2026 - 11 - 09));
        assert_eq!(got[0].description, "Asha's statements are a month old.");
    }

    #[test]
    fn undated_habits_and_past_one_offs_stay_out_of_the_calendar() {
        let once = |key: &str, due: &str| Habit { cadence: Cadence::Once, ..habit(key, "t", Some(due), None) };
        let habits = [
            habit("k", "t", None, None),
            habit("j", "u", Some("2026-12-01"), None),
            once("past", "2026-11-01"),
            once("tax", "2026-12-15"),
            habit("late", "Upload your Acme statement", Some("2026-10-01"), None),
        ];
        let got = from_habits(&habits, date!(2026 - 11 - 09), "https://joy.example");
        let uids: Vec<&str> = got.iter().map(|r| r.uid.as_str()).collect();
        assert_eq!(uids, vec!["joy-habit-j@joy", "joy-habit-tax@joy", "joy-habit-late@joy"]);
        assert_eq!(got[2].on, date!(2026 - 11 - 09), "a late recurring habit shows today");
    }

    #[test]
    fn token_format() {
        let good = "0123456789abcdef".repeat(4);
        assert!(is_token(&good));
        assert_eq!(token_from_file(&format!("{good}.ics")), Some(good.as_str()));
        assert!(!is_token(&good.to_uppercase()));
        assert!(!is_token(&good[1..]));
        assert!(!is_token(&format!("{good}0")));
        assert!(!is_token(&good.replace('a', "g")));
        assert_eq!(token_from_file(&good), None, "the .ics suffix is required");
        assert_eq!(token_from_file(&format!("{good}.json")), None);
        assert_eq!(token_from_file("../etc/passwd.ics"), None);
    }

    fn pdate(id: &str, name: &str, kind: crate::planner::people::DateKind, month: u8, day: u8) -> DateIn {
        DateIn {
            date_id: id.into(),
            person_id: "p1".into(),
            name: name.into(),
            circle: crate::planner::people::Circle::Close,
            kind,
            label: String::new(),
            month,
            day,
            year: Some(1990),
            remind_days: 7,
            gift_budget_paise: None,
            settled_years: vec![],
        }
    }

    #[test]
    fn dates_are_events_without_names_by_default() {
        use crate::planner::people::DateKind;
        let dates = [
            pdate("d1", "Person A", DateKind::Birthday, 12, 14),
            pdate("d2", "Person B", DateKind::Anniversary, 3, 1),
            pdate("d3", "Person C", DateKind::Birthday, 11, 9),
        ];
        let got = from_dates(&dates, &[], date!(2026 - 11 - 09), false, "https://joy.example");
        let uids: Vec<&str> = got.iter().map(|r| r.uid.as_str()).collect();
        assert_eq!(uids, vec!["joy-pdate-d3-2026@joy", "joy-pdate-d1-2026@joy"], "1 Mar is more than 90 days away");
        assert_eq!(got[1].summary, "Birthday (close friend)");
        assert_eq!(got[1].on, date!(2026 - 12 - 14));
        assert!(got.iter().all(|r| !r.summary.contains("Person") && !r.description.contains("Person")));
        assert!(got[1].description.ends_with("https://joy.example/people"));
    }

    #[test]
    fn names_show_only_when_asked() {
        use crate::planner::people::DateKind;
        let dates = [pdate("d1", "Person A", DateKind::Birthday, 12, 14)];
        let got = from_dates(&dates, &[], date!(2026 - 11 - 09), true, "https://x");
        assert_eq!(got[0].summary, "Person A's birthday");
        assert!(got[0].description.contains("Turns 36."));
    }

    #[test]
    fn a_date_ninety_days_out_is_in_and_ninety_one_is_out() {
        use crate::planner::people::DateKind;
        // 9 Nov + 90 days = 7 Feb 2027.
        let dates = [pdate("a", "x", DateKind::Birthday, 2, 7), pdate("b", "x", DateKind::Birthday, 2, 8)];
        let got = from_dates(&dates, &[], date!(2026 - 11 - 09), false, "https://x");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].uid, "joy-pdate-a-2027@joy");
    }

    #[test]
    fn year_wraps_from_december_to_january() {
        use crate::planner::people::DateKind;
        let dates = [pdate("d1", "x", DateKind::Birthday, 1, 5)];
        let got = from_dates(&dates, &[], date!(2026 - 12 - 20), false, "https://x");
        assert_eq!(got[0].on, date!(2027 - 01 - 05));
        assert_eq!(got[0].uid, "joy-pdate-d1-2027@joy");
    }

    #[test]
    fn seasons_are_events_on_the_first_with_their_label() {
        let seasons = [
            SeasonIn { id: "s1".into(), label: "Diwali".into(), month: 12, budget_paise: 500_000 },
            SeasonIn { id: "s2".into(), label: "Rakhi".into(), month: 8, budget_paise: 100_000 },
        ];
        let got = from_dates(&[], &seasons, date!(2026 - 11 - 09), false, "https://x");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].summary, "Gift season: Diwali");
        assert_eq!(got[0].on, date!(2026 - 12 - 01));
        assert_eq!(got[0].uid, "joy-gseason-s1-2026@joy");
        let today_first = from_dates(&[], &seasons, date!(2026 - 12 - 01), false, "https://x");
        assert_eq!(today_first[0].on, date!(2026 - 12 - 01), "the 1st itself counts");
    }

    #[test]
    fn a_private_feed_keeps_every_detail_out() {
        let mut habits = vec![
            habit("catch_up:p1", "Catch up with Person A", Some("2026-11-20"), None),
            habit("gift:d1:2026", "Gift for Person A's birthday on 14 Dec, about Rs 2,000.", Some("2026-12-01"), None),
            habit("bank_statement:acctB", "Upload your Asha bank statement", Some("2026-11-20"), None),
        ];
        habits[2].why = "Your Asha bank statements stop at 1 Oct; the ₹40,000 balance is a guess.".into();
        hide_names(&mut habits);
        assert_eq!(habits[0].title, "Catch up with someone");
        assert_eq!(habits[1].title, "Gift for an upcoming date");
        assert_eq!(habits[2].title, "A money task in joy");
        assert!(habits.iter().all(|h| h.why == "Open joy for the details."));
    }
}
