//! A wedding budget (or any big goal that is paid in pieces): items with an estimate and maybe a
//! quote, dated payment milestones, and one verdict on whether what is set aside will cover each
//! payment by its date. Pure: the handler gathers the rows and passes `today`. All money is paise.
//!
//! How the pieces relate: paying a milestone also records a contribution to the goal (so each
//! partner's share still adds up). That money has been spent, so what is *on hand* for the next
//! payments is the goal's saved total minus what milestones have already paid.

use serde::Serialize;
use time::{Date, Duration, Month};

use super::{goals::months_until, life::inr};

/// The categories an item can sit in, in the order they are shown.
pub const CATEGORIES: &[&str] = &["venue", "catering", "clothes", "jewellery", "photography", "travel", "misc"];

/// Within this share of what is due (10%), having enough still reads as "tight".
const TIGHT_PERCENT: i128 = 10;
/// Contributions in this many days set the expected monthly pace.
const PACE_DAYS: i64 = 90;
const PACE_MONTHS: i64 = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub category: String,
    pub estimate_paise: i64,
    pub quote_paise: Option<i64>,
}

impl Item {
    /// What it is expected to cost: the quote when there is one (it replaces the estimate, higher or lower).
    pub fn planned(&self) -> i64 {
        self.quote_paise.unwrap_or(self.estimate_paise)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    pub id: String,
    pub item_id: String,
    pub due_on: Option<Date>,
    pub amount_paise: i64,
    pub paid: bool,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct CategoryTotal {
    pub category: String,
    pub items: usize,
    /// Quoted items at their quote, plus the rest at their estimate.
    pub planned_paise: i64,
    /// The part of `planned` that is a quote.
    pub committed_paise: i64,
    /// The part of `planned` that is still only an estimate.
    pub estimated_paise: i64,
    pub paid_paise: i64,
    /// Planned minus paid, never below zero.
    pub remaining_paise: i64,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub categories: Vec<CategoryTotal>,
    pub planned_paise: i64,
    pub committed_paise: i64,
    pub estimated_paise: i64,
    pub paid_paise: i64,
    pub remaining_paise: i64,
    /// Unpaid milestones with a date or without: money already pencilled in.
    pub scheduled_paise: i64,
    /// Remaining money no milestone covers yet.
    pub unscheduled_paise: i64,
    /// Unpaid milestones whose date has passed.
    pub overdue_paise: i64,
}

fn rank(category: &str) -> usize {
    CATEGORIES.iter().position(|c| *c == category).unwrap_or(CATEGORIES.len())
}

/// Totals by category and overall. A milestone whose item is gone counts under `misc` (its payment
/// is real money either way). `today` marks which unpaid milestones are overdue.
pub fn budget_summary(items: &[Item], milestones: &[Milestone], today: Date) -> Summary {
    fn slot(cats: &mut Vec<CategoryTotal>, name: &str) -> usize {
        match cats.iter().position(|c| c.category == name) {
            Some(i) => i,
            None => {
                cats.push(CategoryTotal { category: name.to_string(), ..Default::default() });
                cats.len() - 1
            }
        }
    }
    let mut cats: Vec<CategoryTotal> = Vec::new();
    for it in items {
        let i = slot(&mut cats, &it.category);
        let c = &mut cats[i];
        c.items += 1;
        match it.quote_paise {
            Some(q) => c.committed_paise = c.committed_paise.saturating_add(q),
            None => c.estimated_paise = c.estimated_paise.saturating_add(it.estimate_paise),
        }
        c.planned_paise = c.committed_paise.saturating_add(c.estimated_paise);
    }
    let mut s = Summary::default();
    for m in milestones {
        if m.paid {
            let category = items.iter().find(|i| i.id == m.item_id).map_or("misc", |i| i.category.as_str());
            let i = slot(&mut cats, category);
            cats[i].paid_paise = cats[i].paid_paise.saturating_add(m.amount_paise);
        } else {
            s.scheduled_paise = s.scheduled_paise.saturating_add(m.amount_paise);
            if m.due_on.is_some_and(|d| d < today) {
                s.overdue_paise = s.overdue_paise.saturating_add(m.amount_paise);
            }
        }
    }
    cats.sort_by(|a, b| (rank(&a.category), &a.category).cmp(&(rank(&b.category), &b.category)));
    for c in &mut cats {
        c.remaining_paise = (c.planned_paise - c.paid_paise).max(0);
        s.planned_paise = s.planned_paise.saturating_add(c.planned_paise);
        s.committed_paise = s.committed_paise.saturating_add(c.committed_paise);
        s.estimated_paise = s.estimated_paise.saturating_add(c.estimated_paise);
        s.paid_paise = s.paid_paise.saturating_add(c.paid_paise);
        s.remaining_paise = s.remaining_paise.saturating_add(c.remaining_paise);
    }
    s.unscheduled_paise = (s.remaining_paise - s.scheduled_paise).max(0);
    s.categories = cats;
    s
}

/// Money set aside and not yet paid out: saved minus paid, never below zero.
pub fn on_hand(saved_paise: i64, paid_paise: i64) -> i64 {
    (saved_paise - paid_paise).max(0)
}

/// What the couple is likely to put in each month: contributions in the last 90 days, averaged over
/// three months (rounded down, so never an optimistic figure).
pub fn expected_monthly(contributions: &[(Date, i64)], today: Date) -> i64 {
    let from = today - Duration::days(PACE_DAYS);
    let total = contributions.iter().filter(|(d, a)| *d > from && *d <= today && *a > 0).fold(0i64, |t, (_, a)| t.saturating_add(*a));
    total / PACE_MONTHS
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    OnTrack,
    Tight,
    Short,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub status: Status,
    /// How much is missing (short), or how little is spare (tight) at the deciding payment; 0 when on track.
    pub gap_paise: i64,
    /// The date of the deciding payment, when there is one.
    pub by: Option<String>,
    /// One sentence for the top of the page.
    pub line: String,
}

fn day_label(d: Date) -> String {
    let m = match d.month() {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    };
    format!("{} {m} {}", d.day(), d.year())
}

/// Can what is on hand, plus `monthly_paise` put in each month from now, cover every unpaid dated
/// payment by its date? Payments are taken in date order, each needing everything due up to and
/// including it. Short: the largest gap and the date it bites. Tight: covered, but with under 10%
/// of what is due to spare at some payment. Overdue payments are due now (no more months to save).
/// Payments with no date are left out: nothing to judge them against.
pub fn verdict(on_hand_paise: i64, milestones: &[Milestone], monthly_paise: i64, today: Date) -> Verdict {
    let mut due: Vec<(Date, i64)> = milestones.iter().filter(|m| !m.paid).filter_map(|m| m.due_on.map(|d| (d, m.amount_paise))).collect();
    due.sort();
    let (mut cumulative, mut worst_gap, mut worst_gap_on): (i128, i128, Option<Date>) = (0, 0, None);
    let mut tightest: Option<(i128, Date)> = None;
    for (on, amount) in due {
        cumulative += i128::from(amount);
        let months_to_save = if on <= today { 0 } else { months_until(today, on) - 1 };
        let have = i128::from(on_hand_paise) + i128::from(monthly_paise.max(0)) * i128::from(months_to_save);
        let margin = have - cumulative;
        if margin < 0 {
            if -margin > worst_gap {
                worst_gap = -margin;
                worst_gap_on = Some(on);
            }
        } else if margin * 100 < cumulative * TIGHT_PERCENT && tightest.is_none_or(|(m, _)| margin < m) {
            tightest = Some((margin, on));
        }
    }
    let clamp = |v: i128| i64::try_from(v).unwrap_or(i64::MAX);
    if let Some(on) = worst_gap_on {
        let gap = clamp(worst_gap);
        return Verdict { status: Status::Short, gap_paise: gap, by: Some(on.to_string()), line: format!("Short by {} by {}", inr(gap), day_label(on)) };
    }
    if let Some((margin, on)) = tightest {
        let spare = clamp(margin);
        return Verdict { status: Status::Tight, gap_paise: spare, by: Some(on.to_string()), line: format!("Tight: only {} to spare by {}", inr(spare), day_label(on)) };
    }
    Verdict { status: Status::OnTrack, gap_paise: 0, by: None, line: "On track".into() }
}

/// The next payment to make: the earliest unpaid dated one (an overdue one comes first, naturally).
pub fn next_due(milestones: &[Milestone], _today: Date) -> Option<&Milestone> {
    milestones.iter().filter(|m| !m.paid && m.due_on.is_some()).min_by(|a, b| (a.due_on, &a.id).cmp(&(b.due_on, &b.id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    const TODAY: Date = date!(2026 - 11 - 15);

    fn item(id: &str, category: &str, estimate: i64, quote: Option<i64>) -> Item {
        Item { id: id.into(), category: category.into(), estimate_paise: estimate, quote_paise: quote }
    }
    fn ms(id: &str, item_id: &str, due: Option<Date>, amount: i64, paid: bool) -> Milestone {
        Milestone { id: id.into(), item_id: item_id.into(), due_on: due, amount_paise: amount, paid }
    }

    // ---- budget_summary

    #[test]
    fn totals_by_category_in_display_order() {
        let items = [item("a", "catering", 300, None), item("b", "venue", 500, Some(450)), item("c", "venue", 100, None), item("d", "dj", 50, None)];
        let s = budget_summary(&items, &[], TODAY);
        let names: Vec<&str> = s.categories.iter().map(|c| c.category.as_str()).collect();
        assert_eq!(names, ["venue", "catering", "dj"], "known categories first, in order; unknown ones last");
        let venue = &s.categories[0];
        assert_eq!((venue.items, venue.planned_paise, venue.committed_paise, venue.estimated_paise), (2, 550, 450, 100));
        assert_eq!(s.planned_paise, 900);
        assert_eq!((s.committed_paise, s.estimated_paise), (450, 450));
        assert_eq!(s.remaining_paise, 900);
    }

    #[test]
    fn a_quote_lower_than_the_estimate_replaces_it() {
        let low = budget_summary(&[item("a", "venue", 1_000, Some(800))], &[], TODAY);
        assert_eq!((low.planned_paise, low.committed_paise, low.estimated_paise), (800, 800, 0));
        let high = budget_summary(&[item("a", "venue", 1_000, Some(1_200))], &[], TODAY);
        assert_eq!(high.planned_paise, 1_200);
    }

    #[test]
    fn paid_and_remaining_follow_paid_milestones_only() {
        let items = [item("a", "venue", 1_000, None), item("b", "catering", 400, None)];
        let ms = [ms("m1", "a", Some(date!(2026 - 12 - 1)), 300, true), ms("m2", "a", Some(date!(2027 - 1 - 1)), 200, false), ms("m3", "b", None, 100, false)];
        let s = budget_summary(&items, &ms, TODAY);
        assert_eq!(s.paid_paise, 300);
        assert_eq!(s.remaining_paise, 1_100);
        assert_eq!(s.scheduled_paise, 300, "unpaid milestones, dated or not");
        assert_eq!(s.unscheduled_paise, 800);
        assert_eq!(s.categories[0].paid_paise, 300);
        assert_eq!(s.categories[1].paid_paise, 0);
    }

    #[test]
    fn overpaying_one_item_never_makes_remaining_negative_or_offsets_another() {
        let items = [item("a", "venue", 100, None), item("b", "catering", 400, None)];
        let s = budget_summary(&items, &[ms("m1", "a", None, 250, true)], TODAY);
        assert_eq!(s.categories[0].remaining_paise, 0);
        assert_eq!(s.remaining_paise, 400);
    }

    #[test]
    fn overdue_money_is_unpaid_dated_milestones_before_today() {
        let items = [item("a", "venue", 1_000, None)];
        let list = [
            ms("late", "a", Some(date!(2026 - 11 - 14)), 100, false),
            ms("today", "a", Some(TODAY), 50, false),
            ms("paid late", "a", Some(date!(2026 - 10 - 1)), 70, true),
            ms("undated", "a", None, 30, false),
        ];
        assert_eq!(budget_summary(&items, &list, TODAY).overdue_paise, 100);
    }

    #[test]
    fn a_milestone_with_no_item_still_counts_as_money_paid_under_misc() {
        let s = budget_summary(&[item("a", "venue", 1_000, None)], &[ms("m", "gone", None, 250, true), ms("n", "gone", None, 40, false)], TODAY);
        let misc = s.categories.iter().find(|c| c.category == "misc").expect("misc row");
        assert_eq!((misc.paid_paise, misc.planned_paise, misc.items), (250, 0, 0));
        assert_eq!(s.paid_paise, 250);
        assert_eq!(s.scheduled_paise, 40);
    }

    #[test]
    fn nothing_at_all_is_all_zero() {
        assert_eq!(budget_summary(&[], &[], TODAY), Summary::default());
    }

    // ---- on_hand and pace

    #[test]
    fn on_hand_is_what_was_saved_less_what_was_paid_out() {
        assert_eq!(on_hand(1_000, 400), 600);
        assert_eq!(on_hand(100, 400), 0);
    }

    #[test]
    fn pace_is_the_last_ninety_days_over_three_months_rounded_down() {
        let c = [(date!(2026 - 11 - 1), 1_000), (date!(2026 - 9 - 1), 2_000), (date!(2026 - 8 - 1), 9_999), (date!(2026 - 11 - 20), 5_000)];
        // Aug 1 is 106 days back and the Nov 20 one has not happened: only 3,000 counts.
        assert_eq!(expected_monthly(&c, TODAY), 1_000);
        assert_eq!(expected_monthly(&[(TODAY, 100)], TODAY), 33);
        assert_eq!(expected_monthly(&[], TODAY), 0);
    }

    // ---- verdict

    #[test]
    fn no_payments_to_make_is_on_track() {
        let v = verdict(0, &[], 0, TODAY);
        assert_eq!((v.status, v.gap_paise, v.by.as_deref(), v.line.as_str()), (Status::OnTrack, 0, None, "On track"));
    }

    #[test]
    fn over_funded_is_on_track() {
        let v = verdict(10_000_000, &[ms("m", "a", Some(date!(2026 - 12 - 1)), 100_000, false)], 0, TODAY);
        assert_eq!(v.status, Status::OnTrack);
    }

    #[test]
    fn short_names_the_gap_and_the_date_it_bites() {
        // ₹1,000 on hand, ₹3,000 due on 1 Mar 2027. Nov..Mar = 5 months, so 4 more months of saving.
        let due = date!(2027 - 3 - 1);
        let v = verdict(100_000, &[ms("m", "a", Some(due), 300_000, false)], 10_000, TODAY);
        assert_eq!(v.status, Status::Short);
        assert_eq!(v.gap_paise, 300_000 - 100_000 - 40_000);
        assert_eq!(v.by.as_deref(), Some("2027-03-01"));
        assert_eq!(v.line, format!("Short by {} by 1 Mar 2027", inr(160_000)));
    }

    #[test]
    fn monthly_contribution_can_turn_short_into_on_track() {
        let due = [ms("m", "a", Some(date!(2027 - 3 - 1)), 300_000, false)];
        assert_eq!(verdict(100_000, &due, 0, TODAY).status, Status::Short);
        assert_eq!(verdict(100_000, &due, 100_000, TODAY).status, Status::OnTrack, "4 months x 1,000 plus 1,000 covers 3,000 with room");
    }

    #[test]
    fn tight_when_covered_with_less_than_a_tenth_to_spare() {
        let due = [ms("m", "a", Some(date!(2026 - 12 - 1)), 100_000, false)];
        assert_eq!(verdict(105_000, &due, 0, TODAY).status, Status::Tight);
        let v = verdict(105_000, &due, 0, TODAY);
        assert_eq!(v.gap_paise, 5_000, "spare, not a shortfall");
        assert_eq!(verdict(110_000, &due, 0, TODAY).status, Status::OnTrack, "exactly 10% spare is comfortable");
        assert_eq!(verdict(100_000, &due, 0, TODAY).status, Status::Tight, "exactly enough is tight");
        assert_eq!(verdict(99_999, &due, 0, TODAY).status, Status::Short);
    }

    #[test]
    fn an_overdue_payment_is_due_now_and_gets_no_months_of_saving() {
        let due = [ms("m", "a", Some(date!(2026 - 10 - 1)), 100_000, false)];
        let v = verdict(40_000, &due, 1_000_000, TODAY);
        assert_eq!((v.status, v.gap_paise), (Status::Short, 60_000));
        assert_eq!(v.by.as_deref(), Some("2026-10-01"));
    }

    #[test]
    fn a_payment_due_this_month_gets_no_saving_either() {
        let due = [ms("m", "a", Some(date!(2026 - 11 - 30)), 100_000, false)];
        assert_eq!(verdict(0, &due, 1_000_000, TODAY).status, Status::Short);
    }

    #[test]
    fn payments_without_a_date_and_paid_ones_are_not_judged() {
        let list = [ms("a", "x", None, 9_000_000, false), ms("b", "x", Some(date!(2026 - 12 - 1)), 9_000_000, true)];
        assert_eq!(verdict(0, &list, 0, TODAY).status, Status::OnTrack);
    }

    #[test]
    fn each_payment_needs_everything_due_before_it_too() {
        // 500 on hand: the first 400 is fine alone, but the second 400 brings the total to 800.
        let list = [ms("a", "x", Some(date!(2026 - 12 - 1)), 400, false), ms("b", "x", Some(date!(2026 - 12 - 2)), 400, false)];
        let v = verdict(500, &list, 0, TODAY);
        assert_eq!((v.status, v.gap_paise), (Status::Short, 300));
        assert_eq!(v.by.as_deref(), Some("2026-12-02"));
    }

    #[test]
    fn the_worst_gap_wins_not_the_last_one() {
        // 1,000 on hand, 5,000 a month. Dec 1 needs 9,000 and has 6,000 (gap 3,000); the later,
        // smaller payment is comfortably covered, so the early one decides.
        let list = [ms("a", "x", Some(date!(2026 - 12 - 1)), 9_000, false), ms("b", "x", Some(date!(2027 - 6 - 1)), 100, false)];
        let v = verdict(1_000, &list, 5_000, TODAY);
        assert_eq!(v.status, Status::Short);
        assert_eq!(v.by.as_deref(), Some("2026-12-01"));
        assert_eq!(v.gap_paise, 3_000);
    }

    #[test]
    fn a_negative_monthly_pace_is_ignored_and_huge_amounts_do_not_overflow() {
        let due = [ms("m", "a", Some(date!(2027 - 3 - 1)), i64::MAX, false)];
        let v = verdict(0, &due, -5, TODAY);
        assert_eq!(v.status, Status::Short);
        assert_eq!(v.gap_paise, i64::MAX);
        let more = [ms("a", "x", Some(date!(2027 - 3 - 1)), i64::MAX, false), ms("b", "x", Some(date!(2027 - 3 - 2)), i64::MAX, false)];
        assert_eq!(verdict(0, &more, i64::MAX, TODAY).status, Status::OnTrack);
    }

    #[test]
    fn changing_a_milestone_amount_changes_the_verdict() {
        let mut m = ms("m", "a", Some(date!(2027 - 1 - 1)), 300_000, false);
        assert_eq!(verdict(500_000, &[m.clone()], 0, TODAY).status, Status::OnTrack);
        m.amount_paise = 600_000;
        assert_eq!(verdict(500_000, &[m], 0, TODAY).status, Status::Short);
    }

    // ---- next_due

    #[test]
    fn next_due_is_the_earliest_unpaid_dated_payment() {
        let list = [
            ms("far", "a", Some(date!(2027 - 5 - 1)), 1, false),
            ms("paid", "a", Some(date!(2026 - 11 - 20)), 1, true),
            ms("none", "a", None, 1, false),
            ms("soon", "a", Some(date!(2026 - 12 - 1)), 1, false),
        ];
        assert_eq!(next_due(&list, TODAY).map(|m| m.id.as_str()), Some("soon"));
    }

    #[test]
    fn an_overdue_payment_is_the_next_one_due() {
        let list = [ms("soon", "a", Some(date!(2026 - 12 - 1)), 1, false), ms("late", "a", Some(date!(2026 - 10 - 1)), 1, false)];
        assert_eq!(next_due(&list, TODAY).map(|m| m.id.as_str()), Some("late"));
    }

    #[test]
    fn next_due_is_none_when_nothing_is_dated_and_unpaid() {
        assert!(next_due(&[], TODAY).is_none());
        assert!(next_due(&[ms("a", "a", None, 1, false), ms("b", "a", Some(TODAY), 1, true)], TODAY).is_none());
    }
}
