//! The reminder calendar feed: one all-day event per habit that has a due date, rendered as an
//! iCalendar file. Pure: the handler loads the habits and the clock, this module only shapes them.

use time::{format_description::FormatItem, macros::format_description, Date, OffsetDateTime};

use crate::joy::ics::{escape, fold};
use crate::planner::habits::Habit;

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
}
