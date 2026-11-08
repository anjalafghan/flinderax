-- Balance history for the net-worth chart, and automatic posting of salary / fixed payments so
-- balances stay roughly right between the times you type them in.

-- Every balance an account has had. Written by triggers, so every path (manual update, moves,
-- undo, auto-posting) is captured without handler changes.
CREATE TABLE balance_history (
    history_id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    balance_paise INTEGER NOT NULL,
    recorded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts (account_id) ON DELETE CASCADE
);
CREATE INDEX idx_balance_history_account ON balance_history (account_id, recorded_at);

INSERT INTO balance_history (account_id, balance_paise, recorded_at)
    SELECT account_id, balance_paise, updated_at FROM accounts;

CREATE TRIGGER trg_accounts_balance_insert AFTER INSERT ON accounts
BEGIN
    INSERT INTO balance_history (account_id, balance_paise) VALUES (NEW.account_id, NEW.balance_paise);
END;

CREATE TRIGGER trg_accounts_balance_update AFTER UPDATE OF balance_paise ON accounts
WHEN NEW.balance_paise <> OLD.balance_paise
BEGIN
    INSERT INTO balance_history (account_id, balance_paise) VALUES (NEW.account_id, NEW.balance_paise);
END;

-- The local date ('YYYY-MM-DD') the balance was last typed in. Auto-posting only applies
-- income / fixed payments dated after it; typing a balance again resets it.
ALTER TABLE accounts ADD COLUMN balance_set_on TEXT;
UPDATE accounts SET balance_set_on = date(updated_at);

-- One row per (source, date, account) that has been applied, so nothing is posted twice.
CREATE TABLE auto_postings (
    user_id TEXT NOT NULL,
    -- 'income' or 'commitment'
    source_kind TEXT NOT NULL CHECK (source_kind IN ('income', 'commitment')),
    source_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    occurs_on TEXT NOT NULL,
    label TEXT NOT NULL,
    -- signed: + credit, - debit
    amount_paise INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (source_id, occurs_on, account_id),
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts (account_id) ON DELETE CASCADE
);
CREATE INDEX idx_auto_postings_user ON auto_postings (user_id, occurs_on DESC);

ALTER TABLE user_settings ADD COLUMN auto_post INTEGER NOT NULL DEFAULT 1 CHECK (auto_post IN (0, 1));
