//! What the couple seems to enjoy, learned from how past slips ended. It only nudges the draw: a
//! small, bounded multiplier that stays at 1.0 until there is real history, so a handful of skips
//! can never bury a whole kind of date.
//!
//! Each tag keeps a smoothed "liked" score, `(good + PRIOR) / (good + bad + 2 * PRIOR)`. Doing it
//! counts as good (more if rated 4+, bad if rated 2 or less); putting it back counts as a mild bad.
//! A slip's factor is `0.7 + 0.6 * average score of its tags`, so it lives between 0.7 and 1.3.

use std::collections::HashMap;

/// Pseudo-observations on each side: with little data every tag scores near 0.5 (neutral).
const PRIOR: f64 = 3.0;
const SKIP_WEIGHT: f64 = 0.5;

#[derive(Debug, Clone, Default)]
pub struct Taste {
    /// tag → (good, bad)
    tags: HashMap<String, (f64, f64)>,
}

impl Taste {
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

    #[test]
    fn a_few_skips_barely_move_it() {
        let taste = Taste::from_history([("games", "skipped", None), ("games", "skipped", None)]);
        let f = taste.factor(&t("games"));
        assert!(f > 0.93 && f < 1.0, "{f}");
    }
}
