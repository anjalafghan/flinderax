use axum::{
    extract::State,
    http::{ StatusCode},
    Extension, Json,
};
use nanoid::nanoid;
use sqlx::SqlitePool;

use crate::{
    handlers::{color, common::AppError},
    models::{CreateCardPayload, CreateCardResponse},
};

pub async fn create_card(
    State(pool): State<SqlitePool>,
    Extension(user_id): Extension<String>,
    Json(card_details): Json<CreateCardPayload>,
) -> Result<Json<CreateCardResponse>, AppError> {

    let card_id = nanoid!();
    let primary_color = color::pack(card_details.card_primary_color);
    let secondary_color = color::pack(card_details.card_secondary_color);


    sqlx::query!(
        "INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color) VALUES (?, ?, ?, ?, ?, ?)",
        card_id,
        user_id, 
        card_details.card_name,
        card_details.card_bank,
        primary_color,
        secondary_color
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CreateCardResponse{
        card_id,
        status : true
    }))
}

pub async fn update() {}
pub async fn delete() {}
pub async fn get_card() {}

