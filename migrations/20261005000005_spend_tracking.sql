-- Which account pays this card's bill (e.g. Kotak), so the app can say how much to move there.
ALTER TABLE cards ADD COLUMN pay_from_account_id TEXT REFERENCES accounts (account_id) ON DELETE SET NULL;

-- Over how many pay periods an overspend should be paid back (to avoid breaking an FD).
ALTER TABLE user_settings ADD COLUMN recovery_months INTEGER NOT NULL DEFAULT 3
    CHECK (recovery_months BETWEEN 1 AND 24);
