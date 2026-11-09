//! `GET /api/export`: everything that's yours, as one JSON file to keep or move elsewhere.
//! Dates data is included only for your couple, and your partner's own money never is.

use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    Extension,
};
use serde_json::{json, Map, Value};
use sqlx::{Column, Row, TypeInfo};

use crate::{
    handlers::{common::{today_local, AppError}, couple::couple_of},
    models::AppState,
};

/// Columns that are secrets or links that grant access. They never leave in an export, whatever
/// table they sit in (envelope share tokens, reminder feed tokens, hashes of keys and codes).
const SECRET_COLUMNS: &[&str] = &["share_token", "token", "key_hash", "code_hash", "user_password"];

/// Rows of `sql` (bound to `id`) as JSON objects, whatever the columns.
async fn rows(db: &sqlx::SqlitePool, sql: &str, id: &str) -> Result<Vec<Value>, AppError> {
    let out = sqlx::query(sql).bind(id).fetch_all(db).await?;
    Ok(out
        .iter()
        .map(|r| {
            let mut m = Map::new();
            for (i, c) in r.columns().iter().enumerate() {
                let v = match c.type_info().name() {
                    "INTEGER" | "BOOLEAN" => r.try_get::<Option<i64>, _>(i).ok().flatten().map(Value::from),
                    "REAL" => r.try_get::<Option<f64>, _>(i).ok().flatten().map(Value::from),
                    _ => r
                        .try_get::<Option<String>, _>(i)
                        .ok()
                        .flatten()
                        .map(Value::from)
                        .or_else(|| r.try_get::<Option<i64>, _>(i).ok().flatten().map(Value::from)),
                };
                if SECRET_COLUMNS.contains(&c.name()) {
                    continue;
                }
                m.insert(c.name().to_string(), v.unwrap_or(Value::Null));
            }
            Value::Object(m)
        })
        .collect())
}

pub async fn export(State(state): State<AppState>, Extension((user_id, _)): Extension<(String, String)>) -> Result<Response, AppError> {
    let db = &state.db;
    let mut out = Map::new();
    let mine: &[(&str, &str)] = &[
        ("accounts", "SELECT * FROM accounts WHERE user_id = ?"),
        ("balance_history", "SELECT h.* FROM balance_history h JOIN accounts a ON a.account_id = h.account_id WHERE a.user_id = ?"),
        ("cards", "SELECT * FROM cards WHERE user_id = ?"),
        ("card_snapshots", "SELECT s.* FROM card_snapshots s JOIN cards c ON c.card_id = s.card_id WHERE c.user_id = ?"),
        ("card_emis", "SELECT e.* FROM card_emis e JOIN cards c ON c.card_id = e.card_id WHERE c.user_id = ?"),
        ("income_sources", "SELECT * FROM income_sources WHERE user_id = ?"),
        ("commitments", "SELECT * FROM commitments WHERE user_id = ?"),
        ("moves", "SELECT * FROM moves WHERE user_id = ?"),
        ("card_events", "SELECT e.* FROM card_events e JOIN cards c ON c.card_id = e.card_id WHERE c.user_id = ?"),
        ("card_running_state", "SELECT s.* FROM card_running_state s JOIN cards c ON c.card_id = s.card_id WHERE c.user_id = ?"),
        ("deferred_payment_batches", "SELECT * FROM deferred_payment_batches WHERE user_id = ?"),
        ("deferred_payment_items", "SELECT i.* FROM deferred_payment_items i JOIN deferred_payment_batches b ON b.batch_id = i.batch_id WHERE b.user_id = ?"),
        ("pending_charges", "SELECT p.* FROM pending_charges p JOIN cards c ON c.card_id = p.card_id WHERE c.user_id = ?"),
        ("auto_postings", "SELECT * FROM auto_postings WHERE user_id = ?"),
        ("dismissed_recurring", "SELECT * FROM dismissed_recurring WHERE user_id = ?"),
        ("statement_imports", "SELECT * FROM statement_imports WHERE user_id = ?"),
        ("statement_profiles", "SELECT * FROM statement_profiles WHERE user_id = ?"),
        ("tax_entries", "SELECT * FROM tax_entries WHERE user_id = ?"),
        ("life_profiles", "SELECT * FROM life_profiles WHERE user_id = ?"),
        // Everything, the original alert text (`raw_text`) included.
        ("transactions", "SELECT * FROM transactions WHERE user_id = ?"),
        ("merchant_rules", "SELECT * FROM merchant_rules WHERE user_id = ?"),
        ("goals", "SELECT * FROM goals WHERE user_id = ?"),
        ("goal_contributions", "SELECT c.* FROM goal_contributions c JOIN goals g ON g.goal_id = c.goal_id WHERE g.user_id = ?"),
        ("settings", "SELECT * FROM user_settings WHERE user_id = ?"),
        ("checkins", "SELECT * FROM checkins WHERE user_id = ?"),
        ("diet_habits", "SELECT * FROM diet_habits WHERE user_id = ?"),
        ("diet_weeks", "SELECT * FROM diet_weeks WHERE user_id = ?"),
        ("diet_week_habits", "SELECT * FROM diet_week_habits WHERE user_id = ?"),
        ("health_marks", "SELECT * FROM health_marks WHERE user_id = ?"),
        ("people", "SELECT * FROM people WHERE user_id = ?"),
        ("person_dates", "SELECT * FROM person_dates WHERE user_id = ?"),
        ("touches", "SELECT * FROM touches WHERE user_id = ?"),
        ("gifts", "SELECT * FROM gifts WHERE user_id = ?"),
        ("gift_seasons", "SELECT * FROM gift_seasons WHERE user_id = ?"),
    ];
    for (name, sql) in mine {
        out.insert((*name).into(), Value::Array(rows(db, sql, &user_id).await?));
    }
    if let Some((couple, who)) = couple_of(db, &user_id).await? {
        // A surprise still sealed for you stays out of your copy too.
        let dated = format!(
            "SELECT * FROM joy_history WHERE couple_id = ? AND NOT (surprise = 1 AND revealed = 0 AND COALESCE(drawn_by, -1) <> {who})"
        );
        // A slip carrying a surprise still sealed for you stays out of your copy as well.
        let slips = format!(
            "SELECT * FROM joy_activities WHERE couple_id = ? AND id NOT IN (SELECT activity_id FROM joy_history WHERE surprise = 1 AND revealed = 0 AND COALESCE(drawn_by, -1) <> {who})"
        );
        // Notes, tasks and photo records of a date still sealed for you stay out too.
        let sealed = format!(
            "SELECT id FROM joy_history WHERE surprise = 1 AND revealed = 0 AND COALESCE(drawn_by, -1) <> {who}"
        );
        let memories = format!(
            "SELECT m.* FROM joy_memories m JOIN joy_history h ON h.id = m.history_id WHERE h.couple_id = ? AND m.history_id NOT IN ({sealed})"
        );
        let tasks = format!(
            "SELECT t.* FROM joy_tasks t JOIN joy_history h ON h.id = t.history_id WHERE h.couple_id = ? AND t.history_id NOT IN ({sealed})"
        );
        let photos = format!(
            "SELECT p.* FROM joy_history_photos p JOIN joy_history h ON h.id = p.history_id WHERE h.couple_id = ? AND p.history_id NOT IN ({sealed})"
        );
        let dates: &[(&str, &str)] = &[
            ("memories", &memories),
            ("tasks", &tasks),
            ("date_photos", &photos),
            ("checkins", "SELECT * FROM joy_checkins WHERE couple_id = ?"),
            ("month_budget", "SELECT * FROM joy_month_budget WHERE couple_id = ?"),
            ("settings", "SELECT * FROM joy_settings WHERE couple_id = ?"),
            ("vetoes", "SELECT v.* FROM joy_vetoes v JOIN joy_activities a ON a.id = v.activity_id WHERE a.couple_id = ?"),
            ("pairs", "SELECT p.* FROM joy_pairs p JOIN joy_activities a ON a.id = p.a_id WHERE a.couple_id = ?"),
            ("slips", &slips),
            ("dates", &dated),
            ("other_spending", "SELECT * FROM joy_other_spending WHERE couple_id = ?"),
            ("savings", "SELECT * FROM joy_savings WHERE couple_id = ?"),
            ("regulars", "SELECT * FROM joy_regulars WHERE couple_id = ?"),
            ("nights", "SELECT v.* FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id WHERE r.couple_id = ?"),
            ("settlements", "SELECT * FROM joy_settlements WHERE couple_id = ?"),
        ];
        let mut d = Map::new();
        for (name, sql) in dates {
            d.insert((*name).into(), Value::Array(rows(db, sql, &couple).await?));
        }
        out.insert("dates".into(), Value::Object(d));
    }
    let body = json!({ "exported_on": today_local().to_string(), "data": out });
    let name = format!("attachment; filename=\"joy-{}.json\"", today_local());
    Ok((
        [(header::CONTENT_TYPE, "application/json".to_string()), (header::CONTENT_DISPOSITION, name), (header::CACHE_CONTROL, "no-store".to_string())],
        body.to_string(),
    )
        .into_response())
}
