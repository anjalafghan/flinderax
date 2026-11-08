//! The per-couple half of Joy's `config.rs`: names, homes, currency, schedule. Everything that
//! was server-wide there (passcode, cookie key, host, port, API token) lives in Flinderax instead.
//! Joy read these from .env; here they come from the couple's `joy_settings` rows under the same names.

use std::{collections::HashMap, path::{Path, PathBuf}};

use sqlx::SqlitePool;

pub struct Ctx {
    pub currency: String,
    /// The two of us: (name, home lat/lng). Homes only live in settings, never anywhere public.
    pub people: [(String, Option<(f64, f64)>); 2],
    /// Where kept-date photos are stored (one folder per couple).
    pub photos_dir: PathBuf,
    /// Neighbourhood names (e.g. "Bandra") used in personal slip notes. Optional.
    pub areas: [Option<String>; 2],
    /// When we meet and where we start from. None (no DATE_DAYS) = draw for any time.
    pub schedule: Option<crate::joy::schedule::Schedule>,
    /// Where the app lives on the web, for links we send each other.
    pub public_url: String,
}

/// Read a couple's settings the way Joy read its environment: blank means unset, and one
/// matching pair of quotes around a value is stripped.
fn clean(v: &str) -> Option<String> {
    let v = v.trim();
    let v = ['\'', '"']
        .iter()
        .find_map(|q| v.strip_prefix(*q).and_then(|s| s.strip_suffix(*q)))
        .unwrap_or(v);
    (!v.trim().is_empty()).then(|| v.to_string())
}

/// Build a couple's `Ctx` from their `joy_settings`.
pub async fn load(db: &SqlitePool, couple_id: &str, public_url: &str, photos_root: &Path) -> sqlx::Result<Ctx> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM joy_settings WHERE couple_id = ?")
        .bind(couple_id)
        .fetch_all(db)
        .await?;
    let vars: HashMap<String, String> = rows.into_iter().filter_map(|(k, v)| clean(&v).map(|v| (k, v))).collect();
    let var = |k: &str| vars.get(k).cloned();

    Ok(Ctx {
        public_url: public_url.trim_end_matches('/').to_string(),
        currency: var("CURRENCY_SYMBOL").unwrap_or_else(|| "₹".into()),
        people: [
            (var("PERSON_A").unwrap_or_else(|| "You".into()), var("HOME_A").and_then(|s| parse_latlng(&s))),
            (var("PERSON_B").unwrap_or_else(|| "Partner".into()), var("HOME_B").and_then(|s| parse_latlng(&s))),
        ],
        areas: [var("HOME_A_AREA"), var("HOME_B_AREA")],
        schedule: crate::joy::schedule::Schedule::from_vars(var),
        photos_dir: photos_root.join(couple_id),
    })
}

impl Ctx {
    /// A context for unit tests: two named people with homes, no schedule, no weather lookups.
    #[cfg(test)]
    pub fn for_tests() -> Self {
        Self {
            currency: "₹".into(),
            people: [("Rohit".into(), Some((19.00, 73.05))), ("Neha".into(), Some((19.28, 72.87)))],
            areas: [None, None],
            schedule: None,
            public_url: "http://localhost:3000".into(),
            photos_dir: "photos".into(),
        }
    }

    pub fn money(&self, amount: i64) -> String {
        if amount == 0 {
            return "Free".into();
        }
        let digits = amount.to_string();
        let grouped = if self.currency == "₹" { group_indian(&digits) } else { group_western(&digits) };
        format!("{}{}", self.currency, grouped)
    }

    /// Fills `{a}`, `{b}`, `{a_area}`, `{b_area}` in slip text from .env, so the shared
    /// seed stays generic. A `[[…]]` block only shows when every value inside it is set.
    pub fn personalize(&self, text: &str) -> String {
        if !text.contains('{') && !text.contains("[[") {
            return text.to_string();
        }
        let vars: [(&str, Option<&str>); 4] = [
            ("{a_area}", self.areas[0].as_deref()),
            ("{b_area}", self.areas[1].as_deref()),
            ("{a}", Some(&self.people[0].0)),
            ("{b}", Some(&self.people[1].0)),
        ];
        let fill = |s: &str| -> Option<String> {
            let mut out = s.to_string();
            for (key, val) in &vars {
                if out.contains(key) {
                    out = out.replace(key, (*val)?);
                }
            }
            Some(out)
        };
        let mut out = String::new();
        let mut rest = text;
        while let Some(start) = rest.find("[[") {
            out += &fill(&rest[..start]).unwrap_or_default();
            let after = &rest[start + 2..];
            let Some(end) = after.find("]]") else {
                rest = after;
                break;
            };
            out += &fill(&after[..end]).unwrap_or_default();
            rest = &after[end + 2..];
        }
        out += &fill(rest).unwrap_or_default();
        out
    }

    /// Approx road km from each person's home to a venue.
    pub fn distances(&self, venue: Option<(f64, f64)>) -> [Option<f64>; 2] {
        let d = |home: Option<(f64, f64)>| Some(road_km(home?, venue?));
        [d(self.people[0].1), d(self.people[1].1)]
    }
}

/// Accepts "19.13, 72.81" (what Google Maps copies on long-press).
pub fn parse_latlng(s: &str) -> Option<(f64, f64)> {
    let (a, b) = s.split_once(',')?;
    let (lat, lng) = (a.trim().parse::<f64>().ok()?, b.trim().parse::<f64>().ok()?);
    ((-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng)).then_some((lat, lng))
}

/// Straight-line distance × 1.3, a common rule of thumb for city road distance.
pub fn road_km(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (la1, lo1, la2, lo2) = (a.0.to_radians(), a.1.to_radians(), b.0.to_radians(), b.1.to_radians());
    let h = ((la2 - la1) / 2.0).sin().powi(2) + la1.cos() * la2.cos() * ((lo2 - lo1) / 2.0).sin().powi(2);
    6371.0 * 2.0 * h.sqrt().asin() * 1.3
}

fn group_western(d: &str) -> String {
    let mut out = String::new();
    for (i, c) in d.chars().enumerate() {
        if i > 0 && (d.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn group_indian(d: &str) -> String {
    if d.len() <= 3 {
        return d.into();
    }
    let (head, tail) = d.split_at(d.len() - 3);
    let mut out = String::new();
    for (i, c) in head.chars().enumerate() {
        if i > 0 && (head.len() - i) % 2 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{out},{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_money() {
        assert_eq!(group_indian("150000"), "1,50,000");
        assert_eq!(group_indian("1200"), "1,200");
        assert_eq!(group_indian("800"), "800");
        assert_eq!(group_western("1500000"), "1,500,000");
    }

    fn cfg(areas: [Option<&str>; 2]) -> Ctx {
        Ctx {
            currency: "₹".into(),
            people: [("Rohit".into(), None), ("Neha".into(), None)],
            areas: areas.map(|a| a.map(String::from)),
            schedule: None,
            public_url: "http://localhost:3000".into(),
            photos_dir: "photos".into(),
        }
    }

    #[test]
    fn personalizes_slip_text() {
        let mine = cfg([Some("Powai"), Some("Bandra")]);
        let generic = cfg([None, None]);
        let note = "Go early.[[ Close to {b_area}.]]";
        assert_eq!(mine.personalize(note), "Go early. Close to Bandra.");
        assert_eq!(generic.personalize(note), "Go early.");
        assert_eq!(generic.personalize("Hindi rap (watch {b} react)"), "Hindi rap (watch Neha react)");
        assert_eq!(
            mine.personalize("[[Midway between {a_area} and {b_area}. ]]Pin unverified."),
            "Midway between Powai and Bandra. Pin unverified."
        );
        assert_eq!(generic.personalize("plain text"), "plain text");
    }

    #[test]
    fn parses_latlng_and_measures() {
        assert_eq!(parse_latlng("19.13, 72.81"), Some((19.13, 72.81)));
        assert_eq!(parse_latlng("nope"), None);
        // Gateway of India -> Bandra Bandstand is ~13 km straight line
        let km = road_km((18.922, 72.834), (19.045, 72.818)) / 1.3;
        assert!((13.0..15.0).contains(&km), "{km}");
    }
}
