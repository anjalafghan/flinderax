-- Indexes for the hot lookups the ledger, joy and card code run (checked with EXPLAIN QUERY PLAN).
-- card_events: "latest entry for a card" sorts by timestamp; (card_id, timestamp) serves it and replaces the card_id-only index.
DROP INDEX idx_card_events_card_id;
CREATE INDEX idx_card_events_card_time ON card_events (card_id, timestamp);

-- joy_history: per-activity lookups (times_done, did-it guards) and the ON DELETE CASCADE / SET NULL
-- from joy_activities, which otherwise scan the whole history table.
CREATE INDEX joy_history_activity ON joy_history (activity_id, outcome);
CREATE INDEX joy_history_plan_b ON joy_history (plan_b);
-- joy_pairs is keyed (a_id, b_id); lookups and the cascade on b_id need their own index.
CREATE INDEX joy_pairs_b ON joy_pairs (b_id);

-- transactions: per-user category counts, and the SET NULL cascades from cards and accounts.
CREATE INDEX idx_transactions_user_category ON transactions (user_id, category);
CREATE INDEX idx_transactions_card ON transactions (card_id);
CREATE INDEX idx_transactions_account ON transactions (account_id);

-- auto_postings: the dashboard's "latest postings" is ORDER BY occurs_on, created_at DESC LIMIT 10.
DROP INDEX idx_auto_postings_user;
CREATE INDEX idx_auto_postings_user ON auto_postings (user_id, occurs_on DESC, created_at DESC);

-- Couple-scoped tables: every listing and total filters on couple_id, and the cascades from couples scan it.
CREATE INDEX joy_regulars_couple ON joy_regulars (couple_id);
CREATE INDEX joy_savings_couple ON joy_savings (couple_id);
CREATE INDEX joy_other_spending_couple ON joy_other_spending (couple_id);
CREATE INDEX couple_invites_couple ON couple_invites (couple_id);
