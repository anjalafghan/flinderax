//! `/api/year`: the year ahead (twelve months of income, fixed payments and large one-off amounts)
//! and the list of those one-offs (premiums, tax, annual fees, maturities). Money is integer paise.

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    handlers::{
        common::{bad_request, not_found, today_local, AppError},
        couple::couple_of,
        dashboard::load_input,
    },
    models::AppState,
    planner::{
        everyday_spending,
        year_ahead::{next_occurrence, set_aside_monthly, year_view, Direction, Lumpy, YearView},
        CardIn,
    },
    validate,
};

type User = Extension<(String, String)>;

const KINDS: [&str; 6] = ["premium", "tax", "fee", "maturity", "wedding", "other"];

fn parse_date(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

#[derive(Serialize)]
pub struct OutflowOut {
    pub outflow_id: String,
    pub label: String,
    pub on_day: String,
    pub amount_paise: i64,
    pub direction: String,
    pub kind: String,
    pub yearly: bool,
    /// Next date it falls on (None once a one-off has passed).
    pub next_on: Option<String>,
    /// What to put aside each month to have it by then (money going out only).
    pub set_aside_monthly_paise: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct Row {
    outflow_id: String,
    label: String,
    on_day: String,
    amount_paise: i64,
    direction: String,
    kind: String,
    yearly: i64,
}

impl Row {
    fn lumpy(&self) -> Option<Lumpy> {
        Some(Lumpy {
            on: parse_date(&self.on_day)?,
            label: self.label.clone(),
            amount_paise: self.amount_paise,
            direction: if self.direction == "in" { Direction::In } else { Direction::Out },
            yearly: self.yearly == 1,
            kind: self.kind.clone(),
        })
    }
}

async fn rows(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<Row>, AppError> {
    Ok(sqlx::query_as::<_, Row>(
        "SELECT outflow_id, label, on_day, amount_paise, direction, kind, yearly FROM planned_outflows WHERE user_id = ? ORDER BY on_day, outflow_id",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?)
}

/// Every dated amount that should appear in the year view for this user. Other features (wedding
/// payments) add theirs here.
pub async fn lumpy_for(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<Lumpy>, AppError> {
    let mut out: Vec<Lumpy> = rows(db, user_id).await?.iter().filter_map(Row::lumpy).collect();
    out.extend(wedding_lumpy(db, user_id).await?);
    Ok(out)
}

/// Unpaid, dated wedding payments of the couple's goals, as one-off outflows.
async fn wedding_lumpy(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<Lumpy>, AppError> {
    let Some((couple, _)) = couple_of(db, user_id).await? else { return Ok(vec![]) };
    let rows: Vec<(String, i64, String, String)> = sqlx::query_as(
        "SELECT m.due_on, m.amount_paise, i.name, g.name FROM goal_milestones m \
         JOIN goal_items i ON i.item_id = m.item_id JOIN couple_goals g ON g.goal_id = i.goal_id \
         WHERE g.couple_id = ? AND g.archived = 0 AND i.archived = 0 AND m.paid_on IS NULL AND m.due_on IS NOT NULL",
    )
    .bind(&couple)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(due, amount, item, goal)| {
            Some(Lumpy { on: parse_date(&due)?, label: format!("{goal}: {item}"), amount_paise: amount, direction: Direction::Out, yearly: false, kind: "wedding".into() })
        })
        .collect())
}

/// Card bills already run up: the unpaid statement and what has been spent since, each on its due
/// date (today when the date is unknown or past). The cash to pay them is still in the opening
/// balance, so they have to come out of it.
fn card_bills(cards: &[CardIn], today: Date) -> Vec<Lumpy> {
    let mut out = Vec::new();
    for c in cards {
        for (amount, due, what) in [(c.due_now, c.due_date, "bill"), (c.next_bill_est, c.next_due_date, "next bill so far")] {
            if amount > 0 {
                let on = due.filter(|d| *d >= today).unwrap_or(today);
                out.push(Lumpy { on, label: format!("{}: {what}", c.name), amount_paise: amount, direction: Direction::Out, yearly: false, kind: "card".into() });
            }
        }
    }
    out
}

/// `GET /api/year`: the twelve-month view, built on the same inputs as Safe to Spend. Everyday
/// spending is the spending allowance, and card bills already owed come out on their due dates.
pub async fn view(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<YearView>, AppError> {
    let today = today_local();
    let input = load_input(&state.db, &user_id, today).await?;
    let opening: i64 = input.accounts.iter().filter(|a| !a.locked).map(|a| a.balance).sum();
    let mut lumpy = lumpy_for(&state.db, &user_id).await?;
    lumpy.extend(card_bills(&input.cards, today));
    Ok(Json(year_view(today, opening, &input.incomes, &input.commitments, &lumpy, everyday_spending(&input))))
}

/// `GET /api/year/outflows`
pub async fn list(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Vec<OutflowOut>>, AppError> {
    let today = today_local();
    let out = rows(&state.db, &user_id)
        .await?
        .into_iter()
        .map(|r| {
            let l = r.lumpy();
            let next = l.as_ref().and_then(|l| next_occurrence(l, today));
            let set_aside = (r.direction == "out").then(|| next.map(|n| set_aside_monthly(r.amount_paise, today, n))).flatten();
            OutflowOut {
                next_on: next.map(|d| d.to_string()),
                set_aside_monthly_paise: set_aside,
                outflow_id: r.outflow_id,
                label: r.label,
                on_day: r.on_day,
                amount_paise: r.amount_paise,
                direction: r.direction,
                kind: r.kind,
                yearly: r.yearly == 1,
            }
        })
        .collect();
    Ok(Json(out))
}

#[derive(Deserialize)]
pub struct NewOutflow {
    pub label: String,
    pub on_day: String,
    pub amount_paise: i64,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub yearly: bool,
}

#[derive(Serialize)]
pub struct Created {
    pub outflow_id: String,
    pub status: bool,
}

/// `POST /api/year/outflows`
pub async fn create(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<NewOutflow>) -> Result<Json<Created>, AppError> {
    let label = p.label.trim();
    if label.is_empty() || label.chars().count() > 80 {
        return Err(bad_request("Give it a name of up to 80 characters"));
    }
    validate::positive_amount(p.amount_paise).map_err(|e| bad_request(format!("The amount: {e}")))?;
    validate::bounded_amount(p.amount_paise).map_err(|e| bad_request(format!("The amount: {e}")))?;
    if parse_date(&p.on_day).is_none() {
        return Err(bad_request("on_day must look like 2027-03-10"));
    }
    let direction = p.direction.as_deref().unwrap_or("out");
    if !matches!(direction, "out" | "in") {
        return Err(bad_request("direction must be out or in"));
    }
    let kind = p.kind.as_deref().unwrap_or("other");
    if !KINDS.contains(&kind) {
        return Err(bad_request(format!("kind must be one of {}", KINDS.join(", "))));
    }
    let outflow_id = nanoid!();
    sqlx::query(
        "INSERT INTO planned_outflows (outflow_id, user_id, label, on_day, amount_paise, direction, kind, yearly) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&outflow_id)
    .bind(&user_id)
    .bind(label)
    .bind(&p.on_day[..10])
    .bind(p.amount_paise)
    .bind(direction)
    .bind(kind)
    .bind(i64::from(p.yearly))
    .execute(&state.db)
    .await?;
    Ok(Json(Created { outflow_id, status: true }))
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

/// `POST /api/year/outflows/{id}/delete`
pub async fn delete(State(state): State<AppState>, Extension((user_id, _)): User, Path(id): Path<String>) -> Result<Json<Done>, AppError> {
    let done = sqlx::query("DELETE FROM planned_outflows WHERE outflow_id = ? AND user_id = ?").bind(&id).bind(&user_id).execute(&state.db).await?;
    if done.rows_affected() == 0 {
        return Err(not_found("Not found"));
    }
    Ok(Json(Done { status: true }))
}
