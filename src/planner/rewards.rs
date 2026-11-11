//! Rewards: whether a reward is earned yet, and whether claiming it fits the monthly cap.
//! Pure: saved money, streak runs and totals in, a state out.

use serde::Serialize;

/// Shares of a goal that can carry a reward (percent of the target).
pub const GOAL_PCTS: [i64; 4] = [25, 50, 75, 100];
/// Longest streak target a reward may ask for (ten years of days).
pub const MAX_STREAK_TARGET: i64 = 3650;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RewardState {
    /// Not reached yet.
    Locked,
    /// Reached, waiting to be enjoyed.
    Earned,
    Claimed,
}

/// `saved` has reached `pct` percent of `target`. Compares by cross-multiplying in i128 so a huge
/// target can't overflow; a non-positive target never counts as reached.
pub fn goal_reached(saved: i64, target: i64, pct: i64) -> bool {
    target > 0 && i128::from(saved) * 100 >= i128::from(target) * i128::from(pct)
}

/// A streak target is met by the best run so far (a run that later broke still earned it).
pub fn streak_reached(best: u32, target: i64) -> bool {
    target > 0 && i64::from(best) >= target
}

pub fn state(reached: bool, claimed: bool) -> RewardState {
    match (claimed, reached) {
        (true, _) => RewardState::Claimed,
        (false, true) => RewardState::Earned,
        (false, false) => RewardState::Locked,
    }
}

/// How much of the month's cap is left: `None` = no cap. Saturating, never negative.
pub fn cap_left(cap: Option<i64>, spent_this_month: i64) -> Option<i64> {
    cap.map(|c| c.saturating_sub(spent_this_month).max(0))
}

/// Whether a claim of `cost` fits what is left of the cap.
pub fn fits_cap(cap: Option<i64>, spent_this_month: i64, cost: i64) -> bool {
    cap_left(cap, spent_this_month).is_none_or(|left| cost <= left)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_shares() {
        assert!(!goal_reached(24, 100, 25));
        assert!(goal_reached(25, 100, 25));
        assert!(goal_reached(100, 100, 100));
        assert!(!goal_reached(99, 100, 100));
        assert!(goal_reached(150, 100, 100)); // over-saved still counts
        assert!(!goal_reached(-5, 100, 25)); // money taken back out
        assert!(!goal_reached(5, 0, 25));
    }

    #[test]
    fn extreme_amounts_do_not_overflow() {
        assert!(goal_reached(i64::MAX, i64::MAX, 100));
        assert!(!goal_reached(i64::MAX - 1, i64::MAX, 100));
        assert!(!goal_reached(i64::MIN, i64::MAX, 25));
    }

    #[test]
    fn streaks() {
        assert!(streak_reached(7, 7));
        assert!(!streak_reached(6, 7));
        assert!(!streak_reached(0, 0));
    }

    #[test]
    fn states() {
        assert_eq!(state(false, false), RewardState::Locked);
        assert_eq!(state(true, false), RewardState::Earned);
        assert_eq!(state(true, true), RewardState::Claimed);
        assert_eq!(state(false, true), RewardState::Claimed);
    }

    #[test]
    fn cap() {
        assert!(fits_cap(None, i64::MAX, i64::MAX));
        assert!(fits_cap(Some(500), 200, 300));
        assert!(!fits_cap(Some(500), 200, 301));
        assert!(!fits_cap(Some(500), 900, 1)); // already over: nothing fits
        assert!(fits_cap(Some(0), 0, 0)); // a free treat always fits
        assert_eq!(cap_left(Some(i64::MIN), i64::MAX), Some(0));
    }
}
