use crate::handlers::common::{get_key, AppError};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use rusty_paseto::{
    core::{Local, V4},
    prelude::PasetoParser,
};
use sqlx::SqlitePool;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, Level};

use crate::routes;

pub fn build_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest(
            "/user",
            routes::user::routes(pool.clone())
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(middleware::from_fn(token_validator_middleware)),
        )
        .nest(
            "/common",
            routes::common::routes(pool.clone()).layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            ),
        )
}

async fn token_validator_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    match get_token(&headers) {
        Some(token) => {
            let (user_id, role) = parse_token(token)?;
            request.extensions_mut().insert(user_id);
            request.extensions_mut().insert(role);
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(AppError(StatusCode::UNAUTHORIZED, "error".to_string())),
    }
}

pub async fn token_validator_auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    match get_token(&headers) {
        Some(token) => {
            let (user_id, role) = parse_token(token)?;
            if role != "admin" {
                return Err(AppError(
                    StatusCode::FORBIDDEN,
                    "User is not an admin".to_string(),
                ));
            }
            request.extensions_mut().insert(user_id);
            request.extensions_mut().insert(role);
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(AppError(StatusCode::UNAUTHORIZED, "error".to_string())),
    }
}

fn get_token(headers: &HeaderMap) -> Option<&str> {
    let token = headers.get("Authorization").unwrap().to_str().unwrap();
    Some(token)
}

fn parse_token(token: &str) -> Result<(String, String), AppError> {
    let key = get_key()?;
    match PasetoParser::<V4, Local>::default().parse(token, &key) {
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
