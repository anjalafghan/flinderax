use std::sync::Arc;
use serde::{Deserialize, Serialize};

use redis::aio::ConnectionManager;
use sqlx::FromRow;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub redis: Option<ConnectionManager>,
    pub paseto_key: Arc<rusty_paseto::prelude::PasetoSymmetricKey<rusty_paseto::core::V4, rusty_paseto::core::Local>>,
}

#[derive(serde::Deserialize, Clone)]
pub struct CreateUserPayload {
    pub user_name: String,
    pub user_password: String,
    pub user_role: String,
}
#[derive(serde::Serialize)]
pub struct GetUserResponse {
    pub user_id: String,
    pub user_name: String,
    pub user_role: String,
}

#[derive(FromRow, Debug, serde::Serialize)]
pub struct GetUsers {
    pub user_id: String,
    pub user_name: String,
    pub user_password: String,
    pub user_role: String,
}
#[derive(serde::Serialize)]
pub struct CreateUserResponse {
    pub status: bool,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub user_name: String,
    pub user_password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_at: i64,
}

#[derive(Deserialize)]
pub struct CreateCardPayload {
    pub card_name: String,
    pub card_bank: String,
    pub card_primary_color: (u8, u8, u8),
    pub card_secondary_color: (u8, u8, u8),
    pub last_4_digits: Option<String>,
    pub statement_day: Option<i64>,
    pub due_day: Option<i64>,
    pub credit_limit_paise: Option<i64>,
}
#[derive(Serialize)]
pub struct CardResponse {
    pub card_id: String,
    pub status: bool,
}

#[derive(Deserialize)]
pub struct DeleteCardPayload {
    pub card_id: String,
}

#[derive(Deserialize)]
pub struct UpdateCardPayload {
    pub card_id: String,
    pub card_name: String,
    pub card_bank: String,
    pub card_primary_color: (u8, u8, u8),
    pub card_secondary_color: (u8, u8, u8),
    pub last_4_digits: Option<String>,
    /// Omitted fields keep their stored value.
    pub statement_day: Option<i64>,
    pub due_day: Option<i64>,
    pub credit_limit_paise: Option<i64>,
}

#[derive(Deserialize)]
pub struct GetCardForUser {
    pub card_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ShowGetCardResponse {
    pub card_id: String,
    pub card_name: String,
    pub card_bank: String,
    pub card_primary_color: (u8, u8, u8),
    pub card_secondary_color: (u8, u8, u8),
    pub last_4_digits: Option<String>,
    pub last_total_due: Option<f32>,
    pub last_delta: Option<f32>,
    pub statement_day: Option<i64>,
    pub due_day: Option<i64>,
    pub credit_limit_paise: Option<i64>,
}
#[derive(Deserialize)]
pub struct InsertTransactionPayload {
    pub card_id: String,
    pub amount_due: f32,
}

#[derive(Serialize)]
pub struct InsertTransactionResponse {
    pub transaction_id: String,
    pub amount_due: f32,
    pub status: bool,
}

#[derive(Deserialize, Serialize)]
pub struct CardTransactionHistory {
    pub transaction_id: String,
    pub total_due_input: f32,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct DeleteTransactionResponse {
    pub transaction_id: String,
    pub status: bool,
}

#[derive(Deserialize)]
pub struct GetHistoryPayload {
    pub card_id: String,
}

#[derive(Deserialize)]
pub struct ResetTransactionsPayload {
    pub card_id: String,
}

#[derive(Deserialize)]
pub struct DeleteTransactionPayload {
    pub card_id: String,
    pub transaction_id: String,
}

#[derive(Deserialize)]
pub struct DeferUpdatePayload {
    pub card_id: String,
    pub amount: f32,
}

#[derive(Serialize)]
pub struct DeferUpdateResponse {
    pub batch_id: String,
    pub item_id: String,
    pub total_pending: f32,
    pub status: bool,
}

#[derive(Serialize)]
pub struct DeferredBatchStatus {
    pub batch_id: String,
    pub total_amount: f32,
    pub status: String,
    pub items: Vec<DeferredItemStatus>,
}

#[derive(Serialize)]
pub struct DeferredItemStatus {
    pub item_id: String,
    pub card_id: String,
    pub card_name: Option<String>,
    pub amount: f32,
}

#[derive(Deserialize)]
pub struct SettleDeferredPayload {
    pub batch_id: String,
}

#[derive(Serialize)]
pub struct SettleDeferredResponse {
    pub batch_id: String,
    pub total_settled: f32,
    pub status: bool,
}
