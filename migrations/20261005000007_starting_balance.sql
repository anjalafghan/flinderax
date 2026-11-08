-- A card's first snapshot can be a starting balance (money owed before tracking began, so it is
-- not "spending this month") or new spending. Existing first snapshots keep the old behaviour.
ALTER TABLE card_snapshots ADD COLUMN is_starting INTEGER NOT NULL DEFAULT 0 CHECK (is_starting IN (0, 1));
UPDATE card_snapshots SET is_starting = 1
 WHERE rowid IN (SELECT MIN(rowid) FROM card_snapshots GROUP BY card_id);
