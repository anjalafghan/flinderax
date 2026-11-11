//! Life line API, mounted under `/api/plan/lifeline`.
//!
//! * `GET /` returns the line for the plan (the profile plus the choices kept in the plan), the
//!   saved choices, and the working behind the starting figures.
//! * `POST /try` `{base, with}` runs two sets of choices and says what the second changes. It
//!   saves nothing.
//! * `POST /choices`, `PATCH /choices/{id}`, `DELETE /choices/{id}` keep, change and drop choices.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::Date;

use crate::{
    handlers::{
        autopost,
        common::{bad_request, not_found, today_local, AppError},
        life::{load_profile, resolve_expenses},
    },
    models::AppState,
    planner::{
        life::LifeProfile,
        lifeline::{compare, note, simulate, validate, Choice, Consequences, LifelineNote, Line, LineInput, MAX_CHOICES},
    },
    validate::MAX_AMOUNT_PAISE,
};

type User = Extension<(String, String)>;

/// Most choices one user may keep.
const MAX_SAVED: i64 = 30;
const MAX_LABEL: usize = 60;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct SavedChoice {
    pub id: String,
    pub label: String,
    pub in_plan: bool,
    pub choice: Choice,
}

/// Where the line starts from, so the page can show it and the user can correct it.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Working {
    pub accounts_paise: i64,
    pub plan_saved_paise: i64,
    pub loans_paise: i64,
    pub wealth_paise: i64,
    pub expenses_paise: i64,
    pub income_paise: i64,
    pub note: String,
}

#[derive(Serialize)]
pub struct LifelineOut {
    pub saved_profile: bool,
    pub line: Line,
    pub choices: Vec<SavedChoice>,
    pub working: Working,
}

#[derive(Deserialize)]
pub struct TryIn {
    #[serde(default)]
    pub base: Vec<Choice>,
    #[serde(default)]
    pub with: Vec<Choice>,
}

#[derive(Serialize)]
pub struct TryOut {
    pub base: Line,
    pub with: Line,
    pub consequences: Consequences,
}

#[derive(Deserialize)]
pub struct ChoiceIn {
    pub label: String,
    pub choice: Choice,
    #[serde(default)]
    pub in_plan: bool,
}

#[derive(Deserialize)]
pub struct ChoicePatch {
    pub label: Option<String>,
    pub choice: Option<Choice>,
    pub in_plan: Option<bool>,
}

/// The profile, expenses and starting wealth every line for this user is drawn from.
struct Basis {
    profile: LifeProfile,
    saved_profile: bool,
    working: Working,
}

impl Basis {
    fn line(&self, choices: Vec<Choice>, today: Date) -> Line {
        simulate(
            &LineInput {
                profile: self.profile.clone(),
                expenses_paise: self.working.expenses_paise,
                wealth_paise: self.working.wealth_paise,
                choices,
            },
            today,
        )
    }
}

async fn basis(db: &SqlitePool, user_id: &str, today: Date) -> Result<Basis, AppError> {
    let (profile, saved_profile) = load_profile(db, user_id).await?;
    let expenses = resolve_expenses(db, user_id, today, &profile).await?;
    autopost::apply(db, user_id, today).await?;
    let accounts: i64 = sqlx::query_scalar::<_, i64>("SELECT COALESCE(SUM(balance_paise), 0) FROM accounts WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(db)
        .await?;
    let cap = |v: i64| v.clamp(0, MAX_AMOUNT_PAISE);
    let plan_saved = cap(profile.emergency_saved_paise)
        .saturating_add(cap(profile.wedding_saved_paise))
        .saturating_add(cap(profile.child_saved_paise))
        .saturating_add(cap(profile.retirement_saved_paise));
    let loans = cap(profile.loans_paise);
    // The accounts and the plan's "saved so far" figures overlap (the emergency fund sits in a
    // bank account), so adding them would count money twice. The larger of the two is used.
    let owned = accounts.clamp(-MAX_AMOUNT_PAISE, MAX_AMOUNT_PAISE).max(plan_saved);
    let note = if plan_saved >= accounts {
        "Starts from what the life plan says is saved, less loans.".to_string()
    } else {
        "Starts from your account balances, less loans. Add investments held elsewhere to the life plan's saved amounts.".to_string()
    };
    Ok(Basis {
        saved_profile,
        working: Working {
            accounts_paise: accounts,
            plan_saved_paise: plan_saved,
            loans_paise: loans,
            wealth_paise: owned.saturating_sub(loans),
            expenses_paise: expenses.clamp(0, MAX_AMOUNT_PAISE),
            income_paise: cap(profile.my_take_home_paise).saturating_add(cap(profile.partner_take_home_paise)),
            note,
        },
        profile,
    })
}

async fn saved_choices(db: &SqlitePool, user_id: &str) -> Result<Vec<SavedChoice>, AppError> {
    let rows: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT choice_id, label, choice_json, in_plan FROM life_choices WHERE user_id = ? ORDER BY created_at, choice_id",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    // A row that no longer parses (an older shape) is left out, not an error.
    Ok(rows
        .into_iter()
        .filter_map(|(id, label, json, in_plan)| {
            let choice = serde_json::from_str(&json).ok()?;
            Some(SavedChoice { id, label, in_plan: in_plan != 0, choice })
        })
        .collect())
}

fn check_choices(choices: &[Choice], today: Date) -> Result<(), AppError> {
    if choices.len() > MAX_CHOICES {
        return Err(bad_request(format!("At most {MAX_CHOICES} choices at a time.")));
    }
    choices.iter().try_for_each(|c| validate(c, today).map_err(bad_request))
}

fn clean_label(label: &str) -> Result<String, AppError> {
    let label = label.trim();
    if label.is_empty() || label.chars().count() > MAX_LABEL {
        return Err(bad_request(format!("The name must be 1 to {MAX_LABEL} characters.")));
    }
    Ok(label.to_string())
}

/// The life line for the quarterly check-in: today's marks against those kept with the last
/// check-in done before `today`. `None` until the user has saved their details.
pub(crate) async fn checkin_note(db: &SqlitePool, user_id: &str, today: Date) -> Result<Option<LifelineNote>, AppError> {
    let basis = basis(db, user_id, today).await?;
    if !basis.saved_profile {
        return Ok(None);
    }
    let in_plan = saved_choices(db, user_id).await?.into_iter().filter(|c| c.in_plan).map(|c| c.choice).collect();
    let last: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT summary_json FROM checkins WHERE user_id = ? AND done_on < ? ORDER BY done_on DESC, created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .bind(today.to_string())
    .fetch_optional(db)
    .await?;
    // Check-ins saved before the life line existed have no note: nothing to compare with.
    let before: Option<LifelineNote> = last
        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        .and_then(|v| serde_json::from_value(v.get("lifeline")?.clone()).ok());
    Ok(Some(note(&basis.line(in_plan, today).marks, before.as_ref())))
}

pub async fn get(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<LifelineOut>, AppError> {
    let today = today_local();
    let basis = basis(&state.db, &user_id, today).await?;
    let choices = saved_choices(&state.db, &user_id).await?;
    let in_plan = choices.iter().filter(|c| c.in_plan).map(|c| c.choice.clone()).collect();
    Ok(Json(LifelineOut { saved_profile: basis.saved_profile, line: basis.line(in_plan, today), choices, working: basis.working }))
}

pub async fn try_choices(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(body): Json<TryIn>,
) -> Result<Json<TryOut>, AppError> {
    let today = today_local();
    check_choices(&body.base, today)?;
    check_choices(&body.with, today)?;
    let basis = basis(&state.db, &user_id, today).await?;
    let (base, with) = (basis.line(body.base, today), basis.line(body.with, today));
    let consequences = compare(&base, &with);
    Ok(Json(TryOut { base, with, consequences }))
}

pub async fn create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(body): Json<ChoiceIn>,
) -> Result<(StatusCode, Json<SavedChoice>), AppError> {
    let db = &state.db;
    let label = clean_label(&body.label)?;
    validate(&body.choice, today_local()).map_err(bad_request)?;
    let count: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM life_choices WHERE user_id = ?")
        .bind(&user_id)
        .fetch_one(db)
        .await?;
    if count >= MAX_SAVED {
        return Err(AppError(StatusCode::CONFLICT, format!("You can keep {MAX_SAVED} choices. Remove one first.")));
    }
    let id = nanoid!();
    let json = serde_json::to_string(&body.choice).map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query("INSERT INTO life_choices (choice_id, user_id, label, choice_json, in_plan) VALUES (?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&user_id)
        .bind(&label)
        .bind(&json)
        .bind(body.in_plan as i64)
        .execute(db)
        .await?;
    Ok((StatusCode::CREATED, Json(SavedChoice { id, label, in_plan: body.in_plan, choice: body.choice })))
}

pub async fn patch(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
    Json(body): Json<ChoicePatch>,
) -> Result<Json<SavedChoice>, AppError> {
    let db = &state.db;
    let mut current = saved_choices(db, &user_id)
        .await?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| not_found("No such choice."))?;
    if let Some(label) = &body.label {
        current.label = clean_label(label)?;
    }
    if let Some(choice) = body.choice {
        validate(&choice, today_local()).map_err(bad_request)?;
        current.choice = choice;
    }
    if let Some(in_plan) = body.in_plan {
        current.in_plan = in_plan;
    }
    let json = serde_json::to_string(&current.choice).map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query("UPDATE life_choices SET label = ?, choice_json = ?, in_plan = ? WHERE choice_id = ? AND user_id = ?")
        .bind(&current.label)
        .bind(&json)
        .bind(current.in_plan as i64)
        .bind(&id)
        .bind(&user_id)
        .execute(db)
        .await?;
    Ok(Json(current))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let done = sqlx::query("DELETE FROM life_choices WHERE choice_id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    if done.rows_affected() == 0 {
        return Err(not_found("No such choice."));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct Cleared {
    pub removed: u64,
}

/// `POST /api/plan/lifeline/choices/clear`: takes every choice off the line (including any that
/// can no longer be read and so are hidden but still count towards the limit).
pub async fn clear(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Cleared>, AppError> {
    let removed = sqlx::query("DELETE FROM life_choices WHERE user_id = ?").bind(&user_id).execute(&state.db).await?.rows_affected();
    Ok(Json(Cleared { removed }))
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
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    fn user(id: &str) -> User {
        Extension((id.to_string(), "user".to_string()))
    }

    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("{}: {}", e.0, e.1),
        }
    }

    async fn save_profile(s: &AppState, json: &str) {
        sqlx::query("INSERT INTO life_profiles (user_id, profile_json) VALUES ('u', ?)").bind(json).execute(&s.db).await.unwrap();
    }

    fn retire_at(age: i64) -> ChoiceIn {
        ChoiceIn { label: format!("Retire at {age}"), choice: Choice::RetireAt { age }, in_plan: false }
    }

    #[tokio::test]
    async fn a_new_user_gets_a_line_to_ninety_from_the_defaults() {
        let s = state().await;
        let out = ok(get(State(s), user("u")).await).0;
        assert!(!out.saved_profile);
        assert_eq!(out.line.rows.last().unwrap().age, 90);
        assert!(out.choices.is_empty());
        assert_eq!(out.working.wealth_paise, 0);
    }

    #[tokio::test]
    async fn the_line_starts_from_the_larger_of_accounts_and_plan_savings_less_loans() {
        let s = state().await;
        save_profile(&s, r#"{"my_age":29,"my_take_home_paise":15000000,"monthly_expenses_paise":7000000,"emergency_saved_paise":30000000,"retirement_saved_paise":20000000,"loans_paise":10000000}"#).await;
        let out = ok(get(State(s.clone()), user("u")).await).0;
        assert_eq!((out.working.plan_saved_paise, out.working.accounts_paise), (50_000_000, 0));
        assert_eq!(out.working.wealth_paise, 40_000_000);
        assert_eq!(out.working.expenses_paise, 7_000_000);
        assert_eq!(out.line.rows[0].age, 29);

        sqlx::query("INSERT INTO accounts (account_id, user_id, name, kind, balance_paise) VALUES ('a', 'u', 'Bank', 'bank', 90000000)")
            .execute(&s.db)
            .await
            .unwrap();
        let out = ok(get(State(s), user("u")).await).0;
        assert_eq!(out.working.wealth_paise, 80_000_000);
        assert!(out.working.note.contains("account balances"));
    }

    #[tokio::test]
    async fn a_kept_choice_changes_the_plan_line_only_when_it_is_in_the_plan() {
        let s = state().await;
        save_profile(&s, r#"{"my_age":29,"my_take_home_paise":15000000,"monthly_expenses_paise":7000000}"#).await;
        let (code, saved) = ok(create(State(s.clone()), user("u"), Json(retire_at(50))).await);
        assert_eq!(code, StatusCode::CREATED);
        let out = ok(get(State(s.clone()), user("u")).await).0;
        assert_eq!(out.choices, vec![saved.0.clone()]);
        assert_eq!(out.line.marks.retire_age, 60);

        let patched = ok(patch(
            State(s.clone()),
            user("u"),
            Path(saved.0.id.clone()),
            Json(ChoicePatch { label: None, choice: None, in_plan: Some(true) }),
        )
        .await)
        .0;
        assert!(patched.in_plan);
        let out = ok(get(State(s.clone()), user("u")).await).0;
        assert_eq!(out.line.marks.retire_age, 50);

        assert_eq!(ok(delete(State(s.clone()), user("u"), Path(saved.0.id)).await), StatusCode::NO_CONTENT);
        assert!(ok(get(State(s), user("u")).await).0.choices.is_empty());
    }

    #[tokio::test]
    async fn the_check_in_note_compares_with_the_last_saved_check_in() {
        let s = state().await;
        let today = time::macros::date!(2026 - 11 - 10);
        assert!(ok(checkin_note(&s.db, "u", today).await).is_none(), "no saved details, no note");
        save_profile(&s, r#"{"my_age":29,"my_take_home_paise":15000000,"monthly_expenses_paise":7000000}"#).await;
        let first = ok(checkin_note(&s.db, "u", today).await).expect("a note");
        assert!(first.line.starts_with("On today's numbers, work becomes optional at "), "{}", first.line);
        assert!(!first.line.contains("Last check-in"));

        // An older check-in that kept a later age, and one from before the life line existed.
        for (id, day, json) in [
            ("old", "2026-05-01", "{}".to_string()),
            ("last", "2026-08-10", r#"{"lifeline":{"free_at_age":70,"runs_out_at_age":null,"end_age":90,"line":""}}"#.to_string()),
        ] {
            sqlx::query("INSERT INTO checkins (checkin_id, user_id, done_on, period_start, period_end, summary_json) VALUES (?, 'u', ?, ?, ?, ?)")
                .bind(id)
                .bind(day)
                .bind(day)
                .bind(day)
                .bind(json)
                .execute(&s.db)
                .await
                .unwrap();
        }
        let moved = ok(checkin_note(&s.db, "u", today).await).expect("a note");
        assert!(moved.line.ends_with("Last check-in work became optional at 70."), "{}", moved.line);
    }

    #[tokio::test]
    async fn trying_a_choice_says_what_changes_and_saves_nothing() {
        let s = state().await;
        save_profile(&s, r#"{"my_age":29,"my_take_home_paise":15000000,"monthly_expenses_paise":7000000}"#).await;
        let body = TryIn {
            base: vec![],
            with: vec![Choice::JobSwitch { month: "2027-04".into(), take_home_paise: 22_000_000, gap_months: 0, raise_bps: None }],
        };
        let out = ok(try_choices(State(s.clone()), user("u"), Json(body)).await).0;
        assert!(out.consequences.better, "{:?}", out.consequences.lines);
        assert!(out.with.marks.at_end_today_paise > out.base.marks.at_end_today_paise);
        assert!(ok(get(State(s), user("u")).await).0.choices.is_empty());
    }

    #[tokio::test]
    async fn bad_choices_and_other_users_rows_are_refused() {
        let s = state().await;
        let bad = try_choices(State(s.clone()), user("u"), Json(TryIn { base: vec![], with: vec![Choice::RetireAt { age: 5 }] })).await;
        assert_eq!(bad.err().map(|e| e.0), Some(StatusCode::BAD_REQUEST));
        let too_many = TryIn { base: vec![], with: vec![Choice::RetireAt { age: 55 }; MAX_CHOICES + 1] };
        assert_eq!(try_choices(State(s.clone()), user("u"), Json(too_many)).await.err().map(|e| e.0), Some(StatusCode::BAD_REQUEST));
        let unnamed = ChoiceIn { label: "  ".into(), ..retire_at(55) };
        assert_eq!(create(State(s.clone()), user("u"), Json(unnamed)).await.err().map(|e| e.0), Some(StatusCode::BAD_REQUEST));

        let (_, mine) = ok(create(State(s.clone()), user("u"), Json(retire_at(55))).await);
        let theirs = patch(
            State(s.clone()),
            user("other"),
            Path(mine.0.id.clone()),
            Json(ChoicePatch { label: Some("x".into()), choice: None, in_plan: None }),
        )
        .await;
        assert_eq!(theirs.err().map(|e| e.0), Some(StatusCode::NOT_FOUND));
        assert_eq!(delete(State(s.clone()), user("other"), Path(mine.0.id)).await.err().map(|e| e.0), Some(StatusCode::NOT_FOUND));
        assert_eq!(ok(get(State(s), user("u")).await).0.choices.len(), 1);
    }
}
