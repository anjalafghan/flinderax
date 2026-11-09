//! Headers on every response (carried over from Joy): links out don't say where they came from,
//! nothing can frame the app, browsers don't guess content types, the powerful APIs stay off,
//! HTTPS is pinned once seen, and personal API answers are never stored by a cache.

use axum::{
    extract::Request,
    http::{header, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};

const HSTS: &str = "max-age=31536000; includeSubDomains";
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// Build output whose name carries a content hash (`chunk-1dd7yscb.js`, `favicon-18zwz42s.svg`),
/// plus everything under `fonts/`. Those names change whenever the bytes change, so browsers can keep them.
fn is_immutable_asset(path: &str) -> bool {
    if path.starts_with("/fonts/") {
        return true;
    }
    let name = path.rsplit('/').next().unwrap_or_default();
    let Some((stem, ext)) = name.rsplit_once('.') else { return false };
    if !matches!(ext, "js" | "css" | "svg" | "png" | "woff" | "woff2") {
        return false;
    }
    // Bun's hashes are 8 lowercase base-36 characters after the last dash.
    stem.rsplit_once('-')
        .is_some_and(|(_, hash)| hash.len() == 8 && hash.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
}

pub async fn security_headers(req: Request, next: Next) -> Response {
    let https = req.headers().get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https");
    let path = req.uri().path();
    let api = path.starts_with("/api/");
    let cache = if api {
        "no-store"
    } else if is_immutable_asset(path) {
        IMMUTABLE
    } else {
        // index.html, sw.js, robots.txt and SPA routes: always revalidate (a 304 is cheap).
        "no-cache"
    };
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    let set = |h: &mut axum::http::HeaderMap, k: HeaderName, v: &'static str| {
        h.entry(k).or_insert(HeaderValue::from_static(v));
    };
    set(h, header::REFERRER_POLICY, "no-referrer");
    set(h, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    set(h, header::X_FRAME_OPTIONS, "DENY");
    set(h, HeaderName::from_static("cross-origin-opener-policy"), "same-origin");
    set(h, HeaderName::from_static("permissions-policy"), "camera=(), microphone=(), geolocation=(), payment=(), usb=()");
    if https {
        set(h, header::STRICT_TRANSPORT_SECURITY, HSTS);
    }
    set(h, header::CACHE_CONTROL, cache);
    res
}
