-- The life plan inputs for one user, stored as the JSON the planner reads.
CREATE TABLE life_profiles (
    user_id TEXT PRIMARY KEY REFERENCES users(user_id) ON DELETE CASCADE,
    profile_json TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
