-- Large one-off or yearly amounts that are not fixed payments: an insurance premium, an advance-tax
-- instalment, a card's annual fee, an FD or RD maturing. They feed the year-ahead view. Integer paise.
CREATE TABLE planned_outflows (
    outflow_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    label TEXT NOT NULL CHECK (length(label) BETWEEN 1 AND 80),
    on_day TEXT NOT NULL,
    amount_paise INTEGER NOT NULL CHECK (amount_paise > 0),
    direction TEXT NOT NULL DEFAULT 'out' CHECK (direction IN ('out', 'in')),
    kind TEXT NOT NULL DEFAULT 'other' CHECK (kind IN ('premium', 'tax', 'fee', 'maturity', 'wedding', 'other')),
    yearly INTEGER NOT NULL DEFAULT 0 CHECK (yearly IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX planned_outflows_user ON planned_outflows (user_id, on_day);
