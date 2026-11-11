-- Rewards: small treats you choose now and unlock by reaching a goal (or a share of it) or a
-- streak. "Earned" is worked out from the goal's saved money and the streak data, never stored,
-- so it can't drift. Only the claim is stored, once per reward, and a monthly cap keeps treats
-- from eating the goals themselves.
CREATE TABLE rewards (
    reward_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    cost_paise INTEGER NOT NULL DEFAULT 0 CHECK (cost_paise >= 0),
    -- goal: reach `goal_pct` percent of a goal's target (100 = done). streak: a run of `streak_target`.
    trigger_kind TEXT NOT NULL CHECK (trigger_kind IN ('goal', 'streak')),
    goal_id TEXT REFERENCES goals (goal_id) ON DELETE CASCADE,
    goal_pct INTEGER CHECK (goal_pct IN (25, 50, 75, 100)),
    streak_kind TEXT CHECK (streak_kind IN ('logging', 'saved', 'dates')),
    streak_target INTEGER CHECK (streak_target BETWEEN 1 AND 3650),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (trigger_kind = 'goal' AND goal_id IS NOT NULL AND goal_pct IS NOT NULL AND streak_kind IS NULL AND streak_target IS NULL)
        OR (trigger_kind = 'streak' AND streak_kind IS NOT NULL AND streak_target IS NOT NULL AND goal_id IS NULL AND goal_pct IS NULL)
    )
);
CREATE INDEX idx_rewards_user ON rewards (user_id, archived);

-- One claim per reward; the cost is copied so history stays true if the reward is later removed.
CREATE TABLE reward_claims (
    reward_id TEXT PRIMARY KEY REFERENCES rewards (reward_id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    cost_paise INTEGER NOT NULL CHECK (cost_paise >= 0),
    claimed_on TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_reward_claims_user ON reward_claims (user_id, claimed_on);

-- NULL cap = no limit.
CREATE TABLE reward_settings (
    user_id TEXT PRIMARY KEY REFERENCES users (user_id) ON DELETE CASCADE,
    monthly_cap_paise INTEGER CHECK (monthly_cap_paise IS NULL OR monthly_cap_paise >= 0)
);
