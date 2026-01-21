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
