-- Optimized Schema for Flinderax Credit Card Balance App
-- Migration: 20260420000000_initial_schema.sql

PRAGMA foreign_keys = ON;

-- Users table
CREATE TABLE users (
    user_id TEXT PRIMARY KEY,
    user_name TEXT NOT NULL UNIQUE,
    user_password TEXT NOT NULL,
    user_role TEXT NOT NULL CHECK (user_role IN ('admin', 'user')),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Cards table
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

-- Card events (transaction history) - stores the user's input total amount
CREATE TABLE card_events (
    transaction_id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL,
    total_due_input REAL NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (card_id) REFERENCES cards (card_id) ON DELETE CASCADE
);

CREATE INDEX idx_card_events_card_id ON card_events (card_id);

-- Card running state - current balance and last delta
CREATE TABLE card_running_state (
    card_id TEXT PRIMARY KEY,
    last_total_due REAL NOT NULL DEFAULT 0,
    last_delta REAL NOT NULL DEFAULT 0,
    FOREIGN KEY (card_id) REFERENCES cards (card_id) ON DELETE CASCADE
);

-- Deferred payment batches - groups multiple card updates for collective settlement
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

-- Deferred payment items - individual card amounts in a batch
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