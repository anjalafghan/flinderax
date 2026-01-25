use serde::{Deserialize, Serialize};

use sqlx::FromRow;

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
    pub user_password: String,
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
}

#[derive(Deserialize)]
pub struct CreateCardPayload {
    pub card_name: String,
    pub card_bank: String,
    pub card_primary_color: (u8, u8, u8),
    pub card_secondary_color: (u8, u8, u8),
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
}

#[derive(Deserialize)]
pub struct GetCardForUser {
    pub card_id: String,
}

#[derive(Serialize)]
pub struct ShowGetCardResponse {
    pub card_id: String,
    pub card_name: String,
    pub card_bank: String,
    pub card_primary_color: (u8, u8, u8),
    pub card_secondary_color: (u8, u8, u8),
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
