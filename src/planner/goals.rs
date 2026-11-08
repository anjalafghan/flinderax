//! Goals: an emergency fund, a trip, a gadget, the date jar. Each month the savings left after
//! the SIP are shared out by priority, and a goal that can't make its date says what would fix it.
//! Pure: amounts and dates in, a plan out.

use serde::Serialize;
use time::Date;

use super::cycle::add_months;
use crate::validate;

#[derive(Debug, Clone)]
pub struct GoalIn {
    pub id: String,
    pub name: String,
    pub target: i64,
    pub saved: i64,
    /// `None` = no deadline (it gets whatever is left after dated goals).
    pub by: Option<Date>,
    /// 1 = first.
    pub priority: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Done,
    OnTrack,
    Behind,
    /// No deadline: fills from what's left.
    Open,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoalPlan {
    pub id: String,
    pub name: String,
    pub target_paise: i64,
    pub saved_paise: i64,
    /// What this month's savings put towards it.
    pub this_month_paise: i64,
    /// What it needs each month from now to make its date.
    pub needs_monthly_paise: i64,
    pub months_left: i64,
    pub status: GoalStatus,
    /// For a goal that's behind: the month it would be reached at this month's pace.
    pub reached_by: Option<Date>,
    /// For a goal that's behind: what to do about it, plainly.
    pub fixes: Vec<String>,
}

/// Whole months from `today` to `by`, counting the current month (so a goal due this month has 1).
pub fn months_until(today: Date, by: Date) -> i64 {
    let m = i64::from(by.year() - today.year()) * 12 + (i64::from(by.month() as u8) - i64::from(today.month() as u8));
    (m + 1).max(1)
}

/// `today` moved `months` forward. Callers keep `months` within `validate::MAX_MONTHS`.
fn add(today: Date, months: i64) -> Date {
    let (y, m) = add_months(today.year(), today.month(), months as i32);
    super::cycle::clamped(y, m, today.day())
}

/// Shares `available` (this month's savings after the SIP) across `goals` by priority: dated
/// goals get what they need in priority order, then undated ones share what's left in order.
/// All arithmetic saturates, so an absurd target can't wrap into negative money.
pub fn plan(goals: &[GoalIn], available: i64, today: Date, money: impl Fn(i64) -> String) -> Vec<GoalPlan> {
    let mut order: Vec<&GoalIn> = goals.iter().collect();
    order.sort_by_key(|g| (g.by.is_none(), g.priority, g.by));
    let mut left = available.max(0);
    let mut out = Vec::new();
    for g in order {
        let remaining = g.target.saturating_sub(g.saved).max(0);
        let months_left = g.by.map_or(0, |by| months_until(today, by));
        // Undated goals need nothing up front: they share what's left after the dated ones.
        // Ceiling division written so that it cannot overflow: ceil(r / m) = (r - 1) / m + 1.
        let needs = if remaining == 0 || g.by.is_none() { 0 } else { (remaining - 1) / months_left + 1 };
        let give = needs.min(left);
        left -= give;
        let status = match (remaining, g.by) {
            (0, _) => GoalStatus::Done,
            (_, None) => GoalStatus::Open,
            _ if give >= needs => GoalStatus::OnTrack,
            _ => GoalStatus::Behind,
        };
        let mut reached_by = None;
        let mut fixes = Vec::new();
        if status == GoalStatus::Behind {
            if give > 0 {
                // Months from now at this pace; `remaining >= 1` here, so this is at least 1.
                let months_away = (remaining - 1) / give;
                // Beyond the planner's horizon there is no meaningful date to name.
                if months_away <= validate::MAX_MONTHS {
                    let when = add(today, months_away);
                    reached_by = Some(when);
                    fixes.push(format!("Move the date to {}", short_month(when)));
                }
            }
            fixes.push(format!("Put in {} more a month", money(needs - give)));
            fixes.push("Take it from a goal further down the list".to_string());
        }
        out.push(GoalPlan {
            id: g.id.clone(),
            name: g.name.clone(),
            target_paise: g.target,
            saved_paise: g.saved,
            this_month_paise: give,
            needs_monthly_paise: needs,
            months_left,
            status,
            reached_by,
            fixes,
        });
    }
    // Undated goals share what's still left, in priority order, a remaining-sized slice each.
    for p in out.iter_mut().filter(|p| p.status == GoalStatus::Open) {
        let give = p.target_paise.saturating_sub(p.saved_paise).max(0).min(left);
        p.this_month_paise = give;
        left -= give;
    }
    out
}

fn short_month(d: Date) -> String {
    const MON: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    format!("{} {}", MON[d.month() as usize - 1], d.year())
}

/// How many months of needs the savings cover (one decimal, in tenths: 42 = 4.2 months).
pub fn months_covered_tenths(savings: i64, monthly_needs: i64) -> Option<i64> {
    (monthly_needs > 0).then(|| savings.max(0) * 10 / monthly_needs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn g(id: &str, target: i64, saved: i64, by: Option<Date>, priority: i64) -> GoalIn {
        GoalIn { id: id.into(), name: id.into(), target: target * 100, saved: saved * 100, by, priority }
    }

    fn rupees(p: i64) -> String {
        format!("₹{}", p / 100)
    }

    #[test]
    fn months_count_the_current_month() {
        assert_eq!(months_until(date!(2026 - 11 - 08), date!(2026 - 11 - 30)), 1);
        assert_eq!(months_until(date!(2026 - 11 - 08), date!(2027 - 04 - 01)), 6);
    }

    #[test]
    fn dated_goals_get_what_they_need_by_priority() {
        let goals = [
            g("trip", 60_000, 0, Some(date!(2027 - 04 - 01)), 2), // needs 10,000 a month
            g("phone", 30_000, 0, Some(date!(2027 - 01 - 31)), 1), // needs 10,000 a month
            g("emergency", 300_000, 100_000, None, 3),
        ];
        let p = plan(&goals, 25_000 * 100, date!(2026 - 11 - 08), rupees);
        let by_id = |id: &str| p.iter().find(|x| x.id == id).unwrap();
        assert_eq!((by_id("phone").this_month_paise, by_id("phone").status), (1_000_000, GoalStatus::OnTrack));
        assert_eq!((by_id("trip").this_month_paise, by_id("trip").status), (1_000_000, GoalStatus::OnTrack));
        assert_eq!((by_id("emergency").this_month_paise, by_id("emergency").status), (500_000, GoalStatus::Open));
    }

    #[test]
    fn a_goal_that_cant_make_it_says_what_would() {
        let goals = [g("trip", 60_000, 0, Some(date!(2027 - 04 - 01)), 1)];
        let p = plan(&goals, 6_000 * 100, date!(2026 - 11 - 08), rupees);
        assert_eq!(p[0].status, GoalStatus::Behind);
        assert_eq!(p[0].reached_by, Some(date!(2027 - 08 - 08)));
        assert_eq!(p[0].fixes, ["Move the date to Aug 2027", "Put in ₹4000 more a month", "Take it from a goal further down the list"]);
    }

    #[test]
    fn behind_goal_with_tiny_give_does_not_panic() {
        // 1 paisa a month towards a huge target: the date would be millennia away, so no date is named.
        let goals = [GoalIn { id: "x".into(), name: "x".into(), target: i64::MAX, saved: 0, by: Some(date!(2027 - 04 - 01)), priority: 1 }];
        let p = plan(&goals, 1, date!(2026 - 11 - 08), rupees);
        assert_eq!(p[0].status, GoalStatus::Behind);
        assert_eq!(p[0].reached_by, None);
        assert_eq!(p[0].this_month_paise, 1);
        assert!(!p[0].fixes.iter().any(|f| f.starts_with("Move the date")));
    }

    #[test]
    fn i64_max_target_does_not_overflow_or_hand_out_phantom_money() {
        let goals = [
            GoalIn { id: "huge".into(), name: "huge".into(), target: i64::MAX, saved: 0, by: Some(date!(2027 - 01 - 01)), priority: 1 },
            GoalIn { id: "phone".into(), name: "phone".into(), target: 3_000_000, saved: 0, by: Some(date!(2027 - 01 - 31)), priority: 2 },
            GoalIn { id: "undated".into(), name: "undated".into(), target: i64::MAX, saved: -5, by: None, priority: 3 },
        ];
        let available = 2_500_000;
        let p = plan(&goals, available, date!(2026 - 11 - 08), rupees);
        let total: i64 = p.iter().map(|x| x.this_month_paise).sum();
        assert!(p.iter().all(|x| x.this_month_paise >= 0 && x.needs_monthly_paise >= 0));
        assert!(total <= available, "handed out {total} from {available}");
        // The huge goal takes everything; the goals behind it get nothing extra.
        assert_eq!(p[0].this_month_paise, available);
        assert_eq!(p[1].this_month_paise, 0);
        assert_eq!(p[2].this_month_paise, 0);
    }

    #[test]
    fn done_goals_take_nothing() {
        let p = plan(&[g("x", 100, 100, Some(date!(2027 - 01 - 01)), 1)], 50_000, date!(2026 - 11 - 08), rupees);
        assert_eq!((p[0].status, p[0].this_month_paise), (GoalStatus::Done, 0));
    }

    #[test]
    fn emergency_cover() {
        assert_eq!(months_covered_tenths(210_000, 50_000), Some(42));
        assert_eq!(months_covered_tenths(1, 0), None);
    }
}
