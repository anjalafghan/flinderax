//! Built-in merchant memory: well-known Indian merchants and the category they belong to, read
//! from the versioned `data/merchant_seed.json`. Pure: no IO. Keys are `merchant_key` outputs.
//!
//! Lookup order for a payee is the user's own rules, then this seed, then the keyword lists in
//! `alerts::categorize`.

use std::sync::OnceLock;

const SEED_JSON: &str = include_str!("../../data/merchant_seed.json");

struct Seed {
    version: u32,
    /// (merchant key, category), longest key first so the most specific name wins.
    entries: Vec<(String, String)>,
}

fn seed() -> &'static Seed {
    static SEED: OnceLock<Seed> = OnceLock::new();
    SEED.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(SEED_JSON).expect("data/merchant_seed.json is valid JSON");
        let version = v["version"].as_u64().expect("seed has a version") as u32;
        let mut entries: Vec<(String, String)> = v["merchants"]
            .as_object()
            .expect("seed has merchants")
            .iter()
            .map(|(k, c)| (k.clone(), c.as_str().expect("category is a string").to_string()))
            .collect();
        entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
        Seed { version, entries }
    })
}

/// Version of the bundled seed.
pub fn version() -> u32 {
    seed().version
}

/// Every (merchant key, category) pair in the seed.
pub fn entries() -> impl Iterator<Item = (&'static str, &'static str)> {
    seed().entries.iter().map(|(k, c)| (k.as_str(), c.as_str()))
}

/// The seeded category for a `merchant_key`: the longest seed key that appears in it as whole
/// words (`swiggy bangalore` -> swiggy, `swiggy instamart` -> the instamart entry). `None` for an
/// empty key or no match.
pub fn category_for_key(key: &str) -> Option<&'static str> {
    if key.is_empty() {
        return None;
    }
    let padded = format!(" {key} ");
    seed().entries.iter().find(|(k, _)| padded.contains(&format!(" {k} "))).map(|(_, c)| c.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::CATEGORIES;

    #[test]
    fn every_seed_category_is_a_real_app_category() {
        assert!(entries().count() >= 55, "the seed should cover the well-known merchants");
        for (key, cat) in entries() {
            assert!(CATEGORIES.contains(&cat), "seed key {key:?} has unknown category {cat:?}");
            assert_ne!(cat, "uncategorized", "{key:?}");
        }
    }

    #[test]
    fn seed_keys_are_already_in_key_form() {
        for (key, _) in entries() {
            assert_eq!(crate::planner::merchant::merchant_key(key), key, "seed key {key:?} must equal its own merchant_key");
        }
        assert!(version() >= 1);
    }

    #[test]
    fn most_specific_key_wins() {
        assert_eq!(category_for_key("swiggy"), Some("food"));
        assert_eq!(category_for_key("swiggy instamart"), Some("groceries"));
        assert_eq!(category_for_key("amazon"), Some("shopping"));
        assert_eq!(category_for_key("amazon pay"), Some("shopping"));
        assert_eq!(category_for_key("swiggy bangalore"), Some("food"));
        assert_eq!(category_for_key(""), None);
        assert_eq!(category_for_key("swiggyfoo"), None);
    }
}
