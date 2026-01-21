-- Add migration script here
PRAGMA foreign_keys = ON;

-- =========================
-- users
-- =========================
CREATE TABLE IF NOT EXISTS users (
    user_id TEXT PRIMARY KEY,
    user_name TEXT NOT NULL UNIQUE,
    user_password TEXT NOT NULL,
    user_role TEXT NOT NULL CHECK (user_role IN ('admin', 'user')),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- =========================
-- cards
-- =========================
CREATE TABLE IF NOT EXISTS cards (
    card_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    card_name TEXT NOT NULL,
    card_bank TEXT NOT NULL,
    card_default_color TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (user_id)
        REFERENCES users (user_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_cards_user_id
    ON cards (user_id);

-- =========================
-- card_events
-- =========================
CREATE TABLE IF NOT EXISTS card_events (
    transaction_id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL,
    total_due_input INTEGER NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (card_id)
        REFERENCES cards (card_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_card_events_card_id
    ON card_events (card_id);

-- =========================
-- card_running_state
-- =========================
CREATE TABLE IF NOT EXISTS card_running_state (
    card_id TEXT PRIMARY KEY,
    last_total_due INTEGER NOT NULL DEFAULT 0,
    last_delta INTEGER NOT NULL DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (card_id)
        REFERENCES cards (card_id)
        ON DELETE CASCADE
);

