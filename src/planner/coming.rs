//! One "coming up" list: bills, planned dates, birthdays and follow-ups in a single timeline, so
//! nothing depends on remembering which page holds it. Pure: the handler gathers the pieces.

use serde::Serialize;
use time::Date;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Bill,
    Date,
    Birthday,
    FollowUp,
    /// A payment due on a goal's budget (the wedding). Overdue ones stay on the list until paid.
    Milestone,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Item {
    pub on: Date,
    pub kind: Kind,
    pub title: String,
    /// A line of detail: an amount, a person, a time. Empty when there is none.
    pub detail: String,
    /// Where in the app it lives, e.g. "/bills".
    pub link: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Entry {
    #[serde(flatten)]
    pub item: Item,
    pub days_away: i64,
    /// "Today", "Tomorrow", "In 3 days", "Overdue by 2 days".
    pub when: String,
}

/// "Today", "Tomorrow", "In 3 days", "2 days ago".
pub fn when_label(days_away: i64) -> String {
    match days_away {
        0 => "Today".into(),
        1 => "Tomorrow".into(),
        n if n > 1 => format!("In {n} days"),
        -1 => "Overdue by a day".into(),
        n => format!("Overdue by {} days", -n),
    }
}

/// Everything from `today` (overdue follow-ups included) to `days` ahead, soonest first; on the same
/// day, bills before dates before birthdays before follow-ups, then by title.
pub fn merge(items: Vec<Item>, today: Date, days: i64) -> Vec<Entry> {
    let mut out: Vec<Entry> = items
        .into_iter()
        .map(|item| {
            let days_away = (item.on - today).whole_days();
            Entry { when: when_label(days_away), days_away, item }
        })
        .filter(|e| e.days_away <= days && (e.days_away >= 0 || matches!(e.item.kind, Kind::FollowUp | Kind::Milestone)))
        .collect();
    let rank = |k: Kind| match k {
        Kind::Bill => 0,
        Kind::Date => 1,
        Kind::Birthday => 2,
        Kind::FollowUp => 3,
        Kind::Milestone => 4,
    };
    out.sort_by(|a, b| (a.item.on, rank(a.item.kind), &a.item.title).cmp(&(b.item.on, rank(b.item.kind), &b.item.title)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    const TODAY: Date = date!(2026 - 11 - 15);

    fn item(on: Date, kind: Kind, title: &str) -> Item {
        Item { on, kind, title: title.into(), detail: String::new(), link: String::new() }
    }

    #[test]
    fn labels_read_like_a_person_would_say_them() {
        assert_eq!(when_label(0), "Today");
        assert_eq!(when_label(1), "Tomorrow");
        assert_eq!(when_label(5), "In 5 days");
        assert_eq!(when_label(-1), "Overdue by a day");
        assert_eq!(when_label(-4), "Overdue by 4 days");
    }

    #[test]
    fn one_timeline_soonest_first_with_a_stable_order_within_a_day() {
        let items = vec![
            item(date!(2026 - 11 - 17), Kind::Birthday, "Asha"),
            item(date!(2026 - 11 - 16), Kind::FollowUp, "Ask about the move"),
            item(date!(2026 - 11 - 16), Kind::Bill, "Rent"),
            item(date!(2026 - 11 - 16), Kind::Date, "Dinner at Saltt"),
        ];
        let merged = merge(items.clone(), TODAY, 14);
        let got: Vec<&str> = merged.iter().map(|e| e.item.title.as_str()).collect();
        assert_eq!(got, ["Rent", "Dinner at Saltt", "Ask about the move", "Asha"]);
        let again: Vec<String> = merge(items, TODAY, 14).iter().map(|e| e.when.clone()).collect();
        assert_eq!(again, ["Tomorrow", "Tomorrow", "Tomorrow", "In 2 days"]);
    }

    #[test]
    fn an_unpaid_milestone_stays_on_the_list_when_overdue_and_sorts_after_the_rest_of_its_day() {
        let items = vec![
            item(date!(2026 - 11 - 16), Kind::Milestone, "Wedding: Venue"),
            item(date!(2026 - 11 - 16), Kind::Bill, "Rent"),
            item(date!(2026 - 11 - 10), Kind::Milestone, "Wedding: Caterer"),
            item(date!(2026 - 12 - 30), Kind::Milestone, "Wedding: Too far"),
        ];
        let got = merge(items, TODAY, 14);
        let titles: Vec<&str> = got.iter().map(|e| e.item.title.as_str()).collect();
        assert_eq!(titles, ["Wedding: Caterer", "Rent", "Wedding: Venue"]);
        assert_eq!(got[0].when, "Overdue by 5 days");
    }

    #[test]
    fn the_window_ends_and_only_follow_ups_can_be_overdue() {
        let items = vec![
            item(date!(2026 - 11 - 30), Kind::Bill, "Too far"),
            item(date!(2026 - 11 - 14), Kind::Bill, "Paid already"),
            item(date!(2026 - 11 - 13), Kind::FollowUp, "Call back"),
            item(date!(2026 - 11 - 29), Kind::Bill, "Last day"),
        ];
        let got = merge(items, TODAY, 14);
        let titles: Vec<&str> = got.iter().map(|e| e.item.title.as_str()).collect();
        assert_eq!(titles, ["Call back", "Last day"]);
        assert_eq!(got[0].when, "Overdue by 2 days");
    }
}
