//! `POST /api/import`: loads a file made by `GET /api/export` back in, so the export is a real
//! backup. It replaces the caller's money and people data in one transaction: any problem and
//! nothing changes. Dates data (the couple's, shared with a partner) is not touched.

use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::Serialize;
use serde_json::Value;

use crate::{
    handlers::{
        common::AppError,
        export::USER_TABLES,
    },
    models::AppState,
};

#[derive(Serialize)]
pub struct Imported {
    pub status: bool,
    pub rows: usize,
}

fn refused(msg: impl Into<String>) -> AppError {
    AppError(StatusCode::UNPROCESSABLE_ENTITY, msg.into())
}

/// The export key `settings` is the `user_settings` table; every other key is its table's name.
fn table_of(key: &str) -> &str {
    if key == "settings" { "user_settings" } else { key }
}

/// The statement that removes what the export query would have read for this person.
fn wipe_sql(table: &str, select: &str) -> String {
    // "SELECT * FROM t WHERE user_id = ?" -> delete it directly.
    if let Some(rest) = select.strip_prefix("SELECT * FROM ") {
        return format!("DELETE FROM {rest}");
    }
    // "SELECT c.* FROM child c JOIN parent p ON ... WHERE p.user_id = ?" -> delete the child rows it reads.
    let alias = select.strip_prefix("SELECT ").and_then(|r| r.split_once(".*")).map_or("x", |(a, _)| a);
    let rest = select.split_once(" FROM ").map_or("", |(_, r)| r);
    format!("DELETE FROM {table} WHERE rowid IN (SELECT {alias}.rowid FROM {rest})")
}

pub async fn import(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
    Json(file): Json<Value>,
) -> Result<Json<Imported>, AppError> {
    let data = file.get("data").and_then(Value::as_object).ok_or_else(|| refused("That doesn't look like a joy export."))?;
    for (key, rows) in data {
        if key == "dates" {
            continue; // the couple's data is shared with a partner, so it is never replaced from one person's file
        }
        if !USER_TABLES.iter().any(|(k, _)| k == key) {
            return Err(refused(format!("Don't know what “{key}” is.")));
        }
        if !rows.as_array().is_some_and(|r| r.iter().all(Value::is_object)) {
            return Err(refused(format!("“{key}” should be a list of rows.")));
        }
    }

    let mut tx = state.db.begin().await?;
    sqlx::query("PRAGMA defer_foreign_keys = ON").execute(&mut *tx).await?;

    // Children first.
    for (key, select) in USER_TABLES.iter().rev() {
        if data.contains_key(*key) {
            sqlx::query(&wipe_sql(table_of(key), select)).bind(&user_id).execute(&mut *tx).await?;
        }
    }

    let mut total = 0;
    for (key, select) in USER_TABLES {
        let Some(rows) = data.get(*key).and_then(Value::as_array) else { continue };
        let table = table_of(key);
        // Loading a parent can make its triggers write child rows (an account records its first
        // balance); the file's own rows replace those.
        sqlx::query(&wipe_sql(table, select)).bind(&user_id).execute(&mut *tx).await?;
        let columns: Vec<String> = sqlx::query_scalar(&format!("SELECT name FROM pragma_table_info('{table}')")).fetch_all(&mut *tx).await?;
        for row in rows {
            let obj = row.as_object().expect("checked above");
            let names: Vec<&String> = columns.iter().filter(|c| obj.contains_key(c.as_str()) || c.as_str() == "user_id").collect();
            let sql = format!(
                "INSERT INTO {table} ({}) VALUES ({})",
                names.iter().map(|n| format!("\"{n}\"")).collect::<Vec<_>>().join(", "),
                vec!["?"; names.len()].join(", ")
            );
            let mut q = sqlx::query(&sql);
            for name in &names {
                if name.as_str() == "user_id" {
                    q = q.bind(user_id.clone());
                    continue;
                }
                q = match &obj[name.as_str()] {
                    Value::Null => q.bind(None::<String>),
                    Value::Bool(b) => q.bind(i64::from(*b)),
                    Value::Number(n) => match n.as_i64() {
                        Some(i) => q.bind(i),
                        None => q.bind(n.as_f64().unwrap_or_default()),
                    },
                    Value::String(s) => q.bind(s.clone()),
                    _ => return Err(refused(format!("“{key}”.{name} holds something that isn't a value."))),
                };
            }
            q.execute(&mut *tx).await.map_err(|e| match e {
                sqlx::Error::Database(ref d) if d.is_unique_violation() => {
                    AppError(StatusCode::CONFLICT, format!("A row in “{key}” already belongs to another account."))
                }
                sqlx::Error::Database(_) => refused(format!("A row in “{key}” couldn't be loaded.")),
                e => e.into(),
            })?;
            total += 1;
        }
    }
    tx.commit().await?;
    Ok(Json(Imported { status: true, rows: total }))
}
