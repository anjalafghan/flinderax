use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous};
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use joy::{app, models};

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
        let root = joy::joy::photos::photos_root();
        if let Err(e) = joy::joy::db::backfill_visit_history(&pool, &root).await {
            tracing::warn!("could not link regular nights to kept dates: {e}");
        }
    }
    info!("Database connected successfully");

    // `joy enrich-slips [--dry-run]`: fill blank slip fields from data already in the database.
    let args: Vec<String> = env::args().collect();
    if args.get(1).map(String::as_str) == Some("enrich-slips") {
        let dry = args.iter().any(|a| a == "--dry-run");
        let fills = joy::joy::enrich::run(&pool, dry).await?;
        for (id, title, fill) in &fills {
            println!("#{id} {title}: {fill:?}");
        }
        println!("{} {}", fills.len(), if dry { "fills found (dry run, nothing written)." } else { "fills written." });
        return Ok(());
    }

    info!("Running migrations");
    let key_str = env::var("PASETO_KEY").expect("PASETO_KEY missing");
    let key_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, key_str)
        .expect("Failed to decode PASETO_KEY");
    let paseto_key = Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<
        rusty_paseto::core::V4,
        rusty_paseto::core::Local,
    >::from(rusty_paseto::prelude::Key::from(key_bytes.as_slice())));

    let state = models::AppState {
        db: pool.clone(),
        paseto_key,
    };


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
