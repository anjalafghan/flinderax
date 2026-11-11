//! Loads what `planner::calibration` compares: the forecast kept with the last check-in before
//! today, the statements since then, and the saved amounts in the plan now.

use time::{macros::format_description, Date, Duration};

use crate::{
    handlers::{checkin, common::AppError, forecast, life},
    models::AppState,
    planner::{
        calibration::{self, Actual, Calibration, GoalSnap, Snapshot},
        checkin::{report, Period},
        life::{plan, LifeProfile},
    },
};

/// Days in an average month, for the money left each month.
const DAYS_PER_MONTH: f64 = 30.44;

/// The forecast as it stands today, to keep with a check-in.
pub async fn snapshot(state: &AppState, user_id: &str, today: Date) -> Result<Snapshot, AppError> {
    let f = forecast::compute(state, user_id, today).await?;
    Ok(Snapshot {
        made_on: today.to_string(),
        source: f.basis.source.to_string(),
        surplus_paise: f.basis.surplus_paise,
        applied_follow_through_bps: f.basis.follow_through_bps.unwrap_or(10_000),
        goals: f
            .goals
            .iter()
            .map(|g| GoalSnap {
                key: g.key.to_string(),
                label: g.label.to_string(),
                saved_paise: g.saved_paise,
                funded_monthly_paise: g.funded_monthly_paise,
                likelihood_bps: g.likelihood_bps,
            })
            .collect(),
    })
}

/// The forecast kept with the latest check-in done before `today`, set against what happened
/// since. `None` without such a check-in, or when it is too recent to say.
pub async fn latest(state: &AppState, user_id: &str, today: Date) -> Result<Option<Calibration>, AppError> {
    let db = &state.db;
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT done_on, forecast_json FROM checkins
         WHERE user_id = ? AND done_on < ? AND forecast_json IS NOT NULL
         ORDER BY done_on DESC, created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .bind(today.to_string())
    .fetch_optional(db)
    .await?;
    let Some((done_on, json)) = row else { return Ok(None) };
    let Ok(mut snap) = serde_json::from_str::<Snapshot>(&json) else { return Ok(None) };
    // The check-in's own date is what the person remembers.
    snap.made_on = done_on.clone();
    let Some(made_on) = day(&done_on) else { return Ok(None) };

    // Money left each month: from the day after the snapshot to the last day the statements reach.
    let txns = checkin::load_txns(db, user_id).await?;
    let start = made_on + Duration::days(1);
    let last = txns.iter().map(|t| t.on).filter(|d| *d >= start && *d <= today).max();
    let surplus_paise = last.and_then(|end| {
        let days = (end - start).whole_days() + 1;
        let months = days as f64 / DAYS_PER_MONTH;
        let period = Period { start, end };
        let (totals, ..) = report(&txns, period, Period { start: made_on, end: made_on });
        (months >= 0.9).then(|| ((totals.income_paise - totals.spend_paise) as f64 / months).round() as i64)
    });

    // Saved amounts now, from the plan.
    let saved: Option<String> = sqlx::query_scalar("SELECT profile_json FROM life_profiles WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await?;
    let profile: LifeProfile = saved.as_deref().and_then(|j| serde_json::from_str(j).ok()).unwrap_or_default();
    let expenses = life::resolve_expenses(db, user_id, today, &profile).await?;
    let saved = plan(&profile, expenses, today).goals.iter().map(|g| (g.key.to_string(), g.saved_paise)).collect();

    let actual = Actual { days: (today - made_on).whole_days(), surplus_paise, saved };
    Ok(calibration::compare(&snap, &actual))
}

/// "9 Nov 2026"
pub fn day_label(on: &str) -> String {
    day(on).and_then(|d| d.format(format_description!("[day padding:none] [month repr:short] [year]")).ok()).unwrap_or_else(|| on.to_string())
}

fn day(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

#[derive(serde::Serialize)]
pub struct CalibrationReset {
    pub status: bool,
}

/// `POST /api/checkin/calibration/reset`: forgets the forecast kept with check-ins, so the
/// forecast stops being tempered by how the last one turned out. The check-ins themselves stay.
pub async fn reset(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::Extension((user_id, _)): axum::Extension<(String, String)>,
) -> Result<axum::Json<CalibrationReset>, AppError> {
    sqlx::query("UPDATE checkins SET forecast_json = NULL WHERE user_id = ?").bind(&user_id).execute(&state.db).await?;
    Ok(axum::Json(CalibrationReset { status: true }))
}
