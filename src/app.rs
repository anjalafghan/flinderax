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
use time::OffsetDateTime;
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
        // Inside the auth layer, so it knows whose key it is.
        .layer(middleware::from_fn_with_state(state.clone(), crate::idempotency::layer))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            token_validator_middleware,
        ))
}

/// Liveness for Fly: answers 200 only when the database can be queried, so a machine whose
/// migrations failed or whose volume is unreadable is never sent traffic.
async fn healthz(State(state): State<AppState>) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("healthz: database check failed: {e}");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

async fn api_not_found() -> AppError {
    AppError(StatusCode::NOT_FOUND, "Not found".to_string())
}

pub fn build_router(state: AppState) -> Router {
    let dir = static_dir();
    let index = format!("{dir}/index.html");

    Router::new()
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
        .nest("/api/year", protected(&state, routes::year::routes(state.clone())))
        .nest(
            "/api/decisions",
            protected(&state, Router::new().route("/", axum::routing::get(crate::handlers::decisions::get)).with_state(state.clone())),
        )
        .nest("/api/checkin", protected(&state, routes::checkin::routes(state.clone())))
        .nest("/api/diet", protected(&state, routes::diet::routes(state.clone())))
        .nest("/api/people", protected(&state, routes::people::routes(state.clone())))
        .nest("/api/work", protected(&state, routes::work::routes(state.clone())))
        .nest("/api/reminders", protected(&state, routes::reminders::routes(state.clone())))
        .nest("/api/public/reminders", routes::reminders::public_routes(state.clone()).layer(trace_layer()))
        .nest("/api/goals", protected(&state, routes::goals::routes(state.clone())))
        .nest("/api/activity", protected(&state, routes::activity::routes(state.clone())))
        .nest("/api/rewards", protected(&state, routes::rewards::routes(state.clone())))
        .nest(
            "/api/export",
            protected(&state, Router::new().route("/", axum::routing::get(crate::handlers::export::export)).with_state(state.clone())),
        )
        .nest(
            "/api/import",
            protected(
                &state,
                Router::new()
                    .route("/", axum::routing::post(crate::handlers::import::import).layer(axum::extract::DefaultBodyLimit::max(20 * 1024 * 1024)))
                    .with_state(state.clone()),
            ),
        )
        .nest(
            "/api/couple-goals",
            protected(
                &state,
                Router::new()
                    .route("/", axum::routing::get(crate::handlers::couple_goals::list).post(crate::handlers::couple_goals::create))
                    .route("/{id}/contribute", axum::routing::post(crate::handlers::couple_goals::contribute))
                    .route("/{id}/contributions/{cid}/delete", axum::routing::post(crate::handlers::couple_goals::delete_contribution))
                    .route("/{id}/archive", axum::routing::post(crate::handlers::couple_goals::archive))
                    .route("/{id}/budget", axum::routing::get(crate::handlers::wedding::budget))
                    .route("/{id}/items", axum::routing::post(crate::handlers::wedding::create_item))
                    .route("/{id}/items/{item_id}", axum::routing::post(crate::handlers::wedding::update_item))
                    .route("/{id}/items/{item_id}/delete", axum::routing::post(crate::handlers::wedding::delete_item))
                    .route("/{id}/items/{item_id}/milestones", axum::routing::post(crate::handlers::wedding::create_milestone))
                    .route("/{id}/milestones/{mid}", axum::routing::post(crate::handlers::wedding::update_milestone))
                    .route("/{id}/milestones/{mid}/delete", axum::routing::post(crate::handlers::wedding::delete_milestone))
                    .route("/{id}/milestones/{mid}/pay", axum::routing::post(crate::handlers::wedding::pay))
                    .route("/{id}/milestones/{mid}/unpay", axum::routing::post(crate::handlers::wedding::unpay))
                    .with_state(state.clone()),
            ),
        )
        .nest(
            "/api/push",
            protected(
                &state,
                Router::new()
                    .route("/", axum::routing::get(crate::handlers::push::status))
                    .route("/subscribe", axum::routing::post(crate::handlers::push::subscribe))
                    .route("/unsubscribe", axum::routing::post(crate::handlers::push::unsubscribe))
                    .with_state(state.clone()),
            ),
        )
        // Not a session route: a scheduled job proves itself with a secret instead.
        .route("/api/internal/push/run", axum::routing::post(crate::handlers::push::run_round).with_state(state.clone()))
        .route("/api/internal/backup/run", axum::routing::post(crate::handlers::backup::run).with_state(state.clone()))
        .nest("/api/coming-up", protected(&state, Router::new().route("/", axum::routing::get(crate::handlers::coming::get)).with_state(state.clone())))
        .nest("/api/review", protected(&state, routes::review::routes(state.clone())))
        .nest("/api/hook", routes::ingest::hook_routes(state.clone()).layer(trace_layer()))
        // Unknown API paths must 404 instead of falling through to the SPA's index.html.
        .route("/api/{*rest}", any(api_not_found))
        .route("/healthz", axum::routing::get(healthz).with_state(state.clone()))
        // Joy's envelope links were already sent on WhatsApp, so the old /e/{token} address still redirects.
        .route("/e/{token}", axum::routing::get(old_joy_envelope))
        // Static files come from the fallback. It must be set before the layers below: `Router::layer`
        // only wraps what exists when it is called, so headers and compression would skip it.
        // `<file>.br` / `<file>.gz` are served when present, so those aren't compressed on the fly.
        .fallback_service(
            ServeDir::new(&dir)
                .precompressed_br()
                .precompressed_gzip()
                .fallback(axum::handler::HandlerWithoutStateExt::into_service(spa_fallback(&index))),
        )
        .layer(middleware::from_fn(crate::headers::security_headers))
        // Level 4 instead of brotli's default (11): dynamic JSON gets compressed on every request, and
        // on a shared CPU the default costs far more than the few percent of size it saves.
        .layer(CompressionLayer::new().quality(CompressionLevel::Precise(4)))
        // A stuck handler answers 504 instead of holding a connection (and a pool slot) forever.
        // 60 s leaves room for photo and statement uploads on a slow phone connection.
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            std::time::Duration::from_secs(60),
        ))
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
        Some((token, from_cookie)) => {
            // A cookie goes along with any request a page on another site makes to ours, so for
            // anything that changes data it is only trusted when the request also carries a header
            // that only our own page can add (a cross-site request cannot, without our consent).
            if from_cookie && !matches!(*request.method(), axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS) && !headers.contains_key("x-joy") {
                return Err(AppError(StatusCode::FORBIDDEN, "Missing request header.".to_string()));
            }
            let (user_id, role, version) = authenticate(token, &state).await?;
            // Renew a session that is past its halfway point; the client swaps the token in.
            let renewal = renewal(token, &user_id, &role, version, &state);
            request.extensions_mut().insert((user_id, role));
            let mut response = next.run(request).await;
            if let Some((new_token, expires_at)) = renewal {
                if from_cookie {
                    // The web app lives on the cookie: renew that, and keep the token out of reach of scripts.
                    if let Ok(c) = crate::handlers::common::session_cookie(&new_token, crate::handlers::common::is_https(&headers)).parse() {
                        response.headers_mut().append(axum::http::header::SET_COOKIE, c);
                    }
                } else if let (Ok(t), Ok(e)) = (new_token.parse(), expires_at.to_string().parse()) {
                    response.headers_mut().insert("x-refreshed-token", t);
                    response.headers_mut().insert("x-token-expires-at", e);
                }
            }
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
async fn authenticate(token: &str, state: &AppState) -> Result<(String, String, i64), AppError> {
    let (user_id, _token_role, version) = parse_token(token, &state.paseto_key)?;
    let row: Option<(String, i64)> = sqlx::query_as("SELECT user_role, token_version FROM users WHERE user_id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await?;
    match row {
        Some((role, current)) if version == Some(current) => Ok((user_id, role, current)),
        _ => Err(sign_in_again()),
    }
}

/// A new token and its expiry (unix seconds) when `token` has less than half its life left.
fn renewal(token: &str, user_id: &str, role: &str, version: i64, state: &AppState) -> Option<(String, i64)> {
    let left = token_expiry(token, &state.paseto_key)? - OffsetDateTime::now_utc();
    if left >= crate::handlers::common::SESSION_LIFETIME / 2 {
        return None;
    }
    let expiry = OffsetDateTime::now_utc() + crate::handlers::common::SESSION_LIFETIME;
    let new = crate::handlers::common::get_paseto_token(user_id, role.to_string(), version, &state.paseto_key, expiry).ok()?;
    Some((new, expiry.unix_timestamp()))
}

fn token_expiry(token: &str, key: &Arc<PasetoSymmetricKey<V4, Local>>) -> Option<OffsetDateTime> {
    let claims = PasetoParser::<V4, Local>::default().parse(token, key).ok()?;
    OffsetDateTime::parse(claims["exp"].as_str()?, &time::format_description::well_known::Rfc3339).ok()
}

/// The session token and whether it came from the cookie (as opposed to an Authorization header).
fn get_token(headers: &HeaderMap) -> Option<(&str, bool)> {
    if let Some(v) = headers.get("Authorization") {
        let header_value = v.to_str().ok()?;
        return Some((header_value.strip_prefix("Bearer ").map(|t| t.trim()).unwrap_or(header_value), false));
    }
    let cookies = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    cookies
        .split(';')
        .filter_map(|c| c.trim().split_once('='))
        .find(|(name, _)| *name == crate::handlers::common::SESSION_COOKIE)
        .map(|(_, v)| (v, true))
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
