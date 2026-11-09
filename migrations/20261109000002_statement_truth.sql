-- The bank statement becomes the source of truth. Rows the statement confirms are marked
-- on_statement and carry the bank's reference (RRN); rows only an SMS or email alert recorded
-- keep on_statement = 0 until a statement covers them.

ALTER TABLE transactions ADD COLUMN rrn TEXT;
ALTER TABLE transactions ADD COLUMN on_statement INTEGER NOT NULL DEFAULT 0 CHECK (on_statement IN (0, 1));
CREATE INDEX idx_transactions_user_rrn ON transactions (user_id, rrn);

-- Rows already imported from a statement (their keys start 'stmt:') are on a statement.
UPDATE transactions SET on_statement = 1 WHERE dedupe_key LIKE 'stmt:%';

-- Newline-separated names the user holds accounts under. A transfer to or from one is not spending.
ALTER TABLE user_settings ADD COLUMN holder_names TEXT NOT NULL DEFAULT '';

-- One row per statement import: what period it covered and whether its balances chained.
CREATE TABLE statement_imports (
    import_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    account_id TEXT REFERENCES accounts (account_id) ON DELETE SET NULL,
    card_id TEXT REFERENCES cards (card_id) ON DELETE SET NULL,
    first_date TEXT,                                 -- 'YYYY-MM-DD'
    last_date TEXT,                                  -- 'YYYY-MM-DD'
    rows INTEGER NOT NULL,
    opening_paise INTEGER,                           -- NULL when the file has no balance column
    closing_paise INTEGER,
    chain_ok INTEGER NOT NULL DEFAULT 0 CHECK (chain_ok IN (0, 1)),
    imported_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
CREATE INDEX idx_statement_imports_user ON statement_imports (user_id, imported_at DESC);
