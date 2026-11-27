//! Remote MCP at `/mcp`, served from Joy's own machine instead of a second Fly app.
//!
//! The MCP tools are TypeScript (`mcp/src/http.ts`), so Joy starts them under Bun as a child process
//! listening only on loopback, and forwards `/mcp` to it. The Bun side checks the caller's scoped API
//! token against `/api/token-info` on every request and calls Joy back over loopback with that same
//! token, so scopes, audit rows and the privacy rules all apply exactly as for any other token call.
//!
//! Off unless `JOY_MCP_ENTRY` names the script (the Docker image sets it); `/mcp` is then a 404.

use std::sync::OnceLock;
use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderName, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tracing::{error, info, warn};

/// Bigger bodies are refused here before they reach Bun (the same limit http.ts applies).
const MAX_BODY: usize = 1_000_000;

/// Headers passed to Bun. Anything else (cookies above all) stays behind: the MCP side only ever
/// authenticates with the Bearer token.
const REQUEST_HEADERS: [HeaderName; 6] = [
    header::AUTHORIZATION,
    header::CONTENT_TYPE,
    header::ACCEPT,
    HeaderName::from_static("mcp-session-id"),
    HeaderName::from_static("mcp-protocol-version"),
    HeaderName::from_static("last-event-id"),
];

const RESPONSE_HEADERS: [HeaderName; 4] = [
    header::CONTENT_TYPE,
    header::WWW_AUTHENTICATE,
    header::ALLOW,
    HeaderName::from_static("mcp-session-id"),
];

struct Sidecar {
    port: u16,
    client: reqwest::Client,
}

static SIDECAR: OnceLock<Sidecar> = OnceLock::new();

/// The host Claude connects to (`joy.fly.dev`, or `localhost:3000` locally), from `PUBLIC_URL`.
/// Bun's DNS-rebinding check only accepts requests whose Host is in this list.
fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let host = rest.split('/').next()?.trim();
    (!host.is_empty()).then(|| host.to_string())
}

/// Starts the Bun MCP server and keeps it running (restarting with a back-off if it exits).
/// `joy_port` is the port Joy itself listens on, which Bun calls back over loopback.
pub fn start(joy_port: &str) {
    let Ok(entry) = std::env::var("JOY_MCP_ENTRY") else { return };
    let port: u16 = std::env::var("JOY_MCP_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3001);
    let bun = std::env::var("JOY_MCP_BUN").unwrap_or_else(|_| "bun".into());
    let hosts = std::env::var("PUBLIC_URL").ok().and_then(|u| host_of(&u)).unwrap_or_else(|| format!("localhost:{joy_port}"));
    let client = match reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(55)).build() {
        Ok(c) => c,
        Err(e) => {
            error!("MCP: could not build the loopback client: {e}");
            return;
        }
    };
    if SIDECAR.set(Sidecar { port, client }).is_err() {
        return;
    }
    let joy_url = format!("http://127.0.0.1:{joy_port}");
    tokio::spawn(async move {
        let mut backoff = Duration::from_secs(1);
        loop {
            let started = std::time::Instant::now();
            let child = tokio::process::Command::new(&bun)
                .arg("run")
                .arg(&entry)
                .env("PORT", port.to_string())
                .env("JOY_MCP_BIND", "127.0.0.1")
                .env("JOY_URL", &joy_url)
                .env("JOY_MCP_HOSTS", &hosts)
                .env("NODE_ENV", "production")
                .kill_on_drop(true)
                .spawn();
            match child {
                Ok(mut child) => {
                    info!("MCP: started on 127.0.0.1:{port}");
                    match child.wait().await {
                        Ok(status) => warn!("MCP: exited ({status})"),
                        Err(e) => warn!("MCP: lost the process: {e}"),
                    }
                }
                Err(e) => error!("MCP: could not start {bun}: {e}"),
            }
            // A server that ran a while gets restarted at once; one that keeps dying waits longer.
            if started.elapsed() > Duration::from_secs(60) {
                backoff = Duration::from_secs(1);
            }
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_secs(60));
        }
    });
}

/// `GET | POST | DELETE /mcp`: passed to the Bun server with the original Host.
pub async fn proxy(req: Request) -> Response {
    let Some(sidecar) = SIDECAR.get() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let (parts, body) = req.into_parts();
    let Ok(body) = axum::body::to_bytes(body, MAX_BODY).await else {
        return (StatusCode::PAYLOAD_TOO_LARGE, "Request too large").into_response();
    };
    let mut out = sidecar
        .client
        .request(parts.method.clone(), format!("http://127.0.0.1:{}/mcp", sidecar.port))
        .body(body);
    for name in REQUEST_HEADERS.iter().chain([&header::HOST]) {
        if let Some(value) = parts.headers.get(name) {
            out = out.header(name, value);
        }
    }
    let res = match out.send().await {
        Ok(r) => r,
        Err(e) => {
            // Never log the request: the URL is fixed, and the headers carry the token.
            warn!("MCP: sidecar unreachable: {}", if e.is_timeout() { "timeout" } else { "connect" });
            return (StatusCode::SERVICE_UNAVAILABLE, "MCP is starting, try again").into_response();
        }
    };
    let status = res.status();
    let mut builder = Response::builder().status(status);
    for name in &RESPONSE_HEADERS {
        if let Some(value) = res.headers().get(name) {
            builder = builder.header(name, value);
        }
    }
    let bytes = match res.bytes().await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_GATEWAY, "MCP response failed").into_response(),
    };
    builder.body(Body::from(bytes)).unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response())
}

#[cfg(test)]
mod tests {
    #[test]
    fn host_of_strips_scheme_and_path() {
        assert_eq!(super::host_of("https://joy.fly.dev/").as_deref(), Some("joy.fly.dev"));
        assert_eq!(super::host_of("http://localhost:3050").as_deref(), Some("localhost:3050"));
        assert_eq!(super::host_of("https://"), None);
    }
}
