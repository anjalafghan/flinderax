//! `POST /api/plan/project`: the baseline projection plus an optional what-if scenario.
//! The web page calls this on every edit, so it stays a pure computation over the user's data.

use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use time::{macros::format_description, Date};

use crate::{
    validate,
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
    for it in &p.items {
        check_item(it).map_err(bad_request)?;
    }
    let input = load_projection_input(&state.db, &user_id, today_local(), months, spend_pct).await?;
    Ok(Json(project(&input, &p.items)))
}

/// Bounds on one what-if item: money in paise, tenure in months, rate in basis points (up to 60%).
/// The planner also clamps these, but a bad value is better reported than silently changed.
fn check_item(it: &ScenarioItem) -> Result<(), String> {
    validate::positive_amount(it.amount_paise).map_err(|e| format!("\"{}\": {e}", it.name))?;
    if let Some(t) = it.tenure_months {
        validate::months(t).map_err(|e| format!("\"{}\" tenure: {e}", it.name))?;
    }
    if it.rate_bps.is_some_and(|r| !(0..=6_000).contains(&r)) {
        return Err(format!("\"{}\": the rate must be between 0% and 60%", it.name));
    }
    if let Some(t) = it.target_paise {
        validate::positive_amount(t).map_err(|e| format!("\"{}\" target: {e}", it.name))?;
    }
    Ok(())
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
    validate::positive_amount(p.amount_paise).map_err(bad_request)?;
    if let afford::Pay::Emi { months, rate_bps, fee_paise } = p.pay {
        if !(1..=60).contains(&months) || !(0..=6_000).contains(&rate_bps) {
            return Err(bad_request("EMI needs 1-60 months and a rate up to 60%"));
        }
        validate::non_negative_amount(fee_paise).map_err(bad_request)?;
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
        let baseline_short: std::collections::HashMap<String, i64> = project(&input, &[])
            .baseline
            .stress
            .iter()
            .map(|s| (format!("{}:{}", s.month, s.kind), s.amount_paise))
            .collect();
        project(&input, &[item])
            .scenario
            .into_iter()
            .flat_map(|s| s.stress)
            // Only the part of a shortfall the purchase causes: new ones in full, existing ones by
            // how much worse they get.
            .filter_map(|s| {
                let was = baseline_short.get(&format!("{}:{}", s.month, s.kind)).copied().unwrap_or(0);
                (s.amount_paise > was).then(|| s.amount_paise - was)
            })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::projection::ItemKind;
    use time::macros::date;

    fn item(tenure: Option<i64>) -> ScenarioItem {
        ScenarioItem {
            name: "Wedding RD".into(),
            kind: ItemKind::Rd,
            amount_paise: 1_000_000,
            start: date!(2026 - 11 - 10),
            tenure_months: tenure,
            rate_bps: Some(700),
            funded_from: None,
            payout_to: None,
            renewal: None,
            target_paise: None,
            target_date: None,
        }
    }

    #[test]
    fn a_normal_item_passes() {
        assert!(check_item(&item(Some(12))).is_ok());
        assert!(check_item(&item(None)).is_ok());
    }

    #[test]
    fn a_huge_tenure_is_rejected_with_a_message() {
        for t in [i64::MAX, i64::MIN, 1_201, -1] {
            let err = check_item(&item(Some(t))).unwrap_err();
            assert!(err.contains("tenure") && err.contains("Months must be between"), "{err}");
        }
    }

    #[test]
    fn bad_amounts_and_rates_are_rejected() {
        let mut it = item(Some(12));
        it.amount_paise = i64::MAX;
        assert!(check_item(&it).is_err());
        it.amount_paise = 0;
        assert!(check_item(&it).is_err());
        it.amount_paise = 1_000;
        it.rate_bps = Some(6_001);
        assert!(check_item(&it).is_err());
        it.rate_bps = None;
        it.target_paise = Some(i64::MAX);
        assert!(check_item(&it).is_err());
    }
}
