-- A budget under a couple goal (the wedding): items with an estimate and maybe a quote, and dated
-- payment milestones against each. Paying a milestone also records a couple_goal_contributions row
-- (`contribution_id`), so each partner's share keeps working. All money is integer paise.
CREATE TABLE goal_items (
    item_id TEXT PRIMARY KEY,
    goal_id TEXT NOT NULL REFERENCES couple_goals (goal_id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 80),
    category TEXT NOT NULL DEFAULT 'misc' CHECK (category IN ('venue', 'catering', 'clothes', 'jewellery', 'photography', 'travel', 'misc')),
    estimate_paise INTEGER NOT NULL DEFAULT 0 CHECK (estimate_paise >= 0),
    quote_paise INTEGER CHECK (quote_paise IS NULL OR quote_paise >= 0),
    vendor TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX goal_items_goal ON goal_items (goal_id, archived);

CREATE TABLE goal_milestones (
    milestone_id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES goal_items (item_id) ON DELETE CASCADE,
    due_on TEXT,
    amount_paise INTEGER NOT NULL CHECK (amount_paise > 0),
    paid_on TEXT,
    paid_by TEXT REFERENCES users (user_id) ON DELETE SET NULL,
    -- A bank text this payment was matched to, when one was linked.
    transaction_id TEXT,
    -- The contribution recorded when it was paid; taking the payment back removes it.
    contribution_id INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX goal_milestones_item ON goal_milestones (item_id);
CREATE INDEX goal_milestones_due ON goal_milestones (paid_on, due_on);
