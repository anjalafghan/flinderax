//! Payees with uncategorised spending. `list` shows the payees that carry the most unexplained
//! spend; `categorize` gives one payee a category across all of its uncategorised rows, and can
//! remember the choice for future imports.
//!
//! A row's payee is its merchant, or the payee read from its narration when the merchant is
//! blank (statement imports). Payees compare case-insensitively with runs of spaces collapsed.

use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Extension, Json,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use time::Duration;

use crate::{
    alerts::{self, Direction, CATEGORIES},
    handlers::common::{bad_request, today_local, AppError},
    models::AppState,
    narration,
};

type User = Extension<(String, String)>;

const DEFAULT_DAYS: i64 = 365;
const MAX_DAYS: i64 = 3650;
const DEFAULT_LIMIT: i64 = 40;
const MAX_LIMIT: i64 = 200;
/// A payee is treated as a person only when its typical payment is at least Rs 500.
const PERSON_MIN_MEDIAN_PAISE: i64 = 50_000;

/// Words that mark a business name. A name containing one is never suggested as a person.
const BUSINESS_WORDS: &[&str] = &[
    "pvt", "ltd", "llp", "limited", "private", "store", "stores", "shop", "traders", "trading", "enterprises",
    "services", "solutions", "technologies", "tech", "industries", "company", "co", "corp", "corporation",
    "agency", "associates", "hospital", "clinic", "medical", "pharma", "bank", "fund", "trust", "foundation",
    "academy", "school", "college", "institute", "university", "hotel", "restaurant", "cafe", "foods", "mart",
    "motors", "garage", "centre", "center", "digital", "online", "mobile", "electronics", "fashion", "sons",
    "bros", "and", "group", "society", "association", "club", "gym", "salon", "tailors", "dairy", "kitchen",
];

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub days: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct PayeeList {
    /// Every uncategorised debit in the window, including payees cut off by `limit`.
    pub total_uncategorized_paise: i64,
    /// The money in the returned payees.
    pub covered_paise: i64,
    pub payees: Vec<PayeeRow>,
}

#[derive(Serialize)]
pub struct PayeeRow {
    pub payee: String,
    pub count: i64,
    pub paise: i64,
    pub last_on: String,
    /// This payee's share of `total_uncategorized_paise`, in basis points (1/100 of a percent).
    pub share_bps: i64,
    pub suggestion: Option<String>,
    /// One raw narration for the payee, with 12-digit reference numbers masked.
    pub sample: String,
}

/// The key a row or a payee name is grouped and matched by. `None` when there is no name.
fn payee_key(merchant: Option<&str>, raw_text: Option<&str>) -> Option<String> {
    let name = merchant
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .map(str::to_string)
        .or_else(|| raw_text.and_then(|r| narration::parse(r).payee))?;
    let key = name.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    (!key.is_empty()).then_some(key)
}

fn mask_references(raw: &str) -> String {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\d{12,}").expect("valid regex"))
        .replace_all(raw, "XXXXXXXXXXXX")
        .into_owned()
}

/// Two to four alphabetic words, no business word, and a typical payment of Rs 500 or more.
fn looks_like_person(name: &str, median_paise: i64) -> bool {
    let words: Vec<&str> = name.split_whitespace().collect();
    (2..=4).contains(&words.len())
        && words.iter().all(|w| w.chars().all(|c| c.is_ascii_alphabetic()))
        && !words.iter().any(|w| BUSINESS_WORDS.contains(&w.to_ascii_lowercase().as_str()))
        && median_paise >= PERSON_MIN_MEDIAN_PAISE
}

/// A category from the keyword rules, else "people" for a person's name, else nothing.
fn suggest(name: &str, median_paise: i64) -> Option<String> {
    let category = alerts::categorize(Some(name), Direction::Debit, false, &[]);
    if category != "uncategorized" {
        Some(category)
    } else if looks_like_person(name, median_paise) {
        Some("people".into())
    } else {
        None
    }
}

/// `YYYY-MM-DD` for the date `days` before today.
fn days_ago(days: i64) -> String {
    let d = today_local() - Duration::days(days);
    format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
}

struct Group {
    /// The name as last seen (rows are read newest first).
    name: String,
    count: i64,
    paise: i64,
    last_on: String,
    sample: String,
    amounts: Vec<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Query(q): Query<ListQuery>,
) -> Result<Json<PayeeList>, AppError> {
    if q.status.as_deref().is_some_and(|s| s != "uncategorized") {
        return Err(bad_request("status must be 'uncategorized'"));
    }
    let days = q.days.unwrap_or(DEFAULT_DAYS).clamp(1, MAX_DAYS);
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT) as usize;

    // Locked rows stay uncategorised on purpose; categorize() will not touch them, so they are left out.
    let rows = sqlx::query(
        "SELECT occurred_on, amount_paise, merchant, raw_text FROM transactions
         WHERE user_id = ? AND direction = 'debit' AND category = 'uncategorized' AND category_locked = 0
           AND occurred_on >= ?
         ORDER BY occurred_on DESC, txn_id DESC",
    )
    .bind(&user_id)
    .bind(days_ago(days))
    .fetch_all(&state.db)
    .await?;

    let mut total = 0i64;
    let mut groups: HashMap<String, Group> = HashMap::new();
    for row in rows {
        let occurred_on: String = row.try_get("occurred_on")?;
        let paise: i64 = row.try_get("amount_paise")?;
        let merchant: Option<String> = row.try_get("merchant")?;
        let raw: Option<String> = row.try_get("raw_text")?;
        total += paise;

        let Some(key) = payee_key(merchant.as_deref(), raw.as_deref()) else { continue };
        let name = merchant
            .as_deref()
            .map(str::trim)
            .filter(|m| !m.is_empty())
            .map(str::to_string)
            .or_else(|| raw.as_deref().and_then(|r| narration::parse(r).payee))
            .unwrap_or_default();
        let sample = mask_references(raw.as_deref().or(merchant.as_deref()).unwrap_or_default());
        let g = groups.entry(key).or_insert_with(|| Group {
            name,
            count: 0,
            paise: 0,
            last_on: occurred_on.clone(),
            sample,
            amounts: Vec::new(),
        });
        g.count += 1;
        g.paise += paise;
        g.amounts.push(paise);
    }

    let mut groups: Vec<Group> = groups.into_values().collect();
    groups.sort_by(|a, b| b.paise.cmp(&a.paise).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    groups.truncate(limit);

    let mut covered = 0i64;
    let mut payees = Vec::with_capacity(groups.len());
    for mut g in groups {
        g.amounts.sort_unstable();
        let median = g.amounts[g.amounts.len() / 2];
        covered += g.paise;
        payees.push(PayeeRow {
            suggestion: suggest(&g.name, median),
            share_bps: if total > 0 { g.paise * 10_000 / total } else { 0 },
            payee: g.name,
            count: g.count,
            paise: g.paise,
            last_on: g.last_on,
            sample: g.sample,
        });
    }

    Ok(Json(PayeeList { total_uncategorized_paise: total, covered_paise: covered, payees }))
}

#[derive(Deserialize)]
pub struct CategorizePayee {
    pub payee: String,
    pub category: String,
    #[serde(default)]
    pub remember: bool,
}

#[derive(Serialize)]
pub struct Categorized {
    pub updated: u64,
}

pub async fn categorize(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<CategorizePayee>,
) -> Result<Json<Categorized>, AppError> {
    if !CATEGORIES.contains(&p.category.as_str()) {
        return Err(AppError(StatusCode::BAD_REQUEST, "unknown category".into()));
    }
    let key = payee_key(Some(p.payee.as_str()), None).ok_or_else(|| bad_request("payee is required"))?;

    let mut tx = state.db.begin().await?;
    let candidates = sqlx::query(
        "SELECT txn_id, merchant, raw_text FROM transactions
         WHERE user_id = ? AND category = 'uncategorized' AND category_locked = 0",
    )
    .bind(&user_id)
    .fetch_all(&mut *tx)
    .await?;

    let mut updated = 0u64;
    for row in candidates {
        let merchant: Option<String> = row.try_get("merchant")?;
        let raw: Option<String> = row.try_get("raw_text")?;
        if payee_key(merchant.as_deref(), raw.as_deref()).as_deref() != Some(key.as_str()) {
            continue;
        }
        let txn_id: String = row.try_get("txn_id")?;
        updated += sqlx::query(
            "UPDATE transactions SET category = ?
             WHERE user_id = ? AND txn_id = ? AND category = 'uncategorized' AND category_locked = 0",
        )
        .bind(&p.category)
        .bind(&user_id)
        .bind(&txn_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    }

    if p.remember {
        let pattern = alerts::normalize(&p.payee);
        if !pattern.is_empty() {
            sqlx::query(
                "INSERT INTO merchant_rules (user_id, pattern, category) VALUES (?, ?, ?)
                 ON CONFLICT(user_id, pattern) DO UPDATE SET category = excluded.category",
            )
            .bind(&user_id)
            .bind(&pattern)
            .bind(&p.category)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(Json(Categorized { updated }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('other', 'other', 'x', 'user')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32][..]),
        );
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    /// `AppError` has no `Debug`, so unwrap handler results by hand.
    fn unwrap_ok<T>(r: Result<Json<T>, AppError>) -> T {
        match r {
            Ok(Json(v)) => v,
            Err(e) => panic!("handler failed: {}", e.1),
        }
    }

    fn user(id: &str) -> Extension<(String, String)> {
        Extension((id.to_string(), "user".to_string()))
    }

    fn days_back(n: i64) -> String {
        days_ago(n)
    }

    struct Row<'a> {
        id: &'a str,
        user: &'a str,
        on: String,
        paise: i64,
        merchant: Option<&'a str>,
        raw: Option<&'a str>,
        category: &'a str,
        locked: i64,
        direction: &'a str,
    }

    async fn insert(db: &sqlx::SqlitePool, r: Row<'_>) {
        sqlx::query(
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, raw_text,
                                       category, category_locked, source, dedupe_key)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'sms', ?)",
        )
        .bind(r.id)
        .bind(r.user)
        .bind(r.on)
        .bind(r.direction)
        .bind(r.paise)
        .bind(r.merchant)
        .bind(r.raw)
        .bind(r.category)
        .bind(r.locked)
        .bind(format!("fp:{}", r.id))
        .execute(db)
        .await
        .unwrap();
    }

    fn debit<'a>(id: &'a str, user: &'a str, on: String, paise: i64, merchant: Option<&'a str>) -> Row<'a> {
        Row { id, user, on, paise, merchant, raw: None, category: "uncategorized", locked: 0, direction: "debit" }
    }

    async fn seed(db: &sqlx::SqlitePool) {
        let recent = days_back(10);
        let rows = vec![
            debit("r1", "u", recent.clone(), 80_000, Some("SPARSH LEELA RESTAUR")),
            debit("r2", "u", days_back(20), 120_000, Some("SPARSH LEELA RESTAUR")),
            debit("p1", "u", days_back(5), 60_000, Some("RAVI KUMAR")),
            debit("p2", "u", days_back(7), 50_000, Some("ravi   kumar")),
            // Blank merchant: the payee comes from the narration. Its reference is masked in the sample.
            Row {
                raw: Some("UPI-ASHA STORES-Q123456789@YBL-YESB0YBLUPI-612345678902-UPI"),
                ..debit("s1", "u", days_back(3), 3_000, None)
            },
            Row { category: "food", ..debit("done", "u", recent.clone(), 99_000, Some("SWIGGY")) },
            Row { locked: 1, ..debit("lock", "u", recent.clone(), 70_000, Some("LOCKED SHOP")) },
            Row { direction: "credit", ..debit("cr", "u", recent.clone(), 40_000, Some("SALARY CO")) },
            debit("old", "u", days_back(400), 900_000, Some("OLD PAYEE")),
            debit("theirs", "other", recent.clone(), 500_000, Some("RAVI KUMAR")),
        ];
        for r in rows {
            insert(db, r).await;
        }
    }

    async fn category_of(db: &sqlx::SqlitePool, id: &str) -> String {
        sqlx::query_scalar("SELECT category FROM transactions WHERE txn_id = ?").bind(id).fetch_one(db).await.unwrap()
    }

    #[tokio::test]
    async fn listing_groups_sorts_and_does_the_share_maths() {
        let state = test_state().await;
        seed(&state.db).await;
        let list = unwrap_ok(list(State(state.clone()), user("u"), Query(ListQuery { status: None, days: None, limit: None })).await);

        // Uncategorised debits in the window for "u": 80k + 120k + 60k + 50k + 3k = 313k. Not the
        // food, locked, credit, old or other-user rows.
        assert_eq!(list.total_uncategorized_paise, 313_000);
        let names: Vec<(&str, i64, i64)> = list.payees.iter().map(|p| (p.payee.as_str(), p.count, p.paise)).collect();
        assert_eq!(
            names,
            vec![("SPARSH LEELA RESTAUR", 2, 200_000), ("RAVI KUMAR", 2, 110_000), ("ASHA STORES", 1, 3_000)]
        );
        assert_eq!(list.covered_paise, 313_000);
        // 200_000 / 313_000 = 6389.3 bps; 110_000 / 313_000 = 3514.1 bps; both round down.
        assert_eq!(list.payees[0].share_bps, 6389);
        assert_eq!(list.payees[1].share_bps, 3514);
        assert_eq!(list.payees[0].last_on, days_back(10));
        assert!(list.payees[2].sample.contains("XXXXXXXXXXXX"));
        assert!(!list.payees[2].sample.contains("612345678902"));
    }

    #[tokio::test]
    async fn limit_cuts_payees_but_not_the_total() {
        let state = test_state().await;
        seed(&state.db).await;
        let list = unwrap_ok(list(State(state.clone()), user("u"), Query(ListQuery { status: None, days: None, limit: Some(1) })).await);
        assert_eq!(list.payees.len(), 1);
        assert_eq!(list.total_uncategorized_paise, 313_000);
        assert_eq!(list.covered_paise, 200_000);
    }

    #[tokio::test]
    async fn suggestions_name_a_restaurant_and_a_person() {
        let state = test_state().await;
        seed(&state.db).await;
        let list = unwrap_ok(list(State(state.clone()), user("u"), Query(ListQuery { status: None, days: None, limit: None })).await);
        let by_name = |n: &str| list.payees.iter().find(|p| p.payee == n).unwrap();
        assert_eq!(by_name("SPARSH LEELA RESTAUR").suggestion.as_deref(), Some("food"));
        // Median of 60k and 50k payments is 60k (>= Rs 500), and the name is two alphabetic words.
        assert_eq!(by_name("RAVI KUMAR").suggestion.as_deref(), Some("people"));
        // A small payment to a person is not suggested.
        assert_eq!(by_name("ASHA STORES").suggestion, None);
    }

    #[test]
    fn person_heuristic_needs_words_size_and_no_business_word() {
        assert!(looks_like_person("RAVI KUMAR", 60_000));
        assert!(!looks_like_person("RAVI KUMAR", 49_999));
        assert!(!looks_like_person("RAVI", 60_000));
        assert!(!looks_like_person("RAVI KUMAR SHARMA SINGH SAHAB", 60_000));
        assert!(!looks_like_person("RAVI STORES", 60_000));
        assert!(!looks_like_person("RAVI K2", 60_000));
    }

    #[tokio::test]
    async fn categorize_updates_only_this_users_uncategorised_rows_for_the_payee() {
        let state = test_state().await;
        seed(&state.db).await;
        let done = unwrap_ok(categorize(
            State(state.clone()),
            user("u"),
            Json(CategorizePayee { payee: "  ravi KUMAR ".into(), category: "people".into(), remember: false }),
        ).await);
        assert_eq!(done.updated, 2);
        assert_eq!(category_of(&state.db, "p1").await, "people");
        assert_eq!(category_of(&state.db, "p2").await, "people");
        // Another user's row with the same payee is untouched.
        assert_eq!(category_of(&state.db, "theirs").await, "uncategorized");
        // Other payees are untouched.
        assert_eq!(category_of(&state.db, "r1").await, "uncategorized");
        // Already categorised rows keep their category.
        assert_eq!(category_of(&state.db, "done").await, "food");
    }

    #[tokio::test]
    async fn categorize_leaves_locked_rows_alone() {
        let state = test_state().await;
        insert(&state.db, Row { locked: 1, ..debit("locked-ravi", "u", days_back(2), 60_000, Some("RAVI KUMAR")) }).await;
        let done = unwrap_ok(categorize(
            State(state.clone()),
            user("u"),
            Json(CategorizePayee { payee: "RAVI KUMAR".into(), category: "people".into(), remember: false }),
        ).await);
        assert_eq!(done.updated, 0);
        assert_eq!(category_of(&state.db, "locked-ravi").await, "uncategorized");
    }

    #[tokio::test]
    async fn payee_from_the_narration_is_matched_when_the_merchant_is_blank() {
        let state = test_state().await;
        seed(&state.db).await;
        let done = unwrap_ok(categorize(
            State(state.clone()),
            user("u"),
            Json(CategorizePayee { payee: "ASHA STORES".into(), category: "groceries".into(), remember: false }),
        ).await);
        assert_eq!(done.updated, 1);
        assert_eq!(category_of(&state.db, "s1").await, "groceries");
    }

    #[tokio::test]
    async fn remember_creates_a_rule_that_later_imports_pick_up() {
        let state = test_state().await;
        seed(&state.db).await;
        unwrap_ok(categorize(
            State(state.clone()),
            user("u"),
            Json(CategorizePayee { payee: "RAVI KUMAR".into(), category: "people".into(), remember: true }),
        ).await);

        let rules: Vec<(String, String)> =
            sqlx::query_as("SELECT pattern, category FROM merchant_rules WHERE user_id = 'u'")
                .fetch_all(&state.db)
                .await
                .unwrap();
        assert_eq!(rules, vec![("ravi kumar".to_string(), "people".to_string())]);
        assert_eq!(alerts::categorize(Some("RAVI KUMAR"), Direction::Debit, false, &rules), "people");
        assert_eq!(alerts::categorize(Some("Ravi Kumar"), Direction::Debit, false, &rules), "people");
    }

    #[tokio::test]
    async fn invalid_category_is_a_bad_request() {
        let state = test_state().await;
        seed(&state.db).await;
        let err = match categorize(
            State(state.clone()),
            user("u"),
            Json(CategorizePayee { payee: "RAVI KUMAR".into(), category: "nonsense".into(), remember: true }),
        )
        .await
        {
            Ok(_) => panic!("an unknown category must be rejected"),
            Err(e) => e,
        };
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert_eq!(category_of(&state.db, "p1").await, "uncategorized");
    }

    #[tokio::test]
    async fn listing_then_categorising_the_listed_payee_updates_every_row_counted() {
        let state = test_state().await;
        seed(&state.db).await;
        let list = unwrap_ok(list(State(state.clone()), user("u"), Query(ListQuery { status: None, days: None, limit: None })).await);
        for row in &list.payees {
            let done = unwrap_ok(categorize(
                State(state.clone()),
                user("u"),
                Json(CategorizePayee { payee: row.payee.clone(), category: "other".into(), remember: false }),
            ).await);
            assert_eq!(done.updated, row.count as u64, "payee {}", row.payee);
        }
    }
}
