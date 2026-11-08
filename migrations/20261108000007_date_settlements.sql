-- Settling up date money between the two of you: a row each time one of you pays the other back.
CREATE TABLE joy_settlements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    couple_id TEXT NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    payer INTEGER NOT NULL CHECK (payer IN (0, 1)),   -- who paid the other back
    amount_paise INTEGER NOT NULL CHECK (amount_paise > 0),
    on_day TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_joy_settlements_couple ON joy_settlements (couple_id);
