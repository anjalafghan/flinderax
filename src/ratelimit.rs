//! Tiny in-memory fixed-window rate limiter for the auth endpoints (single-process; resets on restart).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(60);

fn buckets() -> &'static Mutex<HashMap<String, (Instant, u32)>> {
    static B: OnceLock<Mutex<HashMap<String, (Instant, u32)>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Counts one hit for `key`; returns false once it has been hit more than `max` times in the window.
pub fn allow(key: &str, max: u32) -> bool {
    let now = Instant::now();
    let mut map = buckets().lock().unwrap_or_else(|e| e.into_inner());
    if map.len() > 10_000 {
        map.retain(|_, (start, _)| now.duration_since(*start) < WINDOW);
    }
    let entry = map.entry(key.to_string()).or_insert((now, 0));
    if now.duration_since(entry.0) >= WINDOW {
        *entry = (now, 0);
    }
    entry.1 += 1;
    entry.1 <= max
}

/// Best-effort client address behind a proxy (Fly sets `fly-client-ip`).
pub fn client_key(headers: &axum::http::HeaderMap) -> String {
    ["fly-client-ip", "x-forwarded-for"]
        .iter()
        .find_map(|h| headers.get(*h)?.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
