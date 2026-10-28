-- 50/30/20-style budgeting. Each commitment belongs to a bucket; settings hold the split and an
-- optional fixed amount for everyday costs (replaces "next period living costs").

ALTER TABLE commitments ADD COLUMN bucket TEXT NOT NULL DEFAULT 'needs'
    CHECK (bucket IN ('needs', 'savings', 'wants'));
UPDATE commitments SET bucket = 'savings' WHERE category IN ('sip', 'rd', 'savings', 'investment');
UPDATE commitments SET bucket = 'wants'   WHERE category IN ('subscription', 'donation', 'entertainment');

ALTER TABLE user_settings ADD COLUMN needs_pct INTEGER NOT NULL DEFAULT 50 CHECK (needs_pct BETWEEN 0 AND 100);
ALTER TABLE user_settings ADD COLUMN savings_pct INTEGER NOT NULL DEFAULT 30 CHECK (savings_pct BETWEEN 0 AND 100);
ALTER TABLE user_settings ADD COLUMN wants_pct INTEGER NOT NULL DEFAULT 20 CHECK (wants_pct BETWEEN 0 AND 100);
-- NULL = derive from what is left of the needs share after listed fixed payments.
ALTER TABLE user_settings ADD COLUMN everyday_paise INTEGER;
UPDATE user_settings SET everyday_paise = next_period_living_paise;
ALTER TABLE user_settings DROP COLUMN next_period_living_paise;
