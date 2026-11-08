-- Link to Joy (the shared date jar): Flinderax pulls its date spending to show a Dates budget and
-- to label the matching bank-alert transactions. Joy never sees anything from here.

CREATE TABLE joy_links (
    user_id TEXT PRIMARY KEY,
    url TEXT NOT NULL,                  -- e.g. https://joy-jar.fly.dev
    token TEXT NOT NULL,                -- Joy's JOY_API_TOKEN
    partner_paise INTEGER NOT NULL DEFAULT 0 CHECK (partner_paise >= 0),  -- partner's monthly share of the date budget
    last_payload TEXT,                  -- last good response, shown (marked stale) when Joy can't be reached
    synced_at DATETIME,
    last_error TEXT,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);

-- 1 once the user picks a category by hand: automatic re-labelling (e.g. as a date) leaves it alone.
ALTER TABLE transactions ADD COLUMN category_locked INTEGER NOT NULL DEFAULT 0 CHECK (category_locked IN (0, 1));
