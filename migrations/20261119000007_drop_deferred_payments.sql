-- Deferred payment batches were never written to by anything; the routes, the dashboard bar and
-- these tables go. (The card_snapshots model is what tracks what a card owes.)
DROP TABLE IF EXISTS deferred_payment_items;
DROP TABLE IF EXISTS deferred_payment_batches;
