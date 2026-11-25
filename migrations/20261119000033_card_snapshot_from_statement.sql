-- A card snapshot read from an uploaded statement's summary box (total due, due date), not typed
-- in from the bank app. statement_on ('YYYY-MM-DD') is the statement's date: the snapshot is dated
-- then, and the same statement imported again is not recorded twice. NULL for a typed snapshot.
ALTER TABLE card_snapshots ADD COLUMN statement_on TEXT;
