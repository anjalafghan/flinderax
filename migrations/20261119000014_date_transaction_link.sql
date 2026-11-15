-- A kept date can be linked to the bank transaction that paid for it, instead of the money side
-- guessing a match by day and amount. One transaction pays for at most one date.
ALTER TABLE joy_history ADD COLUMN txn_id TEXT;
CREATE UNIQUE INDEX joy_history_txn ON joy_history (txn_id) WHERE txn_id IS NOT NULL;
