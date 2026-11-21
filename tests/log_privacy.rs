//! Logs record route TEMPLATES, never real paths or queries: calendar-feed tokens, envelope tokens and
//! photo signatures travel in them, and Fly keeps the logs. This drives real requests through the real
//! router and reads what was actually logged.

use std::sync::{Arc, Mutex};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use joy::{app::build_router, models::AppState};
use rusty_paseto::prelude::{Key, PasetoSymmetricKey};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for Captured {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Captured {
    type Writer = Captured;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl Captured {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

async fn router() -> axum::Router {
    let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    build_router(AppState { db, paseto_key: Arc::new(key) })
}

async fn status_of(app: &axum::Router, uri: &str) -> StatusCode {
    let req = Request::builder().uri(uri).header("fly-client-ip", "10.9.9.9").body(Body::empty()).unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

#[tokio::test]
async fn a_public_feed_request_is_logged_by_template_and_never_by_its_token() {
    let logs = Captured::default();
    let subscriber = tracing_subscriber::fmt().with_writer(logs.clone()).with_ansi(false).with_max_level(tracing::Level::TRACE).finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let app = router().await;
    status_of(&app, "/api/public/reminders/SECRETFEEDTOKEN.ics?sig=SECRETSIGNATURE&exp=99999").await;

    let out = logs.text();
    assert!(out.contains(r#"route="/api/public/reminders/{file}""#), "the template should be logged, got: {out}");
    for secret in ["SECRETFEEDTOKEN", "SECRETSIGNATURE", "99999"] {
        assert!(!out.contains(secret), "{secret} leaked into the logs: {out}");
    }
}

#[tokio::test]
async fn a_path_that_matches_no_route_leaves_no_trace_of_its_path_in_the_logs() {
    let logs = Captured::default();
    let subscriber = tracing_subscriber::fmt().with_writer(logs.clone()).with_ansi(false).with_max_level(tracing::Level::TRACE).finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let app = router().await;
    // Under the public mount but matching nothing: a mistyped or probing request. (Capture itself is
    // proven by the test above.)
    status_of(&app, "/api/public/reminders/a/SECRETPATHPART?token=SECRETQUERYTOKEN").await;

    let out = logs.text();
    for secret in ["SECRETPATHPART", "SECRETQUERYTOKEN"] {
        assert!(!out.contains(secret), "{secret} leaked into the logs: {out}");
    }
}
