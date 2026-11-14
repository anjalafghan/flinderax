-- Follow-ups: a small to-do per person, with an optional due day. Private to each user.
CREATE TABLE followups (
    followup_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES people (person_id) ON DELETE CASCADE,
    text TEXT NOT NULL CHECK (length(text) BETWEEN 1 AND 400),
    due_on TEXT,
    done_on TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_followups_user ON followups (user_id, done_on, due_on);
CREATE INDEX idx_followups_person ON followups (person_id);
