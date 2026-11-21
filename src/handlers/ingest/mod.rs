//! Transactions from bank alerts: the phone forwards SMS / emails to `POST /api/hook/ingest`
//! (authenticated by a long-lived per-user key), or the user pastes one. `crate::alerts` reads
//! the text; this file matches it to a card or account, stores it once, keeps the account balance
//! in step when the alert carries "Avl Bal", and learns merchant categories from corrections.

use std::collections::{BTreeMap, HashSet};

use axum::{
    extract::{multipart::MultipartError, Multipart, Query, State},
    http::{HeaderMap, StatusCode},
    body::Bytes,
    response::{IntoResponse, Response},
    Extension, Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use nanoid::nanoid;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{macros::format_description, Date, Duration};

use crate::{
    alerts::{self, categorize, normalize, Alert, Direction, CATEGORIES},
    handlers::common::{bad_request, not_found, today_local, AppError},
    models::AppState,
    planner::cycle::add_months,
    ratelimit,
    statement::{Mapping, StatementRow},
};

mod statement;
pub use statement::*;


type User = Extension<(String, String)>;

const MAX_TEXT: usize = 2_000;
const HOOK_CALLS_PER_MINUTE: u32 = 120;
/// Alert dates further back than this (or in the future) are treated as "today".
const MAX_ALERT_AGE_DAYS: i64 = 90;

#[derive(Serialize)]
pub struct TxnRow {
    pub txn_id: String,
    pub occurred_on: String,
    pub direction: String,
    pub amount_paise: i64,
    pub merchant: Option<String>,
    pub category: String,
    pub card_id: Option<String>,
    pub account_id: Option<String>,
    /// The card's or account's name.
    pub source_name: Option<String>,
    pub last4: Option<String>,
    pub source: String,
    pub created_at: String,
    /// The category was set by hand, so automatic re-checks leave it alone.
    pub category_locked: bool,
}

#[derive(Serialize)]
pub struct BalanceUpdated {
    pub account_name: String,
    pub balance_paise: i64,
}

/// A newer balance the account already had, kept over the statement's closing balance.
#[derive(Serialize)]
pub struct BalanceKept {
    pub account_name: String,
    pub balance_paise: i64,
    pub set_on: String,
    pub statement_paise: i64,
    pub statement_on: String,
}

#[derive(Serialize)]
pub struct IngestResult {
    /// created | duplicate | unrecognized
    pub status: &'static str,
    pub transaction: Option<TxnRow>,
    pub balance_updated: Option<BalanceUpdated>,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

fn parse_date(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, format_description!("[year]-[month]-[day]")).ok()
}

fn hash_key(key: &str) -> String {
    Sha256::digest(key.trim().as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

async fn load_txn(db: &sqlx::SqlitePool, user_id: &str, txn_id: &str) -> Result<Option<TxnRow>, AppError> {
    Ok(sqlx::query_as!(
        TxnRow,
        r#"SELECT t.txn_id AS "txn_id!", t.occurred_on, t.direction, t.amount_paise, t.merchant, t.category,
                  t.card_id, t.account_id, COALESCE(c.card_name, a.name) AS "source_name?: String", t.last4, t.source,
                  t.created_at AS "created_at!: String", t.category_locked AS "category_locked: bool"
           FROM transactions t
           LEFT JOIN cards c ON c.card_id = t.card_id
           LEFT JOIN accounts a ON a.account_id = t.account_id
           WHERE t.user_id = ? AND t.txn_id = ?"#,
        user_id,
        txn_id
    )
    .fetch_optional(db)
    .await?)
}

async fn load_rules(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<(String, String)>, AppError> {
    Ok(sqlx::query!("SELECT pattern, category FROM merchant_rules WHERE user_id = ?", user_id)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| (r.pattern, r.category))
        .collect())
}

/// Which of the user's cards / accounts an alert is about. Banks print 3 or 4 digits, so the
/// stored digits only have to end with the alert's.
async fn match_instrument(
    db: &sqlx::SqlitePool,
    user_id: &str,
    alert: &Alert,
    mentions_card: bool,
) -> Result<(Option<String>, Option<String>), AppError> {
    let Some(l4) = alert.last4.as_deref() else { return Ok((None, None)) };
    let card = sqlx::query!(
        r#"SELECT card_id AS "card_id!", last_4_digits FROM cards WHERE user_id = ? AND last_4_digits IS NOT NULL"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .find(|c| c.last_4_digits.as_deref().is_some_and(|d| d.ends_with(l4)))
    .map(|c| c.card_id);
    let account = sqlx::query!(
        r#"SELECT account_id AS "account_id!", last_4_digits FROM accounts WHERE user_id = ? AND last_4_digits IS NOT NULL"#,
        user_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .find(|a| a.last_4_digits.as_deref().is_some_and(|d| d.ends_with(l4)))
    .map(|a| a.account_id);
    Ok(match (card, account) {
        (Some(c), Some(_)) if mentions_card => (Some(c), None),
        (_, Some(a)) => (None, Some(a)),
        (c, None) => (c, None),
    })
}

/// How long an unreadable alert stays in the inbox, and how many a person can have at once.
const INBOX_DAYS: i64 = 30;
const INBOX_MAX: i64 = 200;

async fn keep_unrecognized(db: &sqlx::SqlitePool, user_id: &str, text: &str, source: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM unrecognized_alerts WHERE received_at < datetime('now', ?)")
        .bind(format!("-{INBOX_DAYS} days"))
        .execute(db)
        .await?;
    let held: i64 = sqlx::query_scalar("SELECT count(*) FROM unrecognized_alerts WHERE user_id = ?").bind(user_id).fetch_one(db).await?;
    if held >= INBOX_MAX {
        return Ok(());
    }
    sqlx::query("INSERT OR IGNORE INTO unrecognized_alerts (alert_id, user_id, text, source) VALUES (?, ?, ?, ?)")
        .bind(nanoid!())
        .bind(user_id)
        .bind(text)
        .bind(source)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(Serialize)]
pub struct InboxItem {
    pub alert_id: String,
    pub text: String,
    pub source: String,
    pub received_at: String,
}

#[derive(Serialize)]
pub struct Inbox {
    pub count: usize,
    pub items: Vec<InboxItem>,
}

/// `GET /api/ingest/unrecognized`: alert texts the parser could not read, newest first.
pub async fn unrecognized(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Inbox>, AppError> {
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT alert_id, text, source, received_at FROM unrecognized_alerts
         WHERE user_id = ? AND received_at >= datetime('now', ?) ORDER BY received_at DESC, rowid DESC",
    )
    .bind(&user_id)
    .bind(format!("-{INBOX_DAYS} days"))
    .fetch_all(&state.db)
    .await?;
    let items: Vec<InboxItem> =
        rows.into_iter().map(|(alert_id, text, source, received_at)| InboxItem { alert_id, text, source, received_at }).collect();
    Ok(Json(Inbox { count: items.len(), items }))
}

#[derive(Deserialize)]
pub struct DismissAlert {
    pub alert_id: String,
}

/// `POST /api/ingest/unrecognized/dismiss`: done with it (added by hand, or not a transaction).
pub async fn unrecognized_dismiss(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<DismissAlert>,
) -> Result<Json<Done>, AppError> {
    sqlx::query("DELETE FROM unrecognized_alerts WHERE alert_id = ? AND user_id = ?")
        .bind(&p.alert_id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(Done { status: true }))
}

/// Reads `text` and stores it for `user_id`.
pub async fn ingest_text(
    db: &sqlx::SqlitePool,
    user_id: &str,
    text: &str,
    source: &str,
    today: Date,
) -> Result<IngestResult, AppError> {
    if text.len() > MAX_TEXT {
        return Err(bad_request("text is too long"));
    }
    let Some(alert) = alerts::parse(text) else {
        // Texts from the phone are kept for a month so the transaction is not silently missing.
        // A paste shows its answer on the spot, so it is not kept.
        if source != "paste" {
            keep_unrecognized(db, user_id, text, source).await?;
        }
        return Ok(IngestResult { status: "unrecognized", transaction: None, balance_updated: None });
    };
    let mentions_card = alert.card_payment || text.to_ascii_lowercase().contains("card");
    let (card_id, account_id) = match_instrument(db, user_id, &alert, mentions_card).await?;
    let on = alert
        .date
        .filter(|d| *d <= today && *d >= today - Duration::days(MAX_ALERT_AGE_DAYS))
        .unwrap_or(today);
    let mut category = categorize(alert.merchant.as_deref(), alert.direction, alert.card_payment, &load_rules(db, user_id).await?);
    // Money coming back for something you bought is not income.
    if category == "income" && alert.direction == alerts::Direction::Credit && alerts::is_refund_text(text) {
        category = "refund".into();
    }
    let direction = alert.direction.as_str();
    let dedupe_key = match &alert.reference {
        Some(r) => format!("ref:{r}:{direction}"),
        None => format!(
            "fp:{on}:{direction}:{}:{}:{}",
            alert.amount_paise,
            alert.last4.as_deref().unwrap_or(""),
            alert.merchant.as_deref().map(normalize).unwrap_or_default()
        ),
    };

    let txn_id = nanoid!();
    let on_s = on.to_string();
    let inserted = sqlx::query!(
        "INSERT INTO transactions
            (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, card_id, account_id, last4, source, raw_text, dedupe_key)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT (user_id, dedupe_key) DO NOTHING",
        txn_id,
        user_id,
        on_s,
        direction,
        alert.amount_paise,
        alert.merchant,
        category,
        card_id,
        account_id,
        alert.last4,
        source,
        text,
        dedupe_key
    )
    .execute(db)
    .await?
    .rows_affected();
    if inserted == 0 {
        return Ok(IngestResult { status: "duplicate", transaction: None, balance_updated: None });
    }
    // The bank reference lets a later statement row find this alert (the query above is unchanged).
    if let Some(rrn) = crate::narration::rrn_of(text) {
        sqlx::query("UPDATE transactions SET rrn = ? WHERE txn_id = ? AND user_id = ?")
            .bind(rrn)
            .bind(&txn_id)
            .bind(user_id)
            .execute(db)
            .await?;
    }

    // "Avl Bal" is the bank's own figure: it becomes the typed-in balance, unless a newer one exists.
    let mut balance_updated = None;
    if let (Some(acct), Some(bal)) = (&account_id, alert.balance_paise) {
        let res = sqlx::query!(
            "UPDATE accounts SET balance_paise = ?, balance_set_on = ?, updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ? AND user_id = ? AND (balance_set_on IS NULL OR balance_set_on <= ?)",
            bal,
            on_s,
            acct,
            user_id,
            on_s
        )
        .execute(db)
        .await?;
        if res.rows_affected() == 1 {
            let name = sqlx::query_scalar!("SELECT name FROM accounts WHERE account_id = ?", acct).fetch_one(db).await?;
            balance_updated = Some(BalanceUpdated { account_name: name, balance_paise: bal });
        }
    }

    crate::handlers::activity::note(db, user_id, crate::planner::activity::Kind::Txn, &txn_id, 1).await;
    Ok(IngestResult { status: "created", transaction: load_txn(db, user_id, &txn_id).await?, balance_updated })
}

// ---------- the phone's webhook ----------

#[derive(Deserialize)]
pub struct HookPayload {
    pub text: String,
    /// sms (default) | email
    pub source: Option<String>,
}

/// `POST /api/hook/ingest` with header `X-Ingest-Key`. No bearer token: phones can't refresh one.
pub async fn hook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(p): Json<HookPayload>,
) -> Result<Json<IngestResult>, AppError> {
    let unauthorized = || AppError(StatusCode::UNAUTHORIZED, "Invalid ingest key".into());
    let key = headers.get("x-ingest-key").and_then(|v| v.to_str().ok()).ok_or_else(unauthorized)?;
    if !ratelimit::allow(&format!("hook-ip:{}", ratelimit::client_key(&headers)), HOOK_CALLS_PER_MINUTE * 2) {
        return Err(AppError(StatusCode::TOO_MANY_REQUESTS, "Too many requests".into()));
    }
    let hash = hash_key(key);
    let user_id = sqlx::query_scalar!(r#"SELECT user_id AS "user_id!" FROM ingest_keys WHERE key_hash = ?"#, hash)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(unauthorized)?;
    if !ratelimit::allow(&format!("hook-user:{user_id}"), HOOK_CALLS_PER_MINUTE) {
        return Err(AppError(StatusCode::TOO_MANY_REQUESTS, "Too many requests".into()));
    }
    sqlx::query!("UPDATE ingest_keys SET last_used_at = CURRENT_TIMESTAMP WHERE user_id = ?", user_id)
        .execute(&state.db)
        .await?;
    let source = if p.source.as_deref() == Some("email") { "email" } else { "sms" };
    Ok(Json(ingest_text(&state.db, &user_id, &p.text, source, today_local()).await?))
}

// ---------- key management ----------

#[derive(Serialize)]
pub struct KeyStatus {
    pub has_key: bool,
    pub created_at: Option<String>,
    pub last_used_at: Option<String>,
}

#[derive(Serialize)]
pub struct NewKey {
    pub key: String,
}

pub async fn key_status(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<KeyStatus>, AppError> {
    let row = sqlx::query!(
        r#"SELECT created_at AS "created_at?: String", last_used_at AS "last_used_at: String" FROM ingest_keys WHERE user_id = ?"#,
        user_id
    )
    .fetch_optional(&state.db)
    .await?;
    Ok(Json(match row {
        Some(r) => KeyStatus { has_key: true, created_at: r.created_at, last_used_at: r.last_used_at },
        None => KeyStatus { has_key: false, created_at: None, last_used_at: None },
    }))
}

/// Makes a new key (the old one stops working). The key itself is only ever returned here.
pub async fn key_rotate(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<NewKey>, AppError> {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let key = format!("fx_{}", URL_SAFE_NO_PAD.encode(bytes));
    let hash = hash_key(&key);
    sqlx::query!(
        "INSERT INTO ingest_keys (user_id, key_hash) VALUES (?, ?)
         ON CONFLICT(user_id) DO UPDATE SET key_hash = excluded.key_hash, created_at = CURRENT_TIMESTAMP, last_used_at = NULL",
        user_id,
        hash
    )
    .execute(&state.db)
    .await?;
    Ok(Json(NewKey { key }))
}

// ---------- transactions ----------

#[derive(Deserialize)]
pub struct ParsePayload {
    pub text: String,
}

pub async fn parse(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<ParsePayload>,
) -> Result<Json<IngestResult>, AppError> {
    Ok(Json(ingest_text(&state.db, &user_id, &p.text, "paste", today_local()).await?))
}

#[derive(Deserialize)]
pub struct MonthQuery {
    /// YYYY-MM; default the current month.
    pub month: Option<String>,
}

/// First day of `YYYY-MM` and of the month after.
fn month_bounds(ym: &str) -> Option<(String, String)> {
    let idx = crate::planner::parse_year_month(ym)?;
    let (y, m) = (idx.div_euclid(12), idx.rem_euclid(12) + 1);
    let (ny, nm) = add_months(y, time::Month::try_from(m as u8).ok()?, 1);
    Some((format!("{y:04}-{m:02}-01"), format!("{ny:04}-{:02}-01", nm as u8)))
}

pub async fn list(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Query(q): Query<MonthQuery>,
) -> Result<Json<Vec<TxnRow>>, AppError> {
    let today = today_local();
    let ym = q.month.unwrap_or_else(|| format!("{:04}-{:02}", today.year(), today.month() as u8));
    let (from, to) = month_bounds(&ym).ok_or_else(|| bad_request("month must look like 2026-11"))?;
    Ok(Json(
        sqlx::query_as!(
            TxnRow,
            r#"SELECT t.txn_id AS "txn_id!", t.occurred_on, t.direction, t.amount_paise, t.merchant, t.category,
                      t.card_id, t.account_id, COALESCE(c.card_name, a.name) AS "source_name?: String", t.last4, t.source,
                      t.created_at AS "created_at!: String", t.category_locked AS "category_locked: bool"
               FROM transactions t
               LEFT JOIN cards c ON c.card_id = t.card_id
               LEFT JOIN accounts a ON a.account_id = t.account_id
               WHERE t.user_id = ? AND t.occurred_on >= ? AND t.occurred_on < ?
               ORDER BY t.occurred_on DESC, t.created_at DESC"#,
            user_id,
            from,
            to
        )
        .fetch_all(&state.db)
        .await?,
    ))
}

fn check_category(c: &str) -> Result<(), AppError> {
    if CATEGORIES.contains(&c) { Ok(()) } else { Err(bad_request("unknown category")) }
}

#[derive(Deserialize)]
pub struct UpdateTxn {
    pub txn_id: String,
    pub category: String,
    pub merchant: Option<String>,
    #[serde(default)]
    pub remember: bool,
}

#[derive(Serialize)]
pub struct UpdateResult {
    pub status: bool,
    /// Other uncategorised transactions the new rule also applied to.
    pub recategorized: u64,
}

pub async fn update(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<UpdateTxn>,
) -> Result<Json<UpdateResult>, AppError> {
    check_category(&p.category)?;
    let db = &state.db;
    let current = load_txn(db, &user_id, &p.txn_id).await?.ok_or_else(|| not_found("Transaction not found"))?;
    let merchant = p.merchant.map(|m| m.trim().to_string()).filter(|m| !m.is_empty()).or(current.merchant);
    sqlx::query!(
        "UPDATE transactions SET category = ?, merchant = ?, category_locked = 1 WHERE txn_id = ? AND user_id = ?",
        p.category,
        merchant,
        p.txn_id,
        user_id
    )
    .execute(db)
    .await?;

    // Every correction is remembered under the merchant's key, so the next text or statement line
    // from the same merchant comes in categorised. A payee with no key, or "uncategorized", is not.
    if let Some(m) = merchant.as_deref() {
        crate::merchant_rules::remember(db, &user_id, m, &p.category).await?;
    }

    let mut recategorized = 0;
    if let Some(pattern) = merchant.as_deref().map(normalize).filter(|m| p.remember && !m.is_empty()) {
        crate::merchant_rules::upsert_pattern(db, &user_id, &pattern, &p.category).await?;
        let rule = [(pattern, p.category.clone())];
        let others = sqlx::query!(
            r#"SELECT txn_id AS "txn_id!", merchant AS "merchant!" FROM transactions
               WHERE user_id = ? AND category = 'uncategorized' AND merchant IS NOT NULL"#,
            user_id
        )
        .fetch_all(db)
        .await?;
        for o in others {
            if categorize(Some(&o.merchant), Direction::Debit, false, &rule) == p.category {
                recategorized += sqlx::query!("UPDATE transactions SET category = ? WHERE txn_id = ?", p.category, o.txn_id)
                    .execute(db)
                    .await?
                    .rows_affected();
            }
        }
    }
    Ok(Json(UpdateResult { status: true, recategorized }))
}

#[derive(Deserialize)]
pub struct TxnId {
    pub txn_id: String,
}

pub async fn delete(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<TxnId>,
) -> Result<Json<Done>, AppError> {
    let res = sqlx::query!("DELETE FROM transactions WHERE txn_id = ? AND user_id = ?", p.txn_id, user_id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Transaction not found"));
    }
    Ok(Json(Done { status: true }))
}

#[derive(Deserialize)]
pub struct CreateTxn {
    pub occurred_on: String,
    pub direction: String,
    pub amount_paise: i64,
    pub merchant: Option<String>,
    pub category: String,
}

#[derive(Serialize)]
pub struct Created {
    pub txn_id: String,
    pub status: bool,
}

pub async fn create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<CreateTxn>,
) -> Result<Json<Created>, AppError> {
    check_category(&p.category)?;
    parse_date(&p.occurred_on).ok_or_else(|| bad_request("occurred_on must look like 2026-11-02"))?;
    if !matches!(p.direction.as_str(), "debit" | "credit") {
        return Err(bad_request("direction must be debit or credit"));
    }
    if p.amount_paise <= 0 {
        return Err(bad_request("amount_paise must be positive"));
    }
    let txn_id = nanoid!();
    let key = format!("manual:{txn_id}");
    let merchant = p.merchant.map(|m| m.trim().to_string()).filter(|m| !m.is_empty());
    sqlx::query!(
        "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, source, dedupe_key)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'manual', ?)",
        txn_id,
        user_id,
        p.occurred_on,
        p.direction,
        p.amount_paise,
        merchant,
        p.category,
        key
    )
    .execute(&state.db)
    .await?;
    crate::handlers::activity::note(&state.db, &user_id, crate::planner::activity::Kind::Txn, &txn_id, 1).await;
    Ok(Json(Created { txn_id, status: true }))
}

// ---------- summary and rules ----------

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub months: Option<u32>,
}

#[derive(Serialize)]
pub struct CategoryAmount {
    pub category: String,
    pub amount_paise: i64,
}

#[derive(Serialize)]
pub struct MonthSpend {
    pub month: String,
    pub spent_paise: i64,
    pub by_category: Vec<CategoryAmount>,
}

#[derive(Serialize)]
pub struct MerchantSpend {
    pub merchant: String,
    pub amount_paise: i64,
    pub count: i64,
}

#[derive(Serialize)]
pub struct SpendSummary {
    pub months: Vec<MonthSpend>,
    pub top_merchants: Vec<MerchantSpend>,
    pub uncategorized_count: i64,
}

pub async fn summary(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Query(q): Query<SummaryQuery>,
) -> Result<Json<SpendSummary>, AppError> {
    let today = today_local();
    let n = q.months.unwrap_or(6).clamp(1, 24) as i32;
    let month_keys: Vec<String> = (0..n)
        .rev()
        .map(|back| {
            let (y, m) = add_months(today.year(), today.month(), -back);
            format!("{y:04}-{:02}", m as u8)
        })
        .collect();
    let from = format!("{}-01", month_keys[0]);
    let rows = sqlx::query!(
        "SELECT occurred_on, amount_paise, merchant, category FROM transactions
         WHERE user_id = ? AND direction = 'debit' AND occurred_on >= ?",
        user_id,
        from
    )
    .fetch_all(&state.db)
    .await?;

    let current = month_keys.last().cloned().unwrap_or_default();
    let mut per_month: BTreeMap<String, BTreeMap<String, i64>> = month_keys.iter().map(|k| (k.clone(), BTreeMap::new())).collect();
    let mut merchants: BTreeMap<String, (String, i64, i64)> = BTreeMap::new();
    for r in rows.iter().filter(|r| alerts::is_spending(&r.category)) {
        let ym = r.occurred_on.get(..7).unwrap_or_default().to_string();
        if let Some(cats) = per_month.get_mut(&ym) {
            *cats.entry(r.category.clone()).or_default() += r.amount_paise;
        }
        if ym == current
            && let Some(m) = &r.merchant {
                let e = merchants.entry(normalize(m)).or_insert_with(|| (m.clone(), 0, 0));
                e.1 += r.amount_paise;
                e.2 += 1;
            }
    }
    let months = per_month
        .into_iter()
        .map(|(month, cats)| {
            let mut by_category: Vec<CategoryAmount> =
                cats.into_iter().map(|(category, amount_paise)| CategoryAmount { category, amount_paise }).collect();
            by_category.sort_by_key(|a| std::cmp::Reverse(a.amount_paise));
            MonthSpend { spent_paise: by_category.iter().map(|c| c.amount_paise).sum(), month, by_category }
        })
        .collect();
    let mut top_merchants: Vec<MerchantSpend> =
        merchants.into_values().map(|(merchant, amount_paise, count)| MerchantSpend { merchant, amount_paise, count }).collect();
    top_merchants.sort_by_key(|a| std::cmp::Reverse(a.amount_paise));
    top_merchants.truncate(8);

    let uncategorized_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!: i64" FROM transactions WHERE user_id = ? AND category = 'uncategorized'"#,
        user_id
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(SpendSummary { months, top_merchants, uncategorized_count }))
}

#[derive(Serialize)]
pub struct RuleRow {
    pub pattern: String,
    pub category: String,
}

pub async fn rules(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Vec<RuleRow>>, AppError> {
    Ok(Json(
        sqlx::query_as!(RuleRow, "SELECT pattern, category FROM merchant_rules WHERE user_id = ? ORDER BY pattern", user_id)
            .fetch_all(&state.db)
            .await?,
    ))
}

#[derive(Deserialize)]
pub struct RulePattern {
    pub pattern: String,
}

pub async fn rule_delete(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<RulePattern>,
) -> Result<Json<Done>, AppError> {
    sqlx::query!("DELETE FROM merchant_rules WHERE user_id = ? AND pattern = ?", user_id, p.pattern)
        .execute(&state.db)
        .await?;
    Ok(Json(Done { status: true }))
}

#[derive(Serialize)]
pub struct Cleared {
    pub status: bool,
    pub removed: u64,
}

/// `POST /api/ingest/rules/clear`: forgets every category you taught it. Rows already sorted keep
/// the category they have; new ones go back to the automatic keywords.
pub async fn rules_clear(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Cleared>, AppError> {
    let removed = sqlx::query("DELETE FROM merchant_rules WHERE user_id = ?").bind(&user_id).execute(&state.db).await?.rows_affected();
    Ok(Json(Cleared { status: true, removed }))
}

/// `POST /api/ingest/statement/layouts/clear`: forgets the column layouts saved for statements, so
/// the next upload of each is detected afresh.
pub async fn layouts_clear(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Cleared>, AppError> {
    let removed = sqlx::query("DELETE FROM statement_profiles WHERE user_id = ?").bind(&user_id).execute(&state.db).await?.rows_affected();
    Ok(Json(Cleared { status: true, removed }))
}

#[derive(Serialize)]
pub struct Unlocked {
    pub status: bool,
    pub category: String,
}

/// `POST /api/ingest/transactions/unlock`: hands a hand-set category back to the automatic one.
/// The row works out its category again from your rules and the keywords (a statement row also from
/// its structure), and later re-checks may change it as they do for any other row.
pub async fn unlock(State(state): State<AppState>, Extension((user_id, _)): User, Json(p): Json<TxnId>) -> Result<Json<Unlocked>, AppError> {
    let db = &state.db;
    let row: Option<(Option<String>, String, Option<String>, bool, bool)> = sqlx::query_as(
        "SELECT merchant, direction, raw_text, on_statement = 1, card_id IS NOT NULL FROM transactions WHERE txn_id = ? AND user_id = ?",
    )
    .bind(&p.txn_id)
    .bind(&user_id)
    .fetch_optional(db)
    .await?;
    let (merchant, direction, raw, on_statement, on_card) = row.ok_or_else(|| not_found("Transaction not found"))?;
    let credit = direction == Direction::Credit.as_str();
    let dir = if credit { Direction::Credit } else { Direction::Debit };
    let rules = load_rules(db, &user_id).await?;
    let category = match (&raw, on_statement) {
        (Some(raw), true) => {
            let names = load_holder_names(db, &user_id).await?;
            let owner = crate::narration::Owner { holder_names: &names };
            let parts = crate::narration::parse(raw);
            match statement_structure(raw, credit, on_card, &parts, &owner) {
                Some(c) => c.to_string(),
                None => categorize(parts.payee.as_deref(), dir, false, &rules),
            }
        }
        _ => categorize(merchant.as_deref(), dir, false, &rules),
    };
    sqlx::query("UPDATE transactions SET category = ?, category_locked = 0 WHERE txn_id = ? AND user_id = ?")
        .bind(&category)
        .bind(&p.txn_id)
        .bind(&user_id)
        .execute(db)
        .await?;
    Ok(Json(Unlocked { status: true, category }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_bounds_cross_the_year() {
        assert_eq!(month_bounds("2026-12"), Some(("2026-12-01".into(), "2027-01-01".into())));
        assert_eq!(month_bounds("2026-13"), None);
    }

    #[test]
    fn key_hash_ignores_surrounding_whitespace() {
        assert_eq!(hash_key(" fx_abc\n"), hash_key("fx_abc"));
        assert_eq!(hash_key("fx_abc").len(), 64);
    }

    fn statement_row(amount_paise: i64) -> StatementRow {
        use time::macros::date;
        StatementRow {
            date: date!(2026 - 11 - 01),
            direction: Direction::Debit,
            amount_paise,
            narration: "UPI-SWIGGY-swiggy@icici-4305".into(),
            reference: None,
            balance_paise: None,
        }
    }

    #[test]
    fn statement_key_has_no_target_so_reimports_match_across_targets() {
        // The key is built from the row alone: the same row from the same file gets the same key
        // whichever card or account the file is imported to.
        let row = statement_row(145_000);
        assert_eq!(statement_key(&row), statement_key(&row.clone()));
        assert_eq!(statement_key(&row), "stmt:2026-11-01:debit:145000:upi swiggy swiggy icici 4305");
        // Different day, direction or amount is a different row.
        assert_ne!(statement_key(&row), statement_key(&statement_row(145_001)));
        let credit = StatementRow { direction: Direction::Credit, ..row.clone() };
        assert_ne!(statement_key(&row), statement_key(&credit));
    }
}

// ---------- holder names and recategorising ----------

#[derive(Serialize, Deserialize)]
pub struct HolderNames {
    pub names: Vec<String>,
}

/// `GET /api/ingest/holder-names`: the names the user holds accounts under (empty when none are saved).
pub async fn holder_names(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<HolderNames>, AppError> {
    Ok(Json(HolderNames { names: load_holder_names(&state.db, &user_id).await? }))
}

/// `POST /api/ingest/holder-names`: replaces the saved names. Up to 5, each 2 to 60 characters, no
/// line breaks, no repeats (ignoring case).
pub async fn set_holder_names(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<HolderNames>,
) -> Result<Json<HolderNames>, AppError> {
    if p.names.len() > MAX_HOLDER_NAMES {
        return Err(bad_request(format!("Save up to {MAX_HOLDER_NAMES} names.")));
    }
    let mut kept: Vec<String> = Vec::new();
    for name in &p.names {
        let name = name.trim();
        if name.contains(['\n', '\r']) {
            return Err(bad_request("Names can't contain line breaks."));
        }
        let len = name.chars().count();
        if !(2..=60).contains(&len) {
            return Err(bad_request("Each name needs 2 to 60 characters."));
        }
        if !kept.iter().any(|k| k.eq_ignore_ascii_case(name)) {
            kept.push(name.to_string());
        }
    }
    sqlx::query(
        "INSERT INTO user_settings (user_id, holder_names) VALUES (?, ?)
         ON CONFLICT(user_id) DO UPDATE SET holder_names = excluded.holder_names",
    )
    .bind(&user_id)
    .bind(kept.join("\n"))
    .execute(&state.db)
    .await?;
    Ok(Json(HolderNames { names: kept }))
}

#[derive(Serialize)]
pub struct Recategorized {
    pub status: bool,
    /// Statement rows whose category changed.
    pub recategorized: usize,
}

/// `POST /api/ingest/recategorize`: re-runs the import's categories on the user's statement rows: the
/// structure (own accounts, SIPs, salary, fees...), then the user's rules and the shop keywords, so
/// rows imported before a rule or keyword existed catch up. Rows the user set by hand stay as they
/// are, and a row that already has a category is never put back to uncategorized.
pub async fn recategorize(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Recategorized>, AppError> {
    let changed = recategorize_rows(&state.db, &user_id).await?;
    Ok(Json(Recategorized { status: true, recategorized: changed }))
}

/// The work behind [`recategorize`]; also run after each statement import so older rows catch up.
async fn recategorize_rows(db: &sqlx::SqlitePool, user_id: &str) -> Result<usize, AppError> {
    let user_id = user_id.to_string();
    let names = load_holder_names(db, &user_id).await?;
    let owner = crate::narration::Owner { holder_names: &names };
    let rules = load_rules(db, &user_id).await?;
    let rows: Vec<(String, String, String, String, bool)> = sqlx::query_as(
        // The saved payee stands in for a missing narration: before the Kotak formats were read,
        // a narration this code did not know was stored whole as the payee.
        "SELECT txn_id, direction, COALESCE(raw_text, merchant), category, card_id IS NOT NULL FROM transactions
         WHERE user_id = ? AND on_statement = 1 AND category_locked = 0
           AND COALESCE(raw_text, merchant) IS NOT NULL",
    )
    .bind(&user_id)
    .fetch_all(db)
    .await?;
    let mut tx = db.begin().await?;
    let mut changed = 0;
    for (txn_id, direction, raw, current, on_card) in rows {
        let parts = crate::narration::parse(&raw);
        let credit = direction == Direction::Credit.as_str();
        let dir = if credit { Direction::Credit } else { Direction::Debit };
        let category = match statement_structure(&raw, credit, on_card, &parts, &owner) {
            Some(c) => c.to_string(),
            None => categorize(parts.payee.as_deref(), dir, false, &rules),
        };
        if category == current || (category == "uncategorized" && current != "uncategorized") {
            continue;
        }
        sqlx::query("UPDATE transactions SET category = ? WHERE txn_id = ? AND user_id = ?")
            .bind(category)
            .bind(&txn_id)
            .bind(&user_id)
            .execute(&mut *tx)
            .await?;
        changed += 1;
    }
    tx.commit().await?;
    Ok(changed)
}

/// Imports, then re-checks the user's older statement rows against the current rules and keywords.
async fn import_and_catch_up(db: &sqlx::SqlitePool, user_id: &str, input: StatementInput) -> Result<StatementResult, AppError> {
    let mut result = import_statement(db, user_id, input).await?;
    result.older_recategorized = recategorize_rows(db, user_id).await?;
    Ok(result)
}


#[cfg(test)]
mod import_golden;

#[cfg(test)]
mod statement_tests {
    use super::*;
    use axum::{body::Body, http::Request, http::StatusCode};
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    /// HDFC's delimited export: an empty first line, then the header.
    const HEADER: &str = "\r\nDate,Narration,Value Dat,Debit Amount,Credit Amount,Chq/Ref Number,Closing Balance\r\n";
    /// Six synthetic rows, oldest first. Opening balance 10,000.00, closing 55,130.50. Row 3's narration has a comma.
    const ROWS: &str = "01/11/26,UPI-RAVI KUMAR-RAVIK123@OKAXIS-SBIN0001234-612345678901-PAID VIA CRED,01/11/26,250.00,,612345678901,9750.00\r\n\
02/11/26,UPI-ASHA STORES-Q123456789@YBL-YESB0YBLUPI-612345678902-UPI,02/11/26,120.50,,612345678902,9629.50\r\n\
03/11/26,UPI-ACME, PUNE-ACME@OKAXIS-SBIN0001234-612345678903-PAYMENT ON CRED,03/11/26,1000.00,,612345678903,8629.50\r\n\
04/11/26,SALARYFORTHEMONTHOFOCT2026,04/11/26,,50000.00,,58629.50\r\n\
05/11/26,UPI-ASHA RAO-98XXXXXX12@YESCRED-KKBK0001234-612345678904-PAID VIA CRED,05/11/26,3000.00,,612345678904,55629.50\r\n\
06/11/26,UPI-STREAMCO-STREAMCO@YBL-YESB0YBLUPI-612345678905-SUBSCRIPTION DEBIT,06/11/26,499.00,,612345678905,55130.50\r\n";
    const BOUNDARY: &str = "XSTATEMENTX";

    fn csv(rows: &str) -> String {
        format!("{HEADER}{rows}")
    }

    async fn test_state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color) VALUES ('cardA', 'u', 'A', 'HDFC', 0, 0)",
            "INSERT INTO accounts (account_id, user_id, name, kind) VALUES ('acctB', 'u', 'B', 'bank')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32][..]),
        );
        AppState { db, paseto_key: std::sync::Arc::new(key) }
    }

    async fn import(state: &AppState, csv: &str, account: Option<&str>) -> StatementResult {
        let input = StatementInput { csv: csv.to_string(), account_id: account.map(str::to_string), card_id: None, mapping: None };
        import_statement(&state.db, "u", input).await.unwrap_or_else(|e| panic!("import failed: {}", e.1))
    }

    /// An import against the card `cardA`, where a single amount column is spending.
    async fn import_card(state: &AppState, csv: &str) -> StatementResult {
        let input = StatementInput { csv: csv.to_string(), account_id: None, card_id: Some("cardA".into()), mapping: None };
        import_statement(&state.db, "u", input).await.unwrap_or_else(|e| panic!("import failed: {}", e.1))
    }

    async fn category_of(db: &sqlx::SqlitePool, raw: &str) -> String {
        sqlx::query_scalar("SELECT category FROM transactions WHERE raw_text = ?").bind(raw).fetch_one(db).await.unwrap()
    }

    /// Synthetic card export: purchases unsigned, payments and refunds marked "Cr".
    const CARD_CSV: &str = "Date,Description,Amount\n\
12-11-2026,AMAZON PAY,\"2,000.00\"\n\
13-11-2026,PAYMENT RECEIVED,\"5,000.00 Cr\"\n\
14-11-2026,REFUND FLIPKART,\"300.00 Cr\"\n\
15-11-2026,SALARY PAYMENT CREDIT,\"1,000.00 Cr\"\n";

    /// Synthetic layout no detector reads: no column says date or amount.
    const UNKNOWN_CSV: &str = "Posted On,Memo,Out,In\n03/11/2026,Coffee shop,120.00,\n04/11/2026,Salary,,5000.00\n";

    /// The same layout as `UNKNOWN_CSV`, with other rows.
    const LATER_CSV: &str = "Posted On,Memo,Out,In\n10/11/2026,Taxi,250.00,\n11/11/2026,Refund,,80.00\n";

    const UNKNOWN_MAPPING: &str = r#"{"header_row":0,"date":0,"narration":1,"debit":2,"credit":3,"amount":null,"sign":{"kind":"separate_columns"},"balance":null,"reference":null,"date_order":"dmy"}"#;

    async fn count_where(db: &sqlx::SqlitePool, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
    }

    fn user() -> Extension<(String, String)> {
        Extension(("u".to_string(), "user".to_string()))
    }

    /// The bank's alerts for three of the rows, one for an unrelated row (never on the statement), and
    /// one on another card that carries row 6's reference.
    async fn seed_alerts(db: &sqlx::SqlitePool) {
        for q in [
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, account_id, source, dedupe_key)
             VALUES ('alert-asha', 'u', '2026-11-02', 'debit', 12050, 'ASHA STORES', 'acctB', 'sms', 'fp:asha')",
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, account_id, source, dedupe_key)
             VALUES ('alert-salary', 'u', '2026-11-04', 'credit', 5000000, 'acctB', 'sms', 'fp:salary')",
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, account_id, source, dedupe_key)
             VALUES ('alert-unmatched', 'u', '2026-11-03', 'debit', 99900, 'acctB', 'sms', 'fp:unmatched')",
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, card_id, source, dedupe_key, rrn)
             VALUES ('alert-streamco', 'u', '2026-11-06', 'debit', 49900, 'cardA', 'sms', 'fp:stream', '612345678905')",
        ] {
            sqlx::query(q).execute(db).await.unwrap();
        }
    }

    async fn count(db: &sqlx::SqlitePool, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
    }

    #[tokio::test]
    async fn newest_first_statement_sets_the_real_closing_balance() {
        // Newest first, two lines on the last day: the day's first line (listed last) is not the closing one.
        let rows = "06/11/26,UPI-STREAMCO-STREAMCO@YBL-YESB0YBLUPI-612345678905-SUBSCRIPTION DEBIT,06/11/26,499.00,,612345678905,500.00\r\n\
06/11/26,UPI-ASHA RAO-98XXXXXX12@YESCRED-KKBK0001234-612345678904-PAID VIA CRED,06/11/26,3000.00,,612345678904,999.00\r\n\
05/11/26,SALARYFORTHEMONTHOFOCT2026,05/11/26,,5000.00,,3999.00\r\n";
        let state = test_state().await;
        let res = import(&state, &csv(rows), Some("acctB")).await;
        assert!(res.chain_ok);
        assert_eq!(res.balance_updated.map(|b| b.balance_paise), Some(50_000));
        let bal: i64 = sqlx::query_scalar("SELECT balance_paise FROM accounts WHERE account_id = 'acctB'").fetch_one(&state.db).await.unwrap();
        assert_eq!(bal, 50_000);
    }

    #[tokio::test]
    async fn six_row_delimited_file_imports() {
        let state = test_state().await;
        let res = import(&state, &csv(ROWS), Some("acctB")).await;
        assert_eq!((res.total, res.added, res.duplicates, res.skipped), (6, 6, 0, 0));
        assert!(res.chain_ok);
        assert_eq!((res.opening_paise, res.closing_paise), (Some(1_000_000), Some(5_513_050)));
        assert_eq!(res.balance_updated.map(|b| b.balance_paise), Some(5_513_050));
        assert_eq!((res.first_date.as_deref(), res.last_date.as_deref()), (Some("2026-11-01"), Some("2026-11-06")));
        // The comma inside the narration is kept, and the row is stored once with its reference.
        let acme: String = sqlx::query_scalar("SELECT raw_text FROM transactions WHERE rrn = '612345678903'").fetch_one(&state.db).await.unwrap();
        assert_eq!(acme, "UPI-ACME, PUNE-ACME@OKAXIS-SBIN0001234-612345678903-PAYMENT ON CRED");
        assert_eq!(count(&state.db, "SELECT COUNT(*) FROM transactions WHERE on_statement = 1").await, 6);
        assert_eq!(count(&state.db, "SELECT COUNT(*) FROM statement_imports WHERE chain_ok = 1 AND rows = 6").await, 1);
    }

    #[tokio::test]
    async fn newer_typed_balance_is_kept_and_reported() {
        let state = test_state().await;
        sqlx::query("UPDATE accounts SET balance_paise = 7000000, balance_set_on = '2026-11-20' WHERE account_id = 'acctB'")
            .execute(&state.db)
            .await
            .unwrap();
        let res = import(&state, &csv(ROWS), Some("acctB")).await;
        assert!(res.balance_updated.is_none());
        let kept = res.balance_kept.expect("kept balance reported");
        assert_eq!((kept.balance_paise, kept.set_on.as_str()), (7_000_000, "2026-11-20"));
        assert_eq!((kept.statement_paise, kept.statement_on.as_str()), (5_513_050, "2026-11-06"));
        let bal: i64 = sqlx::query_scalar("SELECT balance_paise FROM accounts WHERE account_id = 'acctB'").fetch_one(&state.db).await.unwrap();
        assert_eq!(bal, 7_000_000);
    }

    #[tokio::test]
    async fn older_typed_balance_gives_way_to_the_statement() {
        let state = test_state().await;
        sqlx::query("UPDATE accounts SET balance_paise = 7000000, balance_set_on = '2026-10-01' WHERE account_id = 'acctB'")
            .execute(&state.db)
            .await
            .unwrap();
        let res = import(&state, &csv(ROWS), Some("acctB")).await;
        assert_eq!(res.balance_updated.map(|b| b.balance_paise), Some(5_513_050));
        assert!(res.balance_kept.is_none());
    }

    #[tokio::test]
    async fn re_import_is_all_duplicates() {
        let state = test_state().await;
        seed_alerts(&state.db).await;
        let first = import(&state, &csv(ROWS), Some("acctB")).await;
        // Confirmed alerts keep their own keys, so a second import has to recognise them another way.
        assert_eq!((first.added, first.upgraded, first.not_on_statement), (3, 3, 1));
        let total = count(&state.db, "SELECT COUNT(*) FROM transactions").await;

        let second = import(&state, &csv(ROWS), Some("acctB")).await;
        assert_eq!((second.added, second.upgraded, second.duplicates), (0, 0, 6));
        assert_eq!(count(&state.db, "SELECT COUNT(*) FROM transactions").await, total);
    }

    #[tokio::test]
    async fn alert_with_the_same_rrn_is_upgraded_not_duplicated() {
        let state = test_state().await;
        seed_alerts(&state.db).await;
        // The streamco alert sits on another card, so only the reference can match it.
        import(&state, &csv(ROWS), Some("acctB")).await;
        let (on_statement, rrn, merchant): (i64, Option<String>, Option<String>) =
            sqlx::query_as("SELECT on_statement, rrn, merchant FROM transactions WHERE txn_id = 'alert-streamco'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(on_statement, 1);
        assert_eq!(rrn.as_deref(), Some("612345678905"));
        assert_eq!(merchant.as_deref(), Some("STREAMCO"));
        assert_eq!(count(&state.db, "SELECT COUNT(*) FROM transactions WHERE rrn = '612345678905'").await, 1);
    }

    #[tokio::test]
    async fn own_transfer_is_a_transfer_once_the_holder_name_is_saved() {
        let state = test_state().await;
        import(&state, &csv(ROWS), Some("acctB")).await;
        let before: String = sqlx::query_scalar("SELECT category FROM transactions WHERE rrn = '612345678904'").fetch_one(&state.db).await.unwrap();
        assert_ne!(before, "transfer");

        let saved = set_holder_names(State(state.clone()), user(), Json(HolderNames { names: vec!["Asha Rao".into()] })).await;
        assert_eq!(saved.map(|j| j.0.names).map_err(|e| e.1), Ok(vec!["Asha Rao".to_string()]));
        let changed = recategorize(State(state.clone()), user()).await.unwrap_or_else(|e| panic!("{}", e.1));
        assert!(changed.0.recategorized >= 1);
        let after: String = sqlx::query_scalar("SELECT category FROM transactions WHERE rrn = '612345678904'").fetch_one(&state.db).await.unwrap();
        assert_eq!(after, "transfer");

        // A fresh import with the name already saved categorises the row as a transfer straight away.
        let fresh = test_state().await;
        sqlx::query("INSERT INTO user_settings (user_id, holder_names) VALUES ('u', 'ASHA RAO')").execute(&fresh.db).await.unwrap();
        import(&fresh, &csv(ROWS), Some("acctB")).await;
        let direct: String = sqlx::query_scalar("SELECT category FROM transactions WHERE rrn = '612345678904'").fetch_one(&fresh.db).await.unwrap();
        assert_eq!(direct, "transfer");
    }

    #[tokio::test]
    async fn recategorize_applies_rules_added_after_the_import_but_keeps_hand_set_rows() {
        let state = test_state().await;
        import(&state, &csv(ROWS), Some("acctB")).await;
        let cat = |db: sqlx::SqlitePool| async move {
            sqlx::query_scalar::<_, String>("SELECT category FROM transactions WHERE rrn = '612345678903'").fetch_one(&db).await.unwrap()
        };
        sqlx::query("UPDATE transactions SET category = 'uncategorized' WHERE rrn = '612345678903'").execute(&state.db).await.unwrap();
        sqlx::query("INSERT INTO merchant_rules (user_id, pattern, category) VALUES ('u', 'acme', 'shopping')").execute(&state.db).await.unwrap();
        let _ = recategorize(State(state.clone()), user()).await.unwrap_or_else(|e| panic!("{}", e.1));
        assert_eq!(cat(state.db.clone()).await, "shopping");

        // A row the user set by hand stays.
        sqlx::query("UPDATE transactions SET category = 'gifts', category_locked = 1 WHERE rrn = '612345678903'").execute(&state.db).await.unwrap();
        let _ = recategorize(State(state.clone()), user()).await.unwrap_or_else(|e| panic!("{}", e.1));
        assert_eq!(cat(state.db.clone()).await, "gifts");
    }

    #[tokio::test]
    async fn rows_filed_as_income_before_the_kotak_formats_were_read_are_sorted_from_the_saved_payee() {
        let state = test_state().await;
        sqlx::query("INSERT INTO user_settings (user_id, holder_names) VALUES ('u', 'RAVI KUMAR SHARMA')").execute(&state.db).await.unwrap();
        // Older rows: no raw text, the unread narration stored whole as the payee, filed as income.
        for (id, merchant) in [
            ("k1", "UPI/RAVI KUMAR S/5731242052"),
            ("k2", "TD PREMAT PROCEEDS OF 3750996500"),
            ("k3", "UPI/CRED CLUB/648011099143/PAY"),
            ("k4", "SALARYFORTHEMONTHOFAUG2026ACME"),
        ] {
            sqlx::query("INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, source, dedupe_key, on_statement) VALUES (?, 'u', '2026-10-01', 'credit', 1000, ?, 'income', 'paste', ?, 1)")
                .bind(id).bind(merchant).bind(id)
                .execute(&state.db).await.unwrap();
        }
        let done = recategorize(State(state.clone()), user()).await.unwrap_or_else(|e| panic!("{}", e.1));
        assert_eq!(done.0.recategorized, 3);
        for (id, expected) in [("k1", "transfer"), ("k2", "transfer"), ("k3", "transfer"), ("k4", "income")] {
            let got: String = sqlx::query_scalar("SELECT category FROM transactions WHERE txn_id = ?").bind(id).fetch_one(&state.db).await.unwrap();
            assert_eq!(got, expected, "{id}");
        }
    }

    #[tokio::test]
    async fn correcting_a_category_is_remembered_and_applied_to_the_next_statement_import() {
        let state = test_state().await;
        sqlx::query("INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, source, dedupe_key) VALUES ('t1', 'u', '2026-11-01', 'debit', 1000, 'ACME', 'uncategorized', 'manual', 'k1')")
            .execute(&state.db)
            .await
            .unwrap();
        let body = UpdateTxn { txn_id: "t1".into(), category: "gifts".into(), merchant: None, remember: false };
        // Unknown category: refused, nothing remembered.
        let bad = UpdateTxn { category: "nope".into(), ..body_clone(&body) };
        assert!(update(State(state.clone()), user(), Json(bad)).await.is_err());
        assert!(crate::merchant_rules::lookup(&state.db, "u", "acme").await.unwrap().is_none());
        let ok = UpdateTxn { category: "shopping".into(), ..body_clone(&body) };
        let _ = update(State(state.clone()), user(), Json(ok)).await.unwrap_or_else(|e| panic!("{}", e.1));
        assert_eq!(crate::merchant_rules::lookup(&state.db, "u", "acme").await.unwrap().as_deref(), Some("shopping"));

        // The rule is read back for ingest, so ACME rows from now on come in categorised.
        import(&state, &csv(ROWS), Some("acctB")).await;
        let cat: String = sqlx::query_scalar("SELECT category FROM transactions WHERE rrn = '612345678903'").fetch_one(&state.db).await.unwrap();
        assert_eq!(cat, "shopping");
    }

    fn body_clone(b: &UpdateTxn) -> UpdateTxn {
        UpdateTxn { txn_id: b.txn_id.clone(), category: b.category.clone(), merchant: b.merchant.clone(), remember: b.remember }
    }

    #[tokio::test]
    async fn tampered_balance_fails_the_chain_check() {
        let state = test_state().await;
        let res = import(&state, &csv(&ROWS.replace("9629.50", "9629.00")), Some("acctB")).await;
        assert!(!res.chain_ok);
        assert!(res.opening_paise.is_some());
    }

    #[tokio::test]
    async fn holder_names_are_validated_and_stored_without_duplicates() {
        let state = test_state().await;
        let names = |r: Result<Json<HolderNames>, AppError>| r.map(|j| j.0.names).map_err(|e| e.1);
        let too_many = vec!["Aa", "Bb", "Cc", "Dd", "Ee", "Ff"].into_iter().map(String::from).collect();
        assert!(names(set_holder_names(State(state.clone()), user(), Json(HolderNames { names: too_many })).await).is_err());
        assert!(names(set_holder_names(State(state.clone()), user(), Json(HolderNames { names: vec!["A".into()] })).await).is_err());
        assert!(names(set_holder_names(State(state.clone()), user(), Json(HolderNames { names: vec!["Ab\nCd".into()] })).await).is_err());
        assert_eq!(names(holder_names(State(state.clone()), user()).await), Ok(vec![]));
        let saved = names(set_holder_names(State(state.clone()), user(), Json(HolderNames { names: vec!["Asha Rao".into(), "asha rao".into(), "Ravi K".into()] })).await);
        assert_eq!(saved, Ok(vec!["Asha Rao".to_string(), "Ravi K".to_string()]));
        assert_eq!(names(holder_names(State(state.clone()), user()).await), Ok(vec!["Asha Rao".to_string(), "Ravi K".to_string()]));
    }

    #[test]
    fn holder_suggestion_needs_a_large_debit_and_a_large_credit() {
        use time::macros::date;
        let row = |direction, amount_paise| StatementRow {
            date: date!(2026 - 11 - 05),
            direction,
            amount_paise,
            narration: "UPI-ASHA RAO-98XXXXXX12@YESCRED-KKBK0001234-612345678904-PAID VIA CRED".into(),
            reference: None,
            balance_paise: None,
        };
        assert_eq!(holder_suggestion(&[row(Direction::Debit, 600_000), row(Direction::Credit, 800_000)]).as_deref(), Some("ASHA RAO"));
        assert_eq!(holder_suggestion(&[row(Direction::Debit, 600_000), row(Direction::Credit, 1_000)]), None);
    }

    #[tokio::test]
    async fn card_payments_are_transfers_refunds_are_refunds_and_nothing_is_income() {
        let state = test_state().await;
        let res = import_card(&state, CARD_CSV).await;
        assert_eq!((res.added, res.total), (4, 4));
        assert_eq!(category_of(&state.db, "AMAZON PAY").await, "shopping");
        assert_eq!(category_of(&state.db, "PAYMENT RECEIVED").await, "transfer");
        assert_eq!(category_of(&state.db, "REFUND FLIPKART").await, "refund");
        // The salary-looking narration is a card credit first: a payment in, not income.
        assert_eq!(category_of(&state.db, "SALARY PAYMENT CREDIT").await, "transfer");
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM transactions WHERE card_id = 'cardA' AND category = 'income'").await, 0);
        // Payments and refunds are not spending; the purchase is.
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM transactions WHERE card_id = 'cardA' AND direction = 'debit' AND amount_paise = 200000").await, 1);
    }

    #[tokio::test]
    async fn a_card_payment_that_matches_an_alert_takes_the_statement_category_not_income() {
        let state = test_state().await;
        sqlx::query(
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, card_id, source, dedupe_key, category)
             VALUES ('alert-pay', 'u', '2026-11-13', 'credit', 500000, 'cardA', 'sms', 'fp:pay', 'income')",
        )
        .execute(&state.db)
        .await
        .unwrap();
        let res = import_card(&state, CARD_CSV).await;
        assert_eq!(res.upgraded, 1);
        let category: String = sqlx::query_scalar("SELECT category FROM transactions WHERE txn_id = 'alert-pay'").fetch_one(&state.db).await.unwrap();
        assert_eq!(category, "transfer");
    }

    #[tokio::test]
    async fn bank_debits_that_pay_a_card_bill_are_transfers() {
        let state = test_state().await;
        // Synthetic: a bill paid through a bill-payment app, and an NEFT to a card issuer.
        let bills = "\r\nDate,Narration,Value Dat,Debit Amount,Credit Amount,Chq/Ref Number,Closing Balance\r\n\
07/11/26,UPI-CRED CLUB-CREDCLUB@ICICI-ICIC0000001-612345678906-CRED CLUB,07/11/26,15000.00,,612345678906,40129.50\r\n\
08/11/26,NEFT-HDFC CARDS-PAYMENT-HDFC0000001-N7654321,08/11/26,2500.00,,N7654321,37629.50\r\n";
        import(&state, bills, Some("acctB")).await;
        assert_eq!(category_of(&state.db, "UPI-CRED CLUB-CREDCLUB@ICICI-ICIC0000001-612345678906-CRED CLUB").await, "transfer");
        assert_eq!(category_of(&state.db, "NEFT-HDFC CARDS-PAYMENT-HDFC0000001-N7654321").await, "transfer");
        // The app tags on ordinary rows ("PAID VIA CRED") must not make them transfers.
        import(&state, &csv(ROWS), Some("acctB")).await;
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM transactions WHERE raw_text LIKE '%PAID VIA CRED' AND category = 'transfer'").await, 0);
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM transactions WHERE raw_text LIKE '%PAYMENT ON CRED' AND category = 'transfer'").await, 0);
    }

    #[tokio::test]
    async fn unknown_layout_is_422_needs_mapping_with_a_preview() {
        let state = test_state().await;
        let (status, body) = post_form(&state, "/statement/upload", multipart_fields(&[("account_id", "acctB")], UNKNOWN_CSV.as_bytes())).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["error"], "needs_mapping");
        assert_eq!(json["preview"]["header_row"], 0);
        assert_eq!(json["preview"]["rows"][0], serde_json::json!(["Posted On", "Memo", "Out", "In"]));
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM transactions").await, 0);
    }

    #[tokio::test]
    async fn preview_shows_the_header_and_whether_a_layout_is_known() {
        let state = test_state().await;
        let (status, body) = post_form(&state, "/statement/preview", multipart_fields(&[], UNKNOWN_CSV.as_bytes())).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["known_profile"], false);
        assert_eq!(json["preview"]["guessed"], serde_json::Value::Null);
        assert_eq!(json["signature"].as_str().map(str::len), Some(16));
        // Nothing is stored by a preview.
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM statement_profiles").await, 0);
    }

    #[tokio::test]
    async fn a_saved_layout_is_reused_by_its_header_signature() {
        let state = test_state().await;
        let (status, body) = post_form(
            &state,
            "/statement/upload",
            multipart_fields(
                &[("account_id", "acctB"), ("mapping", UNKNOWN_MAPPING), ("save_profile", "true"), ("label", "Posted layout")],
                UNKNOWN_CSV.as_bytes(),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM statement_profiles WHERE label = 'Posted layout'").await, 1);
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM transactions WHERE on_statement = 1 AND direction = 'debit' AND amount_paise = 12000").await, 1);

        // Another file with the same header, and no mapping: the preview knows it, and the import reads it.
        let (_, body) = post_form(&state, "/statement/preview", multipart_fields(&[], LATER_CSV.as_bytes())).await;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["known_profile"], true);
        let (status, body) = post_form(&state, "/statement/upload", multipart_fields(&[("account_id", "acctB")], LATER_CSV.as_bytes())).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["added"], 2);
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM statement_profiles").await, 1);
    }

    #[tokio::test]
    async fn a_mapping_that_reads_no_rows_is_a_bad_request_and_saves_nothing() {
        let state = test_state().await;
        let wrong = r#"{"header_row":0,"date":0,"narration":1,"debit":9,"credit":9,"amount":null,"sign":{"kind":"separate_columns"},"balance":null,"reference":null,"date_order":"dmy"}"#;
        let (status, body) = post_form(
            &state,
            "/statement/upload",
            multipart_fields(&[("account_id", "acctB"), ("mapping", wrong), ("save_profile", "true")], UNKNOWN_CSV.as_bytes()),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("Check the column mapping"), "{body}");
        assert_eq!(count_where(&state.db, "SELECT COUNT(*) FROM statement_profiles").await, 0);
    }

    #[tokio::test]
    async fn excel_export_is_rejected_by_its_magic_bytes() {
        let state = test_state().await;
        let (status, body) = upload(&state, multipart_body(None, &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["message"], "This is the Excel export. Download the Delimited (.txt) format instead.");
    }

    #[tokio::test]
    async fn upload_route_imports_a_multipart_file() {
        let state = test_state().await;
        let (status, body) = upload(&state, multipart_body(Some("acctB"), csv(ROWS).as_bytes())).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!((json["added"].as_u64(), json["total"].as_u64(), json["chain_ok"].as_bool()), (Some(6), Some(6), Some(true)));
        assert_eq!(count(&state.db, "SELECT COUNT(*) FROM transactions WHERE on_statement = 1").await, 6);
    }

    fn multipart_body(account_id: Option<&str>, file: &[u8]) -> Vec<u8> {
        let mut body = Vec::new();
        if let Some(account) = account_id {
            body.extend_from_slice(format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"account_id\"\r\n\r\n{account}\r\n").as_bytes());
        }
        body.extend_from_slice(
            format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"statement.txt\"\r\nContent-Type: text/plain\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(file);
        body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
        body
    }

    async fn upload(state: &AppState, body: Vec<u8>) -> (StatusCode, String) {
        post_form(state, "/statement/upload", body).await
    }

    /// A multipart body with text fields before the file, for the mapping and profile fields.
    fn multipart_fields(fields: &[(&str, &str)], file: &[u8]) -> Vec<u8> {
        let mut body = Vec::new();
        for (name, value) in fields {
            body.extend_from_slice(format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n").as_bytes());
        }
        body.extend_from_slice(
            format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"statement.txt\"\r\nContent-Type: text/plain\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(file);
        body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
        body
    }

    async fn post_form(state: &AppState, path: &str, body: Vec<u8>) -> (StatusCode, String) {
        let app = crate::routes::ingest::routes(state.clone()).layer(user());
        let req = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", format!("multipart/form-data; boundary={BOUNDARY}"))
            .body(Body::from(body))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }
}
