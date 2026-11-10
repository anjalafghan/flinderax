//! Tiny in-memory fixed-window rate limiter for the auth and hook endpoints (single-process;
//! resets on restart). Keys are hashed, so a caller-chosen key (a 2 MB "username") costs 32 bytes,
//! and the map is swept of expired windows and hard-capped, so it can't grow without bound.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

const WINDOW: Duration = Duration::from_secs(60);
/// Sweep expired windows once the map holds this many keys.
const SWEEP_AT: usize = 10_000;
/// After a sweep, a map still this large is cleared outright: a burst of distinct keys only ever
/// buys the attacker fresh windows, never memory.
const HARD_CAP: usize = 50_000;

type Key = [u8; 32];

fn buckets() -> &'static Mutex<HashMap<Key, (Instant, u32)>> {
    static B: OnceLock<Mutex<HashMap<Key, (Instant, u32)>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(HashMap::new()))
}

fn key_of(key: &str) -> Key {
    Sha256::digest(key.as_bytes()).into()
}

fn with_entry<T>(key: &str, f: impl FnOnce(&mut (Instant, u32), Instant) -> T) -> T {
    let now = Instant::now();
    let mut map = buckets().lock().unwrap_or_else(|e| e.into_inner());
    if map.len() >= SWEEP_AT {
        map.retain(|_, (start, _)| now.duration_since(*start) < WINDOW);
        if map.len() >= HARD_CAP {
            map.clear();
        }
    }
    let entry = map.entry(key_of(key)).or_insert((now, 0));
    if now.duration_since(entry.0) >= WINDOW {
        *entry = (now, 0);
    }
    f(entry, now)
}

/// Counts one hit for `key`; returns false once it has been hit more than `max` times in the window.
pub fn allow(key: &str, max: u32) -> bool {
    with_entry(key, |e, _| {
        e.1 += 1;
        e.1 <= max
    })
}

/// True while `key` has fewer than `max` hits in the window, without counting one. Pair with
/// [`hit`] to count only failures (so nobody can lock a user out by guessing on their behalf).
pub fn under(key: &str, max: u32) -> bool {
    with_entry(key, |e, _| e.1 < max)
}

/// Counts one hit for `key`.
pub fn hit(key: &str) {
    with_entry(key, |e, _| e.1 += 1);
}

/// The client's address as Fly's proxy saw it (`fly-client-ip`, which the proxy overwrites, so a
/// client can't forge it). Off Fly there is no trustworthy header, so every caller shares one
/// bucket. IPv6 addresses are cut to their /64, the block one subscriber controls.
pub fn client_key(headers: &axum::http::HeaderMap) -> String {
    let Some(ip) = headers.get("fly-client-ip").and_then(|v| v.to_str().ok()).map(str::trim) else {
        return "unknown".to_string();
    };
    match ip.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V6(v6)) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
        Ok(v4) => v4.to_string(),
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn huge_keys_cost_a_fixed_size() {
        let big = "x".repeat(2 * 1024 * 1024);
        assert!(allow(&big, 1));
        assert!(!allow(&big, 1));
        assert_eq!(key_of(&big).len(), 32);
    }

    #[test]
    fn under_and_hit_count_only_what_is_hit() {
        let k = "test-under-hit";
        for _ in 0..5 {
            assert!(under(k, 2), "checking alone never uses up the window");
        }
        hit(k);
        hit(k);
        assert!(!under(k, 2));
    }

    #[test]
    fn client_key_trusts_only_flys_header_and_groups_ipv6_by_64() {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", HeaderValue::from_static("1.2.3.4"));
        assert_eq!(client_key(&h), "unknown", "x-forwarded-for is client-controlled");
        h.insert("fly-client-ip", HeaderValue::from_static("203.0.113.9"));
        assert_eq!(client_key(&h), "203.0.113.9");
        h.insert("fly-client-ip", HeaderValue::from_static("2001:db8:aa:bb:1:2:3:4"));
        let a = client_key(&h);
        h.insert("fly-client-ip", HeaderValue::from_static("2001:db8:aa:bb:ffff::9"));
        assert_eq!(a, client_key(&h));
        assert_eq!(a, "2001:db8:aa:bb::/64");
    }
}
