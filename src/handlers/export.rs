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
        ("transactions", "SELECT txn_id, occurred_on, direction, amount_paise, merchant, category, card_id, account_id, last4, source, created_at FROM transactions WHERE user_id = ?"),
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
        let dates: &[(&str, &str)] = &[
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
