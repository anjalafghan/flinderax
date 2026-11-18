-- Rewards can now hang on the activity streak (days in a row you put data in: a statement, a bank
-- text, a manual entry, a check-in). SQLite cannot widen a CHECK in place, so `rewards` is rebuilt
-- with 'activity' allowed as a streak kind. The claims point at rewards, so they are copied aside
-- first and put back after; no reward or claim is lost.
CREATE TABLE reward_claims_keep AS SELECT reward_id, user_id, cost_paise, claimed_on, created_at FROM reward_claims;
CREATE TABLE rewards_keep AS SELECT * FROM rewards;
DROP TABLE reward_claims;
DROP TABLE rewards;

CREATE TABLE rewards (
    reward_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    cost_paise INTEGER NOT NULL DEFAULT 0 CHECK (cost_paise >= 0),
    trigger_kind TEXT NOT NULL CHECK (trigger_kind IN ('goal', 'streak')),
    goal_id TEXT REFERENCES goals (goal_id) ON DELETE CASCADE,
    goal_pct INTEGER CHECK (goal_pct IN (25, 50, 75, 100)),
    streak_kind TEXT CHECK (streak_kind IN ('logging', 'saved', 'dates', 'activity')),
    streak_target INTEGER CHECK (streak_target BETWEEN 1 AND 3650),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (trigger_kind = 'goal' AND goal_id IS NOT NULL AND goal_pct IS NOT NULL AND streak_kind IS NULL AND streak_target IS NULL)
        OR (trigger_kind = 'streak' AND streak_kind IS NOT NULL AND streak_target IS NOT NULL AND goal_id IS NULL AND goal_pct IS NULL)
    )
);
CREATE INDEX idx_rewards_user ON rewards (user_id, archived);
INSERT INTO rewards SELECT * FROM rewards_keep;

CREATE TABLE reward_claims (
    reward_id TEXT PRIMARY KEY REFERENCES rewards (reward_id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    cost_paise INTEGER NOT NULL CHECK (cost_paise >= 0),
    claimed_on TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_reward_claims_user ON reward_claims (user_id, claimed_on);
INSERT INTO reward_claims SELECT reward_id, user_id, cost_paise, claimed_on, created_at FROM reward_claims_keep;

DROP TABLE reward_claims_keep;
DROP TABLE rewards_keep;
