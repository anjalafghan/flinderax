use axum::{
    extract::State,
    http::{ StatusCode},
    Extension, Json,
};
use nanoid::nanoid;
use sqlx::SqlitePool;

use crate::{
    handlers::{color, common::AppError},
    models::{CreateCardPayload, CreateCardResponse, DeleteCardPayload, DeleteCardResponse},
};

pub async fn create_card(
    State(pool): State<SqlitePool>,
    Extension((user_id, _role)): Extension<(String, String)>,
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
pub async fn get_card() {}

pub async fn delete_card(
    State(pool): State<SqlitePool>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(card_details): Json<DeleteCardPayload>,
) -> Result<Json<DeleteCardResponse>, AppError> {

    let card_id =  card_details.card_id;

    let result = sqlx::query!(
        " DELETE FROM cards WHERE card_id = ? AND user_id = ?",
        card_id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
          return Err(AppError(
            StatusCode::NOT_FOUND,
            "Card not found or you don't have permission to delete it".to_string()
        ));
    }

    Ok(Json(DeleteCardResponse{
        card_id,
        status : true
    }))
}
