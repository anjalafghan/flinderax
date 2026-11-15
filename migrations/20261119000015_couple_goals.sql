-- Goals a couple agrees on and both pay into (a wedding, a child). Either of them can add one;
-- both see it; each payment records who made it, so the shares can be told apart.
CREATE TABLE couple_goals (
    goal_id TEXT PRIMARY KEY,
    couple_id TEXT NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 80),
    target_paise INTEGER NOT NULL CHECK (target_paise > 0),
    target_date TEXT,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_by TEXT REFERENCES users (user_id) ON DELETE SET NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX couple_goals_couple ON couple_goals (couple_id, archived);

CREATE TABLE couple_goal_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    goal_id TEXT NOT NULL REFERENCES couple_goals (goal_id) ON DELETE CASCADE,
    user_id TEXT REFERENCES users (user_id) ON DELETE SET NULL,
    -- Their name when they paid, so a deleted account's payments still read as theirs.
    name TEXT NOT NULL DEFAULT '',
    amount_paise INTEGER NOT NULL CHECK (amount_paise > 0),
    on_day TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX couple_goal_contributions_goal ON couple_goal_contributions (goal_id);
