use axum::{Router, routing::get};
use sqlx::SqlitePool;
use std::env;

use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod models;

mod handlers;
mod routes;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "flinderax=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Flinderax application");
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!().run(&pool).await?;
    info!("Database connected successfully");

    info!("Running migrations");
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest(
            "/user",
            routes::user::routes(pool.clone()).layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            ),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
