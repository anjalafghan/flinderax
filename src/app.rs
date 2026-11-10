use crate::handlers::common::AppError;
use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::any,
};
use rusty_paseto::{
    core::{Local, V4},
    prelude::{PasetoParser, PasetoSymmetricKey},
};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::compression::{CompressionLayer, CompressionLevel};
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::{Level, error};

use crate::models::AppState;
use crate::routes;

/// Directory holding the built web app (`bun run build` output). One server serves both
/// the API and the SPA, so there is nothing else to run.
pub fn static_dir() -> String {
    std::env::var("STATIC_DIR").unwrap_or_else(|_| "web/dist".to_string())
}

/// Logs each request by method and route TEMPLATE ("/api/public/reminders/{file}"), never the real
/// path or query: those carry calendar-feed tokens, envelope tokens and photo signatures, and Fly
/// keeps the logs.
fn request_span(req: &axum::http::Request<axum::body::Body>) -> tracing::Span {
    let route = req.extensions().get::<axum::extract::MatchedPath>().map_or("unmatched", |m| m.as_str());
    tracing::info_span!("request", method = %req.method(), route)
}

type MakeSpan = fn(&axum::http::Request<axum::body::Body>) -> tracing::Span;
type Trace = TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    MakeSpan,
    tower_http::trace::DefaultOnRequest,
    DefaultOnResponse,
>;

fn trace_layer() -> Trace {
    TraceLayer::new_for_http()
        .make_span_with(request_span as MakeSpan)
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

/// `router` behind the PASETO token check.
fn protected(state: &AppState, router: Router) -> Router {
    router
        .layer(trace_layer())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            token_validator_middleware,
        ))
}

async fn api_not_found() -> AppError {
    AppError(StatusCode::NOT_FOUND, "Not found".to_string())
}

/// The web app is served by this same server, so cross-origin access is off unless
/// `CORS_ORIGINS` (comma-separated) lists origins that may call the API.
fn cors_layer() -> CorsLayer {
    let origins: Vec<_> = std::env::var("CORS_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|o| o.trim().parse().ok())
        .collect();
    if origins.is_empty() {
        return CorsLayer::new();
    }
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(Any)
        .allow_headers(Any)
}

pub fn build_router(state: AppState) -> Router {
    let dir = static_dir();
    let index = format!("{dir}/index.html");

    Router::new()
        .nest(
            "/api/user",
            routes::user::routes(state.clone())
                .layer(trace_layer())
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    token_validator_middleware,
                )),
        )
        .nest(
            "/api/common",
            routes::common::routes(state.clone()).layer(trace_layer()),
        )
        .nest("/api/card", protected(&state, routes::card::routes(state.clone())))
        .nest("/api/account", protected(&state, routes::account::routes(state.clone())))
        .nest("/api/plan", protected(&state, routes::plan::routes(state.clone())))
        .nest("/api/dashboard", protected(&state, routes::dashboard::routes(state.clone())))
        .nest("/api/moves", protected(&state, routes::moves::routes(state.clone())))
        .nest("/api/ingest", protected(&state, routes::ingest::routes(state.clone())))
        .nest("/api/joy", protected(&state, routes::joy::routes(state.clone())))
        .nest("/api/couple", protected(&state, routes::couple::routes(state.clone())))
        .nest("/api/dates", protected(&state, routes::dates::routes(state.clone())))
        .nest("/api/public/dates", routes::dates::public_routes(state.clone()).layer(trace_layer()))
        .nest("/api/bills", protected(&state, routes::bills::routes(state.clone())))
        .nest("/api/checkin", protected(&state, routes::checkin::routes(state.clone())))
        .nest("/api/diet", protected(&state, routes::diet::routes(state.clone())))
        .nest("/api/people", protected(&state, routes::people::routes(state.clone())))
        .nest("/api/work", protected(&state, routes::work::routes(state.clone())))
        .nest("/api/reminders", protected(&state, routes::reminders::routes(state.clone())))
        .nest("/api/public/reminders", routes::reminders::public_routes(state.clone()).layer(trace_layer()))
        .nest("/api/goals", protected(&state, routes::goals::routes(state.clone())))
        .nest(
            "/api/export",
            protected(&state, Router::new().route("/", axum::routing::get(crate::handlers::export::export)).with_state(state.clone())),
        )
        .nest("/api/review", protected(&state, routes::review::routes(state.clone())))
        .nest("/api/hook", routes::ingest::hook_routes(state.clone()).layer(trace_layer()))
        // Unknown API paths must 404 instead of falling through to the SPA's index.html.
        .route("/api/{*rest}", any(api_not_found))
        // Joy's old addresses (envelope links already sent on WhatsApp, bookmarks) now live under /joy.
        .route("/e/{token}", axum::routing::get(old_joy_envelope))
        .route("/wishlist", axum::routing::get(|| async { axum::response::Redirect::permanent("/joy/slips") }))
        .route("/history", axum::routing::get(|| async { axum::response::Redirect::permanent("/joy/kept") }))
        .route("/regulars", axum::routing::get(|| async { axum::response::Redirect::permanent("/joy/regulars") }))
        .route("/year", axum::routing::get(|| async { axum::response::Redirect::permanent("/joy/year") }))
        .route("/year/{*rest}", axum::routing::get(old_joy_year))
        .route("/who", axum::routing::get(|| async { axum::response::Redirect::permanent("/joy/settings") }))
        .route("/login", axum::routing::get(|| async { axum::response::Redirect::permanent("/auth") }))
        // Static files come from the fallback. It must be set before the layers below: `Router::layer`
        // only wraps what exists when it is called, so headers, CORS and compression would skip it.
        // `<file>.br` / `<file>.gz` are served when present, so those aren't compressed on the fly.
        .fallback_service(
            ServeDir::new(&dir)
                .precompressed_br()
                .precompressed_gzip()
                .fallback(axum::handler::HandlerWithoutStateExt::into_service(spa_fallback(&index))),
        )
        .layer(cors_layer())
        .layer(middleware::from_fn(crate::headers::security_headers))
        // Level 4 instead of brotli's default (11): dynamic JSON gets compressed on every request, and
        // on a shared CPU the default costs far more than the few percent of size it saves.
        .layer(CompressionLayer::new().quality(CompressionLevel::Precise(4)))
        // Safety net: a panic in one request answers 500 instead of taking the server down.
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
}

/// Unknown paths: app routes ("/money", "/joy/slips") get index.html so the client router takes over;
/// a missing file ("/chunk-old.js" after a deploy) gets a plain 404. Answering a script request with
/// HTML makes the browser fail with a MIME error instead of letting the app reload itself.
/// index.html is read once at startup: it's tiny, and this keeps the fallback allocation-free.
fn spa_fallback(index: &str) -> impl Fn(axum::http::Uri) -> std::future::Ready<axum::response::Response> + Clone + Send + Sync + 'static {
    let html: std::sync::Arc<str> = std::fs::read_to_string(index).unwrap_or_default().into();
    move |uri: axum::http::Uri| {
        use axum::response::IntoResponse;
        let last = uri.path().rsplit('/').next().unwrap_or("");
        let is_file = last.contains('.') && !last.ends_with(".html");
        std::future::ready(if is_file {
            axum::http::StatusCode::NOT_FOUND.into_response()
        } else {
            axum::response::Html(html.to_string()).into_response()
        })
    }
}

pub async fn token_validator_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    match get_token(&headers) {
        Some(token) => {
            let (user_id, role) = authenticate(token, &state).await?;
            request.extensions_mut().insert((user_id, role));
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(AppError(StatusCode::UNAUTHORIZED, "error".to_string())),
    }
}

fn sign_in_again() -> AppError {
    AppError(StatusCode::UNAUTHORIZED, "Please sign in again.".to_string())
}

/// A valid token whose user still exists and whose version is still current. Password changes,
/// "sign out everywhere" and account deletion all end older tokens this way. The role comes from
/// the database, so a demotion applies at once.
async fn authenticate(token: &str, state: &AppState) -> Result<(String, String), AppError> {
    let (user_id, _token_role, version) = parse_token(token, &state.paseto_key)?;
    let row: Option<(String, i64)> = sqlx::query_as("SELECT user_role, token_version FROM users WHERE user_id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await?;
    match row {
        Some((role, current)) if version == Some(current) => Ok((user_id, role)),
        _ => Err(sign_in_again()),
    }
}

pub async fn token_validator_auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    match get_token(&headers) {
        Some(token) => {
            let (user_id, role) = authenticate(token, &state).await?;
            if role != "admin" {
                return Err(AppError(
                    StatusCode::FORBIDDEN,
                    "User is not an admin".to_string(),
                ));
            }
            request.extensions_mut().insert((user_id, role));
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(AppError(StatusCode::UNAUTHORIZED, "error".to_string())),
    }
}

fn get_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get("Authorization")?.to_str().ok()?;
    header_value
        .strip_prefix("Bearer ")
        .map(|t| t.trim())
        .or(Some(header_value))
}

fn parse_token(
    token: &str,
    key: &Arc<PasetoSymmetricKey<V4, Local>>,
) -> Result<(String, String, Option<i64>), AppError> {
    match PasetoParser::<V4, Local>::default()
        .check_claim(rusty_paseto::prelude::ExpirationClaim::default())
        .parse(token, key)
    {
        Ok(json_value) => {
            let user_id = json_value["sub"]
                .as_str()
                .ok_or_else(|| {
                    AppError(
                        StatusCode::UNAUTHORIZED,
                        "Missing user_id in token".to_string(),
                    )
                })?
                .to_string();
            let role = json_value["role"]
                .as_str()
                .ok_or_else(|| {
                    AppError(
                        StatusCode::UNAUTHORIZED,
                        "Missing role in token".to_string(),
                    )
                })?
                .to_string();
            // Tokens issued before versions existed have none and no longer pass.
            let version = json_value["tv"].as_i64();
            Ok((user_id, role, version))
        }
        Err(err) => {
            error!("Error parsing token {}", err);
            // The library's reason stays in the log; the client only learns to sign in again.
            Err(AppError(StatusCode::UNAUTHORIZED, "Please sign in again.".to_string()))
        }
    }
}

/// `/e/{token}` (Joy's envelope links) → `/joy/e/{token}`. Tokens are URL-safe; anything else 404s.
async fn old_joy_envelope(axum::extract::Path(token): axum::extract::Path<String>) -> axum::response::Response {
    use axum::response::IntoResponse;
    if token.is_empty() || !token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    }
    axum::response::Redirect::permanent(&format!("/joy/e/{token}")).into_response()
}

/// `/year/2026` and `/year/2026/10` (Joy) → the same under `/joy`.
async fn old_joy_year(axum::extract::Path(rest): axum::extract::Path<String>) -> axum::response::Response {
    use axum::response::IntoResponse;
    if !rest.chars().all(|c| c.is_ascii_digit() || c == '/') {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    }
    axum::response::Redirect::permanent(&format!("/joy/year/{rest}")).into_response()
}
