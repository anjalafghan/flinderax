-- Add composite index for optimizing transaction history queries by card_id + timestamp
CREATE INDEX IF NOT EXISTS idx_card_events_card_id_timestamp
    ON card_events (card_id, timestamp DESC);
