//! One vocabulary for moods. Tags were free text, so "Cosy", "cozy" and "homey" split the same
//! idea three ways and a slip tagged outside the twelve moods could never match a vibe. Saving a
//! slip now maps well-known near-duplicates onto the canonical word; anything else a person types
//! is kept as they wrote it (lower-cased).

/// The moods the jar, the filters and "Meet in the middle" speak in.
pub const CANONICAL: [&str; 12] =
    ["romantic", "chill", "foodie", "cozy", "games", "cultural", "outdoors", "creative", "adventurous", "music", "active", "nightlife"];

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
];

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
    fn every_alias_points_at_a_canonical_word() {
        for (alias, target) in ALIASES {
            assert!(CANONICAL.contains(target), "{alias} -> {target}");
            assert_ne!(alias, target);
        }
    }
}
