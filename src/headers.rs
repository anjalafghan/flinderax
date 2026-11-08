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

pub async fn security_headers(req: Request, next: Next) -> Response {
    let https = req.headers().get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https");
    let api = req.uri().path().starts_with("/api/");
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
    if api {
        set(h, header::CACHE_CONTROL, "no-store");
    }
    res
}
