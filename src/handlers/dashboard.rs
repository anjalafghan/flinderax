//! `GET /api/dashboard/summary`: loads everything for the user and runs the pure planner.

use axum::{extract::State, Extension, Json};
use time::{macros::format_description, Date};

use crate::{
    handlers::{
        card_cycle::load_breakdowns,
        common::{today_local, AppError},
        plan::load_settings,
    },
    models::AppState,
    planner::{self, parse_year_month, AccountIn, CardIn, CommitmentIn, IncomeIn, PlanInput, SettingsIn, Summary},
};

fn parse_db_date(ts: &str) -> Option<Date> {
    Date::parse(ts.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

pub async fn summary(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<Summary>, AppError> {
    let today = today_local();
    let db = &state.db;

    let accounts = sqlx::query!(
        r#"SELECT account_id AS "account_id!", name, balance_paise, locked AS "locked: bool",
                  updated_at AS "updated_at: String"
           FROM accounts WHERE user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|a| AccountIn {
        id: a.account_id,
        name: a.name,
        balance: a.balance_paise,
        locked: a.locked,
        updated: parse_db_date(&a.updated_at),
    })
    .collect();

    let incomes = sqlx::query!(
        "SELECT label, amount_paise, day_of_month, account_id FROM income_sources WHERE user_id = ?",
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|i| IncomeIn {
        label: i.label,
        amount: i.amount_paise,
        day: i.day_of_month as u8,
        account_id: i.account_id,
    })
    .collect();

    let commitments = sqlx::query!(
        r#"SELECT label, amount_paise, day_of_month, account_id, active AS "active: bool", start_month, end_month
           FROM commitments WHERE user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|c| CommitmentIn {
        label: c.label,
        amount: c.amount_paise,
        day: c.day_of_month as u8,
        account_id: c.account_id,
        active: c.active,
        start_month: c.start_month.as_deref().and_then(parse_year_month),
        end_month: c.end_month.as_deref().and_then(parse_year_month),
    })
    .collect();

    let cards = load_breakdowns(db, &user_id, None, today)
        .await?
        .into_iter()
        .map(|b| CardIn {
            name: b.card_name,
            due_now: b.due_now_paise,
            due_date: b.due_date,
            next_bill_est: b.next_bill_est_paise,
            next_due_date: b.next_due_date,
            last_updated: b.last_updated_date,
        })
        .collect();

    let s = load_settings(db, &user_id).await?;
    let input = PlanInput {
        today,
        accounts,
        incomes,
        commitments,
        cards,
        settings: SettingsIn { cushion: s.cushion_paise, next_period_living: s.next_period_living_paise },
    };
    Ok(Json(planner::compute(&input)))
}
