//! Money accounts: bank, cash, FD, RD. Adding a new kind is a CHECK-constraint change only.

use axum::{extract::State, Extension, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

use crate::{
    handlers::common::{bad_request, not_found, AppError},
    models::AppState,
};

type User = Extension<(String, String)>;

#[derive(Serialize)]
pub struct AccountRow {
    pub account_id: String,
    pub name: String,
    pub kind: String,
    pub balance_paise: i64,
    pub locked: bool,
    pub interest_bps: Option<i64>,
    pub maturity_date: Option<String>,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct AccountIdResponse {
    pub account_id: String,
    pub status: bool,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: bool,
}

#[derive(Deserialize)]
pub struct CreateAccountPayload {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub balance_paise: i64,
    /// Defaults to locked for `fd` / `rd`, unlocked otherwise.
    pub locked: Option<bool>,
    pub interest_bps: Option<i64>,
    pub maturity_date: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAccountPayload {
    pub account_id: String,
    pub name: String,
    pub kind: String,
    pub locked: bool,
    pub interest_bps: Option<i64>,
    pub maturity_date: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateBalancePayload {
    pub account_id: String,
    pub balance_paise: i64,
}

#[derive(Deserialize)]
pub struct DeleteAccountPayload {
    pub account_id: String,
}

fn validate(name: &str, kind: &str, maturity: &Option<String>) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(bad_request("name is required"));
    }
    if !matches!(kind, "bank" | "fd" | "rd" | "cash") {
        return Err(bad_request("kind must be one of bank, fd, rd, cash"));
    }
    if let Some(m) = maturity {
        Date::parse(m, format_description!("[year]-[month]-[day]"))
            .map_err(|_| bad_request("maturity_date must look like 2027-03-31"))?;
    }
    Ok(())
}

pub async fn create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<CreateAccountPayload>,
) -> Result<Json<AccountIdResponse>, AppError> {
    validate(&p.name, &p.kind, &p.maturity_date)?;
    let account_id = nanoid!();
    let locked = p.locked.unwrap_or(matches!(p.kind.as_str(), "fd" | "rd"));
    sqlx::query!(
        "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise, locked, interest_bps, maturity_date)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        account_id,
        user_id,
        p.name,
        p.kind,
        p.balance_paise,
        locked,
        p.interest_bps,
        p.maturity_date
    )
    .execute(&state.db)
    .await?;
    Ok(Json(AccountIdResponse { account_id, status: true }))
}

pub async fn update(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<UpdateAccountPayload>,
) -> Result<Json<StatusResponse>, AppError> {
    validate(&p.name, &p.kind, &p.maturity_date)?;
    let res = sqlx::query!(
        "UPDATE accounts SET name = ?, kind = ?, locked = ?, interest_bps = ?, maturity_date = ?
         WHERE account_id = ? AND user_id = ?",
        p.name,
        p.kind,
        p.locked,
        p.interest_bps,
        p.maturity_date,
        p.account_id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Account not found"));
    }
    Ok(Json(StatusResponse { status: true }))
}

pub async fn update_balance(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<UpdateBalancePayload>,
) -> Result<Json<StatusResponse>, AppError> {
    let res = sqlx::query!(
        "UPDATE accounts SET balance_paise = ?, updated_at = CURRENT_TIMESTAMP WHERE account_id = ? AND user_id = ?",
        p.balance_paise,
        p.account_id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Account not found"));
    }
    Ok(Json(StatusResponse { status: true }))
}

pub async fn list(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
) -> Result<Json<Vec<AccountRow>>, AppError> {
    let rows = sqlx::query_as!(
        AccountRow,
        r#"SELECT account_id AS "account_id!", name, kind, balance_paise,
                  locked AS "locked: bool", interest_bps, maturity_date,
                  updated_at AS "updated_at: String"
           FROM accounts WHERE user_id = ? ORDER BY rowid"#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<DeleteAccountPayload>,
) -> Result<Json<StatusResponse>, AppError> {
    // commitments / income that pointed here keep existing with account_id = NULL (ON DELETE SET NULL)
    let res = sqlx::query!(
        "DELETE FROM accounts WHERE account_id = ? AND user_id = ?",
        p.account_id,
        user_id
    )
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Account not found"));
    }
    Ok(Json(StatusResponse { status: true }))
}

/// 400s unless `account_id` (when given) belongs to the user.
pub async fn ensure_account_owned(
    db: &sqlx::SqlitePool,
    account_id: &Option<String>,
    user_id: &str,
) -> Result<(), AppError> {
    if let Some(id) = account_id {
        let found = sqlx::query!(
            "SELECT account_id FROM accounts WHERE account_id = ? AND user_id = ?",
            id,
            user_id
        )
        .fetch_optional(db)
        .await?;
        if found.is_none() {
            return Err(bad_request("account_id does not exist"));
        }
    }
    Ok(())
}
