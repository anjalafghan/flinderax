//! Pure pick logic. No IO: everything the picker needs is passed in, so it is
//! fully deterministic under a seeded RNG and easy to test.

use rand::Rng;

const DAY: i64 = 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Duration {
    Short,
    HalfDay,
    FullDay,
    Weekend,
}

impl Duration {
    pub const ALL: [Duration; 4] = [Self::Short, Self::HalfDay, Self::FullDay, Self::Weekend];

    pub fn key(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::HalfDay => "half_day",
            Self::FullDay => "full_day",
            Self::Weekend => "weekend",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Short => "1–2 hours",
            Self::HalfDay => "Half a day",
            Self::FullDay => "A full day",
            Self::Weekend => "A weekend",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.key() == s)
    }

    /// The time a date takes door to door, as the time we'd need to have: a couple of hours,
    /// half a day, a whole day, or more.
    pub fn of_minutes(m: i64) -> Self {
        match m {
            ..=150 => Self::Short,
            151..=360 => Self::HalfDay,
            361..=720 => Self::FullDay,
            _ => Self::Weekend,
        }
    }
}

/// "home" is at home together, in the same room; "apart" is long distance, each at our own
/// place (on a call, or sending things to each other).
pub const SETTINGS: [(&str, &str); 5] = [
    ("home", "At home, together"),
    ("apart", "Long distance"),
    ("local", "Around town"),
    ("day_trip", "Day trip"),
    ("travel", "Travel"),
];

pub const ENERGIES: [(&str, &str); 3] = [("low", "Low"), ("med", "Medium"), ("high", "High")];

pub fn setting_label(key: &str) -> &str {
    SETTINGS.iter().find(|(k, _)| *k == key).map(|(_, l)| *l).unwrap_or(key)
}

/// The slice of an activity the picker cares about.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: i64,
    pub title: String,
    pub est_cost: i64,
    pub setting: String,
    pub duration: Duration,
    pub tags: Vec<String>,
    pub energy: String,
    pub last_done_at: Option<i64>,
    pub last_skipped_at: Option<i64>,
    /// Approx road km for whichever of us is farther. None = at home / unknown.
    pub distance_km: Option<f64>,
    /// Weekdays it's closed (0 = Sunday … 6 = Saturday).
    pub closed_days: Vec<u8>,
    /// Months it's in season, inclusive, possibly wrapping the new year (11, 5 = Nov–May).
    pub season: Option<(u8, u8)>,
    /// Why it doesn't fit the date we're drawing for ("closes at 9"), or None when it does
    /// (or we're not drawing for a particular date).
    pub window_miss: Option<String>,
    /// Learned liking for this kind of date (see `taste`): about 0.7 to 1.3, 1.0 = no opinion.
    pub taste: f64,
    /// Would rain spoil it? From the slip's cover, falling back to the "outdoors" tag.
    pub outdoor: bool,
    /// Same setting and main tag as one of the last two dates we had.
    pub like_recent: bool,
    /// Both of us marked it "want" (privately).
    pub both_want: bool,
    /// At least one of us marked it "no" (privately). Nobody is told which.
    pub any_no: bool,
    /// Came from a starter pack, not the couple's own pen.
    pub starter: bool,
}

/// Starter slips are drawn a little less than ones the couple wrote.
pub const STARTER_DAMP: f64 = 0.8;

/// A slip we both want comes up this much more often; one either of us ruled out, much less.
pub const BOTH_WANT_BOOST: f64 = 2.0;
pub const ANY_NO_DAMP: f64 = 0.3;

/// How much a kind of date we just had is damped, so the jar does not hand out the same evening twice.
pub const VARIETY_DAMP: f64 = 0.6;

#[derive(Debug, Clone, Default)]
pub struct Filters {
    pub budget: i64,
    pub max_duration: Option<Duration>,
    /// Empty means "anywhere".
    pub settings: Vec<String>,
    pub tag: Option<String>,
    pub energy: Option<String>,
    pub max_km: Option<f64>,
    /// Set by the server from the forecast, not the form: rain is likely tonight.
    pub rain_likely: bool,
    /// Today (local), so closed days and seasons apply. None ignores them.
    pub today: Option<Today>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Today {
    /// 0 = Sunday … 6 = Saturday.
    pub weekday: u8,
    /// 1–12.
    pub month: u8,
}

pub const WEEKDAYS: [(&str, &str); 7] =
    [("sun", "Sunday"), ("mon", "Monday"), ("tue", "Tuesday"), ("wed", "Wednesday"), ("thu", "Thursday"), ("fri", "Friday"), ("sat", "Saturday")];
pub const MONTHS_SHORT: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// "mon,wed" → [1, 3].
pub fn parse_closed(s: &str) -> Vec<u8> {
    s.split(',').filter_map(|d| WEEKDAYS.iter().position(|(k, _)| *k == d.trim()).map(|i| i as u8)).collect()
}

/// In season this month? Handles seasons that wrap the new year.
pub fn in_season(season: Option<(u8, u8)>, month: u8) -> bool {
    match season {
        None => true,
        Some((from, to)) if from <= to => (from..=to).contains(&month),
        Some((from, to)) => month >= from || month <= to,
    }
}

/// "Nov–May", or "Dec" for a single month.
pub fn season_label(from: u8, to: u8) -> String {
    let m = |i: u8| MONTHS_SHORT.get(usize::from(i).saturating_sub(1)).copied().unwrap_or("?");
    if from == to { m(from).into() } else { format!("{}–{}", m(from), m(to)) }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hint {
    EmptyWishlist,
    /// Everything that matched was already rerolled away.
    Exhausted,
    OverBudget { title: String, cost: i64 },
    NeedsMoreTime { title: String, duration: Duration },
    TooFar { title: String, km: i64 },
    OtherSetting { title: String, setting: String },
    /// Fits everything except today: closed on this weekday.
    ClosedToday { title: String, weekday: u8 },
    /// Fits everything except the time of year.
    OutOfSeason { title: String, from: u8, to: u8 },
    /// Fits everything except the date we're drawing for: too far, closed then, too long.
    OutsideWindow { title: String, why: String },
    NoMoodMatch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PickResult {
    Pick(i64),
    NoMatch(Hint),
}

#[derive(Clone, Copy)]
struct Checks {
    budget: bool,
    duration: bool,
    distance: bool,
    setting: bool,
    mood: bool,
    /// Open today and in season.
    open: bool,
    /// Fits the date we're drawing for.
    window: bool,
}

fn checks(c: &Candidate, f: &Filters) -> Checks {
    Checks {
        budget: c.est_cost <= f.budget,
        duration: f.max_duration.is_none_or(|d| c.duration <= d),
        distance: f
            .max_km
            .is_none_or(|m| c.distance_km.is_none_or(|d| d <= m)),
        setting: f.settings.is_empty() || f.settings.contains(&c.setting),
        mood: f.tag.as_ref().is_none_or(|t| c.tags.iter().any(|ct| ct == t))
            && f.energy.as_ref().is_none_or(|e| *e == c.energy),
        open: f.today.is_none_or(|t| !c.closed_days.contains(&t.weekday) && in_season(c.season, t.month)),
        window: c.window_miss.is_none(),
    }
}

/// Higher weight = more likely. Never-done ideas get a boost, older ones
/// climb back up over ~2 months, recently skipped ones are damped.
pub fn weight(c: &Candidate, now: i64, rain_likely: bool) -> f64 {
    let mut w = c.taste * match c.last_done_at {
        None => 3.0,
        Some(t) => 1.0 + ((now - t).max(0) as f64 / (30 * DAY) as f64).min(2.0),
    };
    if c.last_skipped_at.is_some_and(|t| now - t < 3 * DAY) {
        w *= 0.5;
    }
    if c.like_recent {
        w *= VARIETY_DAMP;
    }
    if c.starter {
        w *= STARTER_DAMP;
    }
    if c.any_no {
        w *= ANY_NO_DAMP;
    } else if c.both_want {
        w *= BOTH_WANT_BOOST;
    }
    // Rain doesn't rule outdoor plans out, it just lets the cozy ones float up.
    if rain_likely && c.outdoor {
        w *= 0.3;
    }
    w
}

/// Does this candidate pass every hard filter? (Exclusions aside.)
pub fn matches(c: &Candidate, f: &Filters) -> bool {
    let k = checks(c, f);
    k.budget && k.duration && k.distance && k.setting && k.mood && k.open && k.window
}

pub fn pick<R: Rng>(
    all: &[Candidate],
    f: &Filters,
    excluded: &[i64],
    now: i64,
    rng: &mut R,
) -> PickResult {
    if all.is_empty() {
        return PickResult::NoMatch(Hint::EmptyWishlist);
    }

    let matching: Vec<&Candidate> = all.iter().filter(|c| matches(c, f)).collect();
    let pool: Vec<&Candidate> = matching
        .iter()
        .copied()
        .filter(|c| !excluded.contains(&c.id))
        .collect();

    if pool.is_empty() {
        return PickResult::NoMatch(if matching.is_empty() {
            hint(all, f)
        } else {
            Hint::Exhausted
        });
    }

    let weights: Vec<f64> = pool.iter().map(|c| weight(c, now, f.rain_likely)).collect();
    let mut roll = rng.gen_range(0.0..weights.iter().sum::<f64>());
    for (c, w) in pool.iter().zip(&weights) {
        if roll < *w {
            return PickResult::Pick(c.id);
        }
        roll -= w;
    }
    PickResult::Pick(pool[pool.len() - 1].id)
}

/// Explain which single constraint to relax, preferring the cheapest change.
fn hint(all: &[Candidate], f: &Filters) -> Hint {
    // Candidates that fail exactly the one check `pred` singles out.
    let only_fails = |pred: fn(Checks) -> bool| -> Vec<&Candidate> {
        all.iter().filter(|c| pred(checks(c, f))).collect()
    };

    let mut over_budget =
        only_fails(|k| !k.budget && k.duration && k.distance && k.setting && k.mood && k.open && k.window);
    over_budget.sort_by_key(|c| c.est_cost);
    if let Some(c) = over_budget.first() {
        return Hint::OverBudget { title: c.title.clone(), cost: c.est_cost };
    }

    let mut longer =
        only_fails(|k| k.budget && !k.duration && k.distance && k.setting && k.mood && k.open && k.window);
    longer.sort_by_key(|c| c.duration);
    if let Some(c) = longer.first() {
        return Hint::NeedsMoreTime { title: c.title.clone(), duration: c.duration };
    }

    let mut farther =
        only_fails(|k| k.budget && k.duration && !k.distance && k.setting && k.mood && k.open && k.window);
    farther.sort_by(|a, b| a.distance_km.partial_cmp(&b.distance_km).unwrap());
    if let Some(c) = farther.first() {
        return Hint::TooFar { title: c.title.clone(), km: c.distance_km.unwrap_or(0.0).round() as i64 };
    }

    if let Some(c) = only_fails(|k| k.budget && k.duration && k.distance && !k.setting && k.mood && k.open && k.window)
        .first()
    {
        return Hint::OtherSetting { title: c.title.clone(), setting: c.setting.clone() };
    }

    if let (Some(today), Some(c)) =
        (f.today, only_fails(|k| k.budget && k.duration && k.distance && k.setting && k.mood && !k.open && k.window).first())
    {
        return match c.season {
            Some((from, to)) if !in_season(c.season, today.month) => Hint::OutOfSeason { title: c.title.clone(), from, to },
            _ => Hint::ClosedToday { title: c.title.clone(), weekday: today.weekday },
        };
    }

    if let Some(c) = only_fails(|k| k.budget && k.duration && k.distance && k.setting && k.mood && k.open && !k.window).first() {
        return Hint::OutsideWindow { title: c.title.clone(), why: c.window_miss.clone().unwrap_or_default() };
    }

    Hint::NoMoodMatch
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    const NOW: i64 = 1_800_000_000;

    fn cand(id: i64, cost: i64, setting: &str, duration: Duration) -> Candidate {
        Candidate {
            id,
            title: format!("idea {id}"),
            est_cost: cost,
            setting: setting.into(),
            duration,
            tags: vec!["chill".into()],
            energy: "med".into(),
            last_done_at: None,
            last_skipped_at: None,
            distance_km: None,
            closed_days: vec![],
            season: None,
            window_miss: None,
            taste: 1.0,
            outdoor: false,
            like_recent: false,
            both_want: false,
            any_no: false,
            starter: false,
        }
    }

    fn filters(budget: i64) -> Filters {
        Filters { budget, ..Default::default() }
    }

    fn rng() -> StdRng {
        StdRng::seed_from_u64(7)
    }

    fn picks(all: &[Candidate], f: &Filters, excluded: &[i64], n: usize) -> Vec<PickResult> {
        let mut r = rng();
        (0..n).map(|_| pick(all, f, excluded, NOW, &mut r)).collect()
    }

    #[test]
    fn budget_is_a_hard_filter() {
        let all = [cand(1, 500, "home", Duration::Short), cand(2, 5000, "home", Duration::Short)];
        assert!(picks(&all, &filters(1000), &[], 200).iter().all(|p| *p == PickResult::Pick(1)));
    }

    #[test]
    fn duration_is_a_hard_filter_and_ordered() {
        let all = [cand(1, 0, "home", Duration::HalfDay), cand(2, 0, "home", Duration::Weekend)];
        let f = Filters { max_duration: Some(Duration::FullDay), ..filters(0) };
        assert!(picks(&all, &f, &[], 200).iter().all(|p| *p == PickResult::Pick(1)));
    }

    #[test]
    fn setting_is_a_hard_filter_and_empty_means_any() {
        let all = [cand(1, 0, "home", Duration::Short), cand(2, 0, "travel", Duration::Short)];
        let f = Filters { settings: vec!["travel".into()], ..filters(0) };
        assert!(picks(&all, &f, &[], 200).iter().all(|p| *p == PickResult::Pick(2)));
        let any = picks(&all, &filters(0), &[], 200);
        assert!(any.contains(&PickResult::Pick(1)) && any.contains(&PickResult::Pick(2)));
    }

    #[test]
    fn tag_and_energy_are_hard_filters() {
        let mut a = cand(1, 0, "home", Duration::Short);
        a.tags = vec!["foodie".into()];
        let mut b = cand(2, 0, "home", Duration::Short);
        b.energy = "high".into();
        let all = [a, b];
        let f = Filters { tag: Some("foodie".into()), ..filters(0) };
        assert!(picks(&all, &f, &[], 100).iter().all(|p| *p == PickResult::Pick(1)));
        let f = Filters { energy: Some("high".into()), ..filters(0) };
        assert!(picks(&all, &f, &[], 100).iter().all(|p| *p == PickResult::Pick(2)));
    }

    #[test]
    fn distance_is_a_hard_filter_and_home_always_passes() {
        let mut near = cand(1, 0, "local", Duration::Short);
        near.distance_km = Some(8.0);
        let mut far = cand(2, 0, "local", Duration::Short);
        far.distance_km = Some(60.0);
        let home = cand(3, 0, "home", Duration::Short);
        let f = Filters { max_km: Some(20.0), ..filters(0) };
        let r = picks(&[near, far.clone(), home], &f, &[], 300);
        assert!(r.iter().all(|p| *p != PickResult::Pick(2)));
        assert!(r.contains(&PickResult::Pick(1)) && r.contains(&PickResult::Pick(3)));
        assert_eq!(
            picks(&[far], &f, &[], 1)[0],
            PickResult::NoMatch(Hint::TooFar { title: "idea 2".into(), km: 60 })
        );
    }

    #[test]
    fn matches_agrees_with_pick() {
        let all = [cand(1, 500, "home", Duration::Short), cand(2, 5000, "local", Duration::Weekend)];
        let f = Filters { max_duration: Some(Duration::FullDay), ..filters(1000) };
        let ids: Vec<i64> = all.iter().filter(|c| matches(c, &f)).map(|c| c.id).collect();
        assert_eq!(ids, vec![1]);
        assert!(picks(&all, &f, &[], 50).iter().all(|p| *p == PickResult::Pick(1)));
    }

    #[test]
    fn closed_days_and_seasons_are_hard_filters() {
        let monday = Today { weekday: 1, month: 7 };
        let mut planetarium = cand(1, 0, "local", Duration::Short);
        planetarium.closed_days = parse_closed("mon");
        let mut flamingos = cand(2, 0, "local", Duration::Short);
        flamingos.season = Some((11, 5));
        let open = cand(3, 0, "local", Duration::Short);
        let f = Filters { today: Some(monday), ..filters(0) };
        assert!(picks(&[planetarium.clone(), flamingos.clone(), open], &f, &[], 200).iter().all(|p| *p == PickResult::Pick(3)));
        assert_eq!(
            picks(&[planetarium.clone()], &f, &[], 1)[0],
            PickResult::NoMatch(Hint::ClosedToday { title: "idea 1".into(), weekday: 1 })
        );
        assert_eq!(
            picks(&[flamingos.clone()], &f, &[], 1)[0],
            PickResult::NoMatch(Hint::OutOfSeason { title: "idea 2".into(), from: 11, to: 5 })
        );
        // Without a date, nothing is filtered on it.
        assert_eq!(picks(&[planetarium], &filters(0), &[], 1)[0], PickResult::Pick(1));
    }

    #[test]
    fn seasons_wrap_the_new_year() {
        assert!(in_season(Some((11, 5)), 12) && in_season(Some((11, 5)), 1) && in_season(Some((11, 5)), 5));
        assert!(!in_season(Some((11, 5)), 6) && !in_season(Some((11, 5)), 10));
        assert!(in_season(Some((3, 3)), 3) && !in_season(Some((3, 3)), 4), "single month");
        assert!(in_season(Some((6, 10)), 8) && !in_season(Some((6, 10)), 11));
        assert!(in_season(None, 9), "no season = all year");
        assert_eq!(season_label(11, 5), "Nov–May");
        assert_eq!(season_label(12, 12), "Dec");
        assert_eq!(parse_closed("mon, wed,xyz"), vec![1, 3]);
    }

    #[test]
    fn excluded_ids_are_never_returned() {
        let all: Vec<_> = (1..=5).map(|i| cand(i, 0, "home", Duration::Short)).collect();
        assert!(picks(&all, &filters(0), &[2, 4], 500)
            .iter()
            .all(|p| !matches!(p, PickResult::Pick(2) | PickResult::Pick(4))));
    }

    #[test]
    fn exhausted_when_everything_matching_was_rerolled() {
        let all = [cand(1, 0, "home", Duration::Short)];
        assert_eq!(picks(&all, &filters(0), &[1], 1)[0], PickResult::NoMatch(Hint::Exhausted));
    }

    #[test]
    fn hints_name_the_constraint_to_relax() {
        assert_eq!(picks(&[], &filters(0), &[], 1)[0], PickResult::NoMatch(Hint::EmptyWishlist));

        let all = [cand(1, 3000, "home", Duration::Short), cand(2, 1200, "home", Duration::Short)];
        assert_eq!(
            picks(&all, &filters(500), &[], 1)[0],
            PickResult::NoMatch(Hint::OverBudget { title: "idea 2".into(), cost: 1200 })
        );

        let all = [cand(1, 0, "home", Duration::Weekend)];
        let f = Filters { max_duration: Some(Duration::Short), ..filters(0) };
        assert_eq!(
            picks(&all, &f, &[], 1)[0],
            PickResult::NoMatch(Hint::NeedsMoreTime { title: "idea 1".into(), duration: Duration::Weekend })
        );

        let f = Filters { settings: vec!["travel".into()], ..filters(0) };
        assert_eq!(
            picks(&all, &f, &[], 1)[0],
            PickResult::NoMatch(Hint::OtherSetting { title: "idea 1".into(), setting: "home".into() })
        );

        let f = Filters { tag: Some("nope".into()), ..filters(0) };
        assert_eq!(picks(&all, &f, &[], 1)[0], PickResult::NoMatch(Hint::NoMoodMatch));
    }

    #[test]
    fn never_done_beats_recently_done() {
        let fresh = cand(1, 0, "home", Duration::Short);
        let mut stale = cand(2, 0, "home", Duration::Short);
        stale.last_done_at = Some(NOW - DAY);
        let results = picks(&[fresh, stale], &filters(0), &[], 4000);
        let fresh_wins = results.iter().filter(|p| **p == PickResult::Pick(1)).count();
        // weights 3.0 vs ~1.03 -> ~74%
        assert!((2700..3200).contains(&fresh_wins), "fresh won {fresh_wins}/4000");
    }

    #[test]
    fn taste_scales_the_weight() {
        let mut c = cand(1, 0, "home", Duration::Short);
        let base = weight(&c, NOW, false);
        c.taste = 1.3;
        assert!((weight(&c, NOW, false) - base * 1.3).abs() < 1e-9);
    }

    #[test]
    fn weight_recovers_with_time_and_is_damped_by_skips() {
        let mut c = cand(1, 0, "home", Duration::Short);
        c.last_done_at = Some(NOW - 10 * DAY);
        let recent = weight(&c, NOW, false);
        c.last_done_at = Some(NOW - 100 * DAY);
        let old = weight(&c, NOW, false);
        assert!(old > recent);
        assert_eq!(old, 3.0, "capped");
        c.last_skipped_at = Some(NOW - DAY);
        assert_eq!(weight(&c, NOW, false), 1.5);
    }

    #[test]
    fn slips_the_couple_wrote_come_up_a_little_more_often_than_starter_ones() {
        let mut c = cand(1, 0, "local", Duration::Short);
        let own = weight(&c, NOW, false);
        c.starter = true;
        assert_eq!(weight(&c, NOW, false), own * STARTER_DAMP);
        const { assert!(STARTER_DAMP < 1.0 && STARTER_DAMP > 0.5) } // a nudge, not a ban
    }

    #[test]
    fn what_we_both_want_rises_and_what_either_ruled_out_sinks() {
        let mut c = cand(1, 0, "local", Duration::Short);
        let plain = weight(&c, NOW, false);
        c.both_want = true;
        assert_eq!(weight(&c, NOW, false), plain * BOTH_WANT_BOOST);
        c.both_want = false;
        c.any_no = true;
        assert_eq!(weight(&c, NOW, false), plain * ANY_NO_DAMP);
        c.both_want = true;
        assert_eq!(weight(&c, NOW, false), plain * ANY_NO_DAMP, "a no outweighs the other's want");
    }

    #[test]
    fn a_kind_of_date_we_just_had_is_damped_so_the_jar_does_not_repeat_itself() {
        let mut c = cand(1, 0, "local", Duration::Short);
        let plain = weight(&c, NOW, false);
        c.like_recent = true;
        assert!(weight(&c, NOW, false) < plain);
        assert_eq!(weight(&c, NOW, false), plain * VARIETY_DAMP);
    }

    #[test]
    fn an_outdoor_slip_is_damped_in_rain_whether_or_not_it_carries_the_tag() {
        let mut c = cand(1, 0, "local", Duration::Short);
        c.tags = vec!["chill".into()]; // no "outdoors" tag...
        c.outdoor = true; // ...but its cover says outdoor
        assert!(weight(&c, NOW, true) < weight(&c, NOW, false));
        let mut indoor = cand(2, 0, "local", Duration::Short);
        indoor.tags = vec!["outdoors".into()]; // tagged, but its cover says indoor
        indoor.outdoor = false;
        assert_eq!(weight(&indoor, NOW, true), weight(&indoor, NOW, false), "the cover wins over the tag");
    }

    #[test]
    fn rain_sinks_outdoor_slips_but_never_removes_them() {
        let mut out = cand(1, 0, "local", Duration::Short);
        out.outdoor = true;
        let cozy = cand(2, 0, "home", Duration::Short);
        assert_eq!(weight(&out, NOW, false), weight(&cozy, NOW, false));
        assert!(weight(&out, NOW, true) < weight(&cozy, NOW, true));
        let f = Filters { rain_likely: true, ..filters(0) };
        let r = picks(&[out, cozy], &f, &[], 2000);
        let outdoor = r.iter().filter(|p| **p == PickResult::Pick(1)).count();
        assert!((300..700).contains(&outdoor), "outdoor won {outdoor}/2000, expected ~23%");
    }

    mod properties {
        use super::*;
        use proptest::prelude::*;

        const SETTINGS_POOL: [&str; 4] = ["home", "local", "day_trip", "apart"];
        const TAGS_POOL: [&str; 5] = ["chill", "foodie", "games", "outdoors", "cozy"];

        fn candidate() -> impl Strategy<Value = Candidate> {
            (0i64..40, 0i64..5000, 0usize..4, 0usize..3, proptest::collection::vec(0usize..5, 0..3), proptest::option::of(0.0f64..120.0), any::<bool>(), any::<bool>())
                .prop_map(|(id, cost, setting, duration, tags, km, outdoor, closed)| {
                    let mut c = cand(id, cost, SETTINGS_POOL[setting], [Duration::Short, Duration::HalfDay, Duration::FullDay][duration]);
                    c.tags = tags.into_iter().map(|t| TAGS_POOL[t].to_string()).collect();
                    c.distance_km = km;
                    c.outdoor = outdoor;
                    if closed {
                        c.closed_days = vec![2];
                    }
                    c
                })
        }

        fn filters_strategy() -> impl Strategy<Value = Filters> {
            (proptest::option::of(0i64..6000), proptest::collection::vec(0usize..4, 0..3), proptest::option::of(0usize..5), proptest::option::of(5.0f64..100.0), any::<bool>()).prop_map(
                |(budget, settings, tag, km, rain)| Filters {
                    budget: budget.unwrap_or(i64::MAX),
                    settings: settings.into_iter().map(|i| SETTINGS_POOL[i].to_string()).collect(),
                    tag: tag.map(|t| TAGS_POOL[t].to_string()),
                    max_km: km,
                    rain_likely: rain,
                    today: Some(Today { weekday: 2, month: 6 }),
                    ..Default::default()
                },
            )
        }

        proptest! {
            /// Whatever the jar holds and whatever is asked, a pick fits every filter and was not set aside;
            /// and when nothing is picked, nothing eligible was left.
            #[test]
            fn a_pick_always_satisfies_every_filter_and_never_returns_an_excluded_slip(
                all in proptest::collection::vec(candidate(), 0..25),
                f in filters_strategy(),
                excluded in proptest::collection::vec(0i64..40, 0..10),
                seed in any::<u64>(),
            ) {
                let mut rng = StdRng::seed_from_u64(seed);
                // Ids are unique in a real jar.
                let mut seen = std::collections::HashSet::new();
                let all: Vec<Candidate> = all.into_iter().filter(|c| seen.insert(c.id)).collect();
                match pick(&all, &f, &excluded, NOW, &mut rng) {
                    PickResult::Pick(id) => {
                        let c = all.iter().find(|c| c.id == id).expect("the pick is in the jar");
                        prop_assert!(matches(c, &f), "{c:?} does not fit {f:?}");
                        prop_assert!(!excluded.contains(&id));
                    }
                    PickResult::NoMatch(_) => {
                        prop_assert!(!all.iter().any(|c| matches(c, &f) && !excluded.contains(&c.id)), "something eligible was left");
                    }
                }
            }

            /// The weight of any slip is positive and finite, so the draw can never divide by zero or favour NaN.
            #[test]
            fn weights_are_always_positive_and_finite(c in candidate(), rain in any::<bool>(), done_days in proptest::option::of(0i64..2000)) {
                let mut c = c;
                c.last_done_at = done_days.map(|d| NOW - d * DAY);
                let w = weight(&c, NOW, rain);
                prop_assert!(w.is_finite() && w > 0.0, "{w}");
            }
        }
    }
}
