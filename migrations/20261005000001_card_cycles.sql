-- Card billing cycles, 4-number bank snapshots, EMIs and not-yet-posted charges.
-- Money in this migration is INTEGER paise.

ALTER TABLE cards ADD COLUMN statement_day INTEGER CHECK (statement_day BETWEEN 1 AND 31);
ALTER TABLE cards ADD COLUMN due_day INTEGER CHECK (due_day BETWEEN 1 AND 31);
ALTER TABLE cards ADD COLUMN credit_limit_paise INTEGER;

-- A copy of what the bank app shows. unbilled_* are reference only, never used in maths.
CREATE TABLE card_snapshots (
    snapshot_id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL,
    total_due_paise INTEGER NOT NULL,
    outstanding_paise INTEGER NOT NULL,
    unbilled_spent_paise INTEGER,
    unbilled_credit_paise INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (card_id) REFERENCES cards (card_id) ON DELETE CASCADE
);
CREATE INDEX idx_card_snapshots_card ON card_snapshots (card_id, created_at DESC);

CREATE TABLE card_emis (
    emi_id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL,
    label TEXT NOT NULL,
    monthly_paise INTEGER NOT NULL,
    remaining_principal_paise INTEGER NOT NULL,
    months_left INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (card_id) REFERENCES cards (card_id) ON DELETE CASCADE
);
CREATE INDEX idx_card_emis_card ON card_emis (card_id);

-- Charges (e.g. food delivery) that have not posted to the card yet.
CREATE TABLE pending_charges (
    charge_id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL,
    label TEXT NOT NULL,
    amount_paise INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cleared_at DATETIME,
    FOREIGN KEY (card_id) REFERENCES cards (card_id) ON DELETE CASCADE
);
CREATE INDEX idx_pending_charges_card ON pending_charges (card_id);
