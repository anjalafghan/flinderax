//! One vocabulary for moods. Tags were free text, so "Cosy", "cozy" and "homey" split the same
//! idea three ways and a slip tagged outside the twelve moods could never match a vibe. Saving a
//! slip now maps well-known near-duplicates onto the canonical word; anything else a person types
//! is kept as they wrote it (lower-cased).

/// The moods the jar, the filters and "Meet in the middle" speak in.
pub const CANONICAL: [&str; 23] = [
    "romantic", "chill", "foodie", "cozy", "games", "cultural", "outdoors", "creative", "adventurous", "music", "active", "nightlife",
    // what it is
    "social", "learning", "wellness", "sweet", "movies", "milestone", "spontaneous", "scenic",
    // how much of us it takes: a tired night, a short window, a long stretch
    "low-effort", "quick", "leisurely",
];

/// (what people write, the canonical word)
const ALIASES: &[(&str, &str)] = &[
    ("romance", "romantic"),
    ("date-night", "romantic"),
    ("relaxed", "chill"),
    ("relax", "chill"),
    ("calm", "chill"),
    ("lazy", "chill"),
    ("restaurant", "foodie"),
    ("restaurants", "foodie"),
    ("food", "foodie"),
    ("dining", "foodie"),
    ("dinner", "foodie"),
    ("brunch", "foodie"),
    ("cosy", "cozy"),
    ("homey", "cozy"),
    ("game", "games"),
    ("gaming", "games"),
    ("board games", "games"),
    ("culture", "cultural"),
    ("art", "cultural"),
    ("arts", "cultural"),
    ("museum", "cultural"),
    ("outdoor", "outdoors"),
    ("outside", "outdoors"),
    ("nature", "outdoors"),
    ("creativity", "creative"),
    ("craft", "creative"),
    ("crafts", "creative"),
    ("diy", "creative"),
    ("adventure", "adventurous"),
    ("live music", "music"),
    ("concert", "music"),
    ("sport", "active"),
    ("sports", "active"),
    ("fitness", "active"),
    ("exercise", "active"),
    ("party", "nightlife"),
    ("night", "nightlife"),
    ("friends", "social"),
    ("group", "social"),
    ("hangout", "social"),
    ("class", "learning"),
    ("workshop", "learning"),
    ("course", "learning"),
    ("spa", "wellness"),
    ("self-care", "wellness"),
    ("yoga", "wellness"),
    ("dessert", "sweet"),
    ("cafe", "sweet"),
    ("coffee", "sweet"),
    ("film", "movies"),
    ("cinema", "movies"),
    ("theatre", "movies"),
    ("anniversary", "milestone"),
    ("birthday", "milestone"),
    ("celebration", "milestone"),
    ("last-minute", "spontaneous"),
    ("impulsive", "spontaneous"),
    ("view", "scenic"),
    ("sunset", "scenic"),
    ("drive", "scenic"),
    ("easy", "low-effort"),
    ("effortless", "low-effort"),
    ("low effort", "low-effort"),
    ("tired", "low-effort"),
    ("fast", "quick"),
    ("short", "quick"),
    ("less time", "quick"),
    ("long", "leisurely"),
    ("unhurried", "leisurely"),
    ("slow", "leisurely"),
    ("more time", "leisurely"),
];

/// The effort and time moods a slip has by its own fields, so older slips (and ones nobody tagged)
/// take part in "something easy tonight" and in the taste model without being edited. Never
/// stored: change a slip's duration or energy and these follow.
pub fn implied(duration: crate::joy::picker::Duration, energy: &str) -> Vec<&'static str> {
    use crate::joy::picker::Duration::*;
    let mut v = Vec::new();
    if energy == "low" {
        v.push("low-effort");
    }
    match duration {
        Short => v.push("quick"),
        FullDay | Weekend => v.push("leisurely"),
        HalfDay => {}
    }
    v
}

/// `tags` plus whatever `implied` adds that is not already there (own tags keep their order).
pub fn with_implied(mut tags: Vec<String>, duration: crate::joy::picker::Duration, energy: &str) -> Vec<String> {
    for t in implied(duration, energy) {
        if !tags.iter().any(|x| x == t) {
            tags.push(t.to_string());
        }
    }
    tags
}

/// One tag, lower-cased, with a known alias mapped to its canonical word.
pub fn canonical(tag: &str) -> String {
    let t = tag.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    ALIASES.iter().find(|(a, _)| *a == t).map_or(t, |(_, c)| (*c).to_string())
}

/// A comma-separated list, each tag made canonical, empty ones dropped, repeats removed (order kept).
pub fn normalise(raw: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for t in raw.split(',').map(canonical).filter(|t| !t.is_empty()) {
        if !out.contains(&t) {
            out.push(t);
        }
    }
    out.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_duplicates_collapse_onto_the_canonical_word() {
        assert_eq!(canonical("Cosy"), "cozy");
        assert_eq!(canonical("  Board   Games "), "games");
        assert_eq!(canonical("Restaurant"), "foodie");
        assert_eq!(canonical("outdoor"), "outdoors");
        assert_eq!(canonical("Low  Effort"), "low-effort");
        assert_eq!(canonical("Less time"), "quick");
        assert_eq!(canonical("more time"), "leisurely");
    }

    #[test]
    fn canonical_words_and_custom_tags_are_left_alone() {
        for t in CANONICAL {
            assert_eq!(canonical(t), t);
        }
        assert_eq!(canonical("Pottery"), "pottery", "a tag we don't know is kept, lower-cased");
    }

    #[test]
    fn a_list_is_cleaned_and_deduplicated_in_order() {
        assert_eq!(normalise("Cosy, cozy, Food, , pottery, FOODIE"), "cozy,foodie,pottery");
        assert_eq!(normalise(""), "");
    }

    #[test]
    fn effort_and_time_follow_from_energy_and_duration() {
        use crate::joy::picker::Duration::*;
        assert_eq!(implied(Short, "low"), ["low-effort", "quick"]);
        assert_eq!(implied(HalfDay, "med"), Vec::<&str>::new());
        assert_eq!(implied(Weekend, "high"), ["leisurely"]);
        let own = with_implied(vec!["quick".into(), "foodie".into()], Short, "low");
        assert_eq!(own, ["quick", "foodie", "low-effort"], "no repeats; own tags stay first");
    }

    #[test]
    fn every_alias_points_at_a_canonical_word() {
        for (alias, target) in ALIASES {
            assert!(CANONICAL.contains(target), "{alias} -> {target}");
            assert_ne!(alias, target);
        }
    }
}
