//! Tonight's rain chance from Open-Meteo (free, no key). Cached for an hour and
//! fails open: any error just means "no forecast", never a broken jar.
//!
//! Three things keep it cheap and honest: the cache is per calendar day (an answer from last night
//! is never "tonight"); one fetch at a time, so a burst of requests after the cache expires makes
//! one call, not many, and nobody waits on it once there is an answer from today to give; and when
//! the API is down we keep showing the last good answer from today instead of dropping the hint.

use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Deserialize;
use time::Date;

/// Rain counts as "likely" at or above this chance.
pub const LIKELY_PERCENT: u8 = 60;
const FRESH: Duration = Duration::from_secs(60 * 60);
const RETRY_AFTER_ERROR: Duration = Duration::from_secs(10 * 60);

/// What the last lookup learned: the local day it is for, when, and the answer (None = unknown).
#[derive(Clone, Copy)]
struct Entry {
    day: Date,
    at: Instant,
    value: Option<u8>,
}

/// The cached value to serve now, if the entry is still good for `today`.
fn fresh(e: &Entry, today: Date, now: Instant) -> Option<Option<u8>> {
    let ttl = if e.value.is_some() { FRESH } else { RETRY_AFTER_ERROR };
    (e.day == today && now.duration_since(e.at) < ttl).then_some(e.value)
}

pub struct Weather {
    /// Where to look: between our two homes. None turns the feature off.
    at: Option<(f64, f64)>,
    client: reqwest::Client,
    /// Held only for a moment, never across the network call.
    cache: Mutex<Option<Entry>>,
    /// A refresh is in flight; others keep serving what they have.
    refreshing: AtomicBool,
    /// WEATHER_FAKE_PERCENT, for design work: pretend tonight's rain chance is this.
    fake: Option<u8>,
}

#[derive(Deserialize)]
struct Forecast {
    hourly: Hourly,
}

#[derive(Deserialize)]
struct Hourly {
    time: Vec<String>,
    precipitation_probability: Vec<Option<u8>>,
}

impl Weather {
    pub fn new(at: Option<(f64, f64)>, fake: Option<u8>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("http client");
        Self { at, client, cache: Mutex::new(None), refreshing: AtomicBool::new(false), fake }
    }

    /// Highest rain chance between 17:00 and 23:00 local time today, if we know it.
    ///
    /// Never makes a caller wait on the network when it has anything to say: a fresh answer is
    /// returned as is; an expired one from today is returned at once while one background task
    /// refreshes it; and with nothing at all (first look of the day) the one caller who gets there
    /// first fetches while everyone else carries on without a forecast.
    pub async fn tonight(self: &Arc<Self>) -> Option<u8> {
        if self.fake.is_some() {
            return self.fake;
        }
        let (lat, lng) = self.at?;
        let today = crate::handlers::common::today_local();
        let held = *self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(hit) = held.as_ref().and_then(|e| fresh(e, today, Instant::now())) {
            return hit;
        }
        let last_good = held.filter(|e| e.day == today).and_then(|e| e.value);
        if self.refreshing.swap(true, Ordering::AcqRel) {
            return last_good; // someone is already on it
        }
        if held.is_some_and(|e| e.day == today) {
            let me = Arc::clone(self);
            tokio::spawn(async move { me.refresh(lat, lng, today).await });
            return last_good;
        }
        self.refresh(lat, lng, today).await
    }

    /// Fetch, store, and let the next caller start a refresh again. Returns the new answer.
    async fn refresh(&self, lat: f64, lng: f64, today: Date) -> Option<u8> {
        let fetched = self.fetch(lat, lng).await;
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        let value = match fetched {
            Some(v) => Some(v),
            None => {
                tracing::warn!("weather: no forecast, carrying on without it");
                // Keep today's last good answer rather than losing the hint to a blip.
                cache.as_ref().filter(|e| e.day == today).and_then(|e| e.value)
            }
        };
        *cache = Some(Entry { day: today, at: Instant::now(), value });
        drop(cache);
        self.refreshing.store(false, Ordering::Release);
        value
    }

    async fn fetch(&self, lat: f64, lng: f64) -> Option<u8> {
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={lat:.3}&longitude={lng:.3}\
             &hourly=precipitation_probability&forecast_days=1&timezone=auto"
        );
        let f: Forecast = self.client.get(url).send().await.ok()?.error_for_status().ok()?.json().await.ok()?;
        evening_max(&f.hourly)
    }
}

/// Times look like "2026-10-17T19:00" in the location's own timezone.
fn evening_max(h: &Hourly) -> Option<u8> {
    h.time
        .iter()
        .zip(&h.precipitation_probability)
        .filter(|(t, _)| t.get(11..13).and_then(|hh| hh.parse::<u8>().ok()).is_some_and(|hh| (17..=23).contains(&hh)))
        .filter_map(|(_, p)| *p)
        .max()
}

/// Middle of the two homes (or whichever one is set).
pub fn midpoint(homes: [Option<(f64, f64)>; 2]) -> Option<(f64, f64)> {
    match homes {
        [Some(a), Some(b)] => Some(((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)),
        [Some(a), None] | [None, Some(a)] => Some(a),
        [None, None] => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_evening_peak_only() {
        let h = Hourly {
            time: (0..24).map(|i| format!("2026-10-17T{i:02}:00")).collect(),
            precipitation_probability: (0..24).map(|i| Some(if i == 9 { 95 } else if i == 20 { 70 } else { 10 })).collect(),
        };
        assert_eq!(evening_max(&h), Some(70));
    }

    fn entry(day: Date, age: u64, value: Option<u8>) -> (Entry, Instant) {
        let now = Instant::now();
        (Entry { day, at: now, value }, now + Duration::from_secs(age))
    }

    #[test]
    fn cache_is_per_day_and_expires() {
        let d1 = time::macros::date!(2026 - 11 - 15);
        let d2 = time::macros::date!(2026 - 11 - 16);
        let (e, soon) = entry(d1, 60, Some(80));
        assert_eq!(fresh(&e, d1, soon), Some(Some(80)));
        // Yesterday's answer is never tonight's.
        assert_eq!(fresh(&e, d2, soon), None);
        let (e, later) = entry(d1, 61 * 60, Some(80));
        assert_eq!(fresh(&e, d1, later), None);
        // A failed lookup is retried sooner than a good one expires.
        let (e, t) = entry(d1, 11 * 60, None);
        assert_eq!(fresh(&e, d1, t), None);
        let (e, t) = entry(d1, 5 * 60, None);
        assert_eq!(fresh(&e, d1, t), Some(None));
    }

    #[test]
    fn midpoint_of_homes() {
        assert_eq!(midpoint([Some((19.0, 73.0)), Some((19.2, 72.8))]).map(|(a, b)| (a.round(), b.round())), Some((19.0, 73.0)));
        assert_eq!(midpoint([None, Some((1.0, 2.0))]), Some((1.0, 2.0)));
        assert_eq!(midpoint([None, None]), None);
    }
}
