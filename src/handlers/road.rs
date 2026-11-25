//! `GET /api/plan/road`: the road ahead for the "At a glance" page. Gathers the life plan, the
//! savings goals and the year's one-off payments; `planner::road` puts them in order.

use axum::{extract::State, Extension, Json};

use crate::{
    handlers::{
        common::{today_local, AppError},
        goals, life, year,
    },
    models::AppState,
    planner::{
        life::plan as life_plan,
        road::{road, GoalIn, Inputs, Road},
    },
};

type User = Extension<(String, String)>;

fn parse_day(s: &str) -> Option<time::Date> {
    crate::planner::dates::parse_ymd_prefix(s)
}

pub async fn get(State(state): State<AppState>, Extension((user_id, name)): User) -> Result<Json<Road>, AppError> {
    let today = today_local();
    let db = &state.db;
    let (profile, _) = life::load_profile(db, &user_id).await?;
    let expenses = life::resolve_expenses(db, &user_id, today, &profile).await?;
    let plan = life_plan(&profile, expenses, today);

    let Json(goals_out) = goals::list(State(state.clone()), Extension((user_id.clone(), name))).await?;
    let goals: Vec<GoalIn> = goals_out
        .goals
        .into_iter()
        .map(|g| GoalIn {
            name: g.plan.name,
            target_paise: g.plan.target_paise,
            saved_paise: g.plan.saved_paise,
            target_date: g.target_date.as_deref().and_then(parse_day),
            needs_monthly_paise: g.plan.needs_monthly_paise,
        })
        .collect();
    let payments = year::lumpy_for(db, &user_id).await?;

    Ok(Json(road(&Inputs {
        today,
        life: &plan,
        wedding_dated: profile.wedding_date.is_some(),
        child_dated: profile.child_birth_year.is_some(),
        goals: &goals,
        payments: &payments,
    })))
}
