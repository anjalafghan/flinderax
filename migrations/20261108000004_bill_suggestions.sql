-- "Looks like a subscription" suggestions the user said no to, so they aren't offered again.
CREATE TABLE dismissed_recurring (
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    merchant TEXT NOT NULL,              -- normalised, as alerts::normalize gives it
    dismissed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, merchant)
);
