use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous};
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use joy::{app, models};

/// How often `PRAGMA optimize` refreshes the planner's statistics while the server runs.
const OPTIMIZE_EVERY: Duration = Duration::from_secs(6 * 60 * 60);

/// One small SQLite file on a volume, one process: WAL so reads never wait on the writer, and a
/// few pooled connections (SQLite serialises writers anyway). `foreign_keys` is ON, which the
/// schema relies on for its `ON DELETE CASCADE`s.
fn sqlite_options(database_url: &str) -> Result<SqliteConnectOptions, sqlx::Error> {
    Ok(SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5))
        .pragma("cache_size", "-2000") // 2 MB per connection: the DB is small; keep RSS low
        .pragma("temp_store", "MEMORY")
        .pragma("mmap_size", "33554432")) // 32 MB window, file-backed (reclaimable)
}

async fn optimize(pool: &SqlitePool) {
    if let Err(e) = sqlx::query("PRAGMA optimize").execute(pool).await {
        error!("PRAGMA optimize failed: {e}");
    }
}

/// Resolves on Ctrl-C or SIGTERM (what Fly sends on deploy/stop), so in-flight requests finish and
/// the pool closes cleanly, which checkpoints the WAL into the main file.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // `joy gen-backup-key` and `joy restore-backup`: no server, no database, and no logging, so
    // stdout carries only their output (the key can be captured with `$(joy gen-backup-key)`).
    let cli_args: Vec<String> = env::args().collect();
    if matches!(cli_args.get(1).map(String::as_str), Some("gen-backup-key" | "restore-backup")) {
        dotenvy::dotenv().ok();
        std::process::exit(joy::backup::cli(&cli_args).await);
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // Info, not debug: per-request debug spans cost CPU and log volume on the small VM.
                "joy=info,tower_http=info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Flinderax application");
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");

    let pool = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(3)
        .connect_with(sqlite_options(&database_url)?)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    optimize(&pool).await;
    {
        let root = joy::import_joy::photos_root();
        if let Err(e) = joy::joy::db::backfill_visit_history(&pool, &root).await {
            tracing::warn!("could not link regular nights to kept dates: {e}");
        }
    }
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

    let maintenance = pool.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval_at(tokio::time::Instant::now() + OPTIMIZE_EVERY, OPTIMIZE_EVERY);
        loop {
            tick.tick().await;
            optimize(&maintenance).await;
        }
    });

    let app = app::build_router(state);
    info!("Running Server!");

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    info!("Serving API and web app (from {}) on port {}", app::static_dir(), port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    pool.close().await;
    Ok(())
}
