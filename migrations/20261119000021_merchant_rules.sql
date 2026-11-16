-- Merchant memory: the existing `merchant_rules` table now also remembers every recategorisation.
-- For those rows `pattern` is the planner's `merchant_key` of the payee (exact, one row per key per
-- user: the primary key is already (user_id, pattern)). Older rows keep their words-contained
-- pattern and keep working as before.
ALTER TABLE merchant_rules ADD COLUMN updated_at DATETIME;
-- How many times the user has confirmed this correction (1 when first remembered).
ALTER TABLE merchant_rules ADD COLUMN hits INTEGER NOT NULL DEFAULT 1;
UPDATE merchant_rules SET updated_at = created_at;
