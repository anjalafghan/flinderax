//! Analyses API, mounted at `/api/analyses`: short written reports saved against your account,
//! usually by a connector (Claude) that read your data and is writing back what it found. Every query
//! is scoped by `user_id`. Text only: no amounts or names are computed or shared from here.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    handlers::common::{bad_request, not_found, today_local, AppError},
    models::AppState,
};

type User = Extension<(String, String)>;

/// Most analyses one user keeps; the oldest are dropped past this.
const MAX_KEPT: i64 = 60;
const MAX_TITLE: usize = 120;
const MAX_BODY: usize = 20_000;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct AnalysisRow {
    pub id: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub created_on: String,
}

#[derive(Deserialize)]
pub struct AnalysisBody {
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Serialize)]
pub struct AnalysesOut {
    pub analyses: Vec<AnalysisRow>,
}

type Raw = (String, String, String, String, String);

fn row((id, title, body, source, created_on): Raw) -> AnalysisRow {
    AnalysisRow { id, title, body, source, created_on }
}

pub async fn list(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<AnalysesOut>, AppError> {
    let rows: Vec<Raw> = sqlx::query_as(
        "SELECT analysis_id, title, body, source, created_on FROM analyses WHERE user_id = ? ORDER BY created_on DESC, rowid DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(AnalysesOut { analyses: rows.into_iter().map(row).collect() }))
}

pub async fn create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(input): Json<AnalysisBody>,
) -> Result<(StatusCode, Json<AnalysisRow>), AppError> {
    let title = input.title.trim();
    let body = input.body.trim();
    if title.is_empty() || body.is_empty() {
        return Err(bad_request("An analysis needs a title and some text."));
    }
    if title.chars().count() > MAX_TITLE {
        return Err(bad_request(format!("The title must be {MAX_TITLE} characters or fewer.")));
    }
    if body.chars().count() > MAX_BODY {
        return Err(bad_request(format!("The analysis must be {MAX_BODY} characters or fewer.")));
    }
    let source = match input.source.as_deref().map(str::trim) {
        Some(s) if !s.is_empty() && s.chars().count() <= 40 => s.to_string(),
        _ => "connector".to_string(),
    };
    let item = AnalysisRow { id: nanoid!(), title: title.to_string(), body: body.to_string(), source, created_on: today_local().to_string() };
    sqlx::query("INSERT INTO analyses (analysis_id, user_id, title, body, source, created_on) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&item.id)
        .bind(&user_id)
        .bind(&item.title)
        .bind(&item.body)
        .bind(&item.source)
        .bind(&item.created_on)
        .execute(&state.db)
        .await?;
    // Keep the newest MAX_KEPT.
    sqlx::query(
        "DELETE FROM analyses WHERE user_id = ? AND analysis_id NOT IN
         (SELECT analysis_id FROM analyses WHERE user_id = ? ORDER BY created_on DESC, rowid DESC LIMIT ?)",
    )
    .bind(&user_id)
    .bind(&user_id)
    .bind(MAX_KEPT)
    .execute(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn delete(State(state): State<AppState>, Extension((user_id, _)): User, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let done = sqlx::query("DELETE FROM analyses WHERE analysis_id = ? AND user_id = ?").bind(&id).bind(&user_id).execute(&state.db).await?;
    if done.rows_affected() == 0 {
        return Err(not_found("No such analysis."));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for id in ["u", "other"] {
            sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES (?, ?, 'x', 'user')")
                .bind(id)
                .bind(id)
                .execute(&db)
                .await
                .unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::from(rusty_paseto::prelude::Key::from(&[7u8; 32]));
        AppState { db, paseto_key: std::sync::Arc::new(key) }
    }

    fn user(id: &str) -> User {
        Extension((id.to_string(), "user".to_string()))
    }

    fn body(title: &str, text: &str) -> Json<AnalysisBody> {
        Json(AnalysisBody { title: title.into(), body: text.into(), source: None })
    }

    #[tokio::test]
    async fn saves_lists_and_keeps_each_persons_own() {
        let s = state().await;
        let (code, made) = create(State(s.clone()), user("u"), body(" October ", " Eating out rose. ")).await.ok().unwrap();
        assert_eq!(code, StatusCode::CREATED);
        assert_eq!((made.0.title.as_str(), made.0.body.as_str(), made.0.source.as_str()), ("October", "Eating out rose.", "connector"));
        assert_eq!(list(State(s.clone()), user("u")).await.ok().unwrap().0.analyses.len(), 1);
        assert!(list(State(s.clone()), user("other")).await.ok().unwrap().0.analyses.is_empty());
        // Someone else's id is a plain not-found, and the analysis survives.
        assert!(delete(State(s.clone()), user("other"), Path(made.0.id.clone())).await.is_err());
        assert_eq!(delete(State(s.clone()), user("u"), Path(made.0.id)).await.ok().unwrap(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn refuses_blank_and_oversized() {
        let s = state().await;
        assert!(create(State(s.clone()), user("u"), body("  ", "x")).await.is_err());
        assert!(create(State(s.clone()), user("u"), body("t", "")).await.is_err());
        assert!(create(State(s.clone()), user("u"), body(&"t".repeat(MAX_TITLE + 1), "x")).await.is_err());
        assert!(create(State(s), user("u"), body("t", &"x".repeat(MAX_BODY + 1))).await.is_err());
    }

    #[tokio::test]
    async fn keeps_only_the_newest() {
        let s = state().await;
        for i in 0..(MAX_KEPT + 3) {
            let _ = create(State(s.clone()), user("u"), body(&format!("a{i}"), "x")).await.ok().unwrap();
        }
        let all = list(State(s), user("u")).await.ok().unwrap().0.analyses;
        assert_eq!(all.len() as i64, MAX_KEPT);
        assert!(all.iter().any(|a| a.title == format!("a{}", MAX_KEPT + 2)));
        assert!(!all.iter().any(|a| a.title == "a0"));
    }
}
