use serde::{Deserialize, Serialize};

use sqlx::FromRow;

#[derive(FromRow, Debug, serde::Serialize)]
pub struct Users {
    user_id: String,
    user_name: String,
    user_role: String,
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
pub struct CreateCardResponse {
    pub card_id: String,
    pub status: bool,
}

#[derive(Deserialize)]
pub struct DeleteCardPayload {
    pub card_id: String,
}
#[derive(Serialize)]
pub struct DeleteCardResponse {
    pub card_id: String,
    pub status: bool,
}
