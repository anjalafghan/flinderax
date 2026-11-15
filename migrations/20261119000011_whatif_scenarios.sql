-- What-if scenarios used to live in the browser only. They now follow the person to any device.
CREATE TABLE whatif_scenarios (
    user_id TEXT PRIMARY KEY REFERENCES users (user_id) ON DELETE CASCADE,
    drafts_json TEXT NOT NULL DEFAULT '[]',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
