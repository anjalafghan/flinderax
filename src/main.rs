use sqlx::SqlitePool;
use std::env;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod app;
mod handlers;
mod middleware;
mod models;
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
    info!("Database url is present {}", database_url);
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!().run(&pool).await?;
    info!("Database connected successfully");

    info!("Running migrations");
    info!("Initializing Redis...");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    
    let redis_manager = match redis::Client::open(redis_url) {
        Ok(client) => {
            match tokio::time::timeout(
                std::time::Duration::from_secs(2),
                client.get_connection_manager()
            ).await {
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

    let state = models::AppState {
        db: pool.clone(),
        redis: redis_manager,
    };

    let app = app::build_router(state);
    info!("Running Server!");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
