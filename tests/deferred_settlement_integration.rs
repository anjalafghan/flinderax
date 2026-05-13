use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use sqlx::{SqlitePool, Row};
use std::time::Duration;
usetokio::time::sleep;
use flinderax::{
    models::AppState,
    handlers::card::{
        self, create_card, defer_update, get_deferred_status, settle_deferred, insert_transaction,
    },
    routes::card::routes,
};
use rand::Rng;

// Helper to create a test database
async fn setup_test_db() -> SqlitePool {
    let db = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Run migrations
    sqlx::query(
        r#"
        CREATE TABLE users (
            user_id TEXT PRIMARY KEY,
            user_name TEXT NOT NULL UNIQUE,
            user_password TEXT NOT NULL,
            user_role TEXT NOT NULL CHECK (user_role IN ('admin', 'user')),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE cards (
            card_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            card_name TEXT NOT NULL,
            card_bank TEXT NOT NULL,
            card_primary_color INTEGER NOT NULL,
            card_secondary_color INTEGER NOT NULL,
            last_4_digits TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
        );
        CREATE INDEX idx_cards_user_id ON cards (user_id);
        CREATE TABLE card_events (
            transaction_id TEXT PRIMARY KEY,
            card_id TEXT NOT NULL,
            total_due_input REAL NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (card_id) REFERENCES cards (card_id) ON DELETE CASCADE
        );
        CREATE INDEX idx_card_events_card_id ON card_events (card_id);
        CREATE TABLE card_running_state (
            card_id TEXT PRIMARY KEY,
            last_total_due REAL NOT NULL DEFAULT 0,
            last_delta REAL NOT NULL DEFAULT 0,
            FOREIGN KEY (card_id) REFERENCES cards (card_id) ON DELETE CASCADE
        );
        CREATE TABLE deferred_payment_batches (
            batch_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            total_amount REAL NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'settled', 'cancelled')),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
        );
        CREATE INDEX idx_deferred_batches_user_id ON deferred_payment_batches (user_id);
        CREATE INDEX idx_deferred_batches_status ON deferred_payment_batches (status);
        CREATE TABLE deferred_payment_items (
            item_id TEXT PRIMARY KEY,
            batch_id TEXT NOT NULL,
            card_id TEXT NOT NULL,
            amount REAL NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (batch_id) REFERENCES deferred_payment_batches (batch_id) ON DELETE CASCADE,
            FOREIGN KEY (card_id) REFERENCES cards (card_id) ON DELETE CASCADE
        );
        CREATE INDEX idx_deferred_items_batch_id ON deferred_payment_items (batch_id);
        CREATE INDEX idx_deferred_items_card_id ON deferred_payment_items (card_id);
        "#,
    ).await.unwrap();

    db
}

// Helper to create a test user and return user_id
async fn create_test_user(db: &SqlitePool) -> String {
    let user_id = "test_user_123".to_string();
    sqlx::query(
        "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES (?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind("testuser")
    .bind("password")
    .bind("user")
    .execute(db)
    .await
    .unwrap();
    user_id
}

// Helper to create a test card and return card_id
async fn create_test_card(db: &SqlitePool, user_id: &str, card_name: &str, initial_balance: f32) -> String {
    let card_id = format!("card_{}", rand::thread_rng().gen::<u64>());
    let card_primary_color = 0x112233;
    let card_secondary_color = 0x445566;

    // Insert card
    sqlx::query(
        "INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color, last_4_digits) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&card_id)
    .bind(user_id)
    .bind(card_name)
    .bind("Test Bank")
    .bind(card_primary_color)
    .bind(card_secondary_color)
    .bind("1234")
    .execute(db)
    .await
    .unwrap();

    // Initialize running state
    sqlx::query(
        "INSERT INTO card_running_state (card_id, last_total_due, last_delta) VALUES (?, ?, ?)",
    )
    .bind(&card_id)
    .bind(initial_balance as f64)
    .bind(0.0)
    .execute(db)
    .await
    .unwrap();

    card_id
}

// Helper to get current card state
async fn get_card_state(db: &SqlitePool, card_id: &str) -> (f32, f32) {
    let row = sqlx::query!(
        "SELECT last_total_due, last_delta FROM card_running_state WHERE card_id = ?",
        card_id
    )
    .fetch_optional(db)
    .await
    .unwrap();

    match row {
        Some(r) => (r.last_total_due as f32, r.last_delta as f32),
        None => (0.0, 0.0),
    }
}

// Helper to get transaction count for a card
async fn get_transaction_count(db: &SqlitePool, card_id: &str) -> u64 {
    let row = sqlx::query!(
        "SELECT COUNT(*) as cnt FROM card_events WHERE card_id = ?",
        card_id
    )
    .fetch_one(db)
    .await
    .unwrap();
    row.cnt as u64
}

#[tokio::test]
async fn test_settle_deferred_single_card_positive_amount() {
    // Setup
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;
    let card_id = create_test_card(&db, &user_id, "Test Card 1", 5000.0).await;

    // Simulate deferring a payment of +1000 (increase in total due)
    let batch_id = {
        let state = AppState {
            db: db.clone(),
            redis: None,
            paseto_key: Arc::new( rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()),
        };
        let payload = card::DeferUpdatePayload {
            card_id: card_id.clone(),
            amount: 1000.0,
        };
        let res = defer_update(
            state,
            axum::extract::Extension((user_id.clone(), "user".to_string())),
            axum::Json(payload),
        )
        .await
        .unwrap();
        res.0.batch_id.unwrap()
    };

    // Verify deferred batch was created
    let status = get_deferred_status(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
    )
    .await
    .unwrap();
    assert!(status.0.is_some());
    let status = status.0.unwrap();
    assert_eq!(status.total_amount, 1000.0);
    assert_eq!(status.items.len(), 1);
    assert_eq!(status.items[0].amount, 1000.0);

    // Card balance should still be 5000 (deferred not applied yet)
    let (balance, delta) = get_card_state(&db, &card_id).await;
    assert_eq!(balance, 5000.0);
    assert_eq!(delta, 0.0);

    // Now settle the deferred batch
    let settle_payload = card::SettleDeferredPayload { batch_id };
    let settle_res = settle_deferred(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(settle_payload),
    )
    .await
    .unwrap();

    assert!(settle_res.0.status);

    // Check card balance after settlement
    let (balance, delta) = get_card_state(&db, &card_id).await;
    assert_eq!(balance, 6000.0); // Should be 5000 + 1000
    assert_eq!(delta, 1000.0);   // Delta should be the settled amount

    // Check transaction was recorded
    let tx_count = get_transaction_count(&db, &card_id).await;
    assert_eq!(tx_count, 1);

    // Check the transaction amount
    let tx = sqlx::query!(
        "SELECT total_due_input FROM card_events WHERE card_id = ?",
        card_id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(tx.total_due_input, 6000.0);
}

#[tokio::test]
async fn test_settle_deferred_single_card_negative_amount() {
    // Setup: card with initial balance 5000
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;
    let card_id = create_test_card(&db, &user_id, "Test Card 2", 5000.0).await;

    // Defer a payment of -1000 (payment/reduction)
    let batch_id = {
        let state = AppState {
            db: db.clone(),
            redis: None,
            paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()),
        };
        let payload = card::DeferUpdatePayload {
            card_id: card_id.clone(),
            amount: -1000.0,
        };
        let res = defer_update(
            state,
            axum::extract::Extension((user_id.clone(), "user".to_string())),
            axum::Json(payload),
        )
        .await
        .unwrap();
        res.0.batch_id.unwrap()
    };

    // Balance still 5000
    let (balance, _) = get_card_state(&db, &card_id).await;
    assert_eq!(balance, 5000.0);

    // Settle
    let settle_payload = card::SettleDeferredPayload { batch_id };
    let settle_res = settle_deferred(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(settle_payload),
    )
    .await
    .unwrap();
    assert!(settle_res.0.status);

    // Balance should be 5000 + (-1000) = 4000
    let (balance, delta) = get_card_state(&db, &card_id).await;
    assert_eq!(balance, 4000.0);
    assert_eq!(delta, -1000.0);

    let tx_count = get_transaction_count(&db, &card_id).await;
    assert_eq!(tx_count, 1);

    let tx = sqlx::query!(
        "SELECT total_due_input FROM card_events WHERE card_id = ?",
        card_id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(tx.total_due_input, 4000.0);
}

#[tokio::test]
async fn test_settle_deferred_multiple_cards() {
    // Setup: two cards with different initial balances
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;
    let card1_id = create_test_card(&db, &user_id, "Card 1", 5000.0).await;
    let card2_id = create_test_card(&db, &user_id, "Card 2", 8000.0).await;

    // Defer payments for both cards in same batch
    let batch_id = {
        let state = AppState {
            db: db.clone(),
            redis: None,
            paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()),
        };
        // First card: +2000
        let _ = defer_update(
            state.clone(),
            axum::extract::Extension((user_id.clone(), "user".to_string())),
            axum::Json(card::DeferUpdatePayload {
                card_id: card1_id.clone(),
                amount: 2000.0,
            }),
        )
        .await
        .unwrap();
        // Second card: -1500 (payment)
        let res = defer_update(
            state,
            axum::extract::Extension((user_id.clone(), "user".to_string())),
            axum::Json(card::DeferUpdatePayload {
                card_id: card2_id.clone(),
                amount: -1500.0,
            }),
        )
        .await
        .unwrap();
        res.0.batch_id.unwrap()
    };

    // Verify batch has two items
    let status = get_deferred_status(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
    )
    .await
    .unwrap();
    let status = status.0.unwrap();
    assert_eq!(status.items.len(), 2);
    assert_eq!(status.total_amount, 500.0); // 2000 + (-1500) = 500

    // Card balances unchanged
    let (b1, _) = get_card_state(&db, &card1_id).await;
    let (b2, _) = get_card_state(&db, &card2_id).await;
    assert_eq!(b1, 5000.0);
    assert_eq!(b2, 8000.0);

    // Settle the batch
    let settle_payload = card::SettleDeferredPayload { batch_id };
    let settle_res = settle_deferred(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(settle_payload),
    )
    .await
    .unwrap();
    assert!(settle_res.0.status);

    // Card 1: 5000 + 2000 = 7000
    let (b1, d1) = get_card_state(&db, &card1_id).await;
    assert_eq!(b1, 7000.0);
    assert_eq!(d1, 2000.0);

    // Card 2: 8000 + (-1500) = 6500
    let (b2, d2) = get_card_state(&db, &card2_id).await;
    assert_eq!(b2, 6500.0);
    assert_eq!(d2, -1500.0);

    // Each card should have one transaction
    assert_eq!(get_transaction_count(&db, &card1_id).await, 1);
    assert_eq!(get_transaction_count(&db, &card2_id).await, 1);

    // Check transaction totals
    let tx1 = sqlx::query!(
        "SELECT total_due_input FROM card_events WHERE card_id = ?",
        card1_id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(tx1.total_due_input, 7000.0);

    let tx2 = sqlx::query!(
        "SELECT total_due_input FROM card_events WHERE card_id = ?",
        card2_id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(tx2.total_due_input, 6500.0);
}

#[tokio::test]
async fn test_settle_preserves_existing_transactions() {
    // Setup: card with existing transactions
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;
    let card_id = create_test_card(&db, &user_id, "Test Card 3", 3000.0).await;

    // Manually insert an existing transaction (as if user already had some activity)
    sqlx::query(
        "INSERT INTO card_events (transaction_id, card_id, total_due_input) VALUES (?, ?, ?)",
    )
    .bind("existing_tx_1")
    .bind(&card_id)
    .bind(3000.0)
    .execute(&db)
    .await
    .unwrap();

    // Update running state to match
    sqlx::query(
        "UPDATE card_running_state SET last_total_due = ?, last_delta = ? WHERE card_id = ?",
    )
    .bind(3000.0)
    .bind(3000.0)
    .bind(&card_id)
    .execute(&db)
    .await
    .unwrap();

    let mut tx_count = get_transaction_count(&db, &card_id).await;
    assert_eq!(tx_count, 1);

    // Now defer +2000 more
    let batch_id = {
        let state = AppState {
            db: db.clone(),
            redis: None,
            paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()),
        };
        let res = defer_update(
            state,
            axum::extract::Extension((user_id.clone(), "user".to_string())),
            axum::Json(card::DeferUpdatePayload {
                card_id: card_id.clone(),
                amount: 2000.0,
            }),
        )
        .await
        .unwrap();
        res.0.batch_id.unwrap()
    };

    // Balance still 3000, but we have a deferred item
    let (balance, delta) = get_card_state(&db, &card_id).await;
    assert_eq!(balance, 3000.0);
    assert_eq!(delta, 3000.0); // last_delta from the original transaction

    // Settle
    let settle_payload = card::SettleDeferredPayload { batch_id };
    let settle_res = settle_deferred(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(settle_payload),
    )
    .await
    .unwrap();
    assert!(settle_res.0.status);

    // Final balance: 3000 + 2000 = 5000
    let (balance, delta) = get_card_state(&db, &card_id).await;
    assert_eq!(balance, 5000.0);
    assert_eq!(delta, 2000.0); // last_delta is the most recent transaction amount

    // Should now have 2 transactions total
    tx_count = get_transaction_count(&db, &card_id).await;
    assert_eq!(tx_count, 2);

    // Both transactions should be recorded correctly
    let txs = sqlx::query!(
        "SELECT total_due_input FROM card_events WHERE card_id = ? ORDER BY timestamp ASC",
        card_id
    )
    .fetch_all(&db)
    .await
    .unwrap();
    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0].total_due_input, 3000.0);
    assert_eq!(txs[1].total_due_input, 5000.0);
}

#[tokio::test]
async fn test_multiple_deferred_items_same_card() {
    // Setup: user defers multiple times for same card, then settles
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;
    let card_id = create_test_card(&db, &user_id, "Multi-defer Card", 1000.0).await;

    // First defer: +500
    let res1 = defer_update(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(card::DeferUpdatePayload {
            card_id: card_id.clone(),
            amount: 500.0,
        }),
    )
    .await
    .unwrap();

    // Second defer: +300 (still same batch)
    let res2 = defer_update(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(card::DeferUpdatePayload {
            card_id: card_id.clone(),
            amount: 300.0,
        }),
    )
    .await
    .unwrap();

    // Both should be in same batch
    assert_eq!(res1.0.batch_id, res2.0.batch_id);

    let batch_id = res1.0.batch_id.unwrap();

    // Check batch total = 800
    let status = get_deferred_status(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
    )
    .await
    .unwrap();
    let status = status.0.unwrap();
    assert_eq!(status.items.len(), 2);
    assert_eq!(status.total_amount, 800.0);

    // Balance still 1000
    let (balance, _) = get_card_state(&db, &card_id).await;
    assert_eq!(balance, 1000.0);

    // Settle
    let settle_payload = card::SettleDeferredPayload { batch_id };
    let settle_res = settle_deferred(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(settle_payload),
    )
    .await
    .unwrap();
    assert!(settle_res.0.status);

    // Final balance: 1000 + 800 = 1800
    let (balance, delta) = get_card_state(&db, &card_id).await;
    assert_eq!(balance, 1800.0);
    assert_eq!(delta, 300.0); // last_delta = amount of last settled item

    // Should have 2 transactions (one for each item)
    let tx_count = get_transaction_count(&db, &card_id).await;
    assert_eq!(tx_count, 2);

    // Verify transaction order and amounts
    let txs = sqlx::query!(
        "SELECT total_due_input FROM card_events WHERE card_id = ? ORDER BY timestamp ASC",
        card_id
    )
    .fetch_all(&db)
    .await
    .unwrap();
    assert_eq!(txs.len(), 2);
    // After first item: 1000 + 500 = 1500
    assert_eq!(txs[0].total_due_input, 1500.0);
    // After second item: 1500 + 300 = 1800
    assert_eq!(txs[1].total_due_input, 1800.0);
}

#[tokio::test]
async fn test_settle_cancelled_batch_fails() {
    // Setup
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;
    let card_id = create_test_card(&db, &user_id, "Test Card Cancel", 5000.0).await;

    // Create and then cancel batch
    let batch_id = {
        let state = AppState {
            db: db.clone(),
            redis: None,
            paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()),
        };
        let res = defer_update(
            state,
            axum::extract::Extension((user_id.clone(), "user".to_string())),
            axum::Json(card::DeferUpdatePayload {
                card_id: card_id.clone(),
                amount: 1000.0,
            }),
        )
        .await
        .unwrap();
        res.0.batch_id.unwrap()
    };

    // Cancel the batch
    let cancel_payload = card::SettleDeferredPayload { batch_id: batch_id.clone() };
    let _ = card::cancel_deferred(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(cancel_payload),
    )
    .await
    .unwrap();

    // Try to settle cancelled batch - should fail
    let settle_res = settle_deferred(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(card::SettleDeferredPayload { batch_id }),
    )
    .await;
    assert!(settle_res.is_err());

    // Card balance should remain unchanged (5000)
    let (balance, _) = get_card_state(&db, &card_id).await;
    assert_eq!(balance, 5000.0);
}

#[tokio::test]
async fn test_settle_updates_batch_status() {
    // Setup
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;
    let card_id = create_test_card(&db, &user_id, "Test Card Status", 2000.0).await;

    let batch_id = {
        let state = AppState {
            db: db.clone(),
            redis: None,
            paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()),
        };
        let res = defer_update(
            state,
            axum::extract::Extension((user_id.clone(), "user".to_string())),
            axum::Json(card::DeferUpdatePayload {
                card_id: card_id.clone(),
                amount: 500.0,
            }),
        )
        .await
        .unwrap();
        res.0.batch_id.unwrap()
    };

    // Verify batch status is pending
    let status_before = sqlx::query!(
        "SELECT status FROM deferred_payment_batches WHERE batch_id = ?",
        batch_id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(status_before.status, "pending");

    // Settle
    let settle_payload = card::SettleDeferredPayload { batch_id };
    let _ = settle_deferred(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(settle_payload),
    )
    .await
    .unwrap();

    // Verify batch status is settled
    let status_after = sqlx::query!(
        "SELECT status FROM deferred_payment_batches WHERE batch_id = ?",
        batch_id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(status_after.status, "settled");
}

#[tokio::test]
async fn test_settle_invalid_batch_fails() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    // Try settling a non-existent batch
    let settle_payload = card::SettleDeferredPayload {
        batch_id: "nonexistent".to_string(),
    };
    let res = settle_deferred(
        AppState { db: db.clone(), redis: None, paseto_key: Arc::new(rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::generate(&mut rand::thread_rng()).unwrap()) },
        axum::extraction::Extension((user_id.clone(), "user".to_string())),
        axum::Json(settle_payload),
    )
    .await;
    assert!(res.is_err());
}

// Helper to call get_deferred_status (since it's not public in the module)
mod card {
    pub use crate::handlers::card::*;
    pub type DeferUpdateResponse = crate::models::DeferUpdateResponse;
    pub type SettleDeferredPayload = crate::models::SettleDeferredPayload;
    pub type DeferredBatchStatus = crate::models::DeferredBatchStatus;
}
