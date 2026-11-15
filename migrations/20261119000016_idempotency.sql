-- A POST sent with an Idempotency-Key happens once: a retry (a phone that never heard back, or
-- an offline queue replaying) gets the first answer instead of doing it again.
CREATE TABLE idempotency_keys (
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    -- 0 while the first request is still running
    status INTEGER NOT NULL DEFAULT 0,
    body TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, key)
);
CREATE INDEX idempotency_keys_age ON idempotency_keys (created_at);
