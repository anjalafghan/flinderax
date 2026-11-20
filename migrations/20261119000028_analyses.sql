-- Written analyses: a short report (markdown) saved against your account, by you or by a connector
-- acting as you (for example Claude reading your data and writing back what it found). Private to the
-- person; never shared with a partner. `source` says where it came from.
CREATE TABLE analyses (
    analysis_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'connector',
    created_on TEXT NOT NULL
);
CREATE INDEX idx_analyses_user_created ON analyses (user_id, created_on);
