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
         common::{invalidate_cards_cache, validate_day, AppError},
     },
     models::{
         AppState, CardResponse, CreateCardPayload, DeleteCardPayload, DeleteTransactionPayload,
         DeleteTransactionResponse, GetCardForUser,
         InsertTransactionPayload, InsertTransactionResponse, ResetTransactionsPayload,
         ShowGetCardResponse, UpdateCardPayload, DeferUpdatePayload, DeferUpdateResponse,
         DeferredBatchStatus, DeferredItemStatus, SettleDeferredPayload, SettleDeferredResponse,
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
            last_4_digits: param.last_4_digits,
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

    validate_cycle(&card_details.statement_day, &card_details.due_day)?;
    let card_id = nanoid!();
    let primary_color = color::pack(card_details.card_primary_color);
    let secondary_color = color::pack(card_details.card_secondary_color);

    sqlx::query!(
        "INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color, last_4_digits, statement_day, due_day, credit_limit_paise) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        card_id,
        user_id,
        card_details.card_name,
        card_details.card_bank,
        primary_color,
        secondary_color,
        card_details.last_4_digits,
        card_details.statement_day,
        card_details.due_day,
        card_details.credit_limit_paise
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
    validate_cycle(&update_card_details.statement_day, &update_card_details.due_day)?;
    let card_primary_color = pack(update_card_details.card_primary_color);
    let card_secondary_color = pack(update_card_details.card_secondary_color);
    sqlx::query!(
        "UPDATE cards SET card_name = ?, card_bank = ?, card_primary_color = ?, card_secondary_color = ?, last_4_digits = ?,
                statement_day = COALESCE(?, statement_day), due_day = COALESCE(?, due_day),
                credit_limit_paise = COALESCE(?, credit_limit_paise)
         WHERE card_id = ? AND user_id = ?",
        update_card_details.card_name,
        update_card_details.card_bank,
        card_primary_color,
        card_secondary_color,
        update_card_details.last_4_digits,
        update_card_details.statement_day,
        update_card_details.due_day,
        update_card_details.credit_limit_paise,
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
        "SELECT c.card_id, c.card_name, c.card_bank, c.card_primary_color, c.card_secondary_color, c.last_4_digits,
                c.statement_day, c.due_day, c.credit_limit_paise,
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
        last_4_digits: card.last_4_digits,
        last_total_due: Some(card.last_total_due as f32),
        last_delta: Some(card.last_delta as f32),
        statement_day: card.statement_day,
        due_day: card.due_day,
        credit_limit_paise: card.credit_limit_paise,
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
        "SELECT c.card_id, c.card_name, c.card_bank, c.card_primary_color, c.card_secondary_color, c.last_4_digits,
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
            last_total_due: Some(card.last_total_due as f32),
            last_delta: Some(card.last_delta as f32),
            last_4_digits: card.last_4_digits,
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

/// Appends a `card_events` row and moves `card_running_state` to `amount`, returning
/// `(transaction_id, delta)`. Shared by the plain "insert transaction" flow and bank snapshots.
pub(crate) async fn record_total(
    tx: &mut sqlx::SqliteConnection,
    card_id: &str,
    amount: f32,
) -> Result<(String, f64), AppError> {
    let transaction_id = nanoid!();
    sqlx::query!(
        "INSERT INTO card_events (transaction_id, card_id, total_due_input) VALUES (?, ?, ?)",
        transaction_id,
        card_id,
        amount,
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
             last_total_due = ?
         WHERE card_id = ?
         RETURNING last_delta",
        amount,
        amount,
        card_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Error setting card running state {} ", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok((transaction_id, result.last_delta))
}

pub(crate) fn validate_cycle(statement_day: &Option<i64>, due_day: &Option<i64>) -> Result<(), AppError> {
    if let Some(d) = statement_day {
        validate_day(*d, "statement_day")?;
    }
    if let Some(d) = due_day {
        validate_day(*d, "due_day")?;
    }
    Ok(())
}

/// 404s unless `card_id` belongs to `user_id`.
pub(crate) async fn ensure_card_owned(
    db: &sqlx::SqlitePool,
    card_id: &str,
    user_id: &str,
) -> Result<(), AppError> {
    let owned = sqlx::query!(
        "SELECT card_id FROM cards WHERE card_id = ? AND user_id = ?",
        card_id,
        user_id
    )
    .fetch_optional(db)
    .await?;
    if owned.is_none() {
        return Err(AppError(
            StatusCode::NOT_FOUND,
            "Card not found or you don't have permission to modify it".to_string(),
        ));
    }
    Ok(())
}

pub async fn insert_transaction(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(insert_transaction): Json<InsertTransactionPayload>,
) -> Result<Json<InsertTransactionResponse>, AppError> {
    ensure_card_owned(&state.db, &insert_transaction.card_id, &user_id).await?;
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Error setting transaction check {} ", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let (transaction_id, last_delta) =
        record_total(&mut tx, &insert_transaction.card_id, insert_transaction.amount_due).await?;

    tx.commit()
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    invalidate_cards_cache(&state, &user_id).await;

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
        "UPDATE card_running_state SET last_total_due = 0, last_delta = 0 WHERE card_id = ?",
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

pub async fn delete_transaction(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(payload): Json<DeleteTransactionPayload>,
) -> Result<Json<DeleteTransactionResponse>, AppError> {
    let card_id = &payload.card_id;
    let transaction_id = &payload.transaction_id;

    // Verify card belongs to user
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
            "Card not found or you don't have permission to delete transactions".to_string(),
        ));
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Error starting transaction: {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    // Delete the transaction
    let result = sqlx::query!(
        "DELETE FROM card_events WHERE transaction_id = ? AND card_id = ?",
        transaction_id,
        card_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError(
            StatusCode::NOT_FOUND,
            "Transaction not found".to_string(),
        ));
    }

    // Recalculate last_total_due and last_delta from remaining transactions
    let latest_tx = sqlx::query_as!(
        crate::models::CardTransactionHistory,
        r#"
        SELECT 
            transaction_id as "transaction_id!",
            total_due_input as "total_due_input!: f32",
            timestamp as "timestamp!: String"
        FROM card_events
        WHERE card_id = ?
        ORDER BY timestamp DESC
        LIMIT 1
        "#,
        card_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (new_total_due, new_delta) = if let Some(latest) = latest_tx {
        // Get the second latest to calculate delta
        let second_latest = sqlx::query_as!(
            crate::models::CardTransactionHistory,
            r#"
            SELECT 
                transaction_id as "transaction_id!",
                total_due_input as "total_due_input!: f32",
                timestamp as "timestamp!: String"
            FROM card_events
            WHERE card_id = ? AND transaction_id != ?
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
            card_id,
            latest.transaction_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let delta = if let Some(prev) = second_latest {
            latest.total_due_input - prev.total_due_input
        } else {
            // Only one transaction left - delta equals total
            latest.total_due_input
        };
        (latest.total_due_input, delta)
    } else {
        // No transactions left
        (0.0, 0.0)
    };

    let new_total_due_f32 = new_total_due as f32;
    let new_delta_f32 = new_delta as f32;

    // Update the running state
    sqlx::query!(
        "UPDATE card_running_state SET last_total_due = ?, last_delta = ? WHERE card_id = ?",
        new_total_due_f32,
        new_delta_f32,
        card_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| {
        error!("Error committing transaction: {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    // Invalidate cache
    if let Some(mut redis) = state.redis.clone() {
        let cache_key = format!("user_cards_proto_v2:{}", user_id);
        let _: () = redis.del(cache_key).await.unwrap_or_default();
    }

    Ok(Json(DeleteTransactionResponse {
        transaction_id: transaction_id.to_string(),
        status: true,
    }))
}

pub async fn defer_update(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(payload): Json<DeferUpdatePayload>,
) -> Result<Json<DeferUpdateResponse>, AppError> {
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
            "Card not found or you don't have permission to modify it".to_string(),
        ));
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Error starting transaction: {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let batch_id = sqlx::query!(
        "SELECT batch_id FROM deferred_payment_batches WHERE user_id = ? AND status = 'pending' LIMIT 1",
        user_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let batch_id = if let Some(b) = batch_id {
        b.batch_id.unwrap()
    } else {
        let new_batch_id = nanoid!();
        sqlx::query!(
            "INSERT INTO deferred_payment_batches (batch_id, user_id, total_amount, status) VALUES (?, ?, 0, 'pending')",
            new_batch_id,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        new_batch_id
    };

    let item_id = nanoid!();
    sqlx::query!(
        "INSERT INTO deferred_payment_items (item_id, batch_id, card_id, amount) VALUES (?, ?, ?, ?)",
        item_id,
        batch_id,
        card_id,
        payload.amount
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query!(
        "UPDATE deferred_payment_batches SET total_amount = total_amount + ? WHERE batch_id = ?",
        payload.amount,
        batch_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total = sqlx::query!(
        "SELECT total_amount FROM deferred_payment_batches WHERE batch_id = ?",
        batch_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DeferUpdateResponse {
        batch_id,
        item_id,
        total_pending: total.total_amount as f32,
        status: true,
    }))
}

pub async fn get_deferred_status(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
) -> Result<Json<Option<DeferredBatchStatus>>, AppError> {
    let batch = sqlx::query!(
        "SELECT batch_id, total_amount, status FROM deferred_payment_batches WHERE user_id = ? AND status = 'pending' LIMIT 1",
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(b) = batch {
        let items = sqlx::query!(
            "SELECT di.item_id, di.card_id, di.amount, c.card_name
             FROM deferred_payment_items di
             JOIN cards c ON di.card_id = c.card_id
             WHERE di.batch_id = ?",
            b.batch_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let item_statuses: Vec<DeferredItemStatus> = items
            .into_iter()
            .map(|i| DeferredItemStatus {
                item_id: i.item_id.unwrap(),
                card_id: i.card_id,
                card_name: Some(i.card_name),
                amount: i.amount as f32,
            })
            .collect();

        Ok(Json(Some(DeferredBatchStatus {
            batch_id: b.batch_id.unwrap(),
            total_amount: b.total_amount as f32,
            status: b.status,
            items: item_statuses,
        })))
    } else {
        Ok(Json(None))
    }
}

pub async fn settle_deferred(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(payload): Json<SettleDeferredPayload>,
) -> Result<Json<SettleDeferredResponse>, AppError> {
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Error starting transaction: {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let batch = sqlx::query!(
        "SELECT batch_id, total_amount FROM deferred_payment_batches WHERE batch_id = ? AND user_id = ? AND status = 'pending'",
        payload.batch_id,
        user_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let batch = batch.ok_or_else(|| {
        AppError(StatusCode::NOT_FOUND, "Batch not found or already settled".to_string())
    })?;

    let items = sqlx::query!(
        "SELECT card_id, amount FROM deferred_payment_items WHERE batch_id = ?",
        payload.batch_id
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for item in &items {
        let transaction_id = nanoid!();

        // Get current total before this item's settlement
        let current_state = sqlx::query!(
            "SELECT last_total_due FROM card_running_state WHERE card_id = ?",
            item.card_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let old_total: f32 = if let Some(state) = current_state {
            state.last_total_due as f32
        } else {
            0.0
        };

        let delta = item.amount as f32;
        let new_total = old_total + delta;

        // Insert the transaction event with the new total after settlement
        sqlx::query!(
            "INSERT INTO card_events (transaction_id, card_id, total_due_input) VALUES (?, ?, ?)",
            transaction_id,
            item.card_id,
            new_total
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Update running state
        sqlx::query!(
            "UPDATE card_running_state SET last_total_due = ?, last_delta = ? WHERE card_id = ?",
            new_total,
            delta,
            item.card_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    sqlx::query!(
        "UPDATE deferred_payment_batches SET status = 'settled' WHERE batch_id = ?",
        payload.batch_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(mut redis) = state.redis.clone() {
        let cache_key = format!("user_cards_proto_v2:{}", user_id);
        let _: () = redis.del(cache_key).await.unwrap_or_default();
    }

    Ok(Json(SettleDeferredResponse {
        batch_id: payload.batch_id,
        total_settled: batch.total_amount as f32,
        status: true,
    }))
}

pub async fn cancel_deferred(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(payload): Json<SettleDeferredPayload>,
) -> Result<Json<CardResponse>, AppError> {
    let result = sqlx::query!(
        "UPDATE deferred_payment_batches SET status = 'cancelled' WHERE batch_id = ? AND user_id = ? AND status = 'pending'",
        payload.batch_id,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError(
            StatusCode::NOT_FOUND,
            "Batch not found or already processed".to_string(),
        ));
    }

    Ok(Json(CardResponse {
        card_id: payload.batch_id,
        status: true,
    }))
}
