use serde::{Serialize, Deserialize};

use sqlx::FromRow;

#[derive(FromRow, Debug, serde::Serialize)]
pub struct Users {
    user_id: String,
    user_name: String,
    user_role: String,
}
