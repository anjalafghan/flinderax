//! What the couple seems to enjoy, learned from how past slips ended. It only nudges the draw: a
//! small, bounded multiplier that stays at 1.0 until there is real history, so a handful of skips
//! can never bury a whole kind of date.
//!
//! Each tag keeps a smoothed "liked" score, `(good + PRIOR) / (good + bad + 2 * PRIOR)`. Doing it
//! counts as good (more if rated 4+, bad if rated 2 or less); putting it back counts as a mild bad.
//! A slip's factor is `0.7 + 0.6 * average score of its tags`, so it lives between 0.7 and 1.3.

use std::collections::HashMap;

/// What we felt about things, and what has happened to them, fades with a half-life of about six months.
const HALF_LIFE_DAYS: f64 = 180.0;

/// One past outing, as the model reads it.
#[derive(Debug, Clone, Copy)]
pub struct Event<'a> {
    /// Tags as stored ("romantic,foodie").
    pub tags: &'a str,
    pub setting: &'a str,
    pub energy: &'a str,
    /// What the slip was estimated to cost, in paise.
    pub cost: i64,
    /// "did_it" | "skipped"
    pub outcome: &'a str,
    pub rating: Option<i64>,
    /// How long ago it happened, in days.
    pub age_days: i64,
}

/// Free, under ₹500, under ₹2,000, or dearer. `cost` is in paise.
pub fn cost_band(cost: i64) -> &'static str {
    match cost {
        i64::MIN..=0 => "free",
        1..=49_999 => "cheap",
        50_000..=199_999 => "middling",
        _ => "dear",
    }
}

/// The things about a slip the model learns on: each tag, its setting, energy and cost band.
pub fn features(tags: &[String], setting: &str, energy: &str, cost: i64) -> Vec<String> {
    let mut f: Vec<String> = tags.iter().map(|t| format!("tag:{t}")).collect();
    f.push(format!("setting:{setting}"));
    f.push(format!("energy:{energy}"));
    f.push(format!("cost:{}", cost_band(cost)));
    f
}

/// Pseudo-observations on each side: with little data every tag scores near 0.5 (neutral).
const PRIOR: f64 = 3.0;
const SKIP_WEIGHT: f64 = 0.5;

#[derive(Debug, Clone, Default)]
pub struct Taste {
    /// tag → (good, bad)
    tags: HashMap<String, (f64, f64)>,
}

impl Taste {
    /// Learn from outings with their age: each counts for half as much every six months. A tag, a
    /// setting, an energy level and a cost band each keep their own score.
    pub fn from_events<'a>(events: impl IntoIterator<Item = Event<'a>>) -> Self {
        let mut tags: HashMap<String, (f64, f64)> = HashMap::new();
        for e in events {
            let (good, bad) = match (e.outcome, e.rating) {
                ("did_it", Some(r)) if r <= 2 => (0.0, 1.0),
                ("did_it", Some(r)) if r >= 4 => (1.5, 0.0),
                ("did_it", _) => (1.0, 0.0),
                ("skipped", _) => (0.0, SKIP_WEIGHT),
                _ => continue,
            };
            let fade = 0.5f64.powf(e.age_days.max(0) as f64 / HALF_LIFE_DAYS);
            let list: Vec<String> = e.tags.split(',').map(str::trim).filter(|t| !t.is_empty()).map(String::from).collect();
            for f in features(&list, e.setting, e.energy, e.cost) {
                let slot = tags.entry(f).or_default();
                slot.0 += good * fade;
                slot.1 += bad * fade;
            }
        }
        Self { tags }
    }

    /// Multiplier for the draw weight from a slip's [`features`]: 1.0 with no history, else 0.7 to 1.3.
    pub fn factor_for(&self, features: &[String]) -> f64 {
        if features.is_empty() || self.tags.is_empty() {
            return 1.0;
        }
        let avg = features.iter().map(|f| self.score(f)).sum::<f64>() / features.len() as f64;
        0.7 + 0.6 * avg
    }

    /// `rows` are (tags as stored, "did_it" | "skipped", rating).
    pub fn from_history<'a>(rows: impl IntoIterator<Item = (&'a str, &'a str, Option<i64>)>) -> Self {
        let mut tags: HashMap<String, (f64, f64)> = HashMap::new();
        for (tag_list, outcome, rating) in rows {
            let (good, bad) = match (outcome, rating) {
                ("did_it", Some(r)) if r <= 2 => (0.0, 1.0),
                ("did_it", Some(r)) if r >= 4 => (1.5, 0.0),
                ("did_it", _) => (1.0, 0.0),
                ("skipped", _) => (0.0, SKIP_WEIGHT),
                _ => continue,
            };
            for t in tag_list.split(',').map(str::trim).filter(|t| !t.is_empty()) {
                let e = tags.entry(t.to_string()).or_default();
                e.0 += good;
                e.1 += bad;
            }
        }
        Self { tags }
    }

    fn score(&self, tag: &str) -> f64 {
        let (g, b) = self.tags.get(tag).copied().unwrap_or_default();
        (g + PRIOR) / (g + b + 2.0 * PRIOR)
    }

    /// Multiplier for the draw weight: 1.0 for untagged slips or no history.
    pub fn factor(&self, tags: &[String]) -> f64 {
        if tags.is_empty() || self.tags.is_empty() {
            return 1.0;
        }
        let avg = tags.iter().map(|t| self.score(t)).sum::<f64>() / tags.len() as f64;
        0.7 + 0.6 * avg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Vec<String> {
        s.split(',').map(String::from).collect()
    }

    #[test]
    fn no_history_is_neutral() {
        assert_eq!(Taste::default().factor(&t("chill")), 1.0);
        assert_eq!(Taste::default().factor(&[]), 1.0);
    }

    #[test]
    fn loved_tags_rise_and_skipped_ones_dip_but_stay_bounded() {
        let mut rows = vec![];
        for _ in 0..20 {
            rows.push(("foodie", "did_it", Some(5)));
            rows.push(("games", "skipped", None));
        }
        let taste = Taste::from_history(rows);
        let (up, down) = (taste.factor(&t("foodie")), taste.factor(&t("games")));
        assert!(up > 1.15 && up <= 1.3, "{up}");
        assert!((0.7..0.95).contains(&down), "{down}");
        // A tag it has never seen stays neutral.
        assert!((taste.factor(&t("cozy")) - 1.0).abs() < 1e-9);
    }

    fn ev<'a>(tags: &'a str, setting: &'a str, energy: &'a str, cost: i64, outcome: &'a str, rating: Option<i64>, age_days: i64) -> Event<'a> {
        Event { tags, setting, energy, cost, outcome, rating, age_days }
    }

    fn feats(tags: &str, setting: &str, energy: &str, cost: i64) -> Vec<String> {
        features(&t(tags), setting, energy, cost)
    }

    #[test]
    fn it_learns_more_than_tags() {
        // Five great evenings at home, each with a different tag: no tag repeats, but "home" does.
        let events: Vec<Event> = (["a", "b", "c", "d", "e"]).iter().map(|tag| ev(tag, "home", "low", 30_000, "did_it", Some(5), 5)).collect();
        let taste = Taste::from_events(events);
        let home = taste.factor_for(&feats("new-tag", "home", "high", 500_000));
        let away = taste.factor_for(&feats("new-tag", "day_trip", "high", 500_000));
        assert!(home > away, "home {home} vs away {away}");
        assert!(home > 1.0 && away - 1.0 <= 0.05, "an unseen kind stays near neutral: {away}");
        // And it still reads tags and the cost band.
        assert!(taste.factor_for(&feats("a", "day_trip", "high", 500_000)) > away);
        assert!(taste.factor_for(&feats("new-tag", "day_trip", "high", 30_000)) > away, "cheap is liked too");
    }

    #[test]
    fn what_we_felt_long_ago_counts_for_less() {
        let loved = |age| Taste::from_events((0..6).map(|_| ev("foodie", "local", "med", 100_000, "did_it", Some(5), age)).collect::<Vec<_>>());
        let recent = loved(10).factor_for(&feats("foodie", "local", "med", 100_000));
        let old = loved(720).factor_for(&feats("foodie", "local", "med", 100_000));
        assert!(recent > old && old > 1.0, "{recent} {old}");
        // A hundred-year-old like is almost forgotten.
        let ancient = loved(36_500).factor_for(&feats("foodie", "local", "med", 100_000));
        assert!((ancient - 1.0).abs() < 0.01, "{ancient}");
    }

    #[test]
    fn it_stays_bounded_whatever_the_history() {
        let loved: Vec<Event> = (0..500).map(|_| ev("foodie", "local", "med", 100_000, "did_it", Some(5), 1)).collect();
        let hated: Vec<Event> = (0..500).map(|_| ev("games", "home", "low", 10_000, "did_it", Some(1), 1)).collect();
        let taste = Taste::from_events(loved.into_iter().chain(hated));
        for (tags, setting, energy, cost) in [("foodie", "local", "med", 100_000), ("games", "home", "low", 10_000), ("x", "y", "z", 0)] {
            let f = taste.factor_for(&feats(tags, setting, energy, cost));
            assert!((0.7..=1.3).contains(&f), "{f}");
        }
    }

    #[test]
    fn cost_bands_are_free_cheap_middling_and_dear() {
        assert_eq!(cost_band(0), "free");
        assert_eq!(cost_band(49_999), "cheap");
        assert_eq!(cost_band(50_000), "middling");
        assert_eq!(cost_band(199_999), "middling");
        assert_eq!(cost_band(200_000), "dear");
    }

    #[test]
    fn a_few_skips_barely_move_it() {
        let taste = Taste::from_history([("games", "skipped", None), ("games", "skipped", None)]);
        let f = taste.factor(&t("games"));
        assert!(f > 0.93 && f < 1.0, "{f}");
    }
}
