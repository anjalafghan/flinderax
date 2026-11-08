-- Goals and sinking funds: a target, an optional date, a priority. Money put towards a goal is
-- logged in goal_contributions; a goal's saved amount is their sum.
CREATE TABLE goals (
    goal_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    -- emergency | custom
    kind TEXT NOT NULL DEFAULT 'custom' CHECK (kind IN ('emergency', 'custom')),
    target_paise INTEGER NOT NULL CHECK (target_paise > 0),
    target_date TEXT,                    -- 'YYYY-MM-DD'; NULL = no deadline
    priority INTEGER NOT NULL DEFAULT 1,
    account_id TEXT REFERENCES accounts (account_id) ON DELETE SET NULL,  -- where it's kept
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_goals_user ON goals (user_id, archived);

CREATE TABLE goal_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    goal_id TEXT NOT NULL REFERENCES goals (goal_id) ON DELETE CASCADE,
    amount_paise INTEGER NOT NULL CHECK (amount_paise <> 0),  -- negative = taken back out
    on_day TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_goal_contributions_goal ON goal_contributions (goal_id);
