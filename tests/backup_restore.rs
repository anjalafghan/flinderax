//! The restore drill as a test: snapshot a populated database, open the copy in a fresh pool as a
//! restored machine would, and check it is whole.

use axum::{body::Body, http::{Request, StatusCode}};
use joy::{app::build_router, models::AppState};
use rusty_paseto::prelude::{Key, PasetoSymmetricKey};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::sync::Arc;
use tower::ServiceExt;

async fn open(file: &std::path::Path) -> sqlx::SqlitePool {
    let opts = SqliteConnectOptions::new().filename(file).create_if_missing(true);
    SqlitePoolOptions::new().max_connections(1).connect_with(opts).await.unwrap()
}

#[tokio::test]
async fn snapshot_restores_whole() {
    let dir = std::env::temp_dir().join(format!("joy-restore-drill-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let live = open(&dir.join("live.db")).await;
    sqlx::migrate!().run(&live).await.unwrap();
    for i in 0..3 {
        sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES (?, ?, 'x', 'user')")
            .bind(format!("u{i}")).bind(format!("name{i}")).execute(&live).await.unwrap();
    }
    for i in 0..5 {
        sqlx::query("INSERT INTO unrecognized_alerts (alert_id, user_id, text) VALUES (?, 'u0', ?)")
            .bind(format!("a{i}")).bind(format!("text {i}")).execute(&live).await.unwrap();
    }

    let snap = dir.join("backups/snap.db");
    joy::backup::snapshot_to(&live, &snap).await.unwrap();
    // Writes after the snapshot must not appear in it.
    sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('late', 'late', 'x', 'user')").execute(&live).await.unwrap();
    live.close().await;

    // "Restore": the copy becomes the database of a fresh pool.
    let restored_file = dir.join("restored.db");
    std::fs::copy(&snap, &restored_file).unwrap();
    let db = open(&restored_file).await;

    let ok: String = sqlx::query_scalar("PRAGMA integrity_check").fetch_one(&db).await.unwrap();
    assert_eq!(ok, "ok");
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT 1").fetch_one(&db).await.unwrap(), 1);
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&db).await.unwrap();
    let alerts: i64 = sqlx::query_scalar("SELECT count(*) FROM unrecognized_alerts").fetch_one(&db).await.unwrap();
    assert_eq!((users, alerts), (3, 5));

    // Migrations check: everything is already applied, running again is a no-op, and nothing is dirty.
    let applied_before: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success = 1").fetch_one(&db).await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let applied_after: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success = 1").fetch_one(&db).await.unwrap();
    assert_eq!(applied_before, applied_after);
    assert_eq!(applied_before as usize, sqlx::migrate!().migrations.len());

    // And the app answers /healthz on the restored database.
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    let app = build_router(AppState { db: db.clone(), paseto_key: Arc::new(key) });
    let res = app.oneshot(Request::get("/healthz").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    db.close().await;
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn pull_endpoint_needs_the_token_and_returns_a_database() {
    // Safety: only this test in this binary touches these variables.
    unsafe { std::env::set_var("BACKUP_RUN_TOKEN", "backup-test-token") };
    let dir = std::env::temp_dir().join(format!("joy-pull-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    unsafe { std::env::set_var("BACKUP_DIR", dir.join("backups")) };
    let db = open(&dir.join("live.db")).await;
    sqlx::migrate!().run(&db).await.unwrap();
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    let app = build_router(AppState { db: db.clone(), paseto_key: Arc::new(key) });

    let no = app.clone().oneshot(Request::post("/api/internal/backup/run").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(no.status(), StatusCode::UNAUTHORIZED);
    let wrong = app.clone().oneshot(Request::post("/api/internal/backup/run").header("authorization", "Bearer nope-nope-nope").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);

    let ok = app.oneshot(Request::post("/api/internal/backup/run").header("authorization", "Bearer backup-test-token").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(ok.into_body(), usize::MAX).await.unwrap();
    assert!(bytes.starts_with(b"SQLite format 3\0"));
    assert!(!dir.join("backups").read_dir().unwrap().any(|e| e.unwrap().file_name().to_string_lossy().contains("pull-")), "temp file removed");
    db.close().await;
    std::fs::remove_dir_all(&dir).unwrap();
}
