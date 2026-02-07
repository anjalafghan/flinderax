use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod app;
mod handlers;
mod middleware;
mod models;
mod routes;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/flinderax_backend.rs"));
}

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

    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
    sqlx::migrate!().run(&pool).await?;
    info!("Database connected successfully");

    info!("Running migrations");
    info!("Initializing Redis...");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

    let redis_manager = match redis::Client::open(redis_url) {
        Ok(client) => {
            match tokio::time::timeout(
                std::time::Duration::from_secs(2),
                client.get_connection_manager(),
            )
            .await
            {
                Ok(Ok(manager)) => {
                    info!("Redis connected successfully");
                    Some(manager)
                }
                _ => {
                    error!("Redis connection timed out or failed. Running without cache.");
                    None
                }
            }
        }
        Err(e) => {
            error!("Failed to open Redis client: {}. Running without cache.", e);
            None
        }
    };

    let key_str = env::var("PASETO_KEY").expect("PASETO_KEY missing");
    let key_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, key_str)
        .expect("Failed to decode PASETO_KEY");
    let paseto_key = Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<
        rusty_paseto::core::V4,
        rusty_paseto::core::Local,
    >::from(rusty_paseto::prelude::Key::from(key_bytes.as_slice())));

    let state = models::AppState {
        db: pool.clone(),
        redis: redis_manager,
        paseto_key,
    };

    let app = app::build_router(state);
    info!("Running Server!");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
