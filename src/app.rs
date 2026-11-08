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
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, error};

use crate::models::AppState;
use crate::routes;

/// Directory holding the built web app (`bun run build` output). One server serves both
/// the API and the SPA, so there is nothing else to run.
pub fn static_dir() -> String {
    std::env::var("STATIC_DIR").unwrap_or_else(|_| "web/dist".to_string())
}

fn trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    DefaultMakeSpan,
    tower_http::trace::DefaultOnRequest,
    DefaultOnResponse,
> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
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
        .nest("/api/hook", routes::ingest::hook_routes(state.clone()).layer(trace_layer()))
        // Unknown API paths must 404 instead of falling through to the SPA's index.html.
        .route("/api/{*rest}", any(api_not_found))
        .layer(cors_layer())
        .layer(CompressionLayer::new())
        .fallback_service(ServeDir::new(&dir).fallback(ServeFile::new(index)))
}

async fn token_validator_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    match get_token(&headers) {
        Some(token) => {
            let (user_id, role) = parse_token(token, &state.paseto_key)?;
            request.extensions_mut().insert((user_id, role));
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(AppError(StatusCode::UNAUTHORIZED, "error".to_string())),
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
            let (user_id, role) = parse_token(token, &state.paseto_key)?;
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
) -> Result<(String, String), AppError> {
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
            Ok((user_id, role))
        }
        Err(err) => {
            error!("Error parsing token {}", err);
            Err(AppError(StatusCode::UNAUTHORIZED, err.to_string()))
        }
    }
}
