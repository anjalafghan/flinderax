//! Month close (`GET /api/review/month`) and the India tax tracker (`/api/review/tax...`).

use std::collections::HashSet;

use axum::{
    extract::{Query, State},
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    alerts::is_spending,
    handlers::{
        autopost,
        common::{bad_request, not_found, today_local, AppError},
        dashboard::load_input,
        joy::JoyPayload,
    },
    models::AppState,
    planner::{
        self,
        history::{monthly_net_worth, Point},
        joy::your_share,
        review::{self, DatesFacts, MonthFacts},
        tax::{self, Claims, Payment},
    },
};

type User = Extension<(String, String)>;

fn parse_db_date(ts: &str) -> Option<Date> {
    Date::parse(ts.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

// ---------- month close ----------

#[derive(Deserialize)]
pub struct MonthQuery {
    pub month: Option<String>,
}

#[derive(Serialize)]
pub struct MonthOut {
    pub month: String,
    pub income_paise: i64,
    pub spent_paise: i64,
    pub allowance_paise: i64,
    /// How far spending went past the allowance (0 when within it).
    pub over_by_paise: i64,
    pub top_categories: Vec<review::CategoryAmount>,
    pub dates: Option<DatesFacts>,
    /// Basis points of income; `null` without income.
    pub savings_rate_bps: Option<i64>,
    pub net_worth_change_paise: Option<i64>,
    pub sentences: Vec<String>,
}

/// Net worth at the end of `month` minus the month before; `None` when either is unknown.
async fn net_worth_change(db: &sqlx::SqlitePool, user_id: &str, today: Date, month: &str, prev: &str) -> Result<Option<i64>, AppError> {
    let target = planner::parse_year_month(month).unwrap_or(0);
    let back = (planner::month_index(today) - target + 2).clamp(2, 60) as u32;

    let accounts = sqlx::query!(
        r#"SELECT account_id AS "account_id!", locked AS "locked: bool" FROM accounts WHERE user_id = ?"#,
        user_id
    )
    .fetch_all(db)
    .await?;
    let locked: HashSet<String> = accounts.into_iter().filter(|a| a.locked).map(|a| a.account_id).collect();
    let balances: Vec<Point> = sqlx::query!(
        r#"SELECT h.account_id AS "account_id!", h.balance_paise, h.recorded_at AS "recorded_at: String"
           FROM balance_history h JOIN accounts a ON a.account_id = h.account_id
           WHERE a.user_id = ? ORDER BY h.recorded_at, h.history_id"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .filter_map(|r| Some(Point { at: parse_db_date(&r.recorded_at)?, id: r.account_id, value: r.balance_paise }))
    .collect();
    let debts: Vec<Point> = sqlx::query!(
        r#"SELECT s.card_id AS "card_id!", s.outstanding_paise, s.created_at AS "created_at: String"
           FROM card_snapshots s JOIN cards c ON c.card_id = s.card_id
           WHERE c.user_id = ? ORDER BY s.created_at, s.rowid"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .filter_map(|r| Some(Point { at: parse_db_date(&r.created_at)?, id: r.card_id, value: r.outstanding_paise }))
    .collect();

    let series = monthly_net_worth(&balances, |id| locked.contains(id), &debts, today, back);
    let at = |m: &str| series.iter().find(|p| p.month == m).map(|p| p.net_worth_paise);
    Ok(at(month).zip(at(prev)).map(|(a, b)| a - b))
}

/// Date spending for `month`: from the Dates tables when you're in a couple, else from the last
/// payload a linked Joy app sent.
async fn dates_facts(db: &sqlx::SqlitePool, user_id: &str, month: &str) -> Result<Option<DatesFacts>, AppError> {
    if let Some((couple, who)) = crate::handlers::couple::couple_of(db, user_id).await? {
        let payload = crate::handlers::joy::local_payload(db, &couple).await?;
        let partner = crate::handlers::joy::partner_covers(db, &couple, who, &payload).await?;
        let spent = crate::joy::db::spent_in_month(db, &couple, month).await? * 100;
        let standing = payload.monthly_budget;
        let row = crate::joy::db::month_budget(db, &couple, month).await?;
        let budget = row.map(|(base, carry)| base + carry).or(standing).map(|b| b * 100);
        if spent == 0 && budget.is_none() {
            return Ok(None);
        }
        return Ok(Some(DatesFacts { spent_paise: spent, budget_paise: budget, your_share_paise: your_share(spent, budget, partner) }));
    }
    let Some(link) = sqlx::query!("SELECT partner_paise, last_payload FROM joy_links WHERE user_id = ?", user_id)
        .fetch_optional(db)
        .await?
    else {
        return Ok(None);
    };
    let Some(payload) = link.last_payload.as_deref().and_then(|j| serde_json::from_str::<JoyPayload>(j).ok()) else {
        return Ok(None);
    };
    Ok(payload.months.iter().find(|m| m.month == month).map(|m| {
        let budget = m.budget.map(|b| (b + m.carry) * 100);
        let spent = m.spent * 100;
        DatesFacts { spent_paise: spent, budget_paise: budget, your_share_paise: your_share(spent, budget, link.partner_paise) }
    }))
}

/// `GET /api/review/month?month=2026-10` (default: last month).
pub async fn month(State(state): State<AppState>, Extension((user_id, _)): User, Query(q): Query<MonthQuery>) -> Result<Json<MonthOut>, AppError> {
    let db = &state.db;
    let today = today_local();
    let key = q.month.unwrap_or_else(|| review::last_month(today));
    let (year, mon) = review::parse_month(&key).ok_or_else(|| bad_request("month must look like 2026-10"))?;
    if review::is_future(year, mon, today) {
        return Err(bad_request("That month hasn't happened yet"));
    }
    let (from, to) = review::month_range(year, mon).ok_or_else(|| bad_request("month must look like 2026-10"))?;
    let (from_s, to_s) = (from.to_string(), to.to_string());

    let rows = sqlx::query!(
        r#"SELECT direction, category, SUM(amount_paise) AS "total!: i64", COUNT(*) AS "n!: i64"
           FROM transactions WHERE user_id = ? AND occurred_on >= ? AND occurred_on < ?
           GROUP BY direction, category"#,
        user_id,
        from_s,
        to_s
    )
    .fetch_all(db)
    .await?;
    let has_transactions = rows.iter().any(|r| r.n > 0);
    let income: i64 = rows.iter().filter(|r| r.direction == "credit" && r.category == "income").map(|r| r.total).sum();
    let by_category: Vec<(String, i64)> = rows
        .iter()
        .filter(|r| r.direction == "debit" && is_spending(&r.category))
        .map(|r| (r.category.clone(), r.total))
        .collect();
    let spent: i64 = by_category.iter().map(|(_, a)| a).sum();

    autopost::apply(db, &user_id, today).await?;
    let plan = planner::compute(&load_input(db, &user_id, today).await?);
    let (allowance, needs) = (plan.budget.spending_paise, plan.budget.needs_paise);

    let (py, pm) = planner::cycle::add_months(year, mon, -1);
    let facts = MonthFacts {
        has_transactions,
        income_paise: income,
        spent_paise: spent,
        allowance_paise: allowance,
        top_categories: review::top_categories(&by_category, 3),
        savings_rate_bps: review::savings_rate_bps(income, spent, needs),
        dates: dates_facts(db, &user_id, &key).await?,
        net_worth_change_paise: net_worth_change(db, &user_id, today, &key, &review::month_key(py, pm)).await?,
    };
    Ok(Json(MonthOut {
        month: key,
        income_paise: facts.income_paise,
        spent_paise: facts.spent_paise,
        allowance_paise: allowance,
        over_by_paise: (spent - allowance).max(0),
        sentences: review::sentences(&facts),
        top_categories: facts.top_categories,
        dates: facts.dates,
        savings_rate_bps: facts.savings_rate_bps,
        net_worth_change_paise: facts.net_worth_change_paise,
    }))
}

// ---------- tax ----------

#[derive(Deserialize)]
pub struct TaxQuery {
    pub fy: Option<String>,
}

#[derive(Serialize)]
pub struct SectionOut {
    pub section: &'static str,
    /// Manual entries plus tagged payments made so far this FY.
    pub claimed_paise: i64,
    /// Tagged payments still to come before March.
    pub projected_paise: i64,
    pub limit_paise: Option<i64>,
    /// `limit - claimed`, never below zero.
    pub left_paise: Option<i64>,
    pub note: Option<&'static str>,
}

#[derive(Serialize)]
pub struct EntryOut {
    pub entry_id: String,
    pub section: String,
    pub label: String,
    pub amount_paise: i64,
}

#[derive(Serialize)]
pub struct TaggedOut {
    pub commitment_id: String,
    pub label: String,
    pub tax_section: Option<String>,
    pub paid_paise: i64,
    pub projected_paise: i64,
}

#[derive(Serialize)]
pub struct TaxOut {
    pub fy: String,
    pub from: Date,
    pub to: Date,
    pub sections: Vec<SectionOut>,
    pub entries: Vec<EntryOut>,
    /// Every active fixed payment, so one can be tagged.
    pub commitments: Vec<TaggedOut>,
    pub interest_income_paise: i64,
    pub gross_salary_paise: i64,
    pub regime: tax::Regime,
    pub reminders: Vec<tax::Reminder>,
    /// No realised-gains input yet, so the full exemption shows as left.
    pub ltcg: tax::LtcgHarvest,
}

fn fy_start(raw: Option<&str>, today: Date) -> Result<i32, AppError> {
    match raw {
        None => Ok(tax::fy_start_year(today)),
        Some(s) => tax::parse_fy(s).ok_or_else(|| bad_request("fy must look like 2026-27")),
    }
}

/// `GET /api/review/tax?fy=2026-27` (default: the current FY).
pub async fn tax(State(state): State<AppState>, Extension((user_id, _)): User, Query(q): Query<TaxQuery>) -> Result<Json<TaxOut>, AppError> {
    let db = &state.db;
    let today = today_local();
    let start = fy_start(q.fy.as_deref(), today)?;
    let fy = tax::fy_label(start);
    let (from, to) = tax::fy_bounds(start);

    let entries: Vec<EntryOut> = sqlx::query!(
        r#"SELECT entry_id AS "entry_id!", section, label, amount_paise FROM tax_entries
           WHERE user_id = ? AND fy = ? ORDER BY created_at, rowid"#,
        user_id,
        fy
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|e| EntryOut { entry_id: e.entry_id, section: e.section, label: e.label, amount_paise: e.amount_paise })
    .collect();

    let commitments: Vec<TaggedOut> = sqlx::query!(
        r#"SELECT commitment_id AS "commitment_id!", label, amount_paise, day_of_month, frequency, due_month,
                  start_month, end_month, tax_section
           FROM commitments WHERE user_id = ? AND active = 1 ORDER BY rowid"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|c| {
        let p = Payment {
            amount: c.amount_paise,
            day: c.day_of_month as u8,
            yearly: c.frequency == "yearly",
            due_month: c.due_month.map(|m| m as u8),
            active: true,
            start_month: c.start_month.as_deref().and_then(planner::parse_year_month),
            end_month: c.end_month.as_deref().and_then(planner::parse_year_month),
        };
        let (paid, projected) = tax::payments_in_fy(&p, start, today);
        TaggedOut { commitment_id: c.commitment_id, label: c.label, tax_section: c.tax_section, paid_paise: paid, projected_paise: projected }
    })
    .collect();

    let mut sections = Vec::new();
    let (mut claims, mut full_year) = (Claims::default(), Claims::default());
    for &section in tax::SECTIONS.iter() {
        let manual: i64 = entries.iter().filter(|e| e.section == section).map(|e| e.amount_paise).sum();
        let tagged = commitments.iter().filter(|c| c.tax_section.as_deref() == Some(section));
        let (paid, projected) = tagged.fold((0, 0), |(a, b), c| (a + c.paid_paise, b + c.projected_paise));
        let claimed = manual + paid;
        let limit = tax::limit_for(section);
        claims.set(section, claimed);
        full_year.set(section, claimed + projected);
        sections.push(SectionOut {
            section,
            claimed_paise: claimed,
            projected_paise: projected,
            limit_paise: limit,
            left_paise: limit.map(|l| (l - claimed).max(0)),
            note: tax::section_note(section),
        });
    }

    let (from_s, to_s) = (from.to_string(), to.to_string());
    let interest_income = sqlx::query_scalar!(
        r#"SELECT COALESCE(SUM(amount_paise), 0) AS "total!: i64" FROM transactions
           WHERE user_id = ? AND direction = 'credit' AND occurred_on >= ? AND occurred_on <= ?
             AND (lower(COALESCE(merchant, '') || ' ' || COALESCE(raw_text, '')) LIKE '%interest%'
               OR lower(COALESCE(merchant, '') || ' ' || COALESCE(raw_text, '')) LIKE '%int.pd%'
               OR lower(COALESCE(merchant, '') || ' ' || COALESCE(raw_text, '')) LIKE '%int pd%'
               OR lower(COALESCE(merchant, '') || ' ' || COALESCE(raw_text, '')) LIKE '%int cr%')"#,
        user_id,
        from_s,
        to_s
    )
    .fetch_one(db)
    .await?;

    let monthly_income = sqlx::query_scalar!(r#"SELECT COALESCE(SUM(amount_paise), 0) AS "t!: i64" FROM income_sources WHERE user_id = ?"#, user_id)
        .fetch_one(db)
        .await?;
    let gross = monthly_income * 12;

    Ok(Json(TaxOut {
        fy,
        from,
        to,
        sections,
        entries,
        commitments,
        interest_income_paise: interest_income,
        gross_salary_paise: gross,
        // A whole-year estimate, so the payments still to come count too.
        regime: tax::compare(gross, full_year, start),
        reminders: tax::reminders(start, today),
        ltcg: tax::ltcg_harvest(0, start, today),
    }))
}

#[derive(Deserialize)]
pub struct EntryPayload {
    pub section: String,
    pub label: String,
    pub amount_paise: i64,
    pub fy: Option<String>,
}

#[derive(Serialize)]
pub struct Created {
    pub entry_id: String,
}

fn check_section(s: &str) -> Result<(), AppError> {
    if tax::valid_section(s) {
        Ok(())
    } else {
        Err(bad_request("section must be one of 80C, 80CCD1B, 80D, HRA, 24B, other"))
    }
}

/// `POST /api/review/tax/entry`
pub async fn add_entry(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<EntryPayload>) -> Result<Json<Created>, AppError> {
    check_section(&p.section)?;
    let label = p.label.trim();
    if label.is_empty() || label.len() > 100 {
        return Err(bad_request("Give the entry a short label"));
    }
    if p.amount_paise < 0 {
        return Err(bad_request("The amount can't be negative"));
    }
    let fy = tax::fy_label(fy_start(p.fy.as_deref(), today_local())?);
    let entry_id = nanoid!();
    sqlx::query!(
        "INSERT INTO tax_entries (entry_id, user_id, fy, section, label, amount_paise) VALUES (?, ?, ?, ?, ?, ?)",
        entry_id,
        user_id,
        fy,
        p.section,
        label,
        p.amount_paise
    )
    .execute(&state.db)
    .await?;
    Ok(Json(Created { entry_id }))
}

#[derive(Deserialize)]
pub struct DeletePayload {
    pub entry_id: String,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

/// `POST /api/review/tax/entry/delete`
pub async fn delete_entry(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<DeletePayload>) -> Result<Json<Done>, AppError> {
    let n = sqlx::query!("DELETE FROM tax_entries WHERE entry_id = ? AND user_id = ?", p.entry_id, user_id)
        .execute(&state.db)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(not_found("No such entry"));
    }
    Ok(Json(Done { status: true }))
}

#[derive(Deserialize)]
pub struct TagPayload {
    pub commitment_id: String,
    pub tax_section: Option<String>,
}

/// `POST /api/review/tax/commitment`: tag a fixed payment (an ELSS SIP as 80C), or `null` to untag.
pub async fn tag_commitment(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<TagPayload>) -> Result<Json<Done>, AppError> {
    if let Some(s) = &p.tax_section {
        check_section(s)?;
    }
    let n = sqlx::query!(
        "UPDATE commitments SET tax_section = ? WHERE commitment_id = ? AND user_id = ?",
        p.tax_section,
        p.commitment_id,
        user_id
    )
    .execute(&state.db)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(not_found("No such fixed payment"));
    }
    Ok(Json(Done { status: true }))
}
