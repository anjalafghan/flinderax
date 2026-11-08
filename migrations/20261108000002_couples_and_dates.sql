-- Dates (Joy) inside Flinderax. Two Flinderax users form a couple; everything in the joy_* tables
-- belongs to a couple and is shared by both of them. Money in joy_* tables is whole rupees, as in
-- Joy (converted to paise only where it meets the finance side). Person 0/1 is the user's slot in
-- the couple, so Joy's "who IN (0, 1)" rules carry over unchanged.

-- Slots are fixed: a date, task or memory "by 1" stays theirs. Someone leaving empties their slot
-- (the couple is deleted once both are empty), and a new join fills the empty slot.
CREATE TABLE couples (
    couple_id TEXT PRIMARY KEY,
    user_0 TEXT UNIQUE REFERENCES users (user_id) ON DELETE SET NULL,
    user_1 TEXT UNIQUE REFERENCES users (user_id) ON DELETE SET NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (user_1 IS NULL OR user_0 IS NULL OR user_1 <> user_0)
);

-- Single-use code the partner types to join. Stored as SHA-256; expires after 7 days.
CREATE TABLE couple_invites (
    code_hash TEXT PRIMARY KEY,
    couple_id TEXT NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    expires_at DATETIME NOT NULL,
    used_at DATETIME
);

-- Joy's settings table, plus what used to be .env (PERSON_A, HOME_A, DATE_DAYS, OFFICE...),
-- under the same names so the ported code reads them unchanged.
CREATE TABLE joy_settings (
    couple_id TEXT NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (couple_id, key)
);

CREATE TABLE joy_activities (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    couple_id       TEXT    NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    title           TEXT    NOT NULL,
    notes           TEXT    NOT NULL DEFAULT '',
    est_cost        INTEGER NOT NULL DEFAULT 0,
    setting         TEXT    NOT NULL,
    area            TEXT    NOT NULL DEFAULT '',
    venue           TEXT    NOT NULL DEFAULT '',
    lat             REAL,
    lng             REAL,
    duration        TEXT    NOT NULL,
    tags            TEXT    NOT NULL DEFAULT '',
    energy          TEXT    NOT NULL DEFAULT 'med',
    added_by        TEXT    NOT NULL DEFAULT '',
    archived        INTEGER NOT NULL DEFAULT 0,
    times_done      INTEGER NOT NULL DEFAULT 0,
    last_done_at    INTEGER,
    last_skipped_at INTEGER,
    created_at      INTEGER NOT NULL DEFAULT (unixepoch()),
    generated       INTEGER NOT NULL DEFAULT 0,
    closed_days     TEXT    NOT NULL DEFAULT '',
    season_from     INTEGER,
    season_to       INTEGER,
    est_from_actual INTEGER NOT NULL DEFAULT 0,
    hours           TEXT    NOT NULL DEFAULT '',
    best_time       TEXT    NOT NULL DEFAULT '',
    minutes         INTEGER,
    booking         TEXT    NOT NULL DEFAULT '',
    book_ahead_days INTEGER,
    cover           TEXT    NOT NULL DEFAULT '',
    bring           TEXT    NOT NULL DEFAULT '',
    station         TEXT    NOT NULL DEFAULT '',
    station_walk_min INTEGER,
    share_token     TEXT
);
CREATE INDEX joy_activities_couple ON joy_activities (couple_id, archived);
CREATE UNIQUE INDEX joy_activities_share_token ON joy_activities (share_token) WHERE share_token IS NOT NULL;

CREATE TABLE joy_history (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    couple_id    TEXT    NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    activity_id  INTEGER NOT NULL REFERENCES joy_activities (id) ON DELETE CASCADE,
    picked_at    INTEGER NOT NULL DEFAULT (unixepoch()),
    outcome      TEXT    NOT NULL,
    rating       INTEGER,
    note         TEXT    NOT NULL DEFAULT '',
    drawn_by     INTEGER,
    planned_at   INTEGER,
    surprise     INTEGER NOT NULL DEFAULT 0,
    revealed     INTEGER NOT NULL DEFAULT 0,
    has_photo    INTEGER NOT NULL DEFAULT 0,
    est_cost     INTEGER,
    actual_cost  INTEGER,
    cab_fare_0   INTEGER,
    cab_est_0    INTEGER,
    cab_fare_1   INTEGER,
    cab_est_1    INTEGER,
    from_savings INTEGER,
    plan_b       INTEGER REFERENCES joy_activities (id) ON DELETE SET NULL,
    day_kind     TEXT    NOT NULL DEFAULT '',
    share_token  TEXT
);
CREATE INDEX joy_history_couple ON joy_history (couple_id, picked_at DESC);
CREATE UNIQUE INDEX joy_history_share_token ON joy_history (share_token) WHERE share_token IS NOT NULL;

CREATE TABLE joy_pairs (
    a_id INTEGER NOT NULL REFERENCES joy_activities (id) ON DELETE CASCADE,
    b_id INTEGER NOT NULL REFERENCES joy_activities (id) ON DELETE CASCADE,
    PRIMARY KEY (a_id, b_id),
    CHECK (a_id < b_id)
);

CREATE TABLE joy_vetoes (
    activity_id INTEGER NOT NULL REFERENCES joy_activities (id) ON DELETE CASCADE,
    person      INTEGER NOT NULL CHECK (person IN (0, 1)),
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (activity_id, person)
);

CREATE TABLE joy_checkins (
    couple_id    TEXT    NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    who          INTEGER NOT NULL CHECK (who IN (0, 1)),
    day          TEXT    NOT NULL,
    energy       TEXT    NOT NULL,
    budget       INTEGER,
    max_duration TEXT    NOT NULL DEFAULT '',
    reach        TEXT    NOT NULL DEFAULT 'far',
    vibe         TEXT    NOT NULL DEFAULT '',
    at           INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (couple_id, who, day)
);

CREATE TABLE joy_other_spending (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    couple_id TEXT    NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    amount    INTEGER NOT NULL CHECK (amount > 0),
    note      TEXT    NOT NULL DEFAULT '',
    spent_on  TEXT    NOT NULL DEFAULT (date('now', 'localtime'))
);

CREATE TABLE joy_month_budget (
    couple_id TEXT    NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    month     TEXT    NOT NULL,
    base      INTEGER NOT NULL,
    carry     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (couple_id, month)
);

CREATE TABLE joy_memories (
    history_id INTEGER NOT NULL REFERENCES joy_history (id) ON DELETE CASCADE,
    who        INTEGER NOT NULL CHECK (who IN (0, 1)),
    line       TEXT    NOT NULL,
    at         INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (history_id, who)
);

CREATE TABLE joy_savings (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    couple_id TEXT    NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    amount    INTEGER NOT NULL CHECK (amount != 0),
    kind      TEXT    NOT NULL DEFAULT 'add',
    note      TEXT    NOT NULL DEFAULT '',
    on_day    TEXT    NOT NULL DEFAULT (date('now', 'localtime'))
);

CREATE TABLE joy_tasks (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    history_id INTEGER NOT NULL REFERENCES joy_history (id) ON DELETE CASCADE,
    who        INTEGER NOT NULL CHECK (who IN (0, 1)),
    task       TEXT    NOT NULL,
    due        TEXT    NOT NULL DEFAULT '',
    done_at    INTEGER
);
CREATE INDEX joy_tasks_history ON joy_tasks (history_id);

CREATE TABLE joy_regulars (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    couple_id  TEXT    NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    title      TEXT    NOT NULL,
    place      TEXT    NOT NULL DEFAULT '',
    maps_url   TEXT    NOT NULL DEFAULT '',
    weekdays   TEXT    NOT NULL,
    usual_cost INTEGER NOT NULL DEFAULT 0 CHECK (usual_cost >= 0),
    active     INTEGER NOT NULL DEFAULT 1,
    since      TEXT    NOT NULL DEFAULT (date('now', 'localtime'))
);

CREATE TABLE joy_regular_visits (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    regular_id INTEGER NOT NULL REFERENCES joy_regulars (id) ON DELETE CASCADE,
    day        TEXT    NOT NULL,
    status     TEXT    NOT NULL CHECK (status IN ('went', 'skipped')),
    place      TEXT    NOT NULL DEFAULT '',
    amount     INTEGER NOT NULL DEFAULT 0 CHECK (amount >= 0),
    has_photo  INTEGER NOT NULL DEFAULT 0,
    UNIQUE (regular_id, day)
);
CREATE INDEX joy_regular_visits_day ON joy_regular_visits (day);
