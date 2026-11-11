//! The transfer ledger: "I moved the money", "it was already there", "skip this month".
//! The app can't see the bank, so these clicks are how it learns what has been done.

use std::collections::HashMap;

use axum::{extract::State, Extension, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    handlers::{
        account::ensure_account_owned,
        card::ensure_card_owned,
        common::{bad_request, not_found, AppError},
    },
    models::AppState,
};

type User = Extension<(String, String)>;

#[derive(Deserialize)]
pub struct RecordMove {
    /// `card_bill` | `allocation`
    pub kind: String,
    /// card_bill: `<card_id>:<bill due date>`; allocation: the pay period's start date
    pub key: String,
    pub amount_paise: i64,
    /// `moved` (balances are adjusted) | `already_covered` | `skipped`
    pub mode: String,
    pub from_account_id: Option<String>,
    pub to_account_id: Option<String>,
}

#[derive(Serialize)]
pub struct MoveId {
    pub move_id: String,
    pub status: bool,
}

#[derive(Serialize)]
pub struct MoveRow {
    pub move_id: String,
    pub kind: String,
    pub key: String,
    pub amount_paise: i64,
    pub mode: String,
    pub from_account_id: Option<String>,
    pub to_account_id: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct UndoMove {
    pub move_id: String,
}

#[derive(Serialize)]
pub struct Done {
    pub status: bool,
}

/// `(covered, skipped)` totals per ledger key for one user.
pub async fn coverage(db: &sqlx::SqlitePool, user_id: &str) -> Result<HashMap<String, (i64, i64)>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT move_key, mode, SUM(amount_paise) AS "total!: i64" FROM moves WHERE user_id = ? GROUP BY move_key, mode"#,
        user_id
    )
    .fetch_all(db)
    .await?;
    let mut out: HashMap<String, (i64, i64)> = HashMap::new();
    for r in rows {
        let e = out.entry(r.move_key).or_default();
        if r.mode == "skipped" {
            e.1 += r.total;
        } else {
            e.0 += r.total;
        }
    }
    Ok(out)
}

pub async fn record(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<RecordMove>,
) -> Result<Json<MoveId>, AppError> {
    if !matches!(p.kind.as_str(), "card_bill" | "allocation") {
        return Err(bad_request("kind must be card_bill or allocation"));
    }
    if !matches!(p.mode.as_str(), "moved" | "already_covered" | "skipped") {
        return Err(bad_request("mode must be moved, already_covered or skipped"));
    }
    crate::validate::positive_amount(p.amount_paise).map_err(bad_request)?;
    if p.kind == "card_bill" {
        let card_id = p.key.split(':').next().unwrap_or_default();
        ensure_card_owned(&state.db, card_id, &user_id).await?;
    }
    if p.mode == "skipped" && p.kind == "card_bill" {
        return Err(bad_request("card bills can't be skipped"));
    }
    ensure_account_owned(&state.db, &p.from_account_id, &user_id).await?;
    ensure_account_owned(&state.db, &p.to_account_id, &user_id).await?;
    let moving = p.mode == "moved";
    if moving && (p.from_account_id.is_none() || p.to_account_id.is_none()) {
        return Err(bad_request("a move needs from_account_id and to_account_id"));
    }

    let move_id = nanoid!();
    let mut tx = state.db.begin().await?;
    // A double-tap or a retried request records the same move twice; an identical move in the last
    // few seconds is that, so answer with the one already recorded instead of moving money again.
    let recent: Option<String> = sqlx::query_scalar(
        "SELECT move_id FROM moves
         WHERE user_id = ? AND kind = ? AND move_key = ? AND mode = ? AND amount_paise = ?
           AND from_account_id IS ? AND to_account_id IS ?
           AND created_at >= datetime('now', '-10 seconds')
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&user_id)
    .bind(&p.kind)
    .bind(&p.key)
    .bind(&p.mode)
    .bind(p.amount_paise)
    .bind(&p.from_account_id)
    .bind(&p.to_account_id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(move_id) = recent {
        return Ok(Json(MoveId { move_id, status: true }));
    }
    sqlx::query!(
        "INSERT INTO moves (move_id, user_id, kind, move_key, amount_paise, mode, from_account_id, to_account_id)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        move_id,
        user_id,
        p.kind,
        p.key,
        p.amount_paise,
        p.mode,
        p.from_account_id,
        p.to_account_id
    )
    .execute(&mut *tx)
    .await?;
    if moving {
        adjust(&mut tx, &p.from_account_id, -p.amount_paise).await?;
        adjust(&mut tx, &p.to_account_id, p.amount_paise).await?;
    }
    tx.commit().await?;
    Ok(Json(MoveId { move_id, status: true }))
}

async fn adjust(tx: &mut sqlx::SqliteConnection, account: &Option<String>, delta: i64) -> Result<(), AppError> {
    if let Some(id) = account {
        sqlx::query!(
            "UPDATE accounts SET balance_paise = balance_paise + ?, updated_at = CURRENT_TIMESTAMP WHERE account_id = ?",
            delta,
            id
        )
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

pub async fn undo(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<UndoMove>,
) -> Result<Json<Done>, AppError> {
    let mut tx = state.db.begin().await?;
    let row = sqlx::query!(
        "SELECT amount_paise, mode, from_account_id, to_account_id FROM moves WHERE move_id = ? AND user_id = ?",
        p.move_id,
        user_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| not_found("Move not found"))?;
    sqlx::query!("DELETE FROM moves WHERE move_id = ?", p.move_id).execute(&mut *tx).await?;
    if row.mode == "moved" {
        adjust(&mut tx, &row.from_account_id, row.amount_paise).await?;
        adjust(&mut tx, &row.to_account_id, -row.amount_paise).await?;
    }
    tx.commit().await?;
    Ok(Json(Done { status: true }))
}

pub async fn list(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
) -> Result<Json<Vec<MoveRow>>, AppError> {
    let rows = sqlx::query_as!(
        MoveRow,
        r#"SELECT move_id AS "move_id!", kind, move_key AS "key", amount_paise, mode, from_account_id, to_account_id,
                  created_at AS "created_at: String"
           FROM moves WHERE user_id = ? ORDER BY created_at DESC, rowid DESC LIMIT 50"#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}
