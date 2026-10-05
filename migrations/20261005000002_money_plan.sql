-- Accounts, fixed commitments, income and planner settings. Money is INTEGER paise.

-- bank/cash hold spendable money; fd/rd are savings. `locked` = 1 keeps a balance out of
-- Safe to Spend. New account kinds only need a new CHECK value, no engine change.
CREATE TABLE accounts (
    account_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('bank', 'fd', 'rd', 'cash')),
    balance_paise INTEGER NOT NULL DEFAULT 0,
    locked INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0, 1)),
    interest_bps INTEGER,          -- annual rate in basis points (FD / RD), informational
    maturity_date TEXT,            -- YYYY-MM-DD (FD / RD), informational
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
CREATE INDEX idx_accounts_user ON accounts (user_id);

-- SIPs, rent, donations, subscriptions, an RD instalment, a bills fund...
CREATE TABLE commitments (
    commitment_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    label TEXT NOT NULL,
    amount_paise INTEGER NOT NULL,
    day_of_month INTEGER NOT NULL CHECK (day_of_month BETWEEN 1 AND 31),
    account_id TEXT,
    category TEXT NOT NULL DEFAULT 'other',
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    start_month TEXT,              -- 'YYYY-MM', first month it applies
    end_month TEXT,                -- 'YYYY-MM', last month it applies (e.g. RD final instalment)
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts (account_id) ON DELETE SET NULL
);
CREATE INDEX idx_commitments_user ON commitments (user_id);

CREATE TABLE income_sources (
    income_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    label TEXT NOT NULL,
    amount_paise INTEGER NOT NULL,
    day_of_month INTEGER NOT NULL CHECK (day_of_month BETWEEN 1 AND 31),
    account_id TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts (account_id) ON DELETE SET NULL
);
CREATE INDEX idx_income_user ON income_sources (user_id);

CREATE TABLE user_settings (
    user_id TEXT PRIMARY KEY,
    cushion_paise INTEGER NOT NULL DEFAULT 500000,
    next_period_living_paise INTEGER NOT NULL DEFAULT 1800000,
    week_start INTEGER NOT NULL DEFAULT 1 CHECK (week_start BETWEEN 0 AND 6),
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
