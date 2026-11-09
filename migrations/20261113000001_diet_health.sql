-- Phase 5, diet and health. All private to one user.
-- Up to three habits tracked weekly; inactive ones are kept so past weeks keep their meaning.
CREATE TABLE diet_habits (
    habit_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    preset_key TEXT,
    target_days INTEGER NOT NULL CHECK (target_days BETWEEN 1 AND 7),
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    sort INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_diet_habits_user ON diet_habits (user_id, active);

-- One row per logged week ('YYYY-MM-DD' of the week's first day, per user_settings.week_start).
CREATE TABLE diet_weeks (
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    week_start TEXT NOT NULL,
    feel INTEGER CHECK (feel BETWEEN 1 AND 5),
    note TEXT NOT NULL DEFAULT '',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, week_start)
);

CREATE TABLE diet_week_habits (
    user_id TEXT NOT NULL,
    week_start TEXT NOT NULL,
    habit_id TEXT NOT NULL REFERENCES diet_habits (habit_id) ON DELETE CASCADE,
    days_hit INTEGER NOT NULL CHECK (days_hit BETWEEN 0 AND 7),
    PRIMARY KEY (user_id, week_start, habit_id),
    FOREIGN KEY (user_id, week_start) REFERENCES diet_weeks (user_id, week_start) ON DELETE CASCADE
);

-- Weight and waist (x10, so 70.5 kg is 705), and the date of the last full check-up.
CREATE TABLE health_marks (
    mark_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('weight', 'waist', 'checkup')),
    measured_on TEXT NOT NULL,
    value_x10 INTEGER,
    note TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_health_marks_user ON health_marks (user_id, kind, measured_on DESC);

-- A monthly cap on eating out and delivery; NULL means none.
ALTER TABLE user_settings ADD COLUMN eating_out_cap_paise INTEGER;
