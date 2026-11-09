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
/// What a page may load: our own scripts plus the one inline theme script (by hash, from
/// web/index.html; update it if that script changes), styles from us and Google Fonts (inline
/// styles are used by React), fonts, images (photos come from our own signed links; uploads are
/// previewed as blob:), and API calls only to ourselves. Nothing may frame the app. A script
/// injected by some future bug can't run, and couldn't send a token anywhere if it did.
pub const CSP: &str = "default-src 'self'; \
script-src 'self' 'sha256-dKgSjMjLHzOElpOWR0uUDjeOd7Ua5Zg1zKyZwm4QY8s='; \
style-src 'self' 'unsafe-inline'; \
font-src 'self' data:; \
img-src 'self' data: blob:; \
media-src 'self' blob:; \
connect-src 'self'; \
worker-src 'self'; \
manifest-src 'self'; \
object-src 'none'; \
base-uri 'self'; \
form-action 'self'; \
frame-ancestors 'none'; \
upgrade-insecure-requests";
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
    set(h, header::CONTENT_SECURITY_POLICY, CSP);
    set(h, HeaderName::from_static("cross-origin-opener-policy"), "same-origin");
    set(h, HeaderName::from_static("permissions-policy"), "camera=(), microphone=(), geolocation=(), payment=(), usb=()");
    if https {
        set(h, header::STRICT_TRANSPORT_SECURITY, HSTS);
    }
    set(h, header::CACHE_CONTROL, cache);
    res
}
