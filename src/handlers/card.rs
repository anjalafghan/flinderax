use axum::{
    extract::State,
    http::{ StatusCode},
    Extension, Json,
};
use nanoid::nanoid;
use sqlx::SqlitePool;
use tracing::error;

use crate::{
    handlers::{color::{self,  pack, unpack}, common::AppError},
    models::{CardResponse, CreateCardPayload, DeleteCardPayload, GetCardForUser, InsertTransactionPayload, InsertTransactionResponse, ShowGetCardResponse, UpdateCardPayload  },
};

pub async fn create_card(
    State(pool): State<SqlitePool>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(card_details): Json<CreateCardPayload>,
) -> Result<Json<CardResponse>, AppError> {


    let mut tx = pool
        .begin()
        .await
        .map_err(|e| {
            error!("Error setting transaction check {} ", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

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
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query!("INSERT INTO card_running_state (card_id, last_total_due, last_delta) VALUES (?, ?, ?)"
        ,card_id,
        0.0,
        0.0)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


    tx.commit()
        .await
        .map_err(|e| {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;


    Ok(Json(CardResponse{
        card_id,
        status : true
    }))
}

pub async fn update(State(pool): State<SqlitePool>, Extension((user_id, _role)): Extension<(String, String)>, Json(update_card_details):Json<UpdateCardPayload> ) -> Result<Json<CardResponse>, AppError> {

    let card_primary_color = pack(update_card_details.card_primary_color);
    let card_secondary_color = pack(update_card_details.card_secondary_color);
    sqlx::query!(
        "UPDATE cards SET card_name = ?, card_bank = ?, card_primary_color = ?, card_secondary_color = ? WHERE  card_id = ? AND user_id = ?", 
        update_card_details.card_name,
        update_card_details.card_bank,
        card_primary_color,
        card_secondary_color ,
        update_card_details.card_id,
        user_id
        )
        .execute(&pool)
        .await
        .map_err(|e|{
            AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(Json(CardResponse { card_id: update_card_details.card_id, status: true }))
}

pub async fn get_card(
    State(pool): State<SqlitePool>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(get_card): Json<GetCardForUser>
) -> Result<Json<ShowGetCardResponse>, AppError> {
    let card = sqlx::query!(
        "SELECT c.card_id, c.card_name, c.card_bank, c.card_primary_color, c.card_secondary_color,
                crs.last_total_due, crs.last_delta
         FROM cards c
         LEFT JOIN card_running_state crs ON c.card_id = crs.card_id
         WHERE c.card_id = ? AND c.user_id = ?",
        get_card.card_id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| AppError(
        StatusCode::NOT_FOUND,
        "Card not found or you don't have permission to view it".to_string()
    ))?;

    Ok(Json(ShowGetCardResponse {
        card_id: card.card_id.unwrap(),
        card_name: card.card_name,
        card_bank: card.card_bank,
        card_primary_color: unpack(card.card_primary_color),
        card_secondary_color: unpack(card.card_secondary_color),
        last_total_due: Some(card.last_total_due),
        last_delta: Some(card.last_delta),
    }))
}


pub async fn get_all_cards(
    State(pool): State<SqlitePool>,
    Extension((user_id, _role)): Extension<(String, String)>,
) -> Result<Json<Vec<ShowGetCardResponse>>, AppError> {  // Return Vec wrapped in Json
    let cards = sqlx::query!(
        "SELECT c.card_id, c.card_name, c.card_bank, c.card_primary_color, c.card_secondary_color,
                crs.last_total_due, crs.last_delta
         FROM cards c
         LEFT JOIN card_running_state crs ON c.card_id = crs.card_id
         WHERE c.user_id = ?",
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response: Vec<ShowGetCardResponse> = cards
        .into_iter()
        .map(|card| ShowGetCardResponse {
            card_id: card.card_id.unwrap(),
            card_name: card.card_name,
            card_bank: card.card_bank,
            card_primary_color: unpack(card.card_primary_color),
            card_secondary_color: unpack(card.card_secondary_color),
            last_total_due: card.last_total_due,
            last_delta: card.last_delta,
        })
        .collect();

    Ok(Json(response))
}

pub async fn delete_card(
    State(pool): State<SqlitePool>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(card_details): Json<DeleteCardPayload>,
) -> Result<Json<CardResponse>, AppError> {

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

    Ok(Json(CardResponse{
        card_id,
        status : true
    }))
}


pub async fn insert_transaction(
    State(pool): State<SqlitePool>,
    Extension((_user_id, _role)): Extension<(String, String)>,
    Json(insert_transaction): Json<InsertTransactionPayload>
) -> Result<Json<InsertTransactionResponse>, AppError> {
    let transaction_id = nanoid!();

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| {
            error!("Error setting transaction check {} ", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;


        
    sqlx::query!(
        "INSERT INTO card_events (transaction_id, card_id, total_due_input) VALUES (?, ?, ?)",
        transaction_id,
        insert_transaction.card_id,
        insert_transaction.amount_due,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {

            error!("Error setting card event {} ", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())})?;
    
    let result = sqlx::query!(
        "UPDATE card_running_state
         SET last_delta = ? - last_total_due,
             last_total_due = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE card_id = ?
           AND ? >= last_total_due
         RETURNING last_delta",
        insert_transaction.amount_due,
        insert_transaction.amount_due,
        insert_transaction.card_id,
        insert_transaction.amount_due
    )
    .fetch_one(&mut *tx)  
    .await
    .map_err(|e| {

            error!("Error setting card running state{} ", e);

        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())})?;
    
    let last_delta = result.last_delta; 

    tx.commit()
        .await
        .map_err(|e| {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    
    
    Ok(Json(InsertTransactionResponse {
        transaction_id,
        amount_due: last_delta as f32,
        status: true
    }))
}


pub async fn get_history(
    State(pool): State<SqlitePool>,
    Extension((_user_id, _role)): Extension<(String, String)>,
    Json(payload): Json<crate::models::GetHistoryPayload>,
) -> Result<Json<Vec<crate::models::CardTransactionHistory>>, AppError> {
    let history = sqlx::query_as!(
        crate::models::CardTransactionHistory,
        r#"SELECT 
            transaction_id as "transaction_id!", 
            total_due_input as "total_due_input!: f64", 
            timestamp as "timestamp!: String"
           FROM card_events 
           WHERE card_id = ?
           ORDER BY timestamp DESC"#,
        payload.card_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


    Ok(Json(history))
}
