-- Every balance the bank (or the person) told us, kept so a reconciliation check can compare it with
-- what the transactions imply. 'statement' is the balance at the END of as_of (exact); 'alert' and
-- 'typed' are the balance at some unknown point during that day.
CREATE TABLE balance_snapshots (
    snapshot_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES accounts (account_id) ON DELETE CASCADE,
    as_of TEXT NOT NULL,                                  -- 'YYYY-MM-DD', the bank's date
    balance_paise INTEGER NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('alert', 'statement', 'typed')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (account_id, as_of, source, balance_paise)
);
CREATE INDEX idx_balance_snapshots_account ON balance_snapshots (account_id, as_of);
CREATE INDEX idx_balance_snapshots_user ON balance_snapshots (user_id);

-- Statements whose balances chained: the closing balance is the end of the last day.
INSERT OR IGNORE INTO balance_snapshots (snapshot_id, user_id, account_id, as_of, balance_paise, source)
SELECT lower(hex(randomblob(16))), s.user_id, s.account_id, s.last_date, s.closing_paise, 'statement'
FROM statement_imports s
JOIN accounts a ON a.account_id = s.account_id AND a.user_id = s.user_id
WHERE s.chain_ok = 1 AND s.account_id IS NOT NULL AND s.closing_paise IS NOT NULL AND s.last_date IS NOT NULL;

-- The opening balance is the balance before the first row, i.e. the end of the day before it.
INSERT OR IGNORE INTO balance_snapshots (snapshot_id, user_id, account_id, as_of, balance_paise, source)
SELECT lower(hex(randomblob(16))), s.user_id, s.account_id, date(s.first_date, '-1 day'), s.opening_paise, 'statement'
FROM statement_imports s
JOIN accounts a ON a.account_id = s.account_id AND a.user_id = s.user_id
WHERE s.chain_ok = 1 AND s.account_id IS NOT NULL AND s.opening_paise IS NOT NULL AND date(s.first_date, '-1 day') IS NOT NULL;
