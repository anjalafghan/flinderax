//! A child's education plan, under `/api/plan/education`.
//!
//! * `GET /` returns the saved plan (or a starter for a plan not saved yet) with its summary: the
//!   cost of each stage then, the monthly top-up, a low/base/high range, and what the top-up does to
//!   the life line.
//! * `PUT /` saves the plan (validated; stages are kept in order of age) and returns the same.
//! * `DELETE /` forgets it.
//!
//! The rules are in `planner::education`. The numbers rest on guesses and say so.

use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::Date;

use crate::{
    handlers::{
        common::{bad_request, today_local, AppError},
        life::load_profile,
        lifeline::{basis, saved_choices},
    },
    models::AppState,
    planner::{
        education::{earmark, summarise, validate, Plan, Stage, Summary},
        lifeline::{compare, Choice, Consequences},
    },
};

type User = Extension<(String, String)>;

/// What the plan's top-up does to the life line, against paying the fees out of savings later.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Effect {
    pub consequences: Consequences,
    pub without_at_end_today_paise: i64,
    pub with_at_end_today_paise: i64,
    pub note: String,
}

#[derive(Serialize)]
pub struct EducationOut {
    /// False until a plan has been saved: `plan` is then a starter to edit.
    pub saved: bool,
    pub plan: Plan,
    pub summary: Summary,
    pub lifeline: Effect,
}

#[derive(Deserialize)]
pub struct PlanIn {
    pub child_birth_year: i64,
    pub stages: Vec<Stage>,
    pub inflation_bps: i64,
    pub return_bps: i64,
    #[serde(default)]
    pub corpus_paise: i64,
}

async fn load(db: &SqlitePool, user_id: &str) -> Result<Option<Plan>, AppError> {
    let row: Option<(i64, String, i64, i64, i64)> = sqlx::query_as(
        "SELECT child_birth_year, stages_json, inflation_bps, return_bps, corpus_paise FROM education_plan WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;
    // A row whose stages no longer parse is treated as no plan.
    Ok(row.and_then(|(child_birth_year, json, inflation_bps, return_bps, corpus_paise)| {
        let stages = serde_json::from_str(&json).ok()?;
        Some(Plan { child_birth_year, stages, inflation_bps, return_bps, corpus_paise })
    }))
}

async fn respond(db: &SqlitePool, user_id: &str, today: Date) -> Result<EducationOut, AppError> {
    let saved_plan = load(db, user_id).await?;
    let saved = saved_plan.is_some();
    let plan = match saved_plan {
        Some(p) => p,
        None => {
            let (profile, _) = load_profile(db, user_id).await?;
            let ty = today.year() as i64;
            Plan::starter(profile.child_birth_year.unwrap_or(ty).clamp(ty - 30, ty + 25))
        }
    };
    let summary = summarise(&plan, today);
    let lifeline = effect(db, user_id, today, &plan).await?;
    Ok(EducationOut { saved, plan, summary, lifeline })
}

/// Two life lines that differ only in how the first child's education is paid: from savings when
/// it falls due (the default estimate), or from a pot the plan's top-up fills.
async fn effect(db: &SqlitePool, user_id: &str, today: Date, plan: &Plan) -> Result<Effect, AppError> {
    let basis = basis(db, user_id, today).await?;
    let mut choices: Vec<Choice> =
        saved_choices(db, user_id).await?.into_iter().filter(|c| c.in_plan).map(|c| c.choice).collect();
    // The plan's child is the line's first child in both lines.
    choices.insert(0, Choice::Child { birth_year: plan.child_birth_year, another: false });
    let without = basis.line(choices.clone(), today);
    let with = basis.line_with_earmark(choices, today, earmark(plan, today));
    Ok(Effect {
        consequences: compare(&without, &with),
        without_at_end_today_paise: without.marks.at_end_today_paise,
        with_at_end_today_paise: with.marks.at_end_today_paise,
        note: "Compared with paying the fees out of savings when they fall due, using the default fee estimate. The plan's own fee figures and top-up replace that estimate. Return and inflation are guesses.".to_string(),
    })
}

pub async fn get(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<EducationOut>, AppError> {
    Ok(Json(respond(&state.db, &user_id, today_local()).await?))
}

pub async fn put(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(body): Json<PlanIn>,
) -> Result<Json<EducationOut>, AppError> {
    let today = today_local();
    let mut stages = body.stages;
    for s in &mut stages {
        s.name = s.name.trim().to_string();
    }
    stages.sort_by_key(|s| s.start_age);
    let plan = Plan {
        child_birth_year: body.child_birth_year,
        stages,
        inflation_bps: body.inflation_bps,
        return_bps: body.return_bps,
        corpus_paise: body.corpus_paise,
    };
    validate(&plan, today).map_err(bad_request)?;
    let json = serde_json::to_string(&plan.stages).map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query(
        "INSERT INTO education_plan (user_id, child_birth_year, stages_json, inflation_bps, return_bps, corpus_paise)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET child_birth_year = excluded.child_birth_year, stages_json = excluded.stages_json,
             inflation_bps = excluded.inflation_bps, return_bps = excluded.return_bps, corpus_paise = excluded.corpus_paise,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(&user_id)
    .bind(plan.child_birth_year)
    .bind(&json)
    .bind(plan.inflation_bps)
    .bind(plan.return_bps)
    .bind(plan.corpus_paise)
    .execute(&state.db)
    .await?;
    Ok(Json(respond(&state.db, &user_id, today).await?))
}

pub async fn delete(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM education_plan WHERE user_id = ?").bind(&user_id).execute(&state.db).await?;
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

    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("{}: {}", e.0, e.1),
        }
    }

    fn body(year: i64) -> PlanIn {
        PlanIn {
            child_birth_year: year,
            stages: vec![
                Stage { name: " College ".into(), start_age: 18, today_cost_paise: 150_000_000 },
                Stage { name: "School".into(), start_age: 5, today_cost_paise: 30_000_000 },
            ],
            inflation_bps: 1_000,
            return_bps: 1_100,
            corpus_paise: 5_000_000,
        }
    }

    #[tokio::test]
    async fn before_a_plan_is_saved_a_starter_is_offered() {
        let s = state().await;
        let out = ok(get(State(s), user("u")).await).0;
        assert!(!out.saved);
        assert_eq!(out.plan.stages.len(), 2);
        assert!(out.summary.base.monthly_top_up_paise > 0);
        assert!(out.summary.notes.iter().any(|n| n.contains("not investment advice")));
    }

    #[tokio::test]
    async fn saving_keeps_stages_in_age_order_and_the_summary_follows() {
        let s = state().await;
        let year = today_local().year() as i64;
        let out = ok(put(State(s.clone()), user("u"), Json(body(year))).await).0;
        assert!(out.saved);
        assert_eq!(out.plan.stages.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["School", "College"]);
        assert_eq!(out.summary.stages.len(), 2);
        assert!(out.summary.low.monthly_top_up_paise < out.summary.base.monthly_top_up_paise);
        assert!(out.summary.base.monthly_top_up_paise < out.summary.high.monthly_top_up_paise);
        let again = ok(get(State(s.clone()), user("u")).await).0;
        assert_eq!(again.plan, out.plan);
        assert_eq!(again.summary, out.summary);
        // Saving again replaces: one row per person.
        let _ = ok(put(State(s.clone()), user("u"), Json(PlanIn { corpus_paise: 0, ..body(year) })).await);
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM education_plan").fetch_one(&s.db).await.unwrap();
        assert_eq!(n, 1);
        assert_eq!(ok(get(State(s), user("u")).await).0.plan.corpus_paise, 0);
    }

    #[tokio::test]
    async fn the_top_up_is_shown_against_the_life_line() {
        let s = state().await;
        sqlx::query("INSERT INTO life_profiles (user_id, profile_json) VALUES ('u', ?)")
            .bind(r#"{"my_age":29,"my_take_home_paise":15000000,"monthly_expenses_paise":7000000,"retirement_saved_paise":20000000}"#)
            .execute(&s.db)
            .await
            .unwrap();
        let year = today_local().year() as i64;
        let out = ok(put(State(s), user("u"), Json(body(year + 1))).await).0;
        assert!(!out.lifeline.consequences.lines.is_empty());
        assert!(out.lifeline.without_at_end_today_paise != 0 || out.lifeline.with_at_end_today_paise != 0);
    }

    #[tokio::test]
    async fn bad_plans_are_refused_and_save_nothing() {
        let s = state().await;
        let year = today_local().year() as i64;
        for bad in [
            PlanIn { stages: vec![], ..body(year) },
            PlanIn { child_birth_year: 1900, ..body(year) },
            PlanIn { inflation_bps: 5_000, ..body(year) },
            PlanIn { return_bps: -1, ..body(year) },
            PlanIn { corpus_paise: -5, ..body(year) },
        ] {
            let r = put(State(s.clone()), user("u"), Json(bad)).await;
            assert_eq!(r.err().map(|e| e.0), Some(StatusCode::BAD_REQUEST));
        }
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM education_plan").fetch_one(&s.db).await.unwrap();
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn a_plan_is_your_own_and_can_be_forgotten() {
        let s = state().await;
        let year = today_local().year() as i64;
        let _ = ok(put(State(s.clone()), user("u"), Json(body(year))).await);
        assert!(!ok(get(State(s.clone()), user("other")).await).0.saved);
        assert_eq!(ok(delete(State(s.clone()), user("other")).await), StatusCode::NO_CONTENT);
        assert!(ok(get(State(s.clone()), user("u")).await).0.saved, "someone else's delete touched nothing");
        assert_eq!(ok(delete(State(s.clone()), user("u")).await), StatusCode::NO_CONTENT);
        assert!(!ok(get(State(s), user("u")).await).0.saved);
    }
}
