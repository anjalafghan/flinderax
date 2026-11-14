//! `/api/people/followups`: a small to-do per person ("ask about the surgery") with an optional due
//! day. Private to each user: every query filters by `user_id`, so a foreign or unknown id is a 404.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::SqlitePool;
use time::{Date, Duration};

use crate::{
    handlers::common::{not_found, today_local, AppError},
    models::AppState,
    planner::people::day_of,
};

type User = Extension<(String, String)>;

const MAX_TEXT_CHARS: usize = 120;
/// Open follow-ups one user can hold, so the list can't grow without bound.
const MAX_OPEN: i64 = 200;
/// How far ahead a due day may be.
const MAX_AHEAD_DAYS: i64 = 365 * 5;
/// "Soon" means within this many days.
const SOON_DAYS: i64 = 7;
/// Recently done follow-ups shown under the open ones.
const DONE_SHOWN: i64 = 10;

fn unprocessable(msg: impl Into<String>) -> AppError {
    AppError(StatusCode::UNPROCESSABLE_ENTITY, msg.into())
}

fn double_option<'de, T, D>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(d)?))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FollowState {
    Overdue,
    Today,
    Soon,
    Later,
    /// Open, with no due day.
    Undated,
    Done,
}

/// Where a follow-up stands: its state and the days until it is due (negative when overdue).
pub fn classify(due: Option<Date>, done: bool, today: Date) -> (FollowState, Option<i64>) {
    if done {
        return (FollowState::Done, None);
    }
    let Some(due) = due else { return (FollowState::Undated, None) };
    let days = (due - today).whole_days();
    let state = match days {
        d if d < 0 => FollowState::Overdue,
        0 => FollowState::Today,
        d if d <= SOON_DAYS => FollowState::Soon,
        _ => FollowState::Later,
    };
    (state, Some(days))
}

#[derive(Serialize)]
pub struct FollowUp {
    pub id: String,
    pub person_id: String,
    pub person_name: String,
    pub text: String,
    pub due_on: Option<String>,
    pub done_on: Option<String>,
    pub state: FollowState,
    pub days_away: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct Row {
    followup_id: String,
    person_id: String,
    person_name: String,
    text: String,
    due_on: Option<String>,
    done_on: Option<String>,
}

impl Row {
    fn out(self, today: Date) -> FollowUp {
        let due = self.due_on.as_deref().and_then(day_of);
        let (state, days_away) = classify(due, self.done_on.is_some(), today);
        FollowUp {
            id: self.followup_id,
            person_id: self.person_id,
            person_name: self.person_name,
            text: self.text,
            due_on: self.due_on,
            done_on: self.done_on,
            state,
            days_away,
        }
    }
}

const COLS: &str = "f.followup_id, f.person_id, p.name AS person_name, f.text, f.due_on, f.done_on";

async fn find(db: &SqlitePool, user_id: &str, id: &str) -> Result<Row, AppError> {
    sqlx::query_as::<_, Row>(&format!(
        "SELECT {COLS} FROM followups f JOIN people p ON p.person_id = f.person_id WHERE f.followup_id = ? AND f.user_id = ?"
    ))
    .bind(id)
    .bind(user_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| not_found("Follow-up not found"))
}

fn check_text(t: &str) -> Result<String, AppError> {
    let t = t.trim();
    if t.is_empty() || t.chars().count() > MAX_TEXT_CHARS {
        return Err(unprocessable(format!("Write the follow-up in up to {MAX_TEXT_CHARS} characters.")));
    }
    Ok(t.to_string())
}

fn check_due(s: &str, today: Date) -> Result<String, AppError> {
    let d = day_of(s).filter(|_| s.len() == 10).ok_or_else(|| unprocessable("The due day must look like 2026-12-14."))?;
    if d < today {
        return Err(unprocessable("The due day can't be in the past."));
    }
    if (d - today) > Duration::days(MAX_AHEAD_DAYS) {
        return Err(unprocessable("That due day is too far ahead."));
    }
    Ok(d.to_string())
}

/// Overdue first, then the soonest due; the undated last in the order they were added.
fn order(rows: &mut [FollowUp]) {
    rows.sort_by(|a, b| match (&a.due_on, &b.due_on) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
}

#[derive(Serialize)]
pub struct FollowUpsOut {
    pub open: Vec<FollowUp>,
    pub done: Vec<FollowUp>,
}

/// The follow-ups of everyone not archived, as of `today`.
pub async fn compute(db: &SqlitePool, user_id: &str, today: Date) -> Result<FollowUpsOut, AppError> {
    let rows: Vec<Row> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM followups f JOIN people p ON p.person_id = f.person_id
          WHERE f.user_id = ? AND p.archived = 0 AND f.done_on IS NULL ORDER BY f.created_at, f.followup_id"
    ))
    .bind(user_id)
    .fetch_all(db)
    .await?;
    let mut open: Vec<FollowUp> = rows.into_iter().map(|r| r.out(today)).collect();
    order(&mut open);
    let done: Vec<Row> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM followups f JOIN people p ON p.person_id = f.person_id
          WHERE f.user_id = ? AND p.archived = 0 AND f.done_on IS NOT NULL ORDER BY f.done_on DESC, f.created_at DESC LIMIT ?"
    ))
    .bind(user_id)
    .bind(DONE_SHOWN)
    .fetch_all(db)
    .await?;
    Ok(FollowUpsOut { open, done: done.into_iter().map(|r| r.out(today)).collect() })
}

pub async fn list(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<FollowUpsOut>, AppError> {
    Ok(Json(compute(&state.db, &user_id, today_local()).await?))
}

#[derive(Deserialize)]
pub struct NewFollowUp {
    pub text: String,
    pub due_on: Option<String>,
}

pub async fn add(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(person_id): Path<String>,
    Json(p): Json<NewFollowUp>,
) -> Result<Json<FollowUp>, AppError> {
    let today = today_local();
    let owned: Option<String> = sqlx::query_scalar("SELECT person_id FROM people WHERE person_id = ? AND user_id = ?")
        .bind(&person_id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await?;
    if owned.is_none() {
        return Err(not_found("Person not found"));
    }
    let text = check_text(&p.text)?;
    let due = p.due_on.as_deref().map(|d| check_due(d, today)).transpose()?;
    let open: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM followups WHERE user_id = ? AND done_on IS NULL")
        .bind(&user_id)
        .fetch_one(&state.db)
        .await?;
    if open >= MAX_OPEN {
        return Err(unprocessable("That's a lot of open follow-ups. Tick some off first."));
    }
    let id = nanoid!();
    sqlx::query("INSERT INTO followups (followup_id, user_id, person_id, text, due_on) VALUES (?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&user_id)
        .bind(&person_id)
        .bind(&text)
        .bind(due)
        .execute(&state.db)
        .await?;
    Ok(Json(find(&state.db, &user_id, &id).await?.out(today)))
}

#[derive(Deserialize)]
pub struct FollowUpPatch {
    pub text: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub due_on: Option<Option<String>>,
    pub done: Option<bool>,
}

pub async fn patch(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
    Json(p): Json<FollowUpPatch>,
) -> Result<Json<FollowUp>, AppError> {
    let today = today_local();
    let cur = find(&state.db, &user_id, &id).await?;
    let text = match &p.text {
        Some(t) => check_text(t)?,
        None => cur.text.clone(),
    };
    let due = match &p.due_on {
        Some(Some(d)) => Some(check_due(d, today)?),
        Some(None) => None,
        None => cur.due_on.clone(),
    };
    let done_on = match p.done {
        Some(true) => cur.done_on.clone().or_else(|| Some(today.to_string())),
        Some(false) => None,
        None => cur.done_on.clone(),
    };
    sqlx::query("UPDATE followups SET text = ?, due_on = ?, done_on = ? WHERE followup_id = ? AND user_id = ?")
        .bind(&text)
        .bind(&due)
        .bind(&done_on)
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(find(&state.db, &user_id, &id).await?.out(today)))
}

pub async fn delete(State(state): State<AppState>, Extension((user_id, _)): User, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM followups WHERE followup_id = ? AND user_id = ?").bind(&id).bind(&user_id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Follow-up not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use time::Month;

    fn d(y: i32, m: u8, day: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), day).unwrap()
    }

    #[test]
    fn states_follow_the_due_day() {
        let today = d(2026, 11, 14);
        assert_eq!(classify(None, false, today), (FollowState::Undated, None));
        assert_eq!(classify(Some(d(2026, 11, 13)), false, today), (FollowState::Overdue, Some(-1)));
        assert_eq!(classify(Some(today), false, today), (FollowState::Today, Some(0)));
        assert_eq!(classify(Some(d(2026, 11, 21)), false, today), (FollowState::Soon, Some(7)));
        assert_eq!(classify(Some(d(2026, 11, 22)), false, today), (FollowState::Later, Some(8)));
        assert_eq!(classify(Some(d(2020, 1, 1)), true, today), (FollowState::Done, None));
    }

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

    async fn add_to(s: &AppState, who: &str, person: &str, text: &str, due: Option<String>) -> Result<Json<FollowUp>, AppError> {
        add(State(s.clone()), user(who), Path(person.into()), Json(NewFollowUp { text: text.into(), due_on: due })).await
    }

    fn in_days(n: i64) -> String {
        (today_local() + Duration::days(n)).to_string()
    }

    #[tokio::test]
    async fn add_tick_untick_edit_and_delete() {
        let s = test_state().await;
        let f = ok(add_to(&s, "u", "p1", "  Ask about the surgery ", Some(in_days(3))).await).0;
        assert_eq!((f.text.as_str(), f.person_name.as_str(), f.state), ("Ask about the surgery", "Asha", FollowState::Soon));

        let done = ok(patch(State(s.clone()), user("u"), Path(f.id.clone()), Json(FollowUpPatch { text: None, due_on: None, done: Some(true) })).await).0;
        assert_eq!((done.state, done.done_on.as_deref()), (FollowState::Done, Some(today_local().to_string().as_str())));
        let out = ok(compute(&s.db, "u", today_local()).await);
        assert_eq!((out.open.len(), out.done.len()), (0, 1));

        let undone = ok(patch(State(s.clone()), user("u"), Path(f.id.clone()), Json(FollowUpPatch { text: Some("Send the photos".into()), due_on: Some(None), done: Some(false) })).await).0;
        assert_eq!((undone.state, undone.text.as_str(), undone.due_on), (FollowState::Undated, "Send the photos", None));

        assert_eq!(ok(delete(State(s.clone()), user("u"), Path(f.id.clone())).await), StatusCode::NO_CONTENT);
        assert_eq!(code(delete(State(s.clone()), user("u"), Path(f.id)).await), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn bad_input_is_refused() {
        let s = test_state().await;
        for text in ["", "   ", &"x".repeat(MAX_TEXT_CHARS + 1)] {
            assert_eq!(code(add_to(&s, "u", "p1", text, None).await), StatusCode::UNPROCESSABLE_ENTITY, "{text:?}");
        }
        for due in [in_days(-1), in_days(MAX_AHEAD_DAYS + 1), "soon".into(), "2026-1-5".into(), String::new()] {
            assert_eq!(code(add_to(&s, "u", "p1", "ok", Some(due.clone())).await), StatusCode::UNPROCESSABLE_ENTITY, "{due}");
        }
        assert!(add_to(&s, "u", "p1", &"é".repeat(MAX_TEXT_CHARS), Some(in_days(0))).await.is_ok(), "120 multi-byte characters are fine");
        let f = ok(add_to(&s, "u", "p1", "ok", None).await).0;
        let bad = patch(State(s.clone()), user("u"), Path(f.id), Json(FollowUpPatch { text: None, due_on: Some(Some(in_days(-3))), done: None })).await;
        assert_eq!(code(bad), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn the_list_is_ordered_overdue_first_and_undated_last() {
        let s = test_state().await;
        let later = ok(add_to(&s, "u", "p1", "later", Some(in_days(30))).await).0;
        let none = ok(add_to(&s, "u", "p2", "no date", None).await).0;
        let soon = ok(add_to(&s, "u", "p2", "soon", Some(in_days(2))).await).0;
        let today = ok(add_to(&s, "u", "p1", "today", Some(in_days(0))).await).0;
        // Yesterday can't be added any more, so make one overdue the way time would.
        sqlx::query("UPDATE followups SET due_on = ? WHERE followup_id = ?").bind(in_days(-2)).bind(&later.id).execute(&s.db).await.unwrap();
        let out = ok(compute(&s.db, "u", today_local()).await);
        let ids: Vec<&str> = out.open.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(ids, [later.id.as_str(), today.id.as_str(), soon.id.as_str(), none.id.as_str()]);
        assert_eq!(out.open[0].state, FollowState::Overdue);
    }

    #[tokio::test]
    async fn everything_is_scoped_to_its_owner() {
        let s = test_state().await;
        // Someone else's person can't take my follow-up.
        assert_eq!(code(add_to(&s, "u", "po", "x", None).await), StatusCode::NOT_FOUND);
        let mine = ok(add_to(&s, "u", "p1", "mine", None).await).0;
        let theirs = ok(add_to(&s, "other", "po", "theirs", None).await).0;
        assert_eq!(ok(compute(&s.db, "u", today_local()).await).open.len(), 1);
        assert_eq!(ok(compute(&s.db, "other", today_local()).await).open[0].text, "theirs");
        assert_eq!(code(patch(State(s.clone()), user("other"), Path(mine.id.clone()), Json(FollowUpPatch { text: None, due_on: None, done: Some(true) })).await), StatusCode::NOT_FOUND);
        assert_eq!(code(delete(State(s.clone()), user("other"), Path(mine.id.clone())).await), StatusCode::NOT_FOUND);
        assert_eq!(code(delete(State(s.clone()), user("u"), Path(theirs.id)).await), StatusCode::NOT_FOUND);
        assert_eq!(ok(compute(&s.db, "u", today_local()).await).open[0].id, mine.id);
    }

    #[tokio::test]
    async fn removing_or_archiving_a_person_takes_their_follow_ups_off_the_list() {
        let s = test_state().await;
        let _ = ok(add_to(&s, "u", "p1", "a", None).await);
        let _ = ok(add_to(&s, "u", "p2", "b", None).await);
        sqlx::query("UPDATE people SET archived = 1 WHERE person_id = 'p1'").execute(&s.db).await.unwrap();
        assert_eq!(ok(compute(&s.db, "u", today_local()).await).open.len(), 1);
        sqlx::query("DELETE FROM people WHERE person_id = 'p2'").execute(&s.db).await.unwrap();
        let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM followups WHERE person_id = 'p2'").fetch_one(&s.db).await.unwrap();
        assert_eq!(left, 0, "deleting a person cascades");
    }

    #[tokio::test]
    async fn open_follow_ups_are_capped() {
        let s = test_state().await;
        for i in 0..MAX_OPEN {
            sqlx::query("INSERT INTO followups (followup_id, user_id, person_id, text) VALUES (?, 'u', 'p1', 'x')").bind(format!("f{i}")).execute(&s.db).await.unwrap();
        }
        assert_eq!(code(add_to(&s, "u", "p1", "one more", None).await), StatusCode::UNPROCESSABLE_ENTITY);
        // Ticking one off makes room.
        let _ = ok(patch(State(s.clone()), user("u"), Path("f0".into()), Json(FollowUpPatch { text: None, due_on: None, done: Some(true) })).await);
        assert!(add_to(&s, "u", "p1", "now it fits", None).await.is_ok());
    }
}
