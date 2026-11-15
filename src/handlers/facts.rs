//! `/api/people/facts`: short things to remember about a person. Private to each user: every query
//! filters by `user_id`, so a foreign or unknown id is a 404.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    handlers::common::{not_found, AppError},
    models::AppState,
};

type User = Extension<(String, String)>;

const MAX_TEXT_CHARS: usize = 120;
const MAX_PER_PERSON: i64 = 30;
const MAX_PER_USER: i64 = 500;

#[derive(Serialize, sqlx::FromRow)]
pub struct Fact {
    pub id: String,
    pub person_id: String,
    pub text: String,
}

#[derive(Serialize)]
pub struct FactsOut {
    pub facts: Vec<Fact>,
}

fn unprocessable(msg: impl Into<String>) -> AppError {
    AppError(StatusCode::UNPROCESSABLE_ENTITY, msg.into())
}

fn check_text(t: &str) -> Result<String, AppError> {
    let t = t.trim();
    if t.is_empty() || t.chars().count() > MAX_TEXT_CHARS {
        return Err(unprocessable(format!("Write it in up to {MAX_TEXT_CHARS} characters.")));
    }
    Ok(t.to_string())
}

/// Everyone's facts (not archived people), oldest first within each person.
pub async fn list(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<FactsOut>, AppError> {
    let facts = sqlx::query_as::<_, Fact>(
        "SELECT f.fact_id AS id, f.person_id, f.text FROM person_facts f JOIN people p ON p.person_id = f.person_id
          WHERE f.user_id = ? AND p.archived = 0 ORDER BY f.created_at, f.fact_id",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(FactsOut { facts }))
}

#[derive(Deserialize)]
pub struct NewFact {
    pub text: String,
}

pub async fn add(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(person_id): Path<String>,
    Json(p): Json<NewFact>,
) -> Result<Json<Fact>, AppError> {
    let owned: Option<String> = sqlx::query_scalar("SELECT person_id FROM people WHERE person_id = ? AND user_id = ?")
        .bind(&person_id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await?;
    if owned.is_none() {
        return Err(not_found("Person not found"));
    }
    let text = check_text(&p.text)?;
    let (mine, all): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM person_facts WHERE user_id = ?1 AND person_id = ?2), (SELECT COUNT(*) FROM person_facts WHERE user_id = ?1)",
    )
    .bind(&user_id)
    .bind(&person_id)
    .fetch_one(&state.db)
    .await?;
    if mine >= MAX_PER_PERSON || all >= MAX_PER_USER {
        return Err(unprocessable("That's plenty to remember. Remove one you no longer need first."));
    }
    let id = nanoid!();
    sqlx::query("INSERT INTO person_facts (fact_id, user_id, person_id, text) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&user_id)
        .bind(&person_id)
        .bind(&text)
        .execute(&state.db)
        .await?;
    Ok(Json(Fact { id, person_id, text }))
}

pub async fn delete(State(state): State<AppState>, Extension((user_id, _)): User, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM person_facts WHERE fact_id = ? AND user_id = ?").bind(&id).bind(&user_id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('other', 'other', 'x', 'user')",
            "INSERT INTO people (person_id, user_id, name, circle) VALUES ('p1', 'u', 'Asha', 'family')",
            "INSERT INTO people (person_id, user_id, name, circle) VALUES ('p2', 'u', 'Ravi', 'friend')",
            "INSERT INTO people (person_id, user_id, name, circle) VALUES ('po', 'other', 'Theirs', 'friend')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32][..]),
        );
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    fn user(id: &str) -> User {
        Extension((id.to_string(), String::new()))
    }

    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("handler failed: {} {}", e.0, e.1),
        }
    }

    fn code<T>(r: Result<T, AppError>) -> StatusCode {
        match r {
            Ok(_) => panic!("expected an error"),
            Err(e) => e.0,
        }
    }

    async fn put(s: &AppState, who: &str, person: &str, text: &str) -> Result<Json<Fact>, AppError> {
        add(State(s.clone()), user(who), Path(person.into()), Json(NewFact { text: text.into() })).await
    }

    #[tokio::test]
    async fn add_list_and_delete() {
        let s = test_state().await;
        let f = ok(put(&s, "u", "p1", "  Allergic to peanuts ").await).0;
        assert_eq!(f.text, "Allergic to peanuts");
        let _ = ok(put(&s, "u", "p2", "Loves filter coffee").await);
        let out = ok(list(State(s.clone()), user("u")).await).0;
        assert_eq!(out.facts.len(), 2);
        assert_eq!(ok(delete(State(s.clone()), user("u"), Path(f.id.clone())).await), StatusCode::NO_CONTENT);
        assert_eq!(code(delete(State(s.clone()), user("u"), Path(f.id)).await), StatusCode::NOT_FOUND);
        assert_eq!(ok(list(State(s.clone()), user("u")).await).0.facts.len(), 1);
    }

    #[tokio::test]
    async fn bad_text_is_refused_and_multibyte_counts_by_characters() {
        let s = test_state().await;
        for text in ["", "   ", &"x".repeat(MAX_TEXT_CHARS + 1)] {
            assert_eq!(code(put(&s, "u", "p1", text).await), StatusCode::UNPROCESSABLE_ENTITY, "{text:?}");
        }
        assert!(put(&s, "u", "p1", &"é".repeat(MAX_TEXT_CHARS)).await.is_ok());
    }

    #[tokio::test]
    async fn the_limits_hold() {
        let s = test_state().await;
        for i in 0..MAX_PER_PERSON {
            sqlx::query("INSERT INTO person_facts (fact_id, user_id, person_id, text) VALUES (?, 'u', 'p1', 'x')").bind(format!("f{i}")).execute(&s.db).await.unwrap();
        }
        assert_eq!(code(put(&s, "u", "p1", "one more").await), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(put(&s, "u", "p2", "another person is fine").await.is_ok());
    }

    #[tokio::test]
    async fn scoped_to_the_owner_and_gone_with_the_person() {
        let s = test_state().await;
        assert_eq!(code(put(&s, "u", "po", "x").await), StatusCode::NOT_FOUND);
        let mine = ok(put(&s, "u", "p1", "mine").await).0;
        let _ = ok(put(&s, "other", "po", "theirs").await);
        assert_eq!(ok(list(State(s.clone()), user("other")).await).0.facts[0].text, "theirs");
        assert_eq!(code(delete(State(s.clone()), user("other"), Path(mine.id)).await), StatusCode::NOT_FOUND);
        sqlx::query("UPDATE people SET archived = 1 WHERE person_id = 'p1'").execute(&s.db).await.unwrap();
        assert!(ok(list(State(s.clone()), user("u")).await).0.facts.is_empty(), "archived people drop off the list");
        sqlx::query("DELETE FROM people WHERE person_id = 'p1'").execute(&s.db).await.unwrap();
        let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM person_facts").fetch_one(&s.db).await.unwrap();
        assert_eq!(left, 1, "only the other user's fact is left");
    }
}
