//! The weekly "what needs a decision" list. Pure: the handler counts things, this turns the counts
//! into a short list and a push message. Counts only: never a name or an amount.

use serde::Serialize;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Uncategorized,
    GoalBehind,
    LargeAmountSoon,
    MilestoneSoon,
    BirthdaySoon,
    FollowUpOverdue,
}

/// What the handler found. Every field is a plain count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// Recent transactions still waiting for a category.
    pub uncategorized: usize,
    /// Shared goals past their date or close to it and not nearly funded.
    pub goals_behind: usize,
    /// Large one-off amounts (premiums, tax, maturities) due within the window.
    pub large_amounts_soon: usize,
    /// Wedding payments due within the window.
    pub milestones_soon: usize,
    pub birthdays_soon: usize,
    pub followups_overdue: usize,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub kind: Kind,
    pub count: usize,
    /// "5 transactions need a category".
    pub text: String,
    /// Where to act on it.
    pub link: String,
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The list, most pressing first. Empty when nothing needs a decision.
pub fn decisions(c: &Counts) -> Vec<Decision> {
    let mut out = Vec::new();
    let mut add = |kind: Kind, count: usize, text: String, link: &str| {
        if count > 0 {
            out.push(Decision { kind, count, text, link: link.into() });
        }
    };
    add(Kind::MilestoneSoon, c.milestones_soon, format!("{} due soon", plural(c.milestones_soon, "wedding payment", "wedding payments")), "/");
    add(Kind::LargeAmountSoon, c.large_amounts_soon, format!("{} coming up", plural(c.large_amounts_soon, "large amount", "large amounts")), "/year");
    add(Kind::GoalBehind, c.goals_behind, format!("{} behind", plural(c.goals_behind, "shared goal", "shared goals")), "/");
    add(Kind::FollowUpOverdue, c.followups_overdue, format!("{} overdue", plural(c.followups_overdue, "follow-up", "follow-ups")), "/people");
    add(Kind::BirthdaySoon, c.birthdays_soon, format!("{} this week", plural(c.birthdays_soon, "birthday", "birthdays")), "/people");
    add(Kind::Uncategorized, c.uncategorized, format!("{} need a category", plural(c.uncategorized, "transaction", "transactions")), "/review");
    out
}

/// The weekly push body, or nothing when nothing needs a decision. A count of things, never what they are.
pub fn weekly_body(list: &[Decision]) -> Option<String> {
    let total: usize = list.iter().map(|d| d.count).sum();
    if total == 0 {
        return None;
    }
    Some(format!("This week: {}.", plural(total, "thing needs a decision", "things need a decision")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_found_means_no_list_and_no_message() {
        let list = decisions(&Counts::default());
        assert!(list.is_empty());
        assert_eq!(weekly_body(&list), None);
    }

    #[test]
    fn only_what_is_found_is_listed_and_most_pressing_first() {
        let list = decisions(&Counts { uncategorized: 5, goals_behind: 1, large_amounts_soon: 2, ..Counts::default() });
        let kinds: Vec<Kind> = list.iter().map(|d| d.kind).collect();
        assert_eq!(kinds, vec![Kind::LargeAmountSoon, Kind::GoalBehind, Kind::Uncategorized]);
        assert_eq!(list[0].text, "2 large amounts coming up");
        assert_eq!(list[1].text, "1 shared goal behind");
        assert_eq!(list[2].text, "5 transactions need a category");
        assert_eq!(list[2].link, "/review");
    }

    #[test]
    fn the_push_counts_things_and_names_nothing() {
        let list = decisions(&Counts { uncategorized: 5, milestones_soon: 1, birthdays_soon: 1, ..Counts::default() });
        assert_eq!(weekly_body(&list).as_deref(), Some("This week: 7 things need a decision."));
        let one = decisions(&Counts { followups_overdue: 1, ..Counts::default() });
        assert_eq!(weekly_body(&one).as_deref(), Some("This week: 1 thing needs a decision."));
    }
}
