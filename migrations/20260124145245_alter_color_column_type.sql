-- Simpler approach if no data exists yet
DROP TABLE IF EXISTS cards;

CREATE TABLE cards (
    card_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    card_name TEXT NOT NULL,
    card_bank TEXT NOT NULL,
    card_primary_color INTEGER NOT NULL,
    card_secondary_color INTEGER NOT NULL
);
