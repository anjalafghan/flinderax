//! Link to Joy, the shared date jar. Flinderax pulls Joy's date spending
//! (`GET <joy>/api/v1/spending`, bearer token) to show a Dates budget with your share of it, and
//! labels the matching bank-alert transactions as dates (`planner::joy`). One way only: Joy is
//! shared with a partner, so nothing from here is ever sent there.

use std::time::Duration as StdDuration;

use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    handlers::common::{bad_request, today_local, AppError},
    models::AppState,
    planner::joy::{match_dates, your_share, DateSpend, Debit},
};

type User = Extension<(String, String)>;

const FETCH_TIMEOUT: StdDuration = StdDuration::from_secs(8);
const MONTHS: u32 = 6;
const RECENT: usize = 6;

// ---------- Joy's response (whole rupees) ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoyMonth {
    pub month: String,
    pub budget: Option<i64>,
    #[serde(default)]
    pub carry: i64,
    pub spent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoyEntry {
    pub id: String,
    pub day: String,
    pub amount: i64,
    pub title: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoyPayload {
    #[serde(default)]
    pub currency: String,
    pub monthly_budget: Option<i64>,
    pub months: Vec<JoyMonth>,
    pub entries: Vec<JoyEntry>,
}

fn parse_date(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

/// `https://…`, or `http://` to this machine for local testing. No path, query or credentials.
fn clean_url(raw: &str) -> Result<String, AppError> {
    let url = raw.trim().trim_end_matches('/');
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://").filter(|r| r.starts_with("localhost") || r.starts_with("127.0.0.1")))
        .ok_or_else(|| bad_request("Joy's address must start with https://"))?;
    if rest.is_empty() || rest.contains(['/', '?', '#', '@', ' ']) {
        return Err(bad_request("Use just Joy's address, like https://joy-jar.fly.dev"));
    }
    Ok(url.to_string())
}

/// Asks Joy for its spending. Errors are short sentences shown to the user.
async fn fetch(url: &str, token: &str) -> Result<JoyPayload, String> {
    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .get(format!("{url}/api/v1/spending?months={MONTHS}"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| if e.is_timeout() { "Joy didn't answer in time".to_string() } else { "Couldn't reach Joy".to_string() })?;
    match res.status() {
        s if s.is_success() => res.json::<JoyPayload>().await.map_err(|_| "Joy sent something unexpected".to_string()),
        reqwest::StatusCode::UNAUTHORIZED => Err("Joy didn't accept the token".into()),
        reqwest::StatusCode::NOT_FOUND => Err("Joy's spending link is off (set JOY_API_TOKEN on Joy)".into()),
        s => Err(format!("Joy answered {}", s.as_u16())),
    }
}

/// Labels the debits that match Joy's dates; returns how many changed.
async fn label_dates(db: &sqlx::SqlitePool, user_id: &str, payload: &JoyPayload) -> Result<u64, AppError> {
    let entries: Vec<DateSpend> = payload
        .entries
        .iter()
        .filter_map(|e| Some(DateSpend { day: parse_date(&e.day)?, amount: e.amount * 100 }))
        .filter(|e| e.amount > 0)
        .collect();
    let Some(from) = entries.iter().map(|e| e.day).min() else { return Ok(0) };
    let from = (from - time::Duration::days(1)).to_string();
    let debits: Vec<Debit> = sqlx::query!(
        r#"SELECT txn_id AS "txn_id!", occurred_on, amount_paise, category FROM transactions
           WHERE user_id = ? AND direction = 'debit' AND occurred_on >= ?
             AND (category = 'dates' OR (category_locked = 0 AND category NOT IN ('transfer', 'income')))"#,
        user_id,
        from
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .filter_map(|t| Some(Debit { on: parse_date(&t.occurred_on)?, amount: t.amount_paise, is_date: t.category == "dates", txn_id: t.txn_id }))
    .collect();

    let mut changed = 0;
    for id in match_dates(&entries, &debits) {
        changed += sqlx::query!(
            "UPDATE transactions SET category = 'dates' WHERE txn_id = ? AND user_id = ? AND category_locked = 0",
            id,
            user_id
        )
        .execute(db)
        .await?
        .rows_affected();
    }
    Ok(changed)
}

// ---------- endpoints ----------

#[derive(Deserialize)]
pub struct ConnectPayload {
    pub url: String,
    pub token: String,
    #[serde(default)]
    pub partner_paise: i64,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

/// Saves the link, but only after Joy has answered with that token.
pub async fn connect(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<ConnectPayload>,
) -> Result<Json<Done>, AppError> {
    let url = clean_url(&p.url)?;
    let token = p.token.trim();
    if token.is_empty() {
        return Err(bad_request("Paste Joy's API token"));
    }
    if p.partner_paise < 0 {
        return Err(bad_request("The partner's share can't be negative"));
    }
    let payload = fetch(&url, token).await.map_err(bad_request)?;
    let json = serde_json::to_string(&payload).map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query!(
        "INSERT INTO joy_links (user_id, url, token, partner_paise, last_payload, synced_at, last_error)
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, NULL)
         ON CONFLICT(user_id) DO UPDATE SET url = excluded.url, token = excluded.token, partner_paise = excluded.partner_paise,
            last_payload = excluded.last_payload, synced_at = excluded.synced_at, last_error = NULL",
        user_id,
        url,
        token,
        p.partner_paise,
        json
    )
    .execute(&state.db)
    .await?;
    label_dates(&state.db, &user_id, &payload).await?;
    Ok(Json(Done { status: true }))
}

#[derive(Deserialize)]
pub struct PartnerPayload {
    pub partner_paise: i64,
}

pub async fn set_partner(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<PartnerPayload>,
) -> Result<Json<Done>, AppError> {
    if p.partner_paise < 0 {
        return Err(bad_request("The partner's share can't be negative"));
    }
    sqlx::query!("UPDATE joy_links SET partner_paise = ? WHERE user_id = ?", p.partner_paise, user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(Done { status: true }))
}

pub async fn disconnect(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Done>, AppError> {
    sqlx::query!("DELETE FROM joy_links WHERE user_id = ?", user_id).execute(&state.db).await?;
    Ok(Json(Done { status: true }))
}

#[derive(Serialize)]
pub struct MonthOut {
    pub month: String,
    /// This month's budget including carry-over, if Joy has one.
    pub budget_paise: Option<i64>,
    pub carry_paise: i64,
    pub spent_paise: i64,
    pub your_share_paise: i64,
    /// Budget minus the partner's part.
    pub your_budget_paise: Option<i64>,
}

#[derive(Serialize)]
pub struct RecentOut {
    pub day: String,
    pub title: String,
    pub kind: String,
    pub amount_paise: i64,
    pub your_share_paise: i64,
}

#[derive(Serialize, Default)]
pub struct Summary {
    pub connected: bool,
    pub url: Option<String>,
    pub partner_paise: i64,
    /// Joy couldn't be reached: these are the last figures it sent.
    pub stale: bool,
    pub synced_at: Option<String>,
    pub error: Option<String>,
    pub month: Option<MonthOut>,
    pub recent: Vec<RecentOut>,
    /// Transactions labelled as dates on this refresh.
    pub labelled: u64,
}

/// `GET /api/joy/summary`: refreshes from Joy (falling back to the last figures) and returns this
/// month's date spending with your share.
pub async fn summary(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Summary>, AppError> {
    let db = &state.db;
    let Some(link) = sqlx::query!(
        r#"SELECT url, token, partner_paise, last_payload, synced_at AS "synced_at: String" FROM joy_links WHERE user_id = ?"#,
        user_id
    )
    .fetch_optional(db)
    .await?
    else {
        return Ok(Json(Summary::default()));
    };

    let mut out = Summary { connected: true, url: Some(link.url.clone()), partner_paise: link.partner_paise, ..Default::default() };
    let payload = match fetch(&link.url, &link.token).await {
        Ok(p) => {
            let json = serde_json::to_string(&p).unwrap_or_default();
            sqlx::query!(
                "UPDATE joy_links SET last_payload = ?, synced_at = CURRENT_TIMESTAMP, last_error = NULL WHERE user_id = ?",
                json,
                user_id
            )
            .execute(db)
            .await?;
            out.synced_at = sqlx::query_scalar!(r#"SELECT synced_at AS "s?: String" FROM joy_links WHERE user_id = ?"#, user_id)
                .fetch_one(db)
                .await?;
            out.labelled = label_dates(db, &user_id, &p).await?;
            Some(p)
        }
        Err(e) => {
            sqlx::query!("UPDATE joy_links SET last_error = ? WHERE user_id = ?", e, user_id).execute(db).await?;
            out.stale = true;
            out.synced_at = link.synced_at;
            out.error = Some(e);
            link.last_payload.as_deref().and_then(|j| serde_json::from_str::<JoyPayload>(j).ok())
        }
    };
    let Some(p) = payload else { return Ok(Json(out)) };

    let today = today_local();
    let this_month = format!("{:04}-{:02}", today.year(), today.month() as u8);
    let current = p.months.iter().find(|m| m.month == this_month).or(p.months.last());
    if let Some(m) = current {
        let budget = m.budget.map(|b| (b + m.carry) * 100);
        let spent = m.spent * 100;
        out.month = Some(MonthOut {
            month: m.month.clone(),
            budget_paise: budget,
            carry_paise: m.carry * 100,
            spent_paise: spent,
            your_share_paise: your_share(spent, budget, link.partner_paise),
            your_budget_paise: budget.map(|b| (b - link.partner_paise).max(0)),
        });
        out.recent = p
            .entries
            .iter()
            .filter(|e| e.day.starts_with(&m.month))
            .take(RECENT)
            .map(|e| RecentOut {
                day: e.day.clone(),
                title: e.title.clone(),
                kind: e.kind.clone(),
                amount_paise: e.amount * 100,
                your_share_paise: your_share(e.amount * 100, budget, link.partner_paise),
            })
            .collect();
    }
    Ok(Json(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls() {
        assert_eq!(clean_url(" https://joy-jar.fly.dev/ ").ok().unwrap(), "https://joy-jar.fly.dev");
        assert_eq!(clean_url("http://localhost:3000").ok().unwrap(), "http://localhost:3000");
        assert!(clean_url("http://joy-jar.fly.dev").is_err());
        assert!(clean_url("https://joy-jar.fly.dev/api").is_err());
        assert!(clean_url("https://user@evil.example").is_err());
        assert!(clean_url("ftp://x").is_err());
    }
}
