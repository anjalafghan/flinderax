use crate::handlers::common::AppError;
use std::sync::Arc;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use rusty_paseto::{
    core::{Local, V4},
    prelude::{PasetoParser, PasetoSymmetricKey},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, Level};

use crate::models::AppState;
use crate::routes;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest(
            "/user",
            routes::user::routes(state.clone())
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    token_validator_middleware,
                )),
        )
        .nest(
            "/common",
            routes::common::routes(state.clone()).layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            ),
        )
        .nest(
            "/card",
            routes::card::routes(state.clone())
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    token_validator_middleware,
                )),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
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
    match PasetoParser::<V4, Local>::default().parse(token, key) {
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
