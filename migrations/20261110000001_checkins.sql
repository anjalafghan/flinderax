-- Quarterly check-ins: each finished check-in keeps the report as it was shown.
CREATE TABLE checkins (
    checkin_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    done_on TEXT NOT NULL,          -- 'YYYY-MM-DD'
    period_start TEXT NOT NULL,     -- 'YYYY-MM-DD'
    period_end TEXT NOT NULL,       -- 'YYYY-MM-DD'
    summary_json TEXT NOT NULL,     -- the QuarterReport at the time
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
CREATE INDEX idx_checkins_user ON checkins (user_id, done_on DESC);

-- One private calendar feed per user, keyed by an unguessable token that can be reset.
CREATE TABLE reminder_feeds (
    user_id TEXT PRIMARY KEY,
    token TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
