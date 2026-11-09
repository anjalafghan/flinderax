//! Diet and health: pure rules and the quarterly report section. Phase 5.

use serde::{Deserialize, Serialize};
use time::{Date, Duration, Month};

use crate::planner::{
    checkin::rupees,
    habits::{Area, Cadence, Habit, Impact, Link, Status},
};

/// Ready-made weekly habits: (key, label, target days a week).
pub const PRESETS: [(&str, &str, i64); 6] = [
    ("home_dinner", "Home-cooked dinner", 5),
    ("no_sugary", "No sugary drinks", 6),
    ("fruit_veg", "Fruit or salad every day", 5),
    ("walk", "Walk 30 minutes", 4),
    ("early_dinner", "Dinner before 9 pm", 5),
    ("water", "Eight glasses of water", 5),
];

/// Most habits tracked at once.
pub const MAX_ACTIVE: i64 = 3;

/// Merchants (lowercase fragments) that mean a delivery order when the payment is tagged food.
const DELIVERY_MERCHANTS: [&str; 6] = ["swiggy", "zomato", "eatsure", "box8", "dominos", "faasos"];

const DUE_SOON_DAYS: i64 = 14;
const EATING_OUT_SHARE_BPS: i64 = 4_000;
const CAP_OVER_PACE_PCT: i64 = 115;

/// A food-tagged payment to this merchant is a delivery order.
pub fn is_delivery(merchant: &str) -> bool {
    let m = merchant.to_lowercase();
    DELIVERY_MERCHANTS.iter().any(|d| m.contains(d))
}

/// Delivery orders a week in hundredths, over `days` days.
pub fn per_week_x100(orders: i64, days: i64) -> i64 {
    if days <= 0 {
        return 0;
    }
    (orders * 700 * 2 + days) / (days * 2)
}

/// An active weekly habit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HabitIn {
    pub id: String,
    pub label: String,
    pub target_days: i64,
}

/// What the diet and health habit rules read. Filled by `handlers::diet::habits_input`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DietIn {
    /// Active weekly habits.
    pub habits: Vec<HabitIn>,
    /// First day of the current week, when known.
    pub week_start: Option<Date>,
    pub week_logged: bool,
    pub prev_week_logged: bool,
    /// Days hit per habit id in the last four logged weeks (one list per week).
    pub recent_weeks: Vec<Vec<(String, i64)>>,
    /// Eating out (food) and groceries over the last 90 days.
    pub eating_out_90d_paise: i64,
    pub groceries_90d_paise: i64,
    pub delivery_90d_orders: i64,
    /// Eating out this month so far and over last month.
    pub eating_out_month_paise: i64,
    pub eating_out_last_month_paise: i64,
    pub day: u8,
    pub days_in_month: u8,
    pub cap_paise: Option<i64>,
    pub last_checkup: Option<Date>,
}

impl DietIn {
    /// Eating out as a share of food spend (basis points) over 90 days.
    pub fn share_bps(&self) -> Option<i64> {
        share_bps(self.eating_out_90d_paise, self.groceries_90d_paise)
    }

    /// With a cap set: is this month's eating out over the pace the cap allows?
    pub fn over_cap_pace(&self) -> bool {
        let Some(cap) = self.cap_paise else { return false };
        let pace = cap.saturating_mul(i64::from(self.day)) / i64::from(self.days_in_month.max(1));
        self.eating_out_month_paise.saturating_mul(100) > pace.saturating_mul(CAP_OVER_PACE_PCT)
    }
}

/// Eating out as a share of all food spend, in basis points, rounded.
pub fn share_bps(eating_out: i64, groceries: i64) -> Option<i64> {
    let total = eating_out + groceries;
    (total > 0).then(|| (eating_out * 10_000 * 2 + total) / (total * 2))
}

fn link(label: &str, to: &str) -> Option<Link> {
    Some(Link { label: label.to_string(), to: to.to_string() })
}

fn status_of(due: Date, today: Date) -> Status {
    let days = (due - today).whole_days();
    if days < 0 {
        Status::Overdue
    } else if days <= DUE_SOON_DAYS {
        Status::DueSoon
    } else {
        Status::Ok
    }
}

/// "12 Mar"
pub fn short_day(d: Date) -> String {
    format!("{} {}", d.day(), &month_name(d.month())[..3])
}

fn month_name(m: Month) -> &'static str {
    match m {
        Month::January => "January",
        Month::February => "February",
        Month::March => "March",
        Month::April => "April",
        Month::May => "May",
        Month::June => "June",
        Month::July => "July",
        Month::August => "August",
        Month::September => "September",
        Month::October => "October",
        Month::November => "November",
        Month::December => "December",
    }
}

/// The habit furthest behind its target over the logged weeks: (label, average days, target).
fn furthest_behind(input: &DietIn) -> Option<(String, i64, i64)> {
    let mut worst: Option<(i64, String, i64, i64)> = None;
    for h in &input.habits {
        let days: Vec<i64> = input
            .recent_weeks
            .iter()
            .filter_map(|w| w.iter().find(|(id, _)| *id == h.id).map(|(_, d)| *d))
            .collect();
        if days.is_empty() || h.target_days <= 0 {
            continue;
        }
        let avg = (days.iter().sum::<i64>() as f64 / days.len() as f64).round() as i64;
        let gap_bps = (h.target_days - avg) * 10_000 / h.target_days;
        if gap_bps > 0 && worst.as_ref().is_none_or(|(g, ..)| gap_bps > *g) {
            worst = Some((gap_bps, h.label.clone(), avg, h.target_days));
        }
    }
    worst.map(|(_, label, avg, target)| (label, avg, target))
}

/// The diet and health habits (areas `food` and `health`). Never `high` impact.
pub fn habits(input: &DietIn, today: Date) -> Vec<Habit> {
    let mut out = Vec::new();

    if !input.habits.is_empty() {
        let end = input.week_start.map(|s| s + Duration::days(6));
        let due_soon = !input.week_logged && (end == Some(today) || !input.prev_week_logged);
        let why = match furthest_behind(input) {
            Some((label, avg, target)) => format!("{label} is at {avg} of {target} days lately."),
            None => "Thirty seconds: how many days you hit each one.".to_string(),
        };
        out.push(Habit {
            key: "diet_week".to_string(),
            area: Area::Food,
            title: "Log this week's eating".to_string(),
            why,
            action: link("Log week", "/health"),
            cadence: Cadence::Weekly,
            due_on: end.map(|d| d.to_string()),
            status: if due_soon { Status::DueSoon } else { Status::Ok },
            impact: Impact::Low,
        });
    }

    let share = input.share_bps();
    let over_cap = input.over_cap_pace();
    let heavy = share.is_some_and(|s| s >= EATING_OUT_SHARE_BPS);
    if heavy || over_cap {
        let per_week = (per_week_x100(input.delivery_90d_orders, 90) + 50) / 100;
        let n = (per_week - 1).max(1);
        let why = match input.cap_paise.filter(|_| over_cap) {
            Some(cap) => format!(
                "{} so far this month against a {} cap.",
                rupees(input.eating_out_month_paise),
                rupees(cap)
            ),
            None => format!(
                "Eating out and delivery was {} last month, {}% of food.",
                rupees(input.eating_out_last_month_paise),
                (share.unwrap_or(0) + 50) / 100
            ),
        };
        out.push(Habit {
            key: "eating_out".to_string(),
            area: Area::Food,
            title: format!("Order in at most {n} {} this week", if n == 1 { "time" } else { "times" }),
            why,
            action: link("See food money", "/health"),
            cadence: Cadence::Weekly,
            due_on: None,
            status: if over_cap { Status::DueSoon } else { Status::Ok },
            impact: Impact::Medium,
        });
    }

    out.push(match input.last_checkup {
        None => Habit {
            key: "health_checkup".to_string(),
            area: Area::Health,
            title: "Book a full health check-up".to_string(),
            why: "No check-up recorded yet. Once a year catches things early.".to_string(),
            action: link("Health", "/health"),
            cadence: Cadence::Once,
            due_on: None,
            status: Status::DueSoon,
            impact: Impact::Low,
        },
        Some(last) => {
            let due = last + Duration::days(365);
            Habit {
                key: "health_checkup".to_string(),
                area: Area::Health,
                title: "Annual health check-up".to_string(),
                why: format!("Your last check-up was on {}.", short_day(last)),
                action: link("Health", "/health"),
                cadence: Cadence::Yearly,
                due_on: Some(due.to_string()),
                status: status_of(due, today),
                impact: Impact::Low,
            }
        }
    });
    out
}

/// One weekly diet habit over a check-in period.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DietHabitRate {
    pub label: String,
    pub target_days: i64,
    /// Average days hit as a share of the target, over logged weeks; None with no logged week.
    pub hit_bps: Option<i64>,
    pub weeks_on_target: i64,
    pub weeks_logged: i64,
}

/// The diet section of the quarter report. Its own wins and watch, at most 2 each.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DietReport {
    pub eating_out_paise: i64,
    pub groceries_paise: i64,
    pub prev_eating_out_paise: Option<i64>,
    pub prev_groceries_paise: Option<i64>,
    /// Eating out and delivery as a share of all food spend; None with no food spend.
    pub eating_out_share_bps: Option<i64>,
    /// Delivery orders per week over the period, in hundredths (350 = 3.5 a week).
    pub delivery_per_week_x100: i64,
    pub weeks_logged: i64,
    pub weeks_in_period: i64,
    pub habits: Vec<DietHabitRate>,
    /// Latest weight in the period, kg x10.
    pub weight_x10: Option<i64>,
    /// Change from the latest weight before the period, kg x10; None without both.
    pub weight_change_x10: Option<i64>,
    /// "YYYY-MM-DD"
    pub checkup_due_on: Option<String>,
    pub wins: Vec<String>,
    pub watch: Vec<String>,
}

/// "3.5" or "2": hundredths as a short number of times a week.
fn times(x100: i64) -> String {
    let tenths = (x100 + 5) / 10;
    if tenths % 10 == 0 {
        format!("{}", tenths / 10)
    } else {
        format!("{}.{}", tenths / 10, tenths % 10)
    }
}

/// Fills `wins` and `watch` (at most 2 each) from the numbers already in `report`.
/// `prev_delivery_x100` is the delivery rate in the previous period, when it had any food spend.
/// Weight never appears in this text.
pub fn add_notes(report: &mut DietReport, prev_delivery_x100: Option<i64>, today: Date) {
    let mut wins = Vec::new();
    let mut watch = Vec::new();

    if let Some(due) = report.checkup_due_on.as_deref().and_then(parse_day).filter(|d| *d < today) {
        watch.push(format!("Your yearly check-up is overdue since {}.", short_day(due)));
    }

    let share = report.eating_out_share_bps;
    let prev_share = match (report.prev_eating_out_paise, report.prev_groceries_paise) {
        (Some(e), Some(g)) => share_bps(e, g),
        _ => None,
    };
    if let (Some(now), Some(before)) = (share, prev_share) {
        let (now_pct, before_pct) = ((now + 50) / 100, (before + 50) / 100);
        if now_pct + 3 <= before_pct {
            wins.push(format!("Eating out fell to {now_pct}% of food, from {before_pct}%."));
        } else if now_pct >= before_pct + 3 {
            watch.push(format!("Eating out rose to {now_pct}% of food, from {before_pct}%."));
        }
    }

    if let Some(before) = prev_delivery_x100 {
        let now = report.delivery_per_week_x100;
        if now >= before + 50 && now >= 100 {
            watch.push(format!("Ordered in {} times a week, up from {}.", times(now), times(before)));
        } else if before >= now + 50 && before >= 100 {
            wins.push(format!("Ordered in {} times a week, down from {}.", times(now), times(before)));
        }
    }

    if report.weeks_in_period > 0 {
        let best = report
            .habits
            .iter()
            .filter(|h| h.weeks_on_target >= 3 && h.weeks_on_target * 2 >= report.weeks_in_period)
            .max_by_key(|h| h.weeks_on_target);
        if let Some(h) = best {
            wins.push(format!(
                "{} hit its target in {} of {} weeks.",
                h.label, h.weeks_on_target, report.weeks_in_period
            ));
        }
        let worst = report
            .habits
            .iter()
            .filter(|h| h.weeks_logged >= 3 && h.weeks_on_target * 3 < h.weeks_logged)
            .min_by_key(|h| h.weeks_on_target * 100 / h.weeks_logged);
        if let Some(h) = worst {
            watch.push(format!(
                "{} hit its target in {} of {} logged weeks.",
                h.label, h.weeks_on_target, h.weeks_logged
            ));
        }
    }

    wins.truncate(2);
    watch.truncate(2);
    report.wins = wins;
    report.watch = watch;
}

fn parse_day(s: &str) -> Option<Date> {
    Date::parse(s, time::macros::format_description!("[year]-[month]-[day]")).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn habit_in(id: &str, label: &str, target: i64) -> HabitIn {
        HabitIn { id: id.into(), label: label.into(), target_days: target }
    }

    fn find<'a>(h: &'a [Habit], key: &str) -> Option<&'a Habit> {
        h.iter().find(|x| x.key == key)
    }

    #[test]
    fn delivery_merchants_match_case_insensitively() {
        assert!(is_delivery("SWIGGY Instamart"));
        assert!(is_delivery("zomato"));
        assert!(is_delivery("Domino's Dominos Pizza"));
        assert!(!is_delivery("Corner Cafe"));
        assert_eq!(per_week_x100(18, 90), 140);
        assert_eq!(per_week_x100(0, 90), 0);
        assert_eq!(share_bps(300, 700), Some(3_000));
        assert_eq!(share_bps(0, 0), None);
    }

    #[test]
    fn checkup_never_recorded_then_dated() {
        let today = date!(2026 - 11 - 09);
        let h = habits(&DietIn::default(), today);
        let c = find(&h, "health_checkup").unwrap();
        assert_eq!((c.cadence, c.status, c.due_on.clone()), (Cadence::Once, Status::DueSoon, None));
        assert_eq!(c.title, "Book a full health check-up");
        assert!(find(&h, "diet_week").is_none() && find(&h, "eating_out").is_none());

        let mk = |last: Date| DietIn { last_checkup: Some(last), ..Default::default() };
        let c = &habits(&mk(date!(2025 - 11 - 20)), today)[0];
        assert_eq!((c.cadence, c.status), (Cadence::Yearly, Status::DueSoon));
        assert_eq!(c.due_on.as_deref(), Some("2026-11-20"));
        let c = &habits(&mk(date!(2025 - 03 - 12)), today)[0];
        assert_eq!(c.status, Status::Overdue);
        let c = &habits(&mk(date!(2026 - 06 - 01)), today)[0];
        assert_eq!(c.status, Status::Ok);
        assert_eq!(c.area, Area::Health);
        assert_ne!(c.impact, Impact::High);
    }

    #[test]
    fn diet_week_is_due_through_the_start_of_the_next_week() {
        let monday = date!(2026 - 11 - 09);
        let mut input = DietIn {
            habits: vec![habit_in("a", "Home-cooked dinner", 5)],
            week_start: Some(monday),
            week_logged: false,
            prev_week_logged: true,
            ..Default::default()
        };
        // Midweek, last week logged: fine for now.
        let h = habits(&input, date!(2026 - 11 - 11));
        let w = find(&h, "diet_week").unwrap();
        assert_eq!((w.status, w.due_on.as_deref()), (Status::Ok, Some("2026-11-15")));
        assert_eq!((w.cadence, w.area, w.impact), (Cadence::Weekly, Area::Food, Impact::Low));
        assert_eq!(w.why, "Thirty seconds: how many days you hit each one.");
        // The last day of the week.
        assert_eq!(find(&habits(&input, date!(2026 - 11 - 15)), "diet_week").unwrap().status, Status::DueSoon);
        // Last week not logged.
        input.prev_week_logged = false;
        assert_eq!(find(&habits(&input, date!(2026 - 11 - 10)), "diet_week").unwrap().status, Status::DueSoon);
        // Logged this week.
        input.week_logged = true;
        assert_eq!(find(&habits(&input, date!(2026 - 11 - 15)), "diet_week").unwrap().status, Status::Ok);
    }

    #[test]
    fn diet_week_names_the_habit_furthest_behind() {
        let input = DietIn {
            habits: vec![habit_in("a", "Home-cooked dinner", 5), habit_in("b", "Walk 30 minutes", 4)],
            week_start: Some(date!(2026 - 11 - 09)),
            recent_weeks: vec![
                vec![("a".into(), 3), ("b".into(), 4)],
                vec![("a".into(), 3), ("b".into(), 3)],
                vec![("a".into(), 4), ("b".into(), 4)],
            ],
            ..Default::default()
        };
        let h = habits(&input, date!(2026 - 11 - 10));
        // a averages 3.33 -> 3 of 5; b averages 3.67 -> 4 of 4 (on target).
        assert_eq!(find(&h, "diet_week").unwrap().why, "Home-cooked dinner is at 3 of 5 days lately.");
    }

    #[test]
    fn eating_out_by_share_and_by_cap() {
        let today = date!(2026 - 11 - 09);
        let mut input = DietIn {
            eating_out_90d_paise: 450_000,
            groceries_90d_paise: 550_000,
            delivery_90d_orders: 26,
            eating_out_last_month_paise: 150_000,
            eating_out_month_paise: 10_000,
            day: 9,
            days_in_month: 30,
            ..Default::default()
        };
        // 45% of food, 26 orders in 90 days is about 2 a week.
        let h = habits(&input, today);
        let e = find(&h, "eating_out").unwrap();
        assert_eq!(e.title, "Order in at most 1 time this week");
        assert_eq!(e.why, "Eating out and delivery was ₹1,500 last month, 45% of food.");
        assert_eq!((e.cadence, e.area, e.impact, e.status), (Cadence::Weekly, Area::Food, Impact::Medium, Status::Ok));

        input.delivery_90d_orders = 52; // 4 a week
        assert_eq!(find(&habits(&input, today), "eating_out").unwrap().title, "Order in at most 3 times this week");

        // Under 40% and no cap: nothing.
        input.eating_out_90d_paise = 300_000;
        input.groceries_90d_paise = 700_000;
        assert!(find(&habits(&input, today), "eating_out").is_none());

        // A cap of 3000, day 9 of 30 allows 900; 1,200 is over 1.15x.
        input.cap_paise = Some(300_000);
        input.eating_out_month_paise = 120_000;
        let h = habits(&input, today);
        let e = find(&h, "eating_out").unwrap();
        assert_eq!(e.why, "₹1,200 so far this month against a ₹3,000 cap.");
        assert_eq!(e.status, Status::DueSoon);
        input.eating_out_month_paise = 100_000;
        assert!(find(&habits(&input, today), "eating_out").is_none());
    }

    #[test]
    fn nothing_here_is_high_impact() {
        let input = DietIn {
            habits: vec![habit_in("a", "x", 3)],
            week_start: Some(date!(2026 - 11 - 09)),
            eating_out_90d_paise: 900_000,
            groceries_90d_paise: 100_000,
            ..Default::default()
        };
        assert!(habits(&input, date!(2026 - 11 - 15)).iter().all(|h| h.impact != Impact::High));
    }

    fn report() -> DietReport {
        DietReport {
            eating_out_paise: 310_000,
            groceries_paise: 690_000,
            prev_eating_out_paise: Some(440_000),
            prev_groceries_paise: Some(560_000),
            eating_out_share_bps: Some(3_100),
            delivery_per_week_x100: 350,
            weeks_logged: 10,
            weeks_in_period: 13,
            habits: vec![DietHabitRate {
                label: "Home-cooked dinner".into(),
                target_days: 5,
                hit_bps: Some(8_000),
                weeks_on_target: 9,
                weeks_logged: 10,
            }],
            weight_x10: Some(700),
            weight_change_x10: Some(-12),
            checkup_due_on: Some("2026-03-12".into()),
            ..Default::default()
        }
    }

    #[test]
    fn notes_say_it_plainly_and_cap_at_two() {
        let mut r = report();
        add_notes(&mut r, Some(200), date!(2026 - 11 - 09));
        assert_eq!(
            r.wins,
            vec![
                "Eating out fell to 31% of food, from 44%.".to_string(),
                "Home-cooked dinner hit its target in 9 of 13 weeks.".to_string()
            ]
        );
        assert_eq!(
            r.watch,
            vec![
                "Your yearly check-up is overdue since 12 Mar.".to_string(),
                "Ordered in 3.5 times a week, up from 2.".to_string()
            ]
        );
        for line in r.wins.iter().chain(&r.watch) {
            assert!(!line.contains("kg") && !line.contains('!'), "{line}");
        }
    }

    #[test]
    fn notes_are_quiet_without_a_comparison() {
        let mut r = DietReport { eating_out_share_bps: Some(3_000), ..Default::default() };
        add_notes(&mut r, None, date!(2026 - 11 - 09));
        assert!(r.wins.is_empty() && r.watch.is_empty());

        let mut r = report();
        r.checkup_due_on = Some("2027-01-01".into());
        r.prev_eating_out_paise = None;
        r.prev_groceries_paise = None;
        r.habits[0].weeks_on_target = 2;
        r.habits[0].weeks_logged = 10;
        add_notes(&mut r, None, date!(2026 - 11 - 09));
        assert!(r.wins.is_empty());
        assert_eq!(r.watch, vec!["Home-cooked dinner hit its target in 2 of 10 logged weeks.".to_string()]);
    }

    #[test]
    fn times_formats_hundredths() {
        assert_eq!((times(350), times(200), times(333), times(149)), ("3.5".into(), "2".into(), "3.3".into(), "1.5".into()));
    }
}
