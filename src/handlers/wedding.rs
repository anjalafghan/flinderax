//! `/api/couple-goals/{id}/...`: the budget under a couple goal (a wedding): items with an estimate
//! and maybe a quote, dated payment milestones, one verdict. Either partner can edit; paying a
//! milestone also records a contribution to the goal, in the payer's name. Rules live in
//! `planner::wedding`; this file is HTTP and SQL.

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use time::{Date, Duration};

use crate::{
    handlers::{
        common::{bad_request, not_found, today_local, AppError},
        couple::{couple_of, require_couple},
        couple_goals::{mine, parse_date, record_contribution, Done},
    },
    models::AppState,
    planner::{
        coming::{Item as ComingItem, Kind},
        life::inr,
        wedding::{budget_summary, expected_monthly, next_due, on_hand, verdict, Item, Milestone, Summary, Verdict, CATEGORIES},
    },
    validate,
};

type User = Extension<(String, String)>;

// ---------- reading the budget ----------

#[derive(Serialize)]
pub struct MilestoneOut {
    pub milestone_id: String,
    pub item_id: String,
    pub due_on: Option<String>,
    pub amount_paise: i64,
    /// For filling a form field: "1200" or "1200.50".
    pub amount_input: String,
    pub paid: bool,
    pub paid_on: Option<String>,
    /// Unpaid and the date has passed.
    pub overdue: bool,
}

#[derive(Serialize)]
pub struct ItemOut {
    pub item_id: String,
    pub name: String,
    pub category: String,
    pub estimate_paise: i64,
    pub estimate_input: String,
    pub quote_paise: Option<i64>,
    pub quote_input: String,
    pub vendor: String,
    pub notes: String,
    /// The quote when there is one, else the estimate.
    pub planned_paise: i64,
    pub paid_paise: i64,
    pub remaining_paise: i64,
    pub milestones: Vec<MilestoneOut>,
}

#[derive(Serialize)]
pub struct NextDue {
    pub milestone_id: String,
    pub item_name: String,
    pub due_on: String,
    pub amount_paise: i64,
    pub days_away: i64,
}

#[derive(Serialize)]
pub struct BudgetOut {
    pub goal_id: String,
    pub name: String,
    pub target_paise: i64,
    pub target_date: Option<String>,
    pub saved_paise: i64,
    /// Saved and not yet paid out to anyone.
    pub on_hand_paise: i64,
    /// What the verdict assumes you put in each month (the last 90 days' pace).
    pub expected_monthly_paise: i64,
    pub categories: Vec<String>,
    pub items: Vec<ItemOut>,
    pub summary: Summary,
    pub verdict: Verdict,
    pub next_due: Option<NextDue>,
}

type ItemRow = (String, String, String, i64, Option<i64>, String, String);
type MilestoneRow = (String, String, Option<String>, i64, Option<String>);

fn day(s: &Option<String>) -> Option<Date> {
    s.as_deref().and_then(parse_date)
}

/// `GET /api/couple-goals/{id}/budget`
pub async fn budget(State(state): State<AppState>, Extension((user_id, _)): User, Path(goal_id): Path<String>) -> Result<Json<BudgetOut>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    let today = today_local();
    let (name, target, target_date): (String, i64, Option<String>) =
        sqlx::query_as("SELECT name, target_paise, target_date FROM couple_goals WHERE goal_id = ?").bind(&goal_id).fetch_one(&state.db).await?;
    let item_rows: Vec<ItemRow> = sqlx::query_as(
        "SELECT item_id, name, category, estimate_paise, quote_paise, vendor, notes FROM goal_items WHERE goal_id = ? AND archived = 0 ORDER BY rowid",
    )
    .bind(&goal_id)
    .fetch_all(&state.db)
    .await?;
    let ms_rows: Vec<MilestoneRow> = sqlx::query_as(
        "SELECT m.milestone_id, m.item_id, m.due_on, m.amount_paise, m.paid_on FROM goal_milestones m \
         JOIN goal_items i ON i.item_id = m.item_id WHERE i.goal_id = ? AND i.archived = 0 \
         ORDER BY m.due_on IS NULL, m.due_on, m.rowid",
    )
    .bind(&goal_id)
    .fetch_all(&state.db)
    .await?;
    let pays: Vec<(String, i64)> = sqlx::query_as("SELECT on_day, amount_paise FROM couple_goal_contributions WHERE goal_id = ?").bind(&goal_id).fetch_all(&state.db).await?;
    let saved: i64 = pays.iter().map(|p| p.1).sum();
    let dated_pays: Vec<(Date, i64)> = pays.iter().filter_map(|(d, a)| parse_date(d).map(|d| (d, *a))).collect();

    let items: Vec<Item> =
        item_rows.iter().map(|r| Item { id: r.0.clone(), category: r.2.clone(), estimate_paise: r.3, quote_paise: r.4 }).collect();
    let milestones: Vec<Milestone> =
        ms_rows.iter().map(|r| Milestone { id: r.0.clone(), item_id: r.1.clone(), due_on: day(&r.2), amount_paise: r.3, paid: r.4.is_some() }).collect();

    let summary = budget_summary(&items, &milestones, today);
    let monthly = expected_monthly(&dated_pays, today);
    let hand = on_hand(saved, summary.paid_paise);
    let verdict = verdict(hand, &milestones, monthly, today);
    let next = next_due(&milestones, today).and_then(|m| {
        let item_name = item_rows.iter().find(|r| r.0 == m.item_id).map(|r| r.1.clone()).unwrap_or_default();
        m.due_on.map(|d| NextDue { milestone_id: m.id.clone(), item_name, due_on: d.to_string(), amount_paise: m.amount_paise, days_away: (d - today).whole_days() })
    });

    let items_out = item_rows
        .into_iter()
        .map(|(item_id, name, category, estimate, quote, vendor, notes)| {
            let planned = quote.unwrap_or(estimate);
            let its: Vec<MilestoneOut> = ms_rows
                .iter()
                .filter(|m| m.1 == item_id)
                .map(|m| MilestoneOut {
                    milestone_id: m.0.clone(),
                    item_id: m.1.clone(),
                    due_on: m.2.clone(),
                    amount_paise: m.3,
                    amount_input: validate::plain_rupees(m.3),
                    paid: m.4.is_some(),
                    paid_on: m.4.clone(),
                    overdue: m.4.is_none() && day(&m.2).is_some_and(|d| d < today),
                })
                .collect();
            let paid: i64 = its.iter().filter(|m| m.paid).map(|m| m.amount_paise).sum();
            ItemOut {
                estimate_input: validate::plain_rupees(estimate),
                quote_input: quote.map(validate::plain_rupees).unwrap_or_default(),
                planned_paise: planned,
                paid_paise: paid,
                remaining_paise: (planned - paid).max(0),
                milestones: its,
                item_id,
                name,
                category,
                estimate_paise: estimate,
                quote_paise: quote,
                vendor,
                notes,
            }
        })
        .collect();

    Ok(Json(BudgetOut {
        goal_id,
        name,
        target_paise: target,
        target_date,
        saved_paise: saved,
        on_hand_paise: hand,
        expected_monthly_paise: monthly,
        categories: CATEGORIES.iter().map(|c| (*c).to_string()).collect(),
        items: items_out,
        summary,
        verdict,
        next_due: next,
    }))
}

// ---------- items ----------

/// An amount as paise: `text` ("₹1,200") when given, else `paise`. Rupees typed into a form become
/// paise only through `validate::paise_from_text`.
fn money(paise: Option<i64>, text: Option<&str>) -> Result<Option<i64>, String> {
    match text.map(str::trim).filter(|t| !t.is_empty()) {
        Some(t) => validate::paise_from_text(t).map(Some),
        None => paise.map(validate::non_negative_amount).transpose(),
    }
}

#[derive(Deserialize)]
pub struct ItemIn {
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub estimate_paise: Option<i64>,
    #[serde(default)]
    pub estimate: Option<String>,
    #[serde(default)]
    pub quote_paise: Option<i64>,
    #[serde(default)]
    pub quote: Option<String>,
    #[serde(default)]
    pub vendor: String,
    #[serde(default)]
    pub notes: String,
}

struct CleanItem {
    name: String,
    category: String,
    estimate: i64,
    quote: Option<i64>,
    vendor: String,
    notes: String,
}

fn clean_item(p: ItemIn) -> Result<CleanItem, AppError> {
    let name = p.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(bad_request("Give the item a name of up to 80 characters"));
    }
    let category = p.category.as_deref().map(str::trim).filter(|c| !c.is_empty()).unwrap_or("misc").to_lowercase();
    if !CATEGORIES.contains(&category.as_str()) {
        return Err(bad_request(format!("The category must be one of: {}", CATEGORIES.join(", "))));
    }
    let estimate = money(p.estimate_paise, p.estimate.as_deref()).map_err(|e| bad_request(format!("The estimate: {e}")))?.unwrap_or(0);
    let quote = money(p.quote_paise, p.quote.as_deref()).map_err(|e| bad_request(format!("The quote: {e}")))?;
    let (vendor, notes) = (p.vendor.trim().to_string(), p.notes.trim().to_string());
    if vendor.chars().count() > 80 {
        return Err(bad_request("Keep the vendor under 80 characters"));
    }
    if notes.chars().count() > 300 {
        return Err(bad_request("Keep the notes under 300 characters"));
    }
    Ok(CleanItem { name, category, estimate, quote, vendor, notes })
}

#[derive(Serialize)]
pub struct ItemId {
    pub item_id: String,
    pub status: bool,
}

/// An item of this goal, or a 404.
async fn item_of(db: &sqlx::SqlitePool, goal_id: &str, item_id: &str) -> Result<(), AppError> {
    let found: Option<String> =
        sqlx::query_scalar("SELECT item_id FROM goal_items WHERE item_id = ? AND goal_id = ? AND archived = 0").bind(item_id).bind(goal_id).fetch_optional(db).await?;
    found.map(|_| ()).ok_or_else(|| not_found("Item not found"))
}

/// `POST /api/couple-goals/{id}/items`
pub async fn create_item(State(state): State<AppState>, Extension((user_id, _)): User, Path(goal_id): Path<String>, Json(p): Json<ItemIn>) -> Result<Json<ItemId>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    let c = clean_item(p)?;
    let item_id = nanoid!();
    sqlx::query("INSERT INTO goal_items (item_id, goal_id, name, category, estimate_paise, quote_paise, vendor, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&item_id)
        .bind(&goal_id)
        .bind(c.name)
        .bind(c.category)
        .bind(c.estimate)
        .bind(c.quote)
        .bind(c.vendor)
        .bind(c.notes)
        .execute(&state.db)
        .await?;
    Ok(Json(ItemId { item_id, status: true }))
}

/// `POST /api/couple-goals/{id}/items/{item_id}`: replaces the item's details (a missing quote clears it).
pub async fn update_item(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path((goal_id, item_id)): Path<(String, String)>,
    Json(p): Json<ItemIn>,
) -> Result<Json<Done>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    item_of(&state.db, &goal_id, &item_id).await?;
    let c = clean_item(p)?;
    sqlx::query("UPDATE goal_items SET name = ?, category = ?, estimate_paise = ?, quote_paise = ?, vendor = ?, notes = ? WHERE item_id = ? AND goal_id = ?")
        .bind(c.name)
        .bind(c.category)
        .bind(c.estimate)
        .bind(c.quote)
        .bind(c.vendor)
        .bind(c.notes)
        .bind(&item_id)
        .bind(&goal_id)
        .execute(&state.db)
        .await?;
    Ok(Json(Done { status: true }))
}

/// `POST /api/couple-goals/{id}/items/{item_id}/delete`: removes the item and its unpaid milestones.
/// Refused while a payment against it stands, so what was paid is never lost from the totals.
pub async fn delete_item(State(state): State<AppState>, Extension((user_id, _)): User, Path((goal_id, item_id)): Path<(String, String)>) -> Result<Json<Done>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    item_of(&state.db, &goal_id, &item_id).await?;
    let paid: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM goal_milestones WHERE item_id = ? AND paid_on IS NOT NULL").bind(&item_id).fetch_one(&state.db).await?;
    if paid > 0 {
        return Err(bad_request("Take back its payments first"));
    }
    sqlx::query("DELETE FROM goal_items WHERE item_id = ? AND goal_id = ?").bind(&item_id).bind(&goal_id).execute(&state.db).await?;
    Ok(Json(Done { status: true }))
}

// ---------- milestones ----------

#[derive(Deserialize)]
pub struct MilestoneIn {
    #[serde(default)]
    pub due_on: Option<String>,
    #[serde(default)]
    pub amount_paise: Option<i64>,
    #[serde(default)]
    pub amount: Option<String>,
}

fn clean_milestone(p: MilestoneIn) -> Result<(Option<String>, i64), AppError> {
    let due = p.due_on.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
    let due = match due {
        Some(d) => Some(parse_date(&d).ok_or_else(|| bad_request("due_on must look like 2027-04-01"))?.to_string()),
        None => None,
    };
    let amount = money(p.amount_paise, p.amount.as_deref()).map_err(|e| bad_request(format!("The amount: {e}")))?.ok_or_else(|| bad_request("The amount: give an amount"))?;
    validate::positive_amount(amount).map_err(|e| bad_request(format!("The amount: {e}")))?;
    Ok((due, amount))
}

#[derive(Serialize)]
pub struct MilestoneId {
    pub milestone_id: String,
    pub status: bool,
}

/// What a milestone is, with the item it is under, when it belongs to this goal.
struct Found {
    item_name: String,
    amount: i64,
    paid_on: Option<String>,
    paid_by: Option<String>,
    contribution_id: Option<i64>,
}

async fn milestone_of(db: &sqlx::SqlitePool, goal_id: &str, milestone_id: &str) -> Result<Found, AppError> {
    let row: Option<(String, i64, Option<String>, Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT i.name, m.amount_paise, m.paid_on, m.paid_by, m.contribution_id FROM goal_milestones m \
         JOIN goal_items i ON i.item_id = m.item_id WHERE m.milestone_id = ? AND i.goal_id = ? AND i.archived = 0",
    )
    .bind(milestone_id)
    .bind(goal_id)
    .fetch_optional(db)
    .await?;
    let (item_name, amount, paid_on, paid_by, contribution_id) = row.ok_or_else(|| not_found("Payment not found"))?;
    Ok(Found { item_name, amount, paid_on, paid_by, contribution_id })
}

/// `POST /api/couple-goals/{id}/items/{item_id}/milestones`
pub async fn create_milestone(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path((goal_id, item_id)): Path<(String, String)>,
    Json(p): Json<MilestoneIn>,
) -> Result<Json<MilestoneId>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    item_of(&state.db, &goal_id, &item_id).await?;
    let (due, amount) = clean_milestone(p)?;
    let milestone_id = nanoid!();
    sqlx::query("INSERT INTO goal_milestones (milestone_id, item_id, due_on, amount_paise) VALUES (?, ?, ?, ?)")
        .bind(&milestone_id)
        .bind(&item_id)
        .bind(due)
        .bind(amount)
        .execute(&state.db)
        .await?;
    Ok(Json(MilestoneId { milestone_id, status: true }))
}

/// `POST /api/couple-goals/{id}/milestones/{mid}`: change the date or amount of a payment not yet made.
pub async fn update_milestone(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path((goal_id, mid)): Path<(String, String)>,
    Json(p): Json<MilestoneIn>,
) -> Result<Json<Done>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    if milestone_of(&state.db, &goal_id, &mid).await?.paid_on.is_some() {
        return Err(bad_request("That payment is made. Take it back to change it"));
    }
    let (due, amount) = clean_milestone(p)?;
    sqlx::query("UPDATE goal_milestones SET due_on = ?, amount_paise = ? WHERE milestone_id = ? AND paid_on IS NULL").bind(due).bind(amount).bind(&mid).execute(&state.db).await?;
    Ok(Json(Done { status: true }))
}

/// `POST /api/couple-goals/{id}/milestones/{mid}/delete`: a payment not yet made.
pub async fn delete_milestone(State(state): State<AppState>, Extension((user_id, _)): User, Path((goal_id, mid)): Path<(String, String)>) -> Result<Json<Done>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    if milestone_of(&state.db, &goal_id, &mid).await?.paid_on.is_some() {
        return Err(bad_request("That payment is made. Take it back first"));
    }
    sqlx::query("DELETE FROM goal_milestones WHERE milestone_id = ? AND paid_on IS NULL").bind(&mid).execute(&state.db).await?;
    Ok(Json(Done { status: true }))
}

#[derive(Deserialize, Default)]
pub struct Pay {
    /// The day it was paid; today when left out.
    #[serde(default)]
    pub paid_on: Option<String>,
    /// Paying twice does nothing more (a milestone is paid once), but the key is accepted like every money POST.
    #[serde(default)]
    pub idempotency_key: Option<String>,
}

#[derive(Serialize)]
pub struct Paid {
    pub status: bool,
    /// True when it had already been paid and nothing new was recorded.
    pub already: bool,
}

/// `POST /api/couple-goals/{id}/milestones/{mid}/pay`: mark it paid, in your name, and record the
/// amount as a contribution to the goal so each partner's share still adds up.
pub async fn pay(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path((goal_id, mid)): Path<(String, String)>,
    body: Option<Json<Pay>>,
) -> Result<Json<Paid>, AppError> {
    let p = body.map(|Json(p)| p).unwrap_or_default();
    if p.idempotency_key.as_deref().is_some_and(|k| k.trim().len() > 100) {
        return Err(bad_request("idempotency_key is too long"));
    }
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    let today = today_local();
    let on = match p.paid_on.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        Some(d) => parse_date(d).ok_or_else(|| bad_request("paid_on must look like 2027-04-01"))?,
        None => today,
    };
    if on > today {
        return Err(bad_request("A payment can't be dated in the future"));
    }
    let m = milestone_of(&state.db, &goal_id, &mid).await?;

    let mut tx = state.db.begin().await?;
    let claimed = sqlx::query("UPDATE goal_milestones SET paid_on = ?, paid_by = ? WHERE milestone_id = ? AND paid_on IS NULL")
        .bind(on.to_string())
        .bind(&user_id)
        .bind(&mid)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if claimed == 0 {
        return Ok(Json(Paid { status: true, already: true }));
    }
    let note: String = m.item_name.chars().take(180).collect();
    let cid = record_contribution(&mut *tx, &goal_id, &user_id, m.amount, &on.to_string(), &note).await?;
    sqlx::query("UPDATE goal_milestones SET contribution_id = ? WHERE milestone_id = ?").bind(cid).bind(&mid).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(Paid { status: true, already: false }))
}

/// `POST /api/couple-goals/{id}/milestones/{mid}/unpay`: take back a payment you marked, and the
/// contribution it recorded.
pub async fn unpay(State(state): State<AppState>, Extension((user_id, _)): User, Path((goal_id, mid)): Path<(String, String)>) -> Result<Json<Done>, AppError> {
    let (couple, _) = require_couple(&state.db, &user_id).await?;
    mine(&state.db, &couple, &goal_id).await?;
    let m = milestone_of(&state.db, &goal_id, &mid).await?;
    if m.paid_on.is_none() || m.paid_by.as_deref() != Some(user_id.as_str()) {
        return Err(not_found("Payment not found"));
    }
    let mut tx = state.db.begin().await?;
    if let Some(cid) = m.contribution_id {
        sqlx::query("DELETE FROM couple_goal_contributions WHERE id = ? AND goal_id = ?").bind(cid).bind(&goal_id).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE goal_milestones SET paid_on = NULL, paid_by = NULL, contribution_id = NULL WHERE milestone_id = ?").bind(&mid).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(Done { status: true }))
}

// ---------- coming up ----------

/// The unpaid, dated payments of this person's couple that fall due within `days` days (every
/// overdue one included), as "coming up" items linking to the goal's budget page.
pub async fn milestone_items(db: &sqlx::SqlitePool, user_id: &str, today: Date, days: i64) -> Result<Vec<ComingItem>, AppError> {
    let Some((couple, _)) = couple_of(db, user_id).await? else { return Ok(vec![]) };
    let horizon = (today + Duration::days(days)).to_string();
    let rows: Vec<(String, i64, String, String, String)> = sqlx::query_as(
        "SELECT m.due_on, m.amount_paise, i.name, g.goal_id, g.name FROM goal_milestones m \
         JOIN goal_items i ON i.item_id = m.item_id JOIN couple_goals g ON g.goal_id = i.goal_id \
         WHERE g.couple_id = ? AND g.archived = 0 AND i.archived = 0 AND m.paid_on IS NULL AND m.due_on IS NOT NULL AND m.due_on <= ? \
         ORDER BY m.due_on, m.milestone_id",
    )
    .bind(&couple)
    .bind(horizon)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(due, amount, item, goal_id, goal)| {
            let on = parse_date(&due)?;
            Some(ComingItem { on, kind: Kind::Milestone, title: format!("{goal}: {item}"), detail: inr(amount), link: format!("/wedding/{goal_id}") })
        })
        .collect())
}
