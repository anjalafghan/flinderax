//! The reward fund: money you pay yourself at the end of a month for looking after four parts of
//! life (money, health, relationships, the future). Pure: one month's signals and the months
//! before it in, a score and a payout out.
//!
//! How a month is scored:
//! 1. Each area turns what you did that month into plain units (`raw`). Using the app counts:
//!    a statement or bank text adds money units, a diet week adds health, a kept date or a call
//!    adds relationships, a goal top-up or a plan adds future. Every source is capped so no single
//!    habit can fill an area.
//! 2. The target for an area is learned from your own months: an average that leans on recent
//!    months (each month moves it 40% of the way), plus a 10% stretch, held between a floor (so
//!    doing nothing never pays) and a ceiling (so a huge month can't make the next impossible).
//!    Meeting your usual level earns about 90%; beating it a little earns all of it.
//! 3. Areas are weighted money 40%, the others 20% each, into one score.
//! 4. The payout is the score times ₹2,000, but never more than you actually came in under your
//!    spending allowance that month: a treat is only paid from money really saved. What you don't
//!    earn stays saved; nothing rolls over.

use serde::Serialize;

/// Most a single month can pay (₹2,000).
pub const MAX_PAYOUT_PAISE: i64 = 200_000;
/// Payouts are whole ₹10 steps.
const ROUND_PAISE: i64 = 1_000;
/// Each month moves the learned level this share of the way (percent).
const LEARN_PCT: i64 = 40;
/// The stretch on top of your usual level (percent).
const STRETCH_PCT: i64 = 110;
/// Months of history the learning looks at.
pub const HISTORY_MONTHS: usize = 6;
const FULL: i64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Area {
    Money,
    Health,
    Relationships,
    Future,
}

pub const AREAS: [Area; 4] = [Area::Money, Area::Health, Area::Relationships, Area::Future];

impl Area {
    /// Share of the score, in basis points (they add up to 10,000).
    pub fn weight_bps(self) -> i64 {
        match self {
            Area::Money => 4_000,
            _ => 2_000,
        }
    }

    /// (starting target, floor, ceiling) in units.
    fn bounds(self) -> (i64, i64, i64) {
        match self {
            Area::Money => (60, 20, 200),
            Area::Health => (20, 8, 60),
            Area::Relationships => (20, 8, 60),
            Area::Future => (15, 6, 50),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Area::Money => "Money",
            Area::Health => "Health",
            Area::Relationships => "Relationships",
            Area::Future => "Future",
        }
    }

    /// The easiest next step that raises this area, said plainly.
    pub fn tip(self) -> &'static str {
        match self {
            Area::Money => "Import a statement or log today's spending; staying under your allowance counts too.",
            Area::Health => "Log this week's diet habits, or add a weight or waist reading.",
            Area::Relationships => "Keep a date from the jar, call someone, or tick off a follow-up.",
            Area::Future => "Put a little into a goal, or update a plan on the Life page.",
        }
    }
}

/// What one month held, counted by the handler. Counts are of things done that month.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MonthSignals {
    // money
    /// Activity points (statements, bank texts, manual entries, check-ins), already capped per day.
    pub activity_points: i64,
    /// Days with at least one of those.
    pub active_days: i64,
    pub has_transactions: bool,
    pub spent_paise: i64,
    pub allowance_paise: i64,
    // health
    pub diet_weeks: i64,
    /// Share of habit days hit across the logged weeks; None without any.
    pub habit_hit_bps: Option<i64>,
    pub health_marks: i64,
    pub checkups: i64,
    // relationships
    /// Kept dates whose day fell in the month and has passed.
    pub dates_kept: i64,
    /// Slips drawn from the jar.
    pub jar_draws: i64,
    pub touches: i64,
    pub followups_done: i64,
    pub gifts_given: i64,
    // future
    /// Days with a top-up into a goal.
    pub goal_days: i64,
    /// What went into goals, net of anything taken out.
    pub goal_net_paise: i64,
    pub checkins: i64,
    /// Goals, life choices, planned outflows and what-if drafts added or changed.
    pub plans_touched: i64,
}

fn under_budget(s: &MonthSignals) -> bool {
    s.has_transactions && s.allowance_paise > 0 && s.spent_paise <= s.allowance_paise
}

/// Units for one area. Every source is capped, so one habit alone can't fill an area.
pub fn raw(area: Area, s: &MonthSignals) -> i64 {
    let c = |v: i64, cap: i64| v.clamp(0, cap);
    match area {
        Area::Money => c(s.activity_points, 120) + 2 * c(s.active_days, 31) + if under_budget(s) { 20 } else { 0 },
        Area::Health => {
            // A logged week is worth 5, plus up to 5 more for how many habit days it hit.
            let quality = s.habit_hit_bps.unwrap_or(0).clamp(0, FULL);
            let weeks = c(s.diet_weeks, 6);
            weeks * 5 + weeks * 5 * quality / FULL + 3 * c(s.health_marks, 4) + 10 * c(s.checkups, 1)
        }
        Area::Relationships => {
            10 * c(s.dates_kept, 4) + c(s.jar_draws, 5) + 3 * c(s.touches, 8) + 3 * c(s.followups_done, 6) + 5 * c(s.gifts_given, 3)
        }
        Area::Future => {
            let saving = if s.goal_net_paise > 0 { 4 * c(s.goal_days, 6) } else { 0 };
            saving + 10 * c(s.checkins, 1) + 3 * c(s.plans_touched, 5)
        }
    }
}

/// The learned target for an area from the months before (oldest first). Starts at the area's
/// default and leans each month 40% toward what you really did, then adds the stretch.
pub fn target(area: Area, history: &[i64]) -> i64 {
    let (start, floor, ceil) = area.bounds();
    let skip = history.len().saturating_sub(HISTORY_MONTHS);
    // Learned level in hundredths of a unit, so integer steps don't stall.
    let mut level = start * 100;
    for &r in &history[skip..] {
        let r = r.clamp(0, ceil * 2) * 100;
        level += (r - level) * LEARN_PCT / 100;
    }
    (level * STRETCH_PCT / 100 / 100).clamp(floor, ceil)
}

/// Score of an area in basis points: what you did against its target, at most 100%.
pub fn area_bps(raw: i64, target: i64) -> i64 {
    if target <= 0 {
        return 0;
    }
    (i128::from(raw.max(0)) * i128::from(FULL) / i128::from(target)).min(i128::from(FULL)) as i64
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AreaScore {
    pub area: Area,
    pub label: &'static str,
    pub weight_bps: i64,
    pub raw: i64,
    pub target: i64,
    pub score_bps: i64,
    pub tip: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonthScore {
    pub areas: Vec<AreaScore>,
    /// The weighted score, in basis points.
    pub score_bps: i64,
    /// What the score alone would pay.
    pub earned_paise: i64,
    /// How far under the allowance spending came (0 when over, or without transactions).
    pub saved_paise: i64,
    /// What is really paid: the earned amount, held to what was saved.
    pub payout_paise: i64,
    /// Earned but not paid because not enough was saved.
    pub held_back_paise: i64,
    /// The area worth most to improve next: biggest weighted gap.
    pub focus: Option<Area>,
}

fn round_down(p: i64) -> i64 {
    p.max(0) / ROUND_PAISE * ROUND_PAISE
}

/// Scores one month. `history[i]` holds the earlier months' raw units for area `AREAS[i]`.
pub fn score(s: &MonthSignals, history: &[Vec<i64>; 4]) -> MonthScore {
    let areas: Vec<AreaScore> = AREAS
        .iter()
        .zip(history.iter())
        .map(|(&area, hist)| {
            let raw = raw(area, s);
            let target = target(area, hist);
            AreaScore { area, label: area.label(), weight_bps: area.weight_bps(), raw, target, score_bps: area_bps(raw, target), tip: area.tip() }
        })
        .collect();
    let score_bps: i64 = areas.iter().map(|a| a.weight_bps * a.score_bps / FULL).sum::<i64>().clamp(0, FULL);
    let earned = round_down(MAX_PAYOUT_PAISE * score_bps / FULL);
    let saved = if s.has_transactions { (s.allowance_paise - s.spent_paise).max(0) } else { 0 };
    let payout = round_down(earned.min(saved));
    let focus = areas
        .iter()
        .filter(|a| a.score_bps < FULL)
        .max_by_key(|a| (a.weight_bps * (FULL - a.score_bps), a.weight_bps))
        .map(|a| a.area);
    MonthScore { areas, score_bps, earned_paise: earned, saved_paise: saved, payout_paise: payout, held_back_paise: earned - payout, focus }
}

/// The wallet: everything paid in, less what treats bought from it. Never below zero.
pub fn wallet(paid_in: i64, spent: i64) -> i64 {
    paid_in.saturating_sub(spent).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> [Vec<i64>; 4] {
        [vec![], vec![], vec![], vec![]]
    }

    fn busy() -> MonthSignals {
        MonthSignals {
            activity_points: 80,
            active_days: 20,
            has_transactions: true,
            spent_paise: 3_000_000,
            allowance_paise: 4_000_000,
            diet_weeks: 4,
            habit_hit_bps: Some(7_500),
            health_marks: 2,
            checkups: 0,
            dates_kept: 2,
            jar_draws: 3,
            touches: 4,
            followups_done: 1,
            gifts_given: 0,
            goal_days: 3,
            goal_net_paise: 500_000,
            checkins: 0,
            plans_touched: 2,
        }
    }

    #[test]
    fn doing_nothing_pays_nothing() {
        let s = score(&MonthSignals::default(), &none());
        assert_eq!((s.score_bps, s.earned_paise, s.payout_paise), (0, 0, 0));
        assert_eq!(s.focus, Some(Area::Money));
    }

    #[test]
    fn a_full_month_pays_the_most_and_no_more() {
        let s = score(&busy(), &none());
        assert_eq!(s.score_bps, FULL);
        assert_eq!(s.payout_paise, MAX_PAYOUT_PAISE);
        assert_eq!(s.focus, None);
    }

    #[test]
    fn the_payout_never_exceeds_what_was_really_saved() {
        let mut m = busy();
        m.spent_paise = m.allowance_paise - 123_456; // ₹1,234.56 under
        let s = score(&m, &none());
        assert_eq!(s.payout_paise, 123_000);
        assert_eq!(s.held_back_paise, MAX_PAYOUT_PAISE - 123_000);
        m.spent_paise = m.allowance_paise + 1; // over: nothing saved, nothing paid
        assert_eq!(score(&m, &none()).payout_paise, 0);
        m.has_transactions = false; // no data: nothing known saved
        m.spent_paise = 0;
        assert_eq!(score(&m, &none()).payout_paise, 0);
    }

    #[test]
    fn weights_are_money_heavy() {
        assert_eq!(AREAS.iter().map(|a| a.weight_bps()).sum::<i64>(), FULL);
        let money_only = MonthSignals { activity_points: 120, active_days: 31, has_transactions: true, spent_paise: 0, allowance_paise: 1, ..Default::default() };
        assert_eq!(score(&money_only, &none()).score_bps, 4_000);
    }

    #[test]
    fn the_target_learns_from_your_months_and_stays_in_bounds() {
        assert_eq!(target(Area::Money, &[]), 66); // 60 + 10% stretch
        // A steady 100 a month pulls the target up toward 110.
        let steady = target(Area::Money, &[100; 6]);
        assert!((100..=110).contains(&steady), "{steady}");
        // Quiet months bring it down, but never under the floor.
        assert_eq!(target(Area::Money, &[0; 6]), 20);
        assert_eq!(target(Area::Health, &[1_000_000; 6]), 60);
        // Only the last six months count.
        assert_eq!(target(Area::Money, &[[0; 20].as_slice(), &[100; 6]].concat()), steady);
        // Recent months weigh more than old ones.
        assert!(target(Area::Future, &[0, 0, 0, 30, 30, 30]) > target(Area::Future, &[30, 30, 30, 0, 0, 0]));
    }

    #[test]
    fn matching_your_usual_month_earns_most_but_not_all() {
        let usual = raw(Area::Money, &busy());
        let t = target(Area::Money, &[usual; 6]);
        let bps = area_bps(usual, t);
        assert!((8_500..FULL).contains(&bps), "{bps}");
    }

    #[test]
    fn every_source_is_capped() {
        let huge = MonthSignals {
            activity_points: i64::MAX,
            active_days: i64::MAX,
            diet_weeks: i64::MAX,
            habit_hit_bps: Some(i64::MAX),
            health_marks: i64::MAX,
            checkups: i64::MAX,
            dates_kept: i64::MAX,
            jar_draws: i64::MAX,
            touches: i64::MAX,
            followups_done: i64::MAX,
            gifts_given: i64::MAX,
            goal_days: i64::MAX,
            goal_net_paise: i64::MAX,
            checkins: i64::MAX,
            plans_touched: i64::MAX,
            has_transactions: true,
            spent_paise: i64::MIN / 2,
            allowance_paise: i64::MAX / 2,
        };
        for a in AREAS {
            assert!(raw(a, &huge) < 400, "{a:?}");
        }
        assert_eq!(score(&huge, &none()).payout_paise, MAX_PAYOUT_PAISE);
        // Negative counts are treated as nothing.
        let neg = MonthSignals { activity_points: -50, touches: -3, goal_days: -1, habit_hit_bps: Some(-9), diet_weeks: 2, ..Default::default() };
        assert_eq!(raw(Area::Money, &neg), 0);
        assert_eq!(raw(Area::Health, &neg), 10);
    }

    #[test]
    fn goal_days_only_count_when_money_really_went_in() {
        let mut m = MonthSignals { goal_days: 5, goal_net_paise: 0, ..Default::default() };
        assert_eq!(raw(Area::Future, &m), 0);
        m.goal_net_paise = 1;
        assert_eq!(raw(Area::Future, &m), 20);
    }

    #[test]
    fn the_focus_is_the_biggest_weighted_gap() {
        let mut m = busy();
        m.diet_weeks = 0;
        m.health_marks = 0;
        assert_eq!(score(&m, &none()).focus, Some(Area::Health));
    }

    #[test]
    fn the_wallet_never_goes_negative() {
        assert_eq!(wallet(200_000, 50_000), 150_000);
        assert_eq!(wallet(0, 1), 0);
        assert_eq!(wallet(i64::MIN, i64::MAX), 0);
    }
}
