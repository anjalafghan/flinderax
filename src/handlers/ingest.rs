//! Transactions from bank alerts: the phone forwards SMS / emails to `POST /api/hook/ingest`
//! (authenticated by a long-lived per-user key), or the user pastes one. `crate::alerts` reads
//! the text; this file matches it to a card or account, stores it once, keeps the account balance
//! in step when the alert carries "Avl Bal", and learns merchant categories from corrections.

use std::collections::BTreeMap;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
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
    statement::StatementRow,
};

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
}

#[derive(Serialize)]
pub struct BalanceUpdated {
    pub account_name: String,
    pub balance_paise: i64,
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
                  t.created_at AS "created_at!: String"
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
        return Ok(IngestResult { status: "unrecognized", transaction: None, balance_updated: None });
    };
    let mentions_card = alert.card_payment || text.to_ascii_lowercase().contains("card");
    let (card_id, account_id) = match_instrument(db, user_id, &alert, mentions_card).await?;
    let on = alert
        .date
        .filter(|d| *d <= today && *d >= today - Duration::days(MAX_ALERT_AGE_DAYS))
        .unwrap_or(today);
    let category = categorize(alert.merchant.as_deref(), alert.direction, alert.card_payment, &load_rules(db, user_id).await?);
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
        "INSERT OR IGNORE INTO transactions
            (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, card_id, account_id, last4, source, raw_text, dedupe_key)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                      t.created_at AS "created_at!: String"
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

    let mut recategorized = 0;
    if let Some(pattern) = merchant.as_deref().map(normalize).filter(|m| p.remember && !m.is_empty()) {
        sqlx::query!(
            "INSERT INTO merchant_rules (user_id, pattern, category) VALUES (?, ?, ?)
             ON CONFLICT(user_id, pattern) DO UPDATE SET category = excluded.category",
            user_id,
            pattern,
            p.category
        )
        .execute(db)
        .await?;
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
        if ym == current {
            if let Some(m) = &r.merchant {
                let e = merchants.entry(normalize(m)).or_insert_with(|| (m.clone(), 0, 0));
                e.1 += r.amount_paise;
                e.2 += 1;
            }
        }
    }
    let months = per_month
        .into_iter()
        .map(|(month, cats)| {
            let mut by_category: Vec<CategoryAmount> =
                cats.into_iter().map(|(category, amount_paise)| CategoryAmount { category, amount_paise }).collect();
            by_category.sort_by(|a, b| b.amount_paise.cmp(&a.amount_paise));
            MonthSpend { spent_paise: by_category.iter().map(|c| c.amount_paise).sum(), month, by_category }
        })
        .collect();
    let mut top_merchants: Vec<MerchantSpend> =
        merchants.into_values().map(|(merchant, amount_paise, count)| MerchantSpend { merchant, amount_paise, count }).collect();
    top_merchants.sort_by(|a, b| b.amount_paise.cmp(&a.amount_paise));
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

    #[tokio::test]
    async fn alert_on_one_card_does_not_match_a_statement_row_on_another_account() {
        use sqlx::sqlite::SqlitePoolOptions;
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color) VALUES ('cardA', 'u', 'A', 'HDFC', 0, 0)",
            "INSERT INTO accounts (account_id, user_id, name, kind) VALUES ('acctB', 'u', 'B', 'bank')",
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, card_id, source, dedupe_key)
             VALUES ('t1', 'u', '2026-11-01', 'debit', 145000, 'cardA', 'sms', 'fp:alert-on-card-a')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let mut conn = db.acquire().await.unwrap();
        let on_card_a = alerts_on_target(&mut *conn, "u", "2026-11-01", "debit", 145_000, None, Some("cardA")).await.unwrap();
        let on_account_b = alerts_on_target(&mut *conn, "u", "2026-11-01", "debit", 145_000, Some("acctB"), None).await.unwrap();
        let untargeted = alerts_on_target(&mut *conn, "u", "2026-11-01", "debit", 145_000, None, None).await.unwrap();
        assert_eq!((on_card_a, on_account_b, untargeted), (1, 0, 1));
        // An alert that never got a card or account still matches a targeted import, so it isn't doubled.
        sqlx::query(
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, source, dedupe_key)
             VALUES ('t2', 'u', '2026-11-02', 'debit', 9900, 'sms', 'fp:unscoped')",
        )
        .execute(&mut *conn)
        .await
        .unwrap();
        let unscoped = alerts_on_target(&mut *conn, "u", "2026-11-02", "debit", 9900, Some("acctB"), None).await.unwrap();
        assert_eq!(unscoped, 1);
    }
}

// ---------- statement import ----------

const MAX_STATEMENT_ROWS: usize = 5_000;

#[derive(Deserialize)]
pub struct StatementPayload {
    /// The CSV text of a bank or card statement export.
    pub csv: String,
    /// The bank account or card the statement belongs to (optional; sets last 4 and the balance).
    pub account_id: Option<String>,
    pub card_id: Option<String>,
}

#[derive(Serialize)]
pub struct StatementResult {
    pub total: usize,
    pub added: usize,
    /// Already recorded: the same file or an overlapping one imported before.
    pub duplicates: usize,
    /// Matched a transaction a bank alert had already recorded.
    pub matched_alerts: usize,
    /// Rows under the header that could not be read.
    pub skipped: usize,
    pub uncategorized: usize,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
    pub balance_updated: Option<BalanceUpdated>,
}

/// The identity of a statement row. It deliberately leaves out the card or account the file is
/// imported to, so re-importing the same file against a different target is still a duplicate.
fn statement_key(row: &StatementRow) -> String {
    format!("stmt:{}:{}:{}:{}", row.date, row.direction.as_str(), row.amount_paise, normalize(&row.narration))
}

/// How many bank-alert transactions on this card or account, on this day, in this direction and
/// amount, no statement row has claimed yet. Alerts for other cards or accounts are not counted:
/// an alert on one card must not swallow a row on another account.
async fn alerts_on_target(
    conn: &mut sqlx::SqliteConnection,
    user_id: &str,
    on: &str,
    direction: &str,
    amount_paise: i64,
    account_id: Option<&str>,
    card_id: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM transactions
         WHERE user_id = ?1 AND occurred_on = ?2 AND direction = ?3 AND amount_paise = ?4
           -- An import with no target matches any alert. A targeted import matches alerts on the
           -- same card/account, plus alerts that never got one (else those rows would double).
           AND ((?5 = '' AND ?6 = '')
                OR ((account_id IS NULL OR account_id = ?5) AND (card_id IS NULL OR card_id = ?6)))
           AND dedupe_key NOT LIKE 'stmt:%' AND source != 'manual'",
    )
    .bind(user_id)
    .bind(on)
    .bind(direction)
    .bind(amount_paise)
    .bind(account_id.unwrap_or(""))
    .bind(card_id.unwrap_or(""))
    .fetch_one(conn)
    .await
}

/// `POST /api/ingest/statement`: stores every row of a CSV statement. Nothing is stored twice:
/// re-importing a file, or an overlapping one, only adds what is new, and rows a bank alert already
/// recorded on the same card or account (same day, direction and amount) are matched to it instead
/// of doubled.
pub async fn statement(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<StatementPayload>,
) -> Result<Json<StatementResult>, AppError> {
    let db = &state.db;
    let parsed = crate::statement::parse(&p.csv)
        .ok_or_else(|| bad_request("Couldn't find the transaction table. The file needs a Date column and Debit/Credit or Amount columns."))?;
    if parsed.rows.is_empty() {
        return Err(bad_request("No transactions found in that file."));
    }
    if parsed.rows.len() > MAX_STATEMENT_ROWS {
        return Err(bad_request(format!("Too many rows (limit {MAX_STATEMENT_ROWS}). Import one period at a time.")));
    }

    // Which instrument this is, and its last digits.
    let (mut account_id, mut card_id, mut last4) = (None, None, None);
    if let Some(id) = p.account_id.as_deref().filter(|s| !s.is_empty()) {
        let row = sqlx::query_as::<_, (String, Option<String>)>("SELECT account_id, last_4_digits FROM accounts WHERE account_id = ? AND user_id = ?")
            .bind(id)
            .bind(&user_id)
            .fetch_optional(db)
            .await?
            .ok_or_else(|| not_found("Account not found"))?;
        account_id = Some(row.0);
        last4 = row.1;
    } else if let Some(id) = p.card_id.as_deref().filter(|s| !s.is_empty()) {
        let row = sqlx::query_as::<_, (String, Option<String>)>("SELECT card_id, last_4_digits FROM cards WHERE card_id = ? AND user_id = ?")
            .bind(id)
            .bind(&user_id)
            .fetch_optional(db)
            .await?
            .ok_or_else(|| not_found("Card not found"))?;
        card_id = Some(row.0);
        last4 = row.1;
    }
    let rules = load_rules(db, &user_id).await?;

    let mut res = StatementResult {
        total: parsed.rows.len(),
        added: 0,
        duplicates: 0,
        matched_alerts: 0,
        skipped: parsed.skipped,
        uncategorized: 0,
        first_date: None,
        last_date: None,
        balance_updated: None,
    };
    let mut seen: BTreeMap<String, u32> = BTreeMap::new();
    // How many alert-made transactions of each (day, direction, amount) are still unclaimed.
    let mut alert_budget: BTreeMap<(String, &'static str, i64), i64> = BTreeMap::new();
    let mut tx = db.begin().await?;

    for row in &parsed.rows {
        let on = row.date.to_string();
        let dir = row.direction.as_str();
        res.first_date = Some(res.first_date.take().map_or(on.clone(), |d| d.min(on.clone())));
        res.last_date = Some(res.last_date.take().map_or(on.clone(), |d| d.max(on.clone())));

        // Two identical rows on one day are two purchases; the n-th gets its own key.
        let base = statement_key(row);
        let n = seen.entry(base.clone()).and_modify(|n| *n += 1).or_insert(1);
        let dedupe_key = format!("{base}:{n}");

        let exists: Option<String> = sqlx::query_scalar("SELECT txn_id FROM transactions WHERE user_id = ? AND dedupe_key = ?")
            .bind(&user_id)
            .bind(&dedupe_key)
            .fetch_optional(&mut *tx)
            .await?;
        if exists.is_some() {
            res.duplicates += 1;
            continue;
        }

        let budget_key = (on.clone(), dir, row.amount_paise);
        if !alert_budget.contains_key(&budget_key) {
            let alerts_seen =
                alerts_on_target(&mut tx, &user_id, &on, dir, row.amount_paise, account_id.as_deref(), card_id.as_deref()).await?;
            alert_budget.insert(budget_key.clone(), alerts_seen);
        }
        let left = alert_budget.get_mut(&budget_key).expect("inserted above");
        if *left > 0 {
            *left -= 1;
            res.matched_alerts += 1;
            continue;
        }

        let merchant = crate::statement::merchant_of(&row.narration);
        let lower = row.narration.to_lowercase();
        let card_payment = card_id.is_some()
            && row.direction == Direction::Credit
            && (lower.contains("payment") || lower.contains("thank"));
        let category = categorize(merchant.as_deref().or(Some(row.narration.as_str())), row.direction, card_payment, &rules);
        if category == "uncategorized" {
            res.uncategorized += 1;
        }
        let raw: String = row.narration.chars().take(500).collect();
        sqlx::query(
            "INSERT INTO transactions
                (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, card_id, account_id, last4, source, raw_text, dedupe_key)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'paste', ?, ?)",
        )
        .bind(nanoid!())
        .bind(&user_id)
        .bind(&on)
        .bind(dir)
        .bind(row.amount_paise)
        .bind(&merchant)
        .bind(&category)
        .bind(&card_id)
        .bind(&account_id)
        .bind(&last4)
        .bind(&raw)
        .bind(&dedupe_key)
        .execute(&mut *tx)
        .await?;
        res.added += 1;
    }

    // The statement's own balance, from its newest line, becomes the account balance unless a newer one exists.
    if let Some(acct) = &account_id {
        let newest = parsed.rows.iter().enumerate().filter(|(_, r)| r.balance_paise.is_some()).max_by_key(|(i, r)| (r.date, *i));
        if let Some((_, r)) = newest {
            let bal = r.balance_paise.expect("filtered");
            let on = r.date.to_string();
            let done = sqlx::query(
                "UPDATE accounts SET balance_paise = ?, balance_set_on = ?, updated_at = CURRENT_TIMESTAMP
                 WHERE account_id = ? AND user_id = ? AND (balance_set_on IS NULL OR balance_set_on <= ?)",
            )
            .bind(bal)
            .bind(&on)
            .bind(acct)
            .bind(&user_id)
            .bind(&on)
            .execute(&mut *tx)
            .await?
            .rows_affected();
            if done == 1 {
                let name: String = sqlx::query_scalar("SELECT name FROM accounts WHERE account_id = ?").bind(acct).fetch_one(&mut *tx).await?;
                res.balance_updated = Some(BalanceUpdated { account_name: name, balance_paise: bal });
            }
        }
    }
    tx.commit().await?;
    Ok(Json(res))
}
