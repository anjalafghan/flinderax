-- The reward fund. At the end of each month you are paid (up to ₹2,000) for looking after money,
-- health, relationships and the future, held to what you really came in under your allowance.
-- Payouts build up in a wallet; treats are bought from it.
--
-- 1. Rewards gain a third kind, 'wish': a treat with no trigger, bought from the wallet whenever
--    it holds enough. SQLite cannot widen a CHECK in place, so `rewards` is rebuilt (as in
--    20261119000026), with the claims copied aside and put back.
-- 2. Claims gain `from_wallet`: 1 when the cost came out of the wallet. Claims made before the
--    fund existed stay 0, so they never take money from it.
CREATE TABLE reward_claims_keep AS SELECT reward_id, user_id, cost_paise, claimed_on, created_at FROM reward_claims;
CREATE TABLE rewards_keep AS SELECT * FROM rewards;
DROP TABLE reward_claims;
DROP TABLE rewards;

CREATE TABLE rewards (
    reward_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    cost_paise INTEGER NOT NULL DEFAULT 0 CHECK (cost_paise >= 0),
    trigger_kind TEXT NOT NULL CHECK (trigger_kind IN ('goal', 'streak', 'wish')),
    goal_id TEXT REFERENCES goals (goal_id) ON DELETE CASCADE,
    goal_pct INTEGER CHECK (goal_pct IN (25, 50, 75, 100)),
    streak_kind TEXT CHECK (streak_kind IN ('logging', 'saved', 'dates', 'activity')),
    streak_target INTEGER CHECK (streak_target BETWEEN 1 AND 3650),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (trigger_kind = 'goal' AND goal_id IS NOT NULL AND goal_pct IS NOT NULL AND streak_kind IS NULL AND streak_target IS NULL)
        OR (trigger_kind = 'streak' AND streak_kind IS NOT NULL AND streak_target IS NOT NULL AND goal_id IS NULL AND goal_pct IS NULL)
        OR (trigger_kind = 'wish' AND goal_id IS NULL AND goal_pct IS NULL AND streak_kind IS NULL AND streak_target IS NULL)
    )
);
CREATE INDEX idx_rewards_user ON rewards (user_id, archived);
INSERT INTO rewards SELECT * FROM rewards_keep;

CREATE TABLE reward_claims (
    reward_id TEXT PRIMARY KEY REFERENCES rewards (reward_id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    cost_paise INTEGER NOT NULL CHECK (cost_paise >= 0),
    claimed_on TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    from_wallet INTEGER NOT NULL DEFAULT 0 CHECK (from_wallet IN (0, 1))
);
CREATE INDEX idx_reward_claims_user ON reward_claims (user_id, claimed_on);
INSERT INTO reward_claims (reward_id, user_id, cost_paise, claimed_on, created_at)
    SELECT reward_id, user_id, cost_paise, claimed_on, created_at FROM reward_claims_keep;

DROP TABLE reward_claims_keep;
DROP TABLE rewards_keep;

-- The first month the fund pays for ('YYYY-MM'). Set the first time the fund is looked at.
ALTER TABLE reward_settings ADD COLUMN fund_since TEXT;

-- One row per settled month, written once when the month is over and never changed after, so
-- data added later can't rewrite what a month paid. `detail_json` keeps the area scores as shown.
CREATE TABLE reward_months (
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    month TEXT NOT NULL,
    score_bps INTEGER NOT NULL CHECK (score_bps BETWEEN 0 AND 10000),
    earned_paise INTEGER NOT NULL CHECK (earned_paise >= 0),
    saved_paise INTEGER NOT NULL CHECK (saved_paise >= 0),
    payout_paise INTEGER NOT NULL CHECK (payout_paise BETWEEN 0 AND 200000),
    detail_json TEXT NOT NULL DEFAULT '{}',
    settled_on TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, month)
);
