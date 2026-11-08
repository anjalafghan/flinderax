use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use joy::{app, models};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "joy=debug,tower_http=debug,axum::rejection=trace".into()
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

    // `joy import-joy <joy.db> <user name> [--as a|b] [--photos <dir>] [--dry-run]`
    let args: Vec<String> = env::args().collect();
    if args.get(1).map(String::as_str) == Some("import-joy") {
        let (Some(joy_db), Some(user)) = (args.get(2), args.get(3)) else {
            eprintln!("usage: joy import-joy <joy.db> <user name> [--as a|b] [--photos <dir>] [--dry-run]");
            std::process::exit(2);
        };
        let photos = args.iter().position(|a| a == "--photos").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);
        let dry = args.iter().any(|a| a == "--dry-run");
        let slot = match args.iter().position(|a| a == "--as").and_then(|i| args.get(i + 1)).map(String::as_str) {
            None | Some("a") | Some("A") => 0,
            Some("b") | Some("B") => 1,
            Some(other) => {
                eprintln!("--as must be a or b, not {other}");
                std::process::exit(2);
            }
        };
        let env: Vec<(String, String)> = env::vars().collect();
        let root = joy::import_joy::photos_root();
        let report = joy::import_joy::run(&pool, joy_db.as_ref(), user, slot, &env, photos.as_deref(), &root, dry).await?;
        for (what, n) in &report.counts {
            println!("{what:>16}: {n}");
        }
        println!("{:>16}: {}", "photos", report.photos);
        println!("{}", if dry { "Dry run: nothing was written." } else { "Imported." });
        return Ok(());
    }

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

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    info!("Serving API and web app (from {}) on port {}", app::static_dir(), port);

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
