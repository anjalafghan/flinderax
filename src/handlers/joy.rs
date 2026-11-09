//! Link to Joy, the shared date jar. Flinderax pulls Joy's date spending
//! (`GET <joy>/api/v1/spending`, bearer token) to show a Dates budget with your share of it, and
//! labels the matching bank-alert transactions as dates (`planner::joy`). One way only: Joy is
//! shared with a partner, so nothing from here is ever sent there.

use std::time::Duration as StdDuration;

use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    handlers::{
        common::{bad_request, today_local, AppError},
        couple::couple_of,
    },
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
        return Err(bad_request("Use just Joy's address, like https://joy.fly.dev"));
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
    if let Some((couple, who)) = couple_of(&state.db, &user_id).await? {
        let key = format!("COVERS_{}", 1 - who);
        let rupees = (p.partner_paise / 100).to_string();
        sqlx::query!(
            "INSERT INTO joy_settings (couple_id, key, value) VALUES (?, ?, ?) ON CONFLICT(couple_id, key) DO UPDATE SET value = excluded.value",
            couple,
            key,
            rupees
        )
        .execute(&state.db)
        .await?;
        return Ok(Json(Done { status: true }));
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
    // Dates lives in this app now: read it directly, nothing to fetch and nothing stale.
    if let Some((couple, who)) = couple_of(db, &user_id).await? {
        let p = local_payload(db, &couple).await?;
        let partner = partner_covers(db, &couple, who, &p).await?;
        let mut out = Summary { connected: true, partner_paise: partner, ..Default::default() };
        out.labelled = label_dates(db, &user_id, &p).await?;
        fill_month(&mut out, &p, partner);
        return Ok(Json(out));
    }
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
    if let Some(p) = payload {
        fill_month(&mut out, &p, link.partner_paise);
    }
    Ok(Json(out))
}

/// This month's figures and recent entries from a payload, with your share.
fn fill_month(out: &mut Summary, p: &JoyPayload, partner_paise: i64) {
    let today = today_local();
    let this_month = format!("{:04}-{:02}", today.year(), today.month() as u8);
    let Some(m) = p.months.iter().find(|m| m.month == this_month).or(p.months.last()) else { return };
    let budget = m.budget.map(|b| (b + m.carry) * 100);
    let spent = m.spent * 100;
    out.month = Some(MonthOut {
        month: m.month.clone(),
        budget_paise: budget,
        carry_paise: m.carry * 100,
        spent_paise: spent,
        your_share_paise: your_share(spent, budget, partner_paise),
        your_budget_paise: budget.map(|b| (b - partner_paise).max(0)),
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
            your_share_paise: your_share(e.amount * 100, budget, partner_paise),
        })
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls() {
        assert_eq!(clean_url(" https://joy.fly.dev/ ").ok().unwrap(), "https://joy.fly.dev");
        assert_eq!(clean_url("http://localhost:3000").ok().unwrap(), "http://localhost:3000");
        assert!(clean_url("http://joy.fly.dev").is_err());
        assert!(clean_url("https://joy.fly.dev/api").is_err());
        assert!(clean_url("https://user@evil.example").is_err());
        assert!(clean_url("ftp://x").is_err());
    }
}

// ---------- Dates in this app (couples) ----------

const LOCAL_MONTHS: i64 = 6;

/// The same figures Joy's `/api/v1/spending` gave, read from the Dates tables (whole rupees).
pub async fn local_payload(db: &sqlx::SqlitePool, couple: &str) -> Result<JoyPayload, AppError> {
    use crate::joy::db as jdb;
    let mut months_list = Vec::new();
    for back in (0..LOCAL_MONTHS).rev() {
        months_list.push(jdb::month_offset(db, back).await?);
    }
    let standing = jdb::setting(db, couple, "monthly_budget").await?.and_then(|v| v.trim().parse::<i64>().ok()).filter(|b| *b > 0);
    let mut months = Vec::new();
    for m in &months_list {
        let row = jdb::month_budget(db, couple, m).await?;
        months.push(JoyMonth {
            month: m.clone(),
            budget: row.map(|(base, _)| base).or(standing),
            carry: row.map_or(0, |(_, carry)| carry),
            spent: jdb::spent_in_month(db, couple, m).await?,
        });
    }
    let (oldest, newest) = (months_list[0].clone(), months_list[months_list.len() - 1].clone());
    // Kept dates (an unrevealed surprise keeps its amount but not its name), other date spending,
    // and nights out with a regular: the same rules as Joy's spending API.
    let rows: Vec<(String, String, i64, String, String)> = sqlx::query_as(
        "SELECT 'date:' || h.id, date(COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime'),
                MAX(h.actual_cost - COALESCE(h.from_savings, 0), 0),
                CASE WHEN h.surprise = 1 AND h.revealed = 0 THEN 'A surprise date' ELSE COALESCE(a.title, '') END, 'date'
           FROM joy_history h LEFT JOIN joy_activities a ON a.id = h.activity_id
          WHERE h.couple_id = ?1 AND h.outcome = 'did_it' AND h.actual_cost IS NOT NULL
            AND COALESCE(h.planned_at, h.picked_at) <= unixepoch()
            AND strftime('%Y-%m', COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime') BETWEEN ?2 AND ?3
         UNION ALL
         SELECT 'other:' || id, spent_on, amount, CASE WHEN trim(note) = '' THEN 'Other date spending' ELSE note END, 'other'
           FROM joy_other_spending WHERE couple_id = ?1 AND strftime('%Y-%m', spent_on) BETWEEN ?2 AND ?3
         UNION ALL
         SELECT 'night:' || v.id, v.day, v.amount, CASE WHEN trim(v.place) = '' THEN r.title ELSE v.place END, 'regular'
           FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
          WHERE r.couple_id = ?1 AND v.status = 'went' AND v.amount > 0 AND strftime('%Y-%m', v.day) BETWEEN ?2 AND ?3
         ORDER BY 2 DESC",
    )
    .bind(couple)
    .bind(&oldest)
    .bind(&newest)
    .fetch_all(db)
    .await?;
    let entries = rows
        .into_iter()
        .filter(|r| r.2 > 0)
        .map(|(id, day, amount, title, kind)| JoyEntry { id, day, amount, title, kind })
        .collect();
    Ok(JoyPayload { currency: "₹".into(), monthly_budget: standing, months, entries })
}

/// What the other person covers each month, in paise: their `COVERS_<slot>` setting, else half
/// the budget when there are two of you, else nothing.
pub async fn partner_covers(db: &sqlx::SqlitePool, couple: &str, who: usize, p: &JoyPayload) -> Result<i64, AppError> {
    let key = format!("COVERS_{}", 1 - who);
    if let Some(v) = crate::joy::db::setting(db, couple, &key).await?.and_then(|v| v.trim().parse::<i64>().ok()) {
        return Ok(v.max(0) * 100);
    }
    let two = sqlx::query_scalar!(
        r#"SELECT (user_0 IS NOT NULL AND user_1 IS NOT NULL) AS "two!: bool" FROM couples WHERE couple_id = ?"#,
        couple
    )
    .fetch_one(db)
    .await?;
    Ok(if two { p.monthly_budget.unwrap_or(0) * 100 / 2 } else { 0 })
}

// ---------- settling up ----------

#[derive(Serialize)]
pub struct Settlement {
    pub id: i64,
    pub payer_is_you: bool,
    pub amount_paise: i64,
    pub on_day: String,
    pub note: String,
}

#[derive(Serialize)]
pub struct SettleOut {
    /// Date payments from your bank (transactions labelled Dates).
    pub you_paid_paise: i64,
    pub partner_paid_paise: i64,
    /// Your share of date spending, in basis points (7000 = 70%).
    pub your_share_bps: i64,
    /// Positive: your partner owes you this. Negative: you owe them.
    pub balance_paise: i64,
    pub settlements: Vec<Settlement>,
}

/// Date money between the two of you since the first settlement window (the last 6 months):
/// what each paid (from each person's own transactions labelled Dates), each one's fair share,
/// and repayments logged so far.
pub async fn settle(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<SettleOut>, AppError> {
    let db = &state.db;
    let (couple, who) = crate::handlers::couple::require_couple(db, &user_id).await?;
    let p = local_payload(db, &couple).await?;
    let partner = partner_covers(db, &couple, who, &p).await?;
    let budget = p.monthly_budget.unwrap_or(0) * 100;
    let your_share_bps = if budget > 0 { ((budget - partner).clamp(0, budget) * 10_000) / budget } else { 5_000 };
    let since = format!("{}-01", p.months.first().map(|m| m.month.as_str()).unwrap_or("1970-01"));
    let users = sqlx::query!(r#"SELECT user_0, user_1 FROM couples WHERE couple_id = ?"#, couple).fetch_one(db).await?;
    let paid_by = |uid: Option<String>| {
        let since = since.clone();
        async move {
            match uid {
                Some(u) => sqlx::query_scalar!(
                    r#"SELECT COALESCE(SUM(amount_paise), 0) AS "s!: i64" FROM transactions
                       WHERE user_id = ? AND category = 'dates' AND direction = 'debit' AND occurred_on >= ?"#,
                    u,
                    since
                )
                .fetch_one(db)
                .await,
                None => Ok(0),
            }
        }
    };
    let (mine, theirs) = if who == 0 { (users.user_0, users.user_1) } else { (users.user_1, users.user_0) };
    let you_paid = paid_by(mine).await?;
    let partner_paid = paid_by(theirs).await?;
    let settlements: Vec<Settlement> = sqlx::query!(
        r#"SELECT id AS "id!", payer, amount_paise, on_day, note FROM joy_settlements WHERE couple_id = ? ORDER BY on_day DESC, id DESC"#,
        couple
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|s| Settlement { id: s.id, payer_is_you: s.payer as usize == who, amount_paise: s.amount_paise, on_day: s.on_day, note: s.note })
    .collect();
    // You owe your share of what they paid; they owe their share of what you paid.
    let they_owe = you_paid * (10_000 - your_share_bps) / 10_000;
    let you_owe = partner_paid * your_share_bps / 10_000;
    let repaid: i64 = settlements.iter().map(|s| if s.payer_is_you { s.amount_paise } else { -s.amount_paise }).sum();
    Ok(Json(SettleOut {
        you_paid_paise: you_paid,
        partner_paid_paise: partner_paid,
        your_share_bps,
        balance_paise: they_owe - you_owe + repaid,
        settlements,
    }))
}

#[derive(Deserialize)]
pub struct SettlePayload {
    /// true: you paid your partner back; false: they paid you.
    pub you_paid: bool,
    pub amount_paise: i64,
    pub note: Option<String>,
}

pub async fn settle_record(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<SettlePayload>,
) -> Result<Json<Done>, AppError> {
    if p.amount_paise <= 0 {
        return Err(bad_request("The amount must be more than zero"));
    }
    let (couple, who) = crate::handlers::couple::require_couple(&state.db, &user_id).await?;
    let payer = if p.you_paid { who as i64 } else { 1 - who as i64 };
    let today = today_local().to_string();
    let note = p.note.unwrap_or_default();
    sqlx::query!(
        "INSERT INTO joy_settlements (couple_id, payer, amount_paise, on_day, note) VALUES (?, ?, ?, ?, ?)",
        couple,
        payer,
        p.amount_paise,
        today,
        note
    )
    .execute(&state.db)
    .await?;
    Ok(Json(Done { status: true }))
}
