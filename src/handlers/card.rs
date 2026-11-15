use axum::{
    extract::State,
    http::StatusCode,
    Extension, Json,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, PrimitiveDateTime, macros::format_description};
use nanoid::nanoid;
use tracing::error;

 use crate::{
     handlers::{
         account::ensure_account_owned,
         common::{not_found, validate_day, AppError},
     },
     models::{
         AppState, CardResponse, CreateCardPayload, DeleteCardPayload, DeleteTransactionPayload,
         DeleteTransactionResponse, GetCardForUser,
         ResetTransactionsPayload,
         ShowGetCardResponse, UpdateCardPayload,
         
     },
 };
/// The history timestamp as RFC 3339 (the column holds either that or `YYYY-MM-DD HH:MM:SS`, both UTC).
fn to_rfc3339(timestamp_str: &str) -> String {
    if let Ok(dt) = OffsetDateTime::parse(timestamp_str, &Rfc3339) {
        return dt.format(&Rfc3339).unwrap_or_else(|_| timestamp_str.to_string());
    }
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    PrimitiveDateTime::parse(timestamp_str, &format)
        .ok()
        .and_then(|dt| dt.assume_utc().format(&Rfc3339).ok())
        .unwrap_or_else(|| timestamp_str.to_string())
}

#[derive(serde::Serialize)]
pub struct CardListItem {
    card_id: String,
    card_name: String,
    card_bank: String,
    /// Colours are packed 0xRRGGBB integers (see `pack`).
    card_primary_color: i64,
    card_secondary_color: i64,
    last_total_due: Option<f64>,
    last_delta: Option<f64>,
    last_4_digits: Option<String>,
}

#[derive(serde::Serialize)]
pub struct CardListResponse {
    cards: Vec<CardListItem>,
}

#[derive(serde::Serialize)]
pub struct HistoryItem {
    transaction_id: String,
    total_due_input: f64,
    timestamp: String,
}

#[derive(serde::Serialize)]
pub struct HistoryResponse {
    histories: Vec<HistoryItem>,
}

/// A card colour as one 0xRRGGBB integer.
fn pack(color: (u8, u8, u8)) -> i32 {
    ((color.0 as i32) << 16) | ((color.1 as i32) << 8) | (color.2 as i32)
}

fn unpack(color: i64) -> (u8, u8, u8) {
    (((color >> 16) & 0xFF) as u8, ((color >> 8) & 0xFF) as u8, (color & 0xFF) as u8)
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
    ensure_account_owned(&state.db, &card_details.pay_from_account_id, &user_id).await?;
    let card_id = nanoid!();
    let primary_color = pack(card_details.card_primary_color);
    let secondary_color = pack(card_details.card_secondary_color);

    sqlx::query!(
        "INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color, last_4_digits, statement_day, due_day, credit_limit_paise, pay_from_account_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        card_id,
        user_id,
        card_details.card_name,
        card_details.card_bank,
        primary_color,
        secondary_color,
        card_details.last_4_digits,
        card_details.statement_day,
        card_details.due_day,
        card_details.credit_limit_paise,
        card_details.pay_from_account_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query!(
        "INSERT INTO card_running_state (card_id, last_total_due_paise, last_delta_paise) VALUES (?, 0, 0)",
        card_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


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
    ensure_account_owned(&state.db, &update_card_details.pay_from_account_id, &user_id).await?;
    let card_primary_color = pack(update_card_details.card_primary_color);
    let card_secondary_color = pack(update_card_details.card_secondary_color);
    let res = sqlx::query!(
        "UPDATE cards SET card_name = ?, card_bank = ?, card_primary_color = ?, card_secondary_color = ?, last_4_digits = ?,
                statement_day = COALESCE(?, statement_day), due_day = COALESCE(?, due_day),
                credit_limit_paise = COALESCE(?, credit_limit_paise),
                pay_from_account_id = COALESCE(?, pay_from_account_id)
         WHERE card_id = ? AND user_id = ?",
        update_card_details.card_name,
        update_card_details.card_bank,
        card_primary_color,
        card_secondary_color,
        update_card_details.last_4_digits,
        update_card_details.statement_day,
        update_card_details.due_day,
        update_card_details.credit_limit_paise,
        update_card_details.pay_from_account_id,
        update_card_details.card_id,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    if res.rows_affected() == 0 {
        return Err(not_found("Card not found or you don't have permission to modify it"));
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
                c.statement_day, c.due_day, c.credit_limit_paise, c.pay_from_account_id,
                crs.last_total_due_paise, crs.last_delta_paise
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
        last_total_due: Some(rupees(card.last_total_due_paise)),
        last_delta: Some(rupees(card.last_delta_paise)),
        statement_day: card.statement_day,
        due_day: card.due_day,
        credit_limit_paise: card.credit_limit_paise,
        pay_from_account_id: card.pay_from_account_id,
    }))
}

pub async fn get_all_cards(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
) -> Result<Json<CardListResponse>, AppError> {
    let cards = sqlx::query!(
        "SELECT c.card_id, c.card_name, c.card_bank, c.card_primary_color, c.card_secondary_color, c.last_4_digits,
                crs.last_total_due_paise, crs.last_delta_paise
         FROM cards c
         LEFT JOIN card_running_state crs ON c.card_id = crs.card_id
         WHERE c.user_id = ?",
        user_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let cards = cards
        .into_iter()
        .map(|card| CardListItem {
            card_id: card.card_id.unwrap(),
            card_name: card.card_name,
            card_bank: card.card_bank,
            card_primary_color: card.card_primary_color,
            card_secondary_color: card.card_secondary_color,
            last_total_due: Some(rupees(card.last_total_due_paise)),
            last_delta: Some(rupees(card.last_delta_paise)),
            last_4_digits: card.last_4_digits,
        })
        .collect();

    Ok(Json(CardListResponse { cards }))
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


    Ok(Json(CardResponse {
        card_id,
        status: true,
    }))
}

/// Paise as rupees for the card API, which still speaks rupees (exact: a paisa count under 2^53).
fn rupees(paise: i64) -> f64 {
    paise as f64 / 100.0
}

/// Appends a `card_events` row and moves `card_running_state` to `amount_paise`, returning
/// `(transaction_id, delta_paise)`. Shared by the plain "insert transaction" flow and bank snapshots.
pub(crate) async fn record_total(
    tx: &mut sqlx::SqliteConnection,
    card_id: &str,
    amount_paise: i64,
) -> Result<(String, i64), AppError> {
    let transaction_id = nanoid!();
    sqlx::query!(
        "INSERT INTO card_events (transaction_id, card_id, total_due_input_paise) VALUES (?, ?, ?)",
        transaction_id,
        card_id,
        amount_paise,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Error setting card event {} ", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let result = sqlx::query!(
        "UPDATE card_running_state
         SET last_delta_paise = ? - last_total_due_paise,
             last_total_due_paise = ?
         WHERE card_id = ?
         RETURNING last_delta_paise",
        amount_paise,
        amount_paise,
        card_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Error setting card running state {} ", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok((transaction_id, result.last_delta_paise))
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

pub async fn get_history(
    State(state): State<AppState>,
    Extension((user_id, _role)): Extension<(String, String)>,
    Json(payload): Json<crate::models::GetHistoryPayload>,
) -> Result<Json<HistoryResponse>, AppError> {
    ensure_card_owned(&state.db, &payload.card_id, &user_id).await?;
    let history = sqlx::query_as!(
        crate::models::CardTransactionHistory,
        r#"
        SELECT transaction_id as "transaction_id!",
        total_due_input_paise as "total_due_input_paise!: i64",
        timestamp as "timestamp!: String"
        FROM card_events
        WHERE card_id = ?
        ORDER BY timestamp DESC, rowid DESC"#,
        payload.card_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let histories = history
        .into_iter()
        .map(|h| HistoryItem {
            transaction_id: h.transaction_id,
            total_due_input: rupees(h.total_due_input_paise),
            timestamp: to_rfc3339(&h.timestamp),
        })
        .collect();

    Ok(Json(HistoryResponse { histories }))
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

    // The planner reads the latest balance snapshot, so a reset that left the snapshots would still
    // show the old balance (and old spending). A runtime query keeps the committed .sqlx data valid.
    sqlx::query("DELETE FROM card_snapshots WHERE card_id = ?")
        .bind(card_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Error deleting card snapshots: {}", e);
            AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    sqlx::query!(
        "UPDATE card_running_state SET last_total_due_paise = 0, last_delta_paise = 0 WHERE card_id = ?",
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
            total_due_input_paise as "total_due_input_paise!: i64",
            timestamp as "timestamp!: String"
        FROM card_events
        WHERE card_id = ?
        ORDER BY timestamp DESC, rowid DESC
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
                total_due_input_paise as "total_due_input_paise!: i64",
                timestamp as "timestamp!: String"
            FROM card_events
            WHERE card_id = ? AND transaction_id != ?
            ORDER BY timestamp DESC, rowid DESC
            LIMIT 1
            "#,
            card_id,
            latest.transaction_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let delta = if let Some(prev) = second_latest {
            latest.total_due_input_paise - prev.total_due_input_paise
        } else {
            // Only one transaction left - delta equals total
            latest.total_due_input_paise
        };
        (latest.total_due_input_paise, delta)
    } else {
        // No transactions left
        (0, 0)
    };

    // Update the running state
    sqlx::query!(
        "UPDATE card_running_state SET last_total_due_paise = ?, last_delta_paise = ? WHERE card_id = ?",
        new_total_due,
        new_delta,
        card_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| {
        error!("Error committing transaction: {}", e);
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;


    Ok(Json(DeleteTransactionResponse {
        transaction_id: transaction_id.to_string(),
        status: true,
    }))
}

