//! Goal forecast: a Monte Carlo over the goals of the life plan. Pure: no DB, no clock, no `rand`.
//! Each run uses one fixed-seed PRNG, so the same input always gives the same output.
//! Money is integer paise at the edges and f64 paise inside the simulation; rates are basis points.

use serde::Serialize;
use time::{Date, Month};

use super::life::{future_value, inr, DEBT_RETURN_BPS, EQUITY_RETURN_BPS, GENERAL_INFLATION_BPS, HYBRID_RETURN_BPS};
use crate::validate::MAX_AMOUNT_PAISE;

/// Paths for the forecast the user sees.
pub const DEFAULT_PATHS: u32 = 1_000;
/// Paths for each lever search. Fewer paths keep the search fast; the same seed keeps it stable.
const LEVER_PATHS: u32 = 300;
const MAX_PATHS: u32 = 10_000;
const MAX_HORIZON_MONTHS: i64 = 600;
const LIKELY_BPS: i64 = 8_000;
const POSSIBLE_BPS: i64 = 5_000;
const SEED: u64 = 0x5EED_F0CA_5701_0003;
const MONTHS_PER_YEAR: f64 = 12.0;
/// Annual volatility per mix, in bps. The returns themselves are the life plan's constants.
const EQUITY_VOL_BPS: i64 = 1_600;
const HYBRID_VOL_BPS: i64 = 900;
const DEBT_VOL_BPS: i64 = 150;
/// Inflation used to push a target back by `push_date`. Set to 8% a year for every goal.
const GOAL_INFLATION_BPS: i64 = 800;
const SAVE_STEP_PAISE: i64 = 50_000;
/// ₹10,00,000 a month: the most `save_more` will ask for.
const SAVE_CAP_PAISE: i64 = 100_000_000;
const MAX_DELAY_MONTHS: i64 = 60;
const TRIM_TOP_PCT: i64 = 95;
const TRIM_BOTTOM_PCT: i64 = 50;
const TRIM_STEP_PCT: usize = 5;
const MAX_LEVERS: usize = 3;

/// A goal as the forecast sees it. `months` is the horizon: months to `when`, or the plan's
/// horizon for a goal with no date.
#[derive(Clone, Debug, PartialEq)]
pub struct GoalIn {
    pub key: &'static str,
    pub label: &'static str,
    pub when: Option<Date>,
    pub months: i64,
    pub target_paise: i64,
    pub saved_paise: i64,
    pub needed_monthly_paise: i64,
    /// "debt", "hybrid" or "equity". Anything else is treated as debt, the safest return.
    pub mix: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForecastInput {
    pub today: Date,
    pub surplus_paise: i64,
    pub surplus_sd_paise: i64,
    pub goals: Vec<GoalIn>,
    pub paths: u32,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Likely,
    Possible,
    Unlikely,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LeverKind {
    SaveMore,
    PushDate,
    TrimTarget,
    CutSpend,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Lever {
    pub kind: LeverKind,
    pub text: String,
    pub monthly_paise: Option<i64>,
    pub months: Option<i64>,
    pub target_paise: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct GoalForecast {
    pub key: &'static str,
    pub label: &'static str,
    pub when: Option<String>,
    pub target_paise: i64,
    pub saved_paise: i64,
    pub needed_monthly_paise: i64,
    pub funded_monthly_paise: i64,
    pub likelihood_bps: i64,
    pub p10_paise: i64,
    pub p50_paise: i64,
    pub p90_paise: i64,
    pub status: Status,
    pub levers: Vec<Lever>,
}

/// What the forecast is built on: the statements' averages, or the plan's own numbers.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Basis {
    pub source: &'static str,
    pub months_used: i64,
    pub income_paise: i64,
    pub spend_paise: i64,
    pub invested_paise: i64,
    pub surplus_paise: i64,
    pub surplus_sd_paise: i64,
    pub entered_income_paise: i64,
    pub note: Option<String>,
}

/// The whole forecast, as the API returns it.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Forecast {
    pub basis: Basis,
    pub goals: Vec<GoalForecast>,
    pub path: Vec<YearBand>,
    pub summary: String,
    pub saved_profile: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct YearBand {
    pub year: i64,
    pub p10_paise: i64,
    pub p50_paise: i64,
    pub p90_paise: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForecastOut {
    pub goals: Vec<GoalForecast>,
    pub path: Vec<YearBand>,
    pub summary: String,
}

/// A goal inside the simulation, in f64 paise. `extra` is a monthly amount added on top of the
/// goal's allocation, used only by the `save_more` search.
#[derive(Clone, Copy, Debug)]
struct Slot {
    months: usize,
    target: i64,
    saved: i64,
    need: f64,
    mu: f64,
    sigma: f64,
    extra: f64,
}

impl Slot {
    /// Already covered by what is saved: reached in every path.
    fn done(&self) -> bool {
        self.saved >= self.target
    }
}

/// splitmix64: a small, deterministic generator. Normal draws come from Box–Muller.
struct Rng {
    state: u64,
    spare: Option<f64>,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed, spare: None }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in the open interval (0, 1).
    fn uniform(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64 + 0.5) / 9_007_199_254_740_992.0
    }

    /// Standard normal.
    fn normal(&mut self) -> f64 {
        if let Some(z) = self.spare.take() {
            return z;
        }
        let u1 = self.uniform();
        let u2 = self.uniform();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = std::f64::consts::TAU * u2;
        self.spare = Some(r * theta.sin());
        r * theta.cos()
    }
}

fn mix_params(mix: &str) -> (i64, i64) {
    match mix {
        "equity" => (EQUITY_RETURN_BPS, EQUITY_VOL_BPS),
        "hybrid" => (HYBRID_RETURN_BPS, HYBRID_VOL_BPS),
        _ => (DEBT_RETURN_BPS, DEBT_VOL_BPS),
    }
}

fn slots_for(goals: &[GoalIn]) -> Vec<Slot> {
    goals
        .iter()
        .map(|g| {
            let (mu_bps, vol_bps) = mix_params(g.mix);
            Slot {
                months: g.months.clamp(1, MAX_HORIZON_MONTHS) as usize,
                target: g.target_paise.clamp(0, MAX_AMOUNT_PAISE),
                saved: g.saved_paise.clamp(0, MAX_AMOUNT_PAISE),
                need: g.needed_monthly_paise.max(0) as f64,
                mu: mu_bps as f64 / 10_000.0 / MONTHS_PER_YEAR,
                sigma: vol_bps as f64 / 10_000.0 / MONTHS_PER_YEAR.sqrt(),
                extra: 0.0,
            }
        })
        .collect()
}

/// Surplus growth factor for month `m`: income rises with inflation.
fn growth_at(m: usize) -> f64 {
    (1.0 + GENERAL_INFLATION_BPS as f64 / 10_000.0).powf(m as f64 / MONTHS_PER_YEAR)
}

/// Splits this month's contribution among the goals still open (`m <= months`).
///
/// Each open goal first asks for its monthly need plus whatever it fell behind in earlier lean
/// months (`behind`), so a good month makes up for a bad one, the way a person catches up. When the
/// contribution covers every ask, the rest is shared in proportion to need too: the forecast assumes
/// what you keep goes to your goals. When every open goal needs nothing, it is shared equally.
/// `extra` (a lever's added saving) goes on top.
fn allocate(contrib: f64, slots: &[Slot], behind: &[f64], m: usize, out: &mut [f64]) {
    let mut ask_total = 0.0;
    let mut need_total = 0.0;
    let mut open = 0usize;
    for (s, b) in slots.iter().zip(behind).filter(|(s, _)| m <= s.months) {
        ask_total += s.need + b;
        need_total += s.need;
        open += 1;
    }
    for ((o, s), b) in out.iter_mut().zip(slots).zip(behind) {
        *o = if m > s.months {
            0.0
        } else if ask_total <= 0.0 {
            contrib / open as f64 + s.extra
        } else if contrib >= ask_total {
            let rest = contrib - ask_total;
            let share = if need_total > 0.0 { s.need / need_total } else { 1.0 / open as f64 };
            s.need + b + rest * share + s.extra
        } else {
            contrib * (s.need + b) / ask_total + s.extra
        };
    }
}

/// Pots at each goal's due month, per path, plus the year-end totals of all pots.
struct Sim {
    due: Vec<Vec<f64>>,
    bands: Vec<Vec<f64>>,
}

/// Runs `paths` futures. Each month: contribution, allocation, then each open pot grows by a
/// monthly return. A pot freezes once its goal is due. `want_bands` also records the year-end
/// totals for the fan chart.
fn simulate(slots: &[Slot], surplus: f64, sd: f64, paths: u32, want_bands: bool) -> Sim {
    let horizon = slots.iter().map(|s| s.months).max().unwrap_or(0);
    let growth: Vec<f64> = (1..=horizon).map(growth_at).collect();
    let p = paths as usize;
    let n = slots.len();
    let years = if want_bands { horizon / 12 } else { 0 };
    let mut due = vec![vec![0.0; p]; n];
    let mut bands = vec![vec![0.0; p]; years];
    let mut rng = Rng::new(SEED);
    let mut pots = vec![0.0; n];
    let mut alloc = vec![0.0; n];
    let mut behind = vec![0.0; n];
    for path in 0..p {
        for (pot, s) in pots.iter_mut().zip(slots) {
            *pot = s.saved as f64;
        }
        behind.iter_mut().for_each(|b| *b = 0.0);
        for m in 1..=horizon {
            let shock = if sd > 0.0 { sd * rng.normal() } else { 0.0 };
            let contrib = (surplus * growth[m - 1] + shock).max(0.0);
            allocate(contrib, slots, &behind, m, &mut alloc);
            for (i, s) in slots.iter().enumerate() {
                if m > s.months {
                    continue;
                }
                behind[i] = (behind[i] + s.need - (alloc[i] - s.extra)).max(0.0);
                let r = s.mu + s.sigma * rng.normal();
                pots[i] = (pots[i] + alloc[i]) * (1.0 + r);
                if m == s.months {
                    due[i][path] = pots[i];
                }
            }
            if want_bands && m % 12 == 0 && m / 12 <= years {
                bands[m / 12 - 1][path] = pots.iter().sum();
            }
        }
    }
    Sim { due, bands }
}

/// Share of paths that reach the target at the due month, in bps.
fn reached_bps(slot: &Slot, due: &[f64]) -> i64 {
    if slot.done() {
        return 10_000;
    }
    let hits = due.iter().filter(|&&v| v >= slot.target as f64).count() as i64;
    hits * 10_000 / due.len().max(1) as i64
}

/// Likelihood of one goal in a set of goals, with the given number of paths.
fn hit(slots: &[Slot], i: usize, surplus: f64, sd: f64, paths: u32) -> i64 {
    let sim = simulate(slots, surplus, sd, paths, false);
    reached_bps(&slots[i], &sim.due[i])
}

fn to_paise(x: f64) -> i64 {
    if x.is_nan() {
        0
    } else {
        x.round().clamp(0.0, MAX_AMOUNT_PAISE as f64) as i64
    }
}

/// The value at quantile `q` of an ascending list.
fn percentile(sorted: &[f64], q: f64) -> i64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() - 1) as f64 * q).round() as usize;
    to_paise(sorted[idx])
}

fn sorted(v: &[f64]) -> Vec<f64> {
    let mut out = v.to_vec();
    out.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    out
}

fn status_for(bps: i64) -> Status {
    if bps >= LIKELY_BPS {
        Status::Likely
    } else if bps >= POSSIBLE_BPS {
        Status::Possible
    } else {
        Status::Unlikely
    }
}

/// Smallest extra monthly amount (in `SAVE_STEP_PAISE` steps, up to the cap) that makes goal `i` likely.
fn save_more(slots: &[Slot], i: usize, surplus: f64, sd: f64) -> Option<i64> {
    let check = |k: i64| {
        let mut s = slots.to_vec();
        s[i].extra = (k * SAVE_STEP_PAISE) as f64;
        hit(&s, i, surplus, sd, LEVER_PATHS) >= LIKELY_BPS
    };
    let max_k = SAVE_CAP_PAISE / SAVE_STEP_PAISE;
    if !check(max_k) {
        return None;
    }
    let (mut lo, mut hi) = (0, max_k);
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if check(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some(hi * SAVE_STEP_PAISE)
}

/// Goal `i` with its horizon pushed back `d` months and its target inflated over that time.
fn delayed(slots: &[Slot], i: usize, d: i64) -> Vec<Slot> {
    let mut s = slots.to_vec();
    s[i].months = (s[i].months as i64 + d).clamp(1, MAX_HORIZON_MONTHS) as usize;
    s[i].target = future_value(s[i].target, GOAL_INFLATION_BPS, d);
    s
}

/// Smallest delay in months (up to `MAX_DELAY_MONTHS`) that makes goal `i` likely.
fn push_date(slots: &[Slot], i: usize, surplus: f64, sd: f64) -> Option<i64> {
    let check = |d: i64| hit(&delayed(slots, i, d), i, surplus, sd, LEVER_PATHS) >= LIKELY_BPS;
    if !check(MAX_DELAY_MONTHS) {
        return None;
    }
    let (mut lo, mut hi) = (0, MAX_DELAY_MONTHS);
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if check(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some(hi)
}

/// The largest target, searching down from 95% to 50% in 5% steps, that makes goal `i` likely.
fn trim_target(slots: &[Slot], i: usize, surplus: f64, sd: f64) -> Option<i64> {
    let target = slots[i].target;
    if target <= 0 {
        return None;
    }
    let pcts: Vec<i64> = (TRIM_BOTTOM_PCT..=TRIM_TOP_PCT).rev().step_by(TRIM_STEP_PCT).collect();
    for pct in pcts {
        let new_target = target * pct / 100;
        let mut s = slots.to_vec();
        s[i].target = new_target;
        if hit(&s, i, surplus, sd, LEVER_PATHS) >= LIKELY_BPS {
            return Some(new_target);
        }
    }
    None
}

/// "June 2044" for a date pushed back `d` months.
fn month_year_after(when: Date, d: i64) -> String {
    let idx = when.year() as i64 * 12 + (when.month() as i64 - 1) + d;
    let year = idx.div_euclid(12);
    let month = Month::try_from(idx.rem_euclid(12) as u8 + 1).unwrap_or(when.month());
    format!("{month} {year}")
}

/// The levers for a goal that is not likely, in order, at most `MAX_LEVERS`.
fn levers_for(slots: &[Slot], i: usize, when: Option<Date>, surplus: f64, sd: f64) -> Vec<Lever> {
    let mut out = Vec::new();
    if let Some(extra) = save_more(slots, i, surplus, sd) {
        out.push(Lever {
            kind: LeverKind::SaveMore,
            text: format!("Put {} more a month towards it to make it likely.", inr(extra)),
            monthly_paise: Some(extra),
            months: None,
            target_paise: None,
        });
    }
    if let Some((when, d)) = when.and_then(|w| push_date(slots, i, surplus, sd).map(|d| (w, d))) {
        out.push(Lever {
            kind: LeverKind::PushDate,
            text: format!("Push the date back {d} months, to {}, to make it likely.", month_year_after(when, d)),
            monthly_paise: None,
            months: Some(d),
            target_paise: None,
        });
    }
    if let Some(t) = trim_target(slots, i, surplus, sd) {
        out.push(Lever {
            kind: LeverKind::TrimTarget,
            text: format!("A {} budget instead of {} would be likely.", inr(t), inr(slots[i].target)),
            monthly_paise: None,
            months: None,
            target_paise: Some(t),
        });
    }
    out.truncate(MAX_LEVERS);
    out
}

fn summary(goals: &[GoalForecast]) -> String {
    let likely = goals.iter().filter(|g| g.status == Status::Likely).count();
    let mut text = format!("{likely} of {} goals look likely on what you save today.", goals.len());
    // Any unlikely goal sits below every likely one, so the lowest overall is the hardest.
    if let Some(hardest) = goals.iter().filter(|g| g.status == Status::Unlikely).min_by_key(|g| g.likelihood_bps) {
        // Labels are sentence case ("Emergency fund", "Baby's first year"); mid-sentence they read lower case.
        let label = hardest.label.to_lowercase();
        text.push_str(&format!(" The {label} is the hardest to reach."));
    }
    text
}

/// Runs the forecast. Pure and deterministic: the same input always gives the same output.
pub fn run(input: &ForecastInput) -> ForecastOut {
    let slots = slots_for(&input.goals);
    if slots.is_empty() {
        return ForecastOut { goals: Vec::new(), path: Vec::new(), summary: summary(&[]) };
    }
    let paths = input.paths.clamp(1, MAX_PATHS);
    let surplus = input.surplus_paise as f64;
    let sd = input.surplus_sd_paise.max(0) as f64;
    let sim = simulate(&slots, surplus, sd, paths, true);

    let mut funded = vec![0.0; slots.len()];
    let none_behind = vec![0.0; slots.len()];
    allocate((surplus * growth_at(1)).max(0.0), &slots, &none_behind, 1, &mut funded);

    let mut goals = Vec::with_capacity(slots.len());
    for (i, (g, s)) in input.goals.iter().zip(&slots).enumerate() {
        let bps = reached_bps(s, &sim.due[i]);
        let status = status_for(bps);
        let pots = sorted(&sim.due[i]);
        let levers = if status == Status::Likely {
            Vec::new()
        } else {
            levers_for(&slots, i, g.when, surplus, sd)
        };
        goals.push(GoalForecast {
            key: g.key,
            label: g.label,
            when: g.when.map(|d| d.to_string()),
            target_paise: g.target_paise,
            saved_paise: g.saved_paise,
            needed_monthly_paise: g.needed_monthly_paise,
            funded_monthly_paise: to_paise(funded[i]),
            likelihood_bps: bps,
            p10_paise: percentile(&pots, 0.1),
            p50_paise: percentile(&pots, 0.5),
            p90_paise: percentile(&pots, 0.9),
            status,
            levers,
        });
    }

    let path = sim
        .bands
        .iter()
        .enumerate()
        .map(|(k, totals)| {
            let t = sorted(totals);
            YearBand {
                year: input.today.year() as i64 + k as i64 + 1,
                p10_paise: percentile(&t, 0.1),
                p50_paise: percentile(&t, 0.5),
                p90_paise: percentile(&t, 0.9),
            }
        })
        .collect();

    ForecastOut { summary: summary(&goals), goals, path }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn today() -> Date {
        date!(2026 - 11 - 09)
    }

    fn goal(key: &'static str, months: i64, target: i64, saved: i64, need: i64, mix: &'static str) -> GoalIn {
        GoalIn {
            key,
            label: key,
            when: None,
            months,
            target_paise: target,
            saved_paise: saved,
            needed_monthly_paise: need,
            mix,
        }
    }

    fn input(goals: Vec<GoalIn>, surplus: i64, sd: i64) -> ForecastInput {
        ForecastInput { today: today(), surplus_paise: surplus, surplus_sd_paise: sd, goals, paths: DEFAULT_PATHS }
    }

    fn find<'a>(out: &'a ForecastOut, key: &str) -> &'a GoalForecast {
        out.goals.iter().find(|g| g.key == key).unwrap()
    }

    /// A ₹1,00,000 goal in 24 months, held in debt, with the monthly need the plan would give.
    fn one_goal(saved: i64) -> GoalIn {
        GoalIn { when: Some(date!(2028 - 11 - 01)), ..goal("wedding", 24, 10_000_000, saved, 393_000, "debt") }
    }

    #[test]
    fn the_same_input_gives_the_same_output() {
        let goals = vec![
            goal("emergency", 12, 30_000_000, 5_000_000, 200_000, "debt"),
            GoalIn { when: Some(date!(2031 - 11 - 01)), ..goal("wedding", 60, 250_000_000, 0, 3_000_000, "hybrid") },
            GoalIn { when: Some(date!(2046 - 06 - 01)), ..goal("education", 235, 300_000_000, 0, 500_000, "equity") },
        ];
        let a = run(&input(goals.clone(), 900_000, 80_000));
        let b = run(&input(goals, 900_000, 80_000));
        assert_eq!(a, b);
    }

    #[test]
    fn the_prng_is_deterministic_per_seed() {
        let mut a = Rng::new(SEED);
        let mut b = Rng::new(SEED);
        for _ in 0..100 {
            assert_eq!(a.normal().to_bits(), b.normal().to_bits());
        }
        let mut c = Rng::new(SEED + 1);
        assert_ne!(Rng::new(SEED).normal().to_bits(), c.normal().to_bits());
    }

    #[test]
    fn a_goal_already_fully_saved_is_ten_thousand() {
        let out = run(&input(vec![one_goal(10_000_000)], 0, 0));
        let g = &out.goals[0];
        assert_eq!(g.likelihood_bps, 10_000);
        assert_eq!(g.status, Status::Likely);
        assert!(g.levers.is_empty());
    }

    #[test]
    fn zero_surplus_and_no_savings_is_zero_with_levers() {
        let out = run(&input(vec![one_goal(0)], 0, 0));
        let g = &out.goals[0];
        assert_eq!(g.likelihood_bps, 0);
        assert_eq!(g.status, Status::Unlikely);
        assert!(!g.levers.is_empty(), "an unreachable goal should come with levers");
        assert_eq!(g.levers[0].kind, LeverKind::SaveMore);
        assert!(g.levers[0].text.starts_with("Put "), "{}", g.levers[0].text);
    }

    #[test]
    fn more_surplus_never_lowers_likelihood() {
        let mut last = -1;
        for surplus in [0, 50_000, 100_000, 200_000, 400_000] {
            let out = run(&input(vec![one_goal(0)], surplus, 20_000));
            let bps = out.goals[0].likelihood_bps;
            assert!(bps >= last, "surplus {surplus}: {bps} fell below {last}");
            last = bps;
        }
        assert!(last > 0);
    }

    #[test]
    fn allocation_is_proportional_catches_up_and_shares_the_rest() {
        let slots = slots_for(&[goal("a", 12, 1, 0, 100_000, "debt"), goal("b", 12, 1, 0, 200_000, "debt")]);
        let mut out = [0.0; 2];
        let none = [0.0; 2];

        allocate(150_000.0, &slots, &none, 1, &mut out);
        assert!((out[0] - 50_000.0).abs() < 1e-6 && (out[1] - 100_000.0).abs() < 1e-6, "{out:?}");

        // More than needed: each gets its need, and the rest is shared in proportion to need.
        allocate(600_000.0, &slots, &none, 1, &mut out);
        assert!((out[0] - 200_000.0).abs() < 1e-6 && (out[1] - 400_000.0).abs() < 1e-6, "{out:?}");

        // A goal that fell behind asks for its need plus the shortfall first.
        allocate(400_000.0, &slots, &[100_000.0, 0.0], 1, &mut out);
        assert!((out[0] - 200_000.0).abs() < 1e-6 && (out[1] - 200_000.0).abs() < 1e-6, "{out:?}");

        // A goal past its horizon gets nothing.
        let short = slots_for(&[goal("a", 3, 1, 0, 100_000, "debt"), goal("b", 12, 1, 0, 200_000, "debt")]);
        allocate(150_000.0, &short, &none, 6, &mut out);
        assert_eq!(out[0], 0.0);
        assert!((out[1] - 150_000.0).abs() < 1e-6, "{out:?}");

        // Nothing needed: the contribution is split equally.
        let free = slots_for(&[goal("a", 12, 1, 0, 0, "debt"), goal("b", 12, 1, 0, 0, "debt")]);
        allocate(100_000.0, &free, &none, 1, &mut out);
        assert_eq!(out, [50_000.0, 50_000.0]);
    }

    #[test]
    fn a_lean_month_is_made_up_later() {
        // Needs ₹1,000 a month for 12 months; the surplus averages ₹1,200 but swings by ₹1,500.
        // Without catch-up the capped good months could never cover the lean ones.
        let goals = vec![goal("a", 12, 1_200_000, 0, 100_000, "debt")];
        let out = run(&input(goals, 120_000, 150_000));
        assert!(out.goals[0].likelihood_bps >= 5_000, "{}", out.goals[0].likelihood_bps);
    }

    #[test]
    fn plenty_of_surplus_is_all_put_to_the_goals_in_proportion_to_need() {
        let goals = vec![goal("a", 24, 10_000_000, 0, 300_000, "debt"), goal("b", 60, 10_000_000, 0, 100_000, "hybrid")];
        let out = run(&input(goals, 5_000_000, 0));
        let (a, b) = (find(&out, "a").funded_monthly_paise, find(&out, "b").funded_monthly_paise);
        assert!(a > 300_000 && b > 100_000, "{a} {b}");
        assert!((a - 3 * b).abs() <= 2, "3:1 like the needs: {a} {b}");
        assert!((a + b - (5_000_000.0 * growth_at(1)).round() as i64).abs() <= 2);
    }

    #[test]
    fn save_more_and_push_date_are_found_for_a_reachable_case() {
        // ₹3,000 a month against a goal that needs about ₹3,930: short, but a delay closes it.
        let out = run(&input(vec![one_goal(0)], 300_000, 0));
        let g = &out.goals[0];
        assert_eq!(g.status, Status::Unlikely);
        let save = g.levers.iter().find(|l| l.kind == LeverKind::SaveMore).expect("save_more lever");
        assert!(save.monthly_paise.unwrap() > 0);
        assert_eq!(save.monthly_paise.unwrap() % SAVE_STEP_PAISE, 0);
        let push = g.levers.iter().find(|l| l.kind == LeverKind::PushDate).expect("push_date lever");
        let d = push.months.unwrap();
        assert!((1..=MAX_DELAY_MONTHS).contains(&d));
        assert!(push.text.contains("Push the date back"), "{}", push.text);
        assert!(g.levers.len() <= MAX_LEVERS);
    }

    #[test]
    fn push_date_is_skipped_without_a_date() {
        let out = run(&input(vec![goal("emergency", 12, 30_000_000, 0, 0, "debt")], 0, 0));
        assert!(out.goals[0].levers.iter().all(|l| l.kind != LeverKind::PushDate));
    }

    #[test]
    fn percentiles_are_ordered() {
        let goals = vec![
            goal("emergency", 12, 30_000_000, 5_000_000, 200_000, "debt"),
            GoalIn { when: Some(date!(2031 - 11 - 01)), ..goal("wedding", 60, 250_000_000, 0, 3_000_000, "hybrid") },
        ];
        let out = run(&input(goals, 400_000, 60_000));
        for g in &out.goals {
            assert!(g.p10_paise <= g.p50_paise && g.p50_paise <= g.p90_paise, "{}", g.key);
            assert!((0..=10_000).contains(&g.likelihood_bps));
        }
    }

    #[test]
    fn path_years_ascend_and_run_to_the_last_goal() {
        let goals = vec![
            goal("emergency", 12, 30_000_000, 0, 200_000, "debt"),
            GoalIn { when: Some(date!(2031 - 11 - 01)), ..goal("wedding", 60, 250_000_000, 0, 3_000_000, "hybrid") },
        ];
        let out = run(&input(goals, 400_000, 0));
        let years: Vec<i64> = out.path.iter().map(|b| b.year).collect();
        assert_eq!(years, vec![2027, 2028, 2029, 2030, 2031]);
        for b in &out.path {
            assert!(b.p10_paise <= b.p50_paise && b.p50_paise <= b.p90_paise);
        }
    }

    #[test]
    fn summary_names_the_hardest_goal_when_one_is_unlikely() {
        let out = run(&input(vec![one_goal(10_000_000), goal("wedding_far", 24, 10_000_000, 0, 0, "debt")], 0, 0));
        assert!(out.summary.starts_with("1 of 2 goals look likely on what you save today."), "{}", out.summary);
        assert!(out.summary.ends_with("The wedding_far is the hardest to reach."), "{}", out.summary);
    }

    #[test]
    fn no_goals_gives_an_empty_forecast() {
        let out = run(&input(Vec::new(), 100_000, 0));
        assert!(out.goals.is_empty() && out.path.is_empty());
    }

    #[test]
    fn the_forecast_is_fast_enough_for_a_thousand_paths_of_five_goals() {
        let goals = vec![
            goal("emergency", 12, 30_000_000, 0, 200_000, "debt"),
            goal("wedding", 60, 250_000_000, 0, 3_000_000, "hybrid"),
            goal("child_birth", 120, 30_000_000, 0, 100_000, "hybrid"),
            goal("child_education", 240, 300_000_000, 0, 400_000, "equity"),
            goal("retirement", 600, 1_000_000_000, 0, 200_000, "equity"),
        ];
        let start = std::time::Instant::now();
        let out = simulate(&slots_for(&goals), 400_000.0, 60_000.0, DEFAULT_PATHS, true);
        let elapsed = start.elapsed();
        assert_eq!(out.due.len(), 5);
        eprintln!("forecast simulation: {elapsed:?}");
        assert!(elapsed.as_secs_f64() < 2.0, "took {elapsed:?}");
    }
}
