//! Predicted against actual. Each finished check-in keeps a [`Snapshot`] of the forecast as it
//! stood that day. The next check-in sets it against what happened: the money left each month
//! (from the statements) and what each goal really gained (from the saved amounts in the plan).
//! The share of the expected goal money that really arrived (`follow_through_bps`) then tempers
//! the next forecast, so it stops assuming money that never reaches the goals.

use serde::{Deserialize, Serialize};

use crate::planner::checkin::rupees;

/// Days in an average month, for turning a check-in period into months.
const DAYS_PER_MONTH: f64 = 30.44;
/// A goal counts as on pace when it gained at least this share of what was expected.
const ON_PACE_BPS: i64 = 9_000;
/// The follow-through the forecast will use never drops below this, so one quarter with the saved
/// amounts left stale can't wipe out every goal.
pub const MIN_FOLLOW_THROUGH_BPS: i64 = 2_500;
/// Fewest days between snapshot and check-in before a comparison means anything.
const MIN_DAYS: i64 = 28;

/// One goal as the forecast saw it at a check-in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalSnap {
    pub key: String,
    pub label: String,
    pub saved_paise: i64,
    /// What the forecast expected to go to this goal each month.
    pub funded_monthly_paise: i64,
    pub likelihood_bps: i64,
}

/// The forecast at a finished check-in, kept to compare with later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// "YYYY-MM-DD"
    pub made_on: String,
    /// "statements" or "profile".
    pub source: String,
    /// Expected money left each month.
    pub surplus_paise: i64,
    /// The share of that money the forecast assumed would reach the goals (100% until a check-in
    /// has shown otherwise).
    #[serde(default = "full")]
    pub applied_follow_through_bps: i64,
    pub goals: Vec<GoalSnap>,
}

fn full() -> i64 {
    10_000
}

/// One goal: what was expected to be added since the snapshot, and what was.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalCheck {
    pub key: String,
    pub label: String,
    pub expected_paise: i64,
    pub actual_paise: i64,
    pub on_pace: bool,
}

/// The last forecast set against what happened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    /// When the forecast being checked was made.
    pub since: String,
    pub months: f64,
    pub expected_surplus_paise: i64,
    /// Money really left each month over the period, when the statements cover it.
    pub actual_surplus_paise: Option<i64>,
    pub goals: Vec<GoalCheck>,
    /// Share of the expected goal money that arrived, when any was expected and the saved amounts
    /// were updated since. Capped at 100%.
    pub follow_through_bps: Option<i64>,
    /// The share of the monthly surplus the next forecast should assume reaches the goals: what
    /// the checked forecast assumed, times what arrived of it. Never below the floor.
    pub next_follow_through_bps: Option<i64>,
    /// Plain sentences for the report.
    pub lines: Vec<String>,
}

/// What happened since the snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actual {
    /// Days from the snapshot to now.
    pub days: i64,
    /// Money left each month over those days, from the statements.
    pub surplus_paise: Option<i64>,
    /// Saved amount now, per goal key.
    pub saved: Vec<(String, i64)>,
}

/// Sets a snapshot against what happened. `None` when too little time has passed to say.
pub fn compare(snap: &Snapshot, actual: &Actual) -> Option<Calibration> {
    if actual.days < MIN_DAYS {
        return None;
    }
    let months = actual.days as f64 / DAYS_PER_MONTH;
    let goals: Vec<GoalCheck> = snap
        .goals
        .iter()
        .filter_map(|g| {
            let now = actual.saved.iter().find(|(k, _)| *k == g.key)?.1;
            let expected = (g.funded_monthly_paise.max(0) as f64 * months).round() as i64;
            let gained = now - g.saved_paise;
            Some(GoalCheck {
                key: g.key.clone(),
                label: g.label.clone(),
                expected_paise: expected,
                actual_paise: gained,
                on_pace: expected == 0 || gained.saturating_mul(10_000) >= expected.saturating_mul(ON_PACE_BPS),
            })
        })
        .collect();
    let expected_total: i64 = goals.iter().map(|g| g.expected_paise).sum();
    let gained_total: i64 = goals.iter().map(|g| g.actual_paise.max(0)).sum();
    let any_updated = goals.iter().any(|g| g.actual_paise != 0);
    let follow_through_bps = (expected_total > 0 && any_updated).then(|| (gained_total.saturating_mul(10_000) / expected_total).clamp(0, 10_000));

    let mut lines = Vec::new();
    if let Some(a) = actual.surplus_paise {
        let e = snap.surplus_paise;
        lines.push(if e > 0 && (a - e).abs() * 10 <= e {
            format!("You kept about {} a month, close to the {} the forecast expected.", rupees(a), rupees(e))
        } else if a > e {
            format!("You kept {} a month, more than the {} the forecast expected.", rupees(a), rupees(e))
        } else {
            format!("You kept {} a month, less than the {} the forecast expected.", rupees(a), rupees(e))
        });
    }
    if !goals.is_empty() && !any_updated && expected_total > 0 {
        lines.push("None of your saved amounts changed since then. Update them in your plan to see how your goals did.".to_string());
    } else {
        for g in goals.iter().filter(|g| g.expected_paise > 0) {
            let label = g.label.to_lowercase();
            lines.push(if g.on_pace {
                format!("Your {label} goal gained {}, on pace with the {} expected.", rupees(g.actual_paise), rupees(g.expected_paise))
            } else if g.actual_paise <= 0 {
                format!("Nothing reached your {label} goal; the forecast expected {}.", rupees(g.expected_paise))
            } else {
                format!("Your {label} goal gained {} of the {} expected.", rupees(g.actual_paise), rupees(g.expected_paise))
            });
        }
    }
    let next_follow_through_bps =
        follow_through_bps.map(|f| (snap.applied_follow_through_bps.clamp(0, 10_000) * f / 10_000).clamp(MIN_FOLLOW_THROUGH_BPS, 10_000));
    Some(Calibration {
        next_follow_through_bps,
        since: snap.made_on.clone(),
        months: (months * 10.0).round() / 10.0,
        expected_surplus_paise: snap.surplus_paise,
        actual_surplus_paise: actual.surplus_paise,
        goals,
        follow_through_bps,
        lines,
    })
}

/// The monthly money the forecast should assume reaches the goals, given the share from the last
/// check-in (`next_follow_through_bps`). Never more than the surplus, never less than
/// [`MIN_FOLLOW_THROUGH_BPS`] of it.
pub fn tempered_surplus(surplus_paise: i64, follow_through_bps: Option<i64>) -> i64 {
    match follow_through_bps {
        Some(f) if surplus_paise > 0 => surplus_paise.saturating_mul(f.clamp(MIN_FOLLOW_THROUGH_BPS, 10_000)) / 10_000,
        _ => surplus_paise,
    }
}

/// The note the forecast shows when it tempers the surplus.
pub fn tempered_note(follow_through_bps: i64, since: &str) -> String {
    let pct = (follow_through_bps.clamp(MIN_FOLLOW_THROUGH_BPS, 10_000) + 50) / 100;
    format!("Since your check-in on {since}, about {pct}% of the money you had left each month reached your goals. This forecast assumes the same.")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> Snapshot {
        Snapshot {
            made_on: "2026-08-10".into(),
            source: "statements".into(),
            surplus_paise: 4_000_000,
            applied_follow_through_bps: 10_000,
            goals: vec![
                GoalSnap { key: "emergency".into(), label: "Emergency fund".into(), saved_paise: 10_000_000, funded_monthly_paise: 2_000_000, likelihood_bps: 6_900 },
                GoalSnap { key: "retirement".into(), label: "Retirement".into(), saved_paise: 50_000_000, funded_monthly_paise: 1_000_000, likelihood_bps: 9_300 },
            ],
        }
    }

    fn actual(days: i64, emergency: i64, retirement: i64) -> Actual {
        Actual { days, surplus_paise: Some(3_000_000), saved: vec![("emergency".into(), emergency), ("retirement".into(), retirement)] }
    }

    #[test]
    fn too_soon_to_say() {
        assert!(compare(&snap(), &actual(10, 0, 0)).is_none());
    }

    #[test]
    fn measures_each_goal_against_what_was_expected() {
        // 91 days ≈ 3 months: emergency expected ≈ ₹59.8k, retirement ≈ ₹29.9k.
        let c = compare(&snap(), &actual(91, 10_000_000 + 6_000_000, 50_000_000 + 1_000_000)).unwrap();
        assert_eq!(c.months, 3.0);
        let e = &c.goals[0];
        assert!(e.on_pace, "{e:?}");
        let r = &c.goals[1];
        assert!(!r.on_pace && r.actual_paise == 1_000_000, "{r:?}");
        let ft = c.follow_through_bps.unwrap();
        assert!((7_700..=7_900).contains(&ft), "{ft}");
        assert!(c.lines[0].starts_with("You kept ₹30,000 a month, less than the ₹40,000"), "{:?}", c.lines);
        assert!(c.lines.iter().any(|l| l.starts_with("Your retirement goal gained ₹10,000 of the")), "{:?}", c.lines);
    }

    #[test]
    fn stale_saved_amounts_ask_for_an_update_and_do_not_count() {
        let c = compare(&snap(), &actual(91, 10_000_000, 50_000_000)).unwrap();
        assert_eq!(c.follow_through_bps, None);
        assert!(c.lines.iter().any(|l| l.contains("Update them in your plan")), "{:?}", c.lines);
    }

    #[test]
    fn a_tempered_forecast_that_was_met_keeps_its_tempering() {
        // The checked forecast already assumed half; everything it expected arrived, so the next
        // one should still assume half, not jump back to everything.
        let mut s = snap();
        s.applied_follow_through_bps = 5_000;
        let c = compare(&s, &actual(91, 99_000_000, 99_000_000)).unwrap();
        assert_eq!((c.follow_through_bps, c.next_follow_through_bps), (Some(10_000), Some(5_000)));
        // A fifth of a half is below the floor, so the floor holds.
        let c = compare(&s, &actual(91, 10_000_000 + 1_200_000, 50_000_000 + 600_000)).unwrap();
        assert_eq!(c.next_follow_through_bps, Some(MIN_FOLLOW_THROUGH_BPS));
    }

    #[test]
    fn old_snapshots_without_the_share_read_as_full() {
        let j = r#"{"made_on":"2026-08-10","source":"statements","surplus_paise":1,"goals":[]}"#;
        assert_eq!(serde_json::from_str::<Snapshot>(j).unwrap().applied_follow_through_bps, 10_000);
    }

    #[test]
    fn follow_through_never_goes_over_full() {
        let c = compare(&snap(), &actual(91, 99_000_000, 99_000_000)).unwrap();
        assert_eq!(c.follow_through_bps, Some(10_000));
    }

    #[test]
    fn a_goal_that_lost_money_counts_as_nothing_reached() {
        let c = compare(&snap(), &actual(91, 9_000_000, 50_000_000 + 3_000_000)).unwrap();
        assert!(c.lines.iter().any(|l| l.starts_with("Nothing reached your emergency fund goal")), "{:?}", c.lines);
    }

    #[test]
    fn goals_missing_from_the_plan_now_are_skipped() {
        let a = Actual { days: 91, surplus_paise: None, saved: vec![("emergency".into(), 12_000_000)] };
        let c = compare(&snap(), &a).unwrap();
        assert_eq!(c.goals.len(), 1);
        assert!(!c.lines.iter().any(|l| l.starts_with("You kept")));
    }

    #[test]
    fn tempering_scales_and_has_a_floor() {
        assert_eq!(tempered_surplus(4_000_000, Some(5_000)), 2_000_000);
        assert_eq!(tempered_surplus(4_000_000, Some(0)), 1_000_000);
        assert_eq!(tempered_surplus(4_000_000, None), 4_000_000);
        assert_eq!(tempered_surplus(-100, Some(5_000)), -100);
        assert_eq!(tempered_note(4_960, "9 Nov 2026"), "Since your check-in on 9 Nov 2026, about 50% of the money you had left each month reached your goals. This forecast assumes the same.");
    }

    #[test]
    fn snapshots_round_trip_as_json() {
        let s = snap();
        assert_eq!(serde_json::from_str::<Snapshot>(&serde_json::to_string(&s).unwrap()).unwrap(), s);
    }
}
