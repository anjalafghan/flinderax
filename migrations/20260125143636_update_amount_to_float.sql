
PRAGMA foreign_keys = ON;

-- Step 1: Create new table with REAL columns
CREATE TABLE card_running_state_new (
    card_id TEXT PRIMARY KEY,
    last_total_due REAL NOT NULL DEFAULT 0,
    last_delta REAL NOT NULL DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (card_id)
        REFERENCES cards (card_id)
        ON DELETE CASCADE
);

-- Step 2: Copy existing data
INSERT INTO card_running_state_new (card_id, last_total_due, last_delta, updated_at)
SELECT card_id, CAST(last_total_due AS REAL), CAST(last_delta AS REAL), updated_at
FROM card_running_state;

-- Step 3: Drop old table
DROP TABLE card_running_state;

-- Step 4: Rename new table
ALTER TABLE card_running_state_new RENAME TO card_running_state;
