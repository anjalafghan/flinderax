//! Life plan API. `GET /api/plan/life` returns the saved profile (or defaults) with its plan.
//! `POST /api/plan/life/profile` validates a profile, saves it, and returns the plan.

use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::Serialize;
use sqlx::SqlitePool;
use time::Date;

use crate::{
    handlers::{
        autopost,
        common::{bad_request, today_local, AppError},
        dashboard::load_input,
    },
    models::AppState,
    planner::{
        self,
        life::{parse_date, plan as build_plan, LifePlan, LifeProfile},
    },
    validate,
};

type User = Extension<(String, String)>;

#[derive(Serialize)]
pub struct LifeOut {
    pub saved: bool,
    pub profile: LifeProfile,
    pub plan: LifePlan,
}

pub async fn get(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<LifeOut>, AppError> {
    let today = today_local();
    let saved: Option<String> = sqlx::query_scalar::<_, String>("SELECT profile_json FROM life_profiles WHERE user_id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await?;
    let profile = match &saved {
        Some(json) => serde_json::from_str(json).unwrap_or_default(),
        None => LifeProfile::default(),
    };
    Ok(Json(respond(&state.db, &user_id, today, profile, saved.is_some()).await?))
}

pub async fn save(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(mut profile): Json<LifeProfile>,
) -> Result<Json<LifeOut>, AppError> {
    let db = &state.db;
    let today = today_local();
    profile.wedding_date = profile.wedding_date.take().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    validate_profile(&profile, today)?;
    let json = serde_json::to_string(&profile)
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query(
        "INSERT INTO life_profiles (user_id, profile_json, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(user_id) DO UPDATE SET profile_json = excluded.profile_json, updated_at = excluded.updated_at",
    )
    .bind(&user_id)
    .bind(&json)
    .execute(db)
    .await?;
    Ok(Json(respond(db, &user_id, today, profile, true).await?))
}

/// The plan for a profile. Expenses come from the profile, else from the budget Safe to Spend uses.
async fn respond(db: &SqlitePool, user_id: &str, today: Date, profile: LifeProfile, saved: bool) -> Result<LifeOut, AppError> {
    let expenses = resolve_expenses(db, user_id, today, &profile).await?;
    let plan = build_plan(&profile, expenses, today);
    Ok(LifeOut { saved, profile, plan })
}

/// Monthly expenses: the profile's figure if given, else needs + spending from the plan's budget,
/// else half of the couple's combined take-home.
async fn resolve_expenses(db: &SqlitePool, user_id: &str, today: Date, profile: &LifeProfile) -> Result<i64, AppError> {
    if let Some(expenses) = profile.monthly_expenses_paise {
        return Ok(expenses);
    }
    autopost::apply(db, user_id, today).await?;
    let input = load_input(db, user_id, today).await?;
    let budget = planner::compute(&input).budget;
    let planned = budget.needs_paise.saturating_add(budget.spending_paise);
    if planned > 0 {
        return Ok(planned);
    }
    let take_home = profile.my_take_home_paise.max(0).saturating_add(profile.partner_take_home_paise.max(0));
    Ok(take_home / 2)
}

fn money(label: &str, v: i64, check: fn(i64) -> Result<i64, String>) -> Result<(), AppError> {
    check(v).map(|_| ()).map_err(|e| bad_request(format!("{label}: {e}")))
}

fn validate_profile(p: &LifeProfile, today: Date) -> Result<(), AppError> {
    fn age(label: &str, v: i64) -> Result<(), AppError> {
        if (18..=80).contains(&v) {
            Ok(())
        } else {
            Err(bad_request(format!("{label} must be between 18 and 80.")))
        }
    }
    age("Your age", p.my_age)?;
    if let Some(a) = p.partner_age {
        age("Partner's age", a)?;
    }
    if !(40..=75).contains(&p.retirement_age) {
        return Err(bad_request("Retirement age must be between 40 and 75."));
    }
    if p.retirement_age <= p.my_age {
        return Err(bad_request("Retirement age must be more than your age."));
    }
    money("Your take-home", p.my_take_home_paise, validate::non_negative_amount)?;
    money("Partner's take-home", p.partner_take_home_paise, validate::non_negative_amount)?;
    if let Some(e) = p.monthly_expenses_paise {
        money("Monthly expenses", e, validate::non_negative_amount)?;
    }
    if let Some(b) = p.wedding_budget_paise {
        money("Wedding budget", b, validate::positive_amount)?;
    }
    money("Wedding saved", p.wedding_saved_paise, validate::non_negative_amount)?;
    money("Child education cost", p.child_education_today_paise, validate::positive_amount)?;
    money("Child savings", p.child_saved_paise, validate::non_negative_amount)?;
    money("Retirement savings", p.retirement_saved_paise, validate::non_negative_amount)?;
    money("Emergency savings", p.emergency_saved_paise, validate::non_negative_amount)?;
    money("Term life cover", p.term_cover_paise, validate::non_negative_amount)?;
    money("Health cover", p.health_cover_paise, validate::non_negative_amount)?;
    money("Loans", p.loans_paise, validate::non_negative_amount)?;
    if let Some(s) = &p.wedding_date {
        let when = parse_date(s).ok_or_else(|| bad_request("Wedding date must be YYYY-MM-DD."))?;
        if when < today {
            return Err(bad_request("Wedding date can't be in the past."));
        }
    }
    if let Some(y) = p.child_birth_year {
        let (lo, hi) = (today.year() as i64 - 17, today.year() as i64 + 15);
        if !(lo..=hi).contains(&y) {
            return Err(bad_request(format!("Child's birth year must be between {lo} and {hi}.")));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')")
            .execute(&db)
            .await
            .unwrap();
        let key = rusty_paseto::prelude::PasetoSymmetricKey::from(rusty_paseto::prelude::Key::from(&[7u8; 32]));
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    fn user() -> User {
        Extension(("u".to_string(), "user".to_string()))
    }

    fn ok<T>(r: Result<Json<T>, AppError>) -> T {
        match r {
            Ok(j) => j.0,
            Err(e) => panic!("unexpected error {}: {}", e.0, e.1),
        }
    }

    #[tokio::test]
    async fn a_saved_profile_reads_back() {
        let st = state().await;
        let before = ok(get(State(st.clone()), user()).await);
        assert!(!before.saved);
        assert_eq!(before.profile, LifeProfile::default());

        let profile = LifeProfile {
            my_age: 31,
            partner_age: Some(29),
            my_take_home_paise: 15_000_000,
            partner_take_home_paise: 10_000_000,
            monthly_expenses_paise: Some(6_000_000),
            wedding_date: Some("2029-02-14".into()),
            child_birth_year: Some(2030),
            retirement_age: 58,
            ..Default::default()
        };
        let saved = ok(save(State(st.clone()), user(), Json(profile.clone())).await);
        assert!(saved.saved);
        assert_eq!(saved.plan.monthly.surplus_paise, 19_000_000);

        let read = ok(get(State(st.clone()), user()).await);
        assert!(read.saved);
        assert_eq!(read.profile, profile);
        let keys: Vec<&str> = read.plan.goals.iter().map(|g| g.key).collect();
        assert_eq!(keys, vec!["emergency", "wedding", "child_birth", "child_education", "retirement"]);
    }

    #[tokio::test]
    async fn a_second_save_replaces_the_first() {
        let st = state().await;
        let first = LifeProfile { my_age: 30, ..Default::default() };
        let second = LifeProfile { my_age: 35, retirement_age: 62, ..Default::default() };
        ok(save(State(st.clone()), user(), Json(first)).await);
        ok(save(State(st.clone()), user(), Json(second)).await);
        let read = ok(get(State(st.clone()), user()).await);
        assert_eq!(read.profile.my_age, 35);
        assert_eq!(read.profile.retirement_age, 62);
    }

    #[tokio::test]
    async fn an_invalid_profile_is_refused_with_400() {
        let st = state().await;
        let bad = LifeProfile { my_age: 40, retirement_age: 40, ..Default::default() };
        match save(State(st.clone()), user(), Json(bad)).await {
            Err(AppError(code, _)) => assert_eq!(code, StatusCode::BAD_REQUEST),
            Ok(_) => panic!("retirement age equal to age should be refused"),
        }
        let past = LifeProfile { wedding_date: Some("2001-01-01".into()), ..Default::default() };
        assert!(matches!(save(State(st.clone()), user(), Json(past)).await, Err(AppError(StatusCode::BAD_REQUEST, _))));
    }
}
