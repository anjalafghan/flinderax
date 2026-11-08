-- Transactions read from bank alert SMS / emails (or pasted / typed), with merchant categories
-- the app learns from corrections, and a long-lived key the phone uses to send alerts.

-- Lets an alert for "A/c XX1234" be matched to a bank account.
ALTER TABLE accounts ADD COLUMN last_4_digits TEXT;

CREATE TABLE transactions (
    txn_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    occurred_on TEXT NOT NULL,                       -- 'YYYY-MM-DD'
    direction TEXT NOT NULL CHECK (direction IN ('debit', 'credit')),
    amount_paise INTEGER NOT NULL CHECK (amount_paise > 0),
    merchant TEXT,
    category TEXT NOT NULL DEFAULT 'uncategorized',
    card_id TEXT REFERENCES cards (card_id) ON DELETE SET NULL,
    account_id TEXT REFERENCES accounts (account_id) ON DELETE SET NULL,
    last4 TEXT,
    source TEXT NOT NULL CHECK (source IN ('sms', 'email', 'paste', 'manual')),
    raw_text TEXT,
    -- The bank reference when there is one, otherwise date/amount/last4/merchant: the same alert
    -- arriving twice (SMS and email) is stored once.
    dedupe_key TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE,
    UNIQUE (user_id, dedupe_key)
);
CREATE INDEX idx_transactions_user_date ON transactions (user_id, occurred_on DESC);

-- "Merchant contains <pattern> -> <category>", learned when the user re-categorises.
CREATE TABLE merchant_rules (
    user_id TEXT NOT NULL,
    pattern TEXT NOT NULL,                           -- normalised: lower-case words
    category TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, pattern),
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);

-- One key per user; only its SHA-256 is stored.
CREATE TABLE ingest_keys (
    user_id TEXT PRIMARY KEY,
    key_hash TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
