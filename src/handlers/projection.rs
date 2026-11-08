//! `POST /api/plan/project`: the baseline projection plus an optional what-if scenario.
//! The web page calls this on every edit, so it stays a pure computation over the user's data.

use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use time::{macros::format_description, Date};

use crate::{
    handlers::{
        common::{bad_request, today_local, AppError},
        dashboard::load_input,
    },
    models::AppState,
    planner::{
        emi_effective, month_index,
        projection::{project, DepositIn, EmiIn, Projection, ProjectionInput, Renewal, ScenarioItem, MAX_MONTHS},
        parse_year_month,
    },
};

#[derive(Deserialize)]
pub struct ProjectPayload {
    /// Months ahead, default 24.
    pub months: Option<i64>,
    /// Percent of the spending allowance you actually spend each month, default 100.
    pub spend_pct: Option<i64>,
    #[serde(default)]
    pub items: Vec<ScenarioItem>,
}

pub async fn project_handler(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Json(p): Json<ProjectPayload>,
) -> Result<Json<Projection>, AppError> {
    let months = p.months.unwrap_or(24);
    let spend_pct = p.spend_pct.unwrap_or(100);
    if !(1..=MAX_MONTHS).contains(&months) {
        return Err(bad_request(format!("months must be between 1 and {MAX_MONTHS}")));
    }
    if !(0..=300).contains(&spend_pct) {
        return Err(bad_request("spend_pct must be between 0 and 300"));
    }
    if p.items.len() > 10 {
        return Err(bad_request("at most 10 scenario items"));
    }
    let input = load_projection_input(&state.db, &user_id, today_local(), months, spend_pct).await?;
    Ok(Json(project(&input, &p.items)))
}

/// Everything the projection needs: the plan, FDs/RDs and card EMIs.
pub async fn load_projection_input(
    db: &sqlx::SqlitePool,
    user_id: &str,
    today: Date,
    months: i64,
    spend_pct: i64,
) -> Result<ProjectionInput, AppError> {
    let plan = load_input(db, user_id, today).await?;

    let deposits = sqlx::query!(
        r#"SELECT account_id AS "account_id!", kind, interest_bps, maturity_date, renewal, payout_account_id
           FROM accounts WHERE user_id = ? AND kind IN ('fd', 'rd')"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|a| DepositIn {
        account_id: a.account_id,
        is_rd: a.kind == "rd",
        rate_bps: a.interest_bps.unwrap_or(0),
        maturity: a
            .maturity_date
            .as_deref()
            .and_then(|d| Date::parse(d, format_description!("[year]-[month]-[day]")).ok()),
        renewal: Renewal::parse(&a.renewal),
        payout_to: a.payout_account_id,
    })
    .collect();

    let now_idx = month_index(today);
    let emis = sqlx::query!(
        r#"SELECT e.monthly_paise, e.remaining_principal_paise, e.months_left,
                  COALESCE(e.as_of_month, strftime('%Y-%m', e.created_at)) AS "as_of_month!: String"
           FROM card_emis e JOIN cards c ON c.card_id = e.card_id WHERE c.user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .filter_map(|e| {
        let (monthly, _, left) = emi_effective(
            e.monthly_paise,
            e.remaining_principal_paise,
            e.months_left,
            parse_year_month(&e.as_of_month).unwrap_or(now_idx),
            now_idx,
        );
        (left > 0).then_some(EmiIn { monthly, months_left: left })
    })
    .collect();

    Ok(ProjectionInput { plan, deposits, emis, months, spend_pct })
}

#[derive(Deserialize)]
pub struct AffordPayload {
    pub amount_paise: i64,
    pub pay: crate::planner::afford::Pay,
}

/// `POST /api/plan/afford`: can I buy this now or on EMI? Uses the same projection as What-if,
/// 12 months ahead, to see whether a lump on a given date leaves any month short.
pub async fn afford_handler(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Json(p): Json<AffordPayload>,
) -> Result<Json<crate::planner::afford::Verdict>, AppError> {
    use crate::planner::{afford, compute, format_inr, projection::ItemKind};
    if p.amount_paise <= 0 {
        return Err(bad_request("The amount must be more than zero"));
    }
    if let afford::Pay::Emi { months, rate_bps, fee_paise } = p.pay {
        if !(1..=60).contains(&months) || !(0..=6_000).contains(&rate_bps) || fee_paise < 0 {
            return Err(bad_request("EMI needs 1-60 months, a rate up to 60% and a fee of zero or more"));
        }
    }
    const HORIZON: i64 = 12;
    let today = today_local();
    let input = load_projection_input(&state.db, &user_id, today, HORIZON, 100).await?;
    let summary = compute(&input.plan);
    let short_if_spent_on = |date: Date| {
        let item = ScenarioItem {
            name: "Purchase".into(),
            kind: ItemKind::OneOff,
            amount_paise: p.amount_paise,
            start: date,
            tenure_months: None,
            rate_bps: None,
            funded_from: None,
            payout_to: None,
            renewal: None,
            target_paise: None,
            target_date: None,
        };
        let baseline_short: std::collections::HashSet<String> =
            project(&input, &[]).baseline.stress.iter().map(|s| format!("{}:{}", s.month, s.kind)).collect();
        project(&input, &[item])
            .scenario
            .into_iter()
            .flat_map(|s| s.stress)
            // Only shortfalls the purchase causes, not ones already in the plan.
            .filter(|s| !baseline_short.contains(&format!("{}:{}", s.month, s.kind)))
            .map(|s| s.amount_paise)
            .max()
    };
    Ok(Json(afford::afford(
        p.amount_paise,
        p.pay,
        today,
        summary.safe_to_spend_paise,
        summary.budget.rest_savings_paise,
        HORIZON,
        short_if_spent_on,
        format_inr,
    )))
}
