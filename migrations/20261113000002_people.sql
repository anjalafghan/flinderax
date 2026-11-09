-- Phase 5, relationships. A short private list of people, their dates, catch-ups and gifts.
-- No contact details are stored.
CREATE TABLE people (
    person_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    circle TEXT NOT NULL CHECK (circle IN ('family', 'close', 'friend', 'work', 'other')),
    every_days INTEGER CHECK (every_days BETWEEN 7 AND 365),
    last_contact_on TEXT,
    note TEXT NOT NULL DEFAULT '',
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_people_user ON people (user_id, archived);

-- Repeating dates: month and day, the year only when known (for "turns 60").
CREATE TABLE person_dates (
    date_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES people (person_id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('birthday', 'anniversary', 'remembrance', 'other')),
    label TEXT NOT NULL DEFAULT '',
    month INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12),
    day INTEGER NOT NULL CHECK (day BETWEEN 1 AND 31),
    year INTEGER,
    remind_days INTEGER NOT NULL DEFAULT 7 CHECK (remind_days BETWEEN 0 AND 60),
    gift_budget_paise INTEGER CHECK (gift_budget_paise > 0),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_person_dates_user ON person_dates (user_id, month, day);

CREATE TABLE touches (
    touch_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES people (person_id) ON DELETE CASCADE,
    on_day TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('call', 'visit', 'message', 'met')),
    cost_paise INTEGER CHECK (cost_paise > 0),
    note TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_touches_person ON touches (person_id, on_day DESC);
CREATE INDEX idx_touches_user ON touches (user_id, on_day);

CREATE TABLE gifts (
    gift_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES people (person_id) ON DELETE CASCADE,
    date_id TEXT REFERENCES person_dates (date_id) ON DELETE SET NULL,
    for_year INTEGER,
    status TEXT NOT NULL DEFAULT 'idea' CHECK (status IN ('idea', 'bought', 'given')),
    title TEXT NOT NULL,
    amount_paise INTEGER CHECK (amount_paise > 0),
    on_day TEXT,
    txn_id TEXT REFERENCES transactions (txn_id) ON DELETE SET NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_gifts_user ON gifts (user_id, status);
CREATE INDEX idx_gifts_person ON gifts (person_id);

-- Festival gifting (Diwali, Rakhi, ...): one hand-set budget and month per season, every year.
CREATE TABLE gift_seasons (
    season_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    month INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12),
    budget_paise INTEGER NOT NULL CHECK (budget_paise > 0),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_gift_seasons_user ON gift_seasons (user_id);

-- Calendar events name people only when this is on.
ALTER TABLE user_settings ADD COLUMN feed_names INTEGER NOT NULL DEFAULT 0 CHECK (feed_names IN (0, 1));
