use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, PrimitiveDateTime, macros::format_description};
use nanoid::nanoid;
use prost::Message;
use redis::AsyncCommands;
use tracing::error;

use crate::{
    handlers::{
        color::{self, pack, unpack},
        common::AppError,
    },
    models::{
        AppState, CardResponse, CreateCardPayload, DeleteCardPayload, GetCardForUser,
        InsertTransactionPayload, InsertTransactionResponse, ResetTransactionsPayload,
        ShowGetCardResponse, UpdateCardPayload,
    },
};
struct Timestamp {
    seconds: i64,
    nanos: i32,
}

fn parse_timestamp(timestamp_str: &str) -> Timestamp {
    if let Ok(dt) = OffsetDateTime::parse(timestamp_str, &Rfc3339) {
        Timestamp {
            seconds: dt.unix_timestamp(),
            nanos: dt.nanosecond() as i32,
        }
    } else {
        let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        if let Ok(primitive_dt) = PrimitiveDateTime::parse(timestamp_str, &format) {
            let dt = primitive_dt.assume_utc();
            Timestamp {
                seconds: dt.unix_timestamp(),
                nanos: dt.nanosecond() as i32,
            }
        } else {
            Timestamp {
                seconds: 0,
                nanos: 0,
            }
        }
    }
}

impl From<crate::models::CardTransactionHistory> for crate::proto::CardTransactionHistory {
    fn from(value: crate::models::CardTransactionHistory) -> Self {
        let timestamp = parse_timestamp(&value.timestamp);
        crate::proto::CardTransactionHistory {
            transaction_id: value.transaction_id,
            total_due_input: value.total_due_input,
            timestamp_seconds: timestamp.seconds,
            timestamp_nanos: timestamp.nanos,
        }
    }
}

impl From<crate::models::ShowGetCardResponse> for crate::proto::Card {
    fn from(param: crate::models::ShowGetCardResponse) -> Self {
        crate::proto::Card {
            card_id: param.card_id,
            card_name: param.card_name,
            card_bank: param.card_bank,
            card_primary_color: color::pack(param.card_primary_color),
            card_secondary_color: color::pack(param.card_secondary_color),
            last_total_due: param.last_total_due,
            last_delta: param.last_delta,
        }
    }
}

pub async fn create_card(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(card_details): Json<CreateCardPayload>,
) -> Result<Json<CardResponse>, AppError> {
    let mut tx = state.db.begin().await.map_err(|e| {
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

    sqlx::query!(
        "INSERT INTO card_running_state (card_id, last_total_due, last_delta) VALUES (?, ?, ?)",
        card_id,
        0.0,
        0.0
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Invalidate cache
    if let Some(mut redis) = state.redis.clone() {
        let cache_key = format!("user_cards_proto_v2:{}", user_id);
        let _: () = redis.del(cache_key).await.unwrap_or_default();
    }

    Ok(Json(CardResponse {
        card_id,
        status: true,
    }))
}

pub async fn update(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(update_card_details): Json<UpdateCardPayload>,
) -> Result<Json<CardResponse>, AppError> {
    let card_primary_color = pack(update_card_details.card_primary_color);
    let card_secondary_color = pack(update_card_details.card_secondary_color);
    sqlx::query!(
        "UPDATE cards SET card_name = ?, card_bank = ?, card_primary_color = ?, card_secondary_color = ? WHERE card_id = ? AND user_id = ?",
        update_card_details.card_name,
        update_card_details.card_bank,
        card_primary_color,
        card_secondary_color,
        update_card_details.card_id,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    // Invalidate cache
    if let Some(mut redis) = state.redis.clone() {
        let cache_key = format!("user_cards_proto_v2:{}", user_id);
        let _: () = redis.del(cache_key).await.unwrap_or_default();
    }

    Ok(Json(CardResponse {
        card_id: update_card_details.card_id,
        status: true,
    }))
}

pub async fn get_card(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(get_card): Json<GetCardForUser>,
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
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| {
        AppError(
            StatusCode::NOT_FOUND,
            "Card not found or you don't have permission to view it".to_string(),
        )
    })?;

    Ok(Json(ShowGetCardResponse {
        card_id: card.card_id.unwrap(),
        card_name: card.card_name,
        card_bank: card.card_bank,
        card_primary_color: unpack(card.card_primary_color),
        card_secondary_color: unpack(card.card_secondary_color),
        last_total_due: Some(card.last_total_due as f32),
        last_delta: Some(card.last_delta as f32),
    }))
}

pub async fn get_all_cards(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("user_cards_proto_v2:{}", user_id);

    // Check Redis cache if available
    if let Some(mut redis) = state.redis.clone() {
        if let Ok(Some(cached_data)) = redis.get::<_, Option<Vec<u8>>>(&cache_key).await {
            return Ok(([(header::CONTENT_TYPE, "application/x-protobuf")], cached_data));
        }
    }

    let cards = sqlx::query!(
        "SELECT c.card_id, c.card_name, c.card_bank, c.card_primary_color, c.card_secondary_color,
                crs.last_total_due, crs.last_delta
         FROM cards c
         LEFT JOIN card_running_state crs ON c.card_id = crs.card_id
         WHERE c.user_id = ?",
        user_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let proto_cards: Vec<crate::proto::Card> = cards
        .into_iter()
        .map(|card| crate::proto::Card {
            card_id: card.card_id.unwrap(),
            card_name: card.card_name,
            card_bank: card.card_bank,
            card_primary_color: card.card_primary_color as i32,
            card_secondary_color: card.card_secondary_color as i32,
            last_total_due: card.last_total_due.map(|v| v as f32),
            last_delta: card.last_delta.map(|v| v as f32),
        })
        .collect();

    let card_list = crate::proto::CardList { cards: proto_cards };

    let mut buf = Vec::new();
    card_list.encode(&mut buf).map_err(|e: prost::EncodeError| {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    // Cache the result if Redis is available
    if let Some(mut redis) = state.redis.clone() {
        let _: () = redis
            .set_ex(cache_key, buf.clone(), 3600)
            .await
            .unwrap_or_default();
    }

    Ok(([(header::CONTENT_TYPE, "application/x-protobuf")], buf))
}

pub async fn delete_card(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(card_details): Json<DeleteCardPayload>,
) -> Result<Json<CardResponse>, AppError> {
    let card_id = card_details.card_id;

    let result = sqlx::query!(
        "DELETE FROM cards WHERE card_id = ? AND user_id = ?",
        card_id,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError(
            StatusCode::NOT_FOUND,
            "Card not found or you don't have permission to delete it".to_string(),
        ));
    }

    // Invalidate cache
    if let Some(mut redis) = state.redis.clone() {
        let cache_key = format!("user_cards_proto_v2:{}", user_id);
        let _: () = redis.del(cache_key).await.unwrap_or_default();
    }

    Ok(Json(CardResponse {
        card_id,
        status: true,
    }))
}

pub async fn insert_transaction(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(insert_transaction): Json<InsertTransactionPayload>,
) -> Result<Json<InsertTransactionResponse>, AppError> {
    let transaction_id = nanoid!();
    let mut tx = state.db.begin().await.map_err(|e| {
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
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let result = sqlx::query!(
        "UPDATE card_running_state
         SET last_delta = ? - last_total_due,
             last_total_due = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE card_id = ?
         RETURNING last_delta",
        insert_transaction.amount_due,
        insert_transaction.amount_due,
        insert_transaction.card_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Error setting card running state {} ", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let last_delta = result.last_delta;
    tx.commit()
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Invalidate cache
    if let Some(mut redis) = state.redis.clone() {
        let cache_key = format!("user_cards_proto_v2:{}", user_id);
        let _: () = redis.del(cache_key).await.unwrap_or_default();
    }

    Ok(Json(InsertTransactionResponse {
        transaction_id,
        amount_due: last_delta as f32,
        status: true,
    }))
}

pub async fn get_history(
    State(state): State<AppState>,
    Extension((_user_id, _role)): Extension<(String, String)>,
    Json(payload): Json<crate::models::GetHistoryPayload>,
) -> Result<impl IntoResponse, AppError> {
    let history = sqlx::query_as!(
        crate::models::CardTransactionHistory,
        r#"
        SELECT transaction_id as "transaction_id!",
        total_due_input as "total_due_input!: f32",
        timestamp as "timestamp!: String"
        FROM card_events
        WHERE card_id = ?
        ORDER BY timestamp DESC"#,
        payload.card_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let proto_histories: Vec<crate::proto::CardTransactionHistory> =
        history.into_iter().map(|h| h.into()).collect();

    let response = crate::proto::CardHistoryList {
        histories: proto_histories,
    };

    let mut buf = Vec::new();
    response.encode(&mut buf).map_err(|e: prost::EncodeError| {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(([(header::CONTENT_TYPE, "application/x-protobuf")], buf))
}

pub async fn reset_transactions(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(payload): Json<ResetTransactionsPayload>,
) -> Result<Json<CardResponse>, AppError> {
    let card_id = &payload.card_id;

    let card_exists = sqlx::query!(
        "SELECT card_id FROM cards WHERE card_id = ? AND user_id = ?",
        card_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if card_exists.is_none() {
        return Err(AppError(
            StatusCode::NOT_FOUND,
            "Card not found or you don't have permission to reset it".to_string(),
        ));
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Error starting transaction: {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    sqlx::query!("DELETE FROM card_events WHERE card_id = ?", card_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Error deleting card events: {}", e);
            AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    sqlx::query!(
        "UPDATE card_running_state SET last_total_due = 0, last_delta = 0, updated_at = CURRENT_TIMESTAMP WHERE card_id = ?",
        card_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Error resetting card running state: {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    tx.commit().await.map_err(|e| {
        error!("Error committing transaction: {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    if let Some(mut redis) = state.redis.clone() {
        let cache_key = format!("user_cards_proto_v2:{}", user_id);
        let _: () = redis.del(cache_key).await.unwrap_or_default();
    }

    Ok(Json(CardResponse {
        card_id: card_id.to_string(),
        status: true,
    }))
}
