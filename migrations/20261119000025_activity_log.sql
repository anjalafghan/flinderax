-- Activity ledger: one row per thing you did that puts data into the app (a statement imported,
-- a bank text or manual entry logged, a check-in finished). `day` is the day you did it, never the
-- day of the data, so importing old history cannot be farmed. `ref_id` names the thing (import id,
-- transaction id, check-in id) so a retry or a re-upload is counted once. Points are integers and
-- already capped per kind per day when written; streaks and totals are derived from this table.
CREATE TABLE activity_log (
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('statement', 'txn', 'checkin')),
    day TEXT NOT NULL,
    ref_id TEXT NOT NULL,
    points INTEGER NOT NULL CHECK (points >= 0),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, kind, day, ref_id)
);
CREATE INDEX idx_activity_log_user_day ON activity_log (user_id, day);
