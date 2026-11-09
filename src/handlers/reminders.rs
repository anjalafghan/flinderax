//! The reminder calendar feed. `link` and `reset_link` are behind login and manage the user's private
//! URL; `feed` is public and is keyed only by the unguessable token in the file name.

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use rand::RngCore;
use serde::Serialize;
use time::OffsetDateTime;

use crate::{
    handlers::{
        common::{today_local, AppError},
        habits, people,
    },
    models::AppState,
    planner::reminders::{from_dates, from_habits, hide_names, render, token_from_file, Reminder},
};

const DEFAULT_PUBLIC_URL: &str = "https://joy.fly.dev";

#[derive(Serialize)]
pub struct LinkOut {
    pub url: String,
}

/// `GET /api/reminders/link`: the user's feed URL, created on first use.
pub async fn link(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<LinkOut>, AppError> {
    sqlx::query(
        "INSERT INTO reminder_feeds (user_id, token) VALUES (?, ?) ON CONFLICT(user_id) DO NOTHING",
    )
    .bind(&user_id)
    .bind(new_token())
    .execute(&state.db)
    .await?;
    let token: String = sqlx::query_scalar("SELECT token FROM reminder_feeds WHERE user_id = ?")
        .bind(&user_id)
        .fetch_one(&state.db)
        .await?;
    Ok(Json(LinkOut { url: feed_url(&token) }))
}

/// `POST /api/reminders/link/reset`: a new token. The old URL stops working at once.
pub async fn reset_link(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<LinkOut>, AppError> {
    let token = new_token();
    sqlx::query(
        "INSERT INTO reminder_feeds (user_id, token) VALUES (?, ?) \
         ON CONFLICT(user_id) DO UPDATE SET token = excluded.token",
    )
    .bind(&user_id)
    .bind(&token)
    .execute(&state.db)
    .await?;
    Ok(Json(LinkOut { url: feed_url(&token) }))
}

/// `GET /api/public/reminders/{file}`: the calendar for the user who owns the token in `file`.
pub async fn feed(State(state): State<AppState>, Path(file): Path<String>) -> Response {
    match feed_body(&state, &file).await {
        Ok(Some(body)) => (
            [
                (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
                (header::CACHE_CONTROL, "private, no-store"),
            ],
            body,
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => e.into_response(),
    }
}

/// The calendar text for a feed file, or None when the name or token is unknown.
async fn feed_body(state: &AppState, file: &str) -> Result<Option<String>, AppError> {
    let Some(token) = token_from_file(file) else {
        return Ok(None);
    };
    let user_id: Option<String> = sqlx::query_scalar("SELECT user_id FROM reminder_feeds WHERE token = ?")
        .bind(token)
        .fetch_optional(&state.db)
        .await?;
    let Some(user_id) = user_id else {
        return Ok(None);
    };
    let today = today_local();
    let mut habits = habits::compute(state, &user_id, today).await?;
    let names = people::feed_names(&state.db, &user_id).await?;
    if !names {
        hide_names(&mut habits);
    }
    let url = public_url();
    let mut reminders = from_habits(&habits, today, &url);
    let (dates, seasons) = people::feed_input(&state.db, &user_id).await?;
    reminders.extend(from_dates(&dates, &seasons, today, names, &url));
    let status = crate::handlers::checkin::compute_status(state, &user_id, today).await?;
    reminders.push(checkin_reminder(&status.next_due_on, today, &url));
    let now = OffsetDateTime::now_utc().unix_timestamp();
    Ok(Some(render(&reminders, now)))
}

/// The quarterly check-in, on its due day (today when it is overdue).
fn checkin_reminder(next_due_on: &str, today: time::Date, public_url: &str) -> Reminder {
    let due = time::Date::parse(next_due_on, time::macros::format_description!("[year]-[month]-[day]")).unwrap_or(today);
    Reminder {
        uid: "joy-checkin@joy".into(),
        on: due.max(today),
        summary: "Quarterly check-in with joy".into(),
        description: format!(
            "About 20 minutes: upload your statements, confirm balances, sort new payees and read your quarter.\n{public_url}/checkin"
        ),
    }
}

fn feed_url(token: &str) -> String {
    format!("{}/api/public/reminders/{token}.ics", public_url())
}

/// `PUBLIC_URL` without a trailing slash.
fn public_url() -> String {
    std::env::var("PUBLIC_URL")
        .ok()
        .map(|v| v.trim().trim_end_matches('/').to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_PUBLIC_URL.to_string())
}

/// 32 random bytes as 64 lowercase hex characters.
fn new_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('v', 'v', 'x', 'user')",
            "INSERT INTO accounts (account_id, user_id, name, kind) VALUES ('acctB', 'u', 'Asha bank', 'bank')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32][..]),
        );
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    fn user(id: &str) -> Extension<(String, String)> {
        Extension((id.to_string(), String::new()))
    }

    /// The `{token}.ics` file name from a feed URL.
    fn file_of(url: &str) -> String {
        url.rsplit('/').next().unwrap().to_string()
    }

    async fn get_feed(state: &AppState, file: &str) -> (StatusCode, Option<String>, String) {
        let res = feed(State(state.clone()), Path(file.to_string())).await;
        let status = res.status();
        let ctype = res.headers().get(header::CONTENT_TYPE).map(|v| v.to_str().unwrap().to_string());
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        (status, ctype, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn link_is_created_once_and_is_stable() {
        let state = test_state().await;
        let first = link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url;
        let second = link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url;
        assert_eq!(first, second);
        assert!(first.ends_with(".ics") && first.contains("/api/public/reminders/"));
        let token = file_of(&first);
        assert!(crate::planner::reminders::token_from_file(&token).is_some());
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reminder_feeds").fetch_one(&state.db).await.unwrap();
        assert_eq!(rows, 1);
    }

    #[tokio::test]
    async fn each_user_gets_their_own_token() {
        let state = test_state().await;
        let a = link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url;
        let b = link(State(state.clone()), user("v")).await.map_err(|e| e.1).unwrap().0.url;
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn reset_changes_the_token_and_the_old_one_stops_working() {
        let state = test_state().await;
        let old = link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url;
        let old_file = file_of(&old);
        assert_eq!(get_feed(&state, &old_file).await.0, StatusCode::OK);

        let new = reset_link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url;
        assert_ne!(old, new);
        assert_eq!(get_feed(&state, &old_file).await.0, StatusCode::NOT_FOUND);
        assert_eq!(get_feed(&state, &file_of(&new)).await.0, StatusCode::OK);
        assert_eq!(link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url, new);
    }

    #[tokio::test]
    async fn reset_without_a_link_creates_one() {
        let state = test_state().await;
        let url = reset_link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url;
        assert_eq!(get_feed(&state, &file_of(&url)).await.0, StatusCode::OK);
    }

    #[tokio::test]
    async fn feed_serves_a_calendar_with_private_headers() {
        let state = test_state().await;
        let url = link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url;
        // A statement that ended long ago makes the next upload due in the past, so it shows up today.
        sqlx::query(
            "INSERT INTO statement_imports (import_id, user_id, account_id, first_date, last_date, rows) \
             VALUES ('imp1', 'u', 'acctB', '2025-12-01', '2026-01-01', 3)",
        )
        .execute(&state.db)
        .await
        .unwrap();
        let res = feed(State(state.clone()), Path(file_of(&url))).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[header::CONTENT_TYPE], "text/calendar; charset=utf-8");
        assert_eq!(res.headers()[header::CACHE_CONTROL], "private, no-store");
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.starts_with("BEGIN:VCALENDAR\r\n"));
        assert!(text.ends_with("END:VCALENDAR\r\n"));
        assert!(text.contains("UID:joy-habit-bank_statement-acctB@joy\r\n"), "{text}");
        assert!(text.contains("SUMMARY:Upload your Asha bank statement\r\n"), "{text}");
        assert!(text.contains(&format!("{}/spending\r\n", public_url())), "action link in the description: {text}");
        assert!(text.contains("UID:joy-checkin@joy\r\n"), "the check-in is in the feed: {text}");
        assert!(text.contains("SUMMARY:Quarterly check-in with joy\r\n"), "{text}");
    }

    #[tokio::test]
    async fn bad_or_unknown_files_are_404() {
        let state = test_state().await;
        let url = link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url;
        let token = file_of(&url);
        let bare = token.trim_end_matches(".ics").to_string();
        assert_eq!(get_feed(&state, &bare).await.0, StatusCode::NOT_FOUND, "suffix required");
        assert_eq!(get_feed(&state, &format!("{}.ics", "a".repeat(64))).await.0, StatusCode::NOT_FOUND);
        assert_eq!(get_feed(&state, "not-a-token.ics").await.0, StatusCode::NOT_FOUND);
        let upper = format!("{}.ics", token.trim_end_matches(".ics").to_uppercase());
        assert_eq!(get_feed(&state, &upper).await.0, StatusCode::NOT_FOUND, "tokens are lowercase");
    }

    #[test]
    fn tokens_are_64_lowercase_hex() {
        let t = new_token();
        assert_eq!(t.len(), 64);
        assert!(crate::planner::reminders::is_token(&t));
        assert_ne!(t, new_token());
    }

    #[test]
    fn public_url_has_a_default() {
        // Only checks the shape: the env var is not set by the test runner.
        let url = feed_url(&"a".repeat(64));
        assert!(url.starts_with("http"));
        assert!(url.ends_with(&format!("/api/public/reminders/{}.ics", "a".repeat(64))));
        assert!(!url.contains("//api"));
    }

    /// Unfolds the calendar text so long lines can be searched.
    fn unfolded(text: &str) -> String {
        text.replace("\r\n ", "")
    }

    async fn feed_with_people(state: &AppState, names: bool) -> String {
        let today = today_local();
        let url = link(State(state.clone()), user("u")).await.map_err(|e| e.1).unwrap().0.url;
        for q in [
            "INSERT INTO people (person_id, user_id, name, circle, every_days, created_at) VALUES ('p1', 'u', 'Person A', 'family', 28, '2026-01-01 10:00:00')",
            "INSERT INTO people (person_id, user_id, name, circle) VALUES ('p2', 'v', 'Person X', 'family')",
        ] {
            sqlx::query(q).execute(&state.db).await.unwrap();
        }
        sqlx::query("INSERT INTO gift_seasons (season_id, user_id, label, month, budget_paise) VALUES ('s1', 'u', 'Diwali', ?, 500000)")
            .bind(today.month().next() as i64)
            .execute(&state.db)
            .await
            .unwrap();
        let soon = today + time::Duration::days(10);
        let far = today + time::Duration::days(120);
        sqlx::query("INSERT INTO person_dates (date_id, user_id, person_id, kind, month, day, year, gift_budget_paise) VALUES ('d1', 'u', 'p1', 'birthday', ?, ?, 1966, 200000)")
            .bind(soon.month() as i64)
            .bind(soon.day() as i64)
            .execute(&state.db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO person_dates (date_id, user_id, person_id, kind, month, day) VALUES ('d2', 'u', 'p1', 'anniversary', ?, ?)")
            .bind(far.month() as i64)
            .bind(far.day() as i64)
            .execute(&state.db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO person_dates (date_id, user_id, person_id, kind, month, day) VALUES ('d3', 'v', 'p2', 'birthday', ?, ?)")
            .bind(soon.month() as i64)
            .bind(soon.day() as i64)
            .execute(&state.db)
            .await
            .unwrap();
        if names {
            sqlx::query("INSERT INTO user_settings (user_id, feed_names) VALUES ('u', 1)").execute(&state.db).await.unwrap();
        }
        let (status, _, body) = get_feed(state, &file_of(&url)).await;
        assert_eq!(status, StatusCode::OK);
        unfolded(&body)
    }

    #[tokio::test]
    async fn the_feed_keeps_names_out_by_default() {
        let state = test_state().await;
        let soon = today_local() + time::Duration::days(10);
        let text = feed_with_people(&state, false).await;
        assert!(text.contains("SUMMARY:Birthday (family)\r\n"), "{text}");
        assert!(text.contains(&format!("UID:joy-pdate-d1-{}@joy\r\n", soon.year())), "{text}");
        assert!(text.contains("SUMMARY:Gift season: Diwali\r\n"), "{text}");
        assert!(text.contains("SUMMARY:Catch up with someone\r\n"), "{text}");
        assert!(text.contains("SUMMARY:Gift for an upcoming date\r\n"), "{text}");
        assert!(!text.contains("Person A") && !text.contains("Person X"), "no names anywhere: {text}");
        assert!(!text.contains("d3"), "another user's date is not in this feed");
        assert!(!text.contains("joy-pdate-d2"), "120 days away is out of range");
    }

    #[tokio::test]
    async fn the_feed_shows_names_when_the_user_turns_them_on() {
        let state = test_state().await;
        let text = feed_with_people(&state, true).await;
        assert!(text.contains("SUMMARY:Person A's birthday\r\n"), "{text}");
        assert!(text.contains("SUMMARY:Catch up with Person A\r\n"), "{text}");
        assert!(text.contains("SUMMARY:Gift for Person A's birthday on "), "{text}");
        assert!(!text.contains("Person X"), "another user's person never appears");
    }
}
