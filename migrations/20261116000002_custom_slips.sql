-- A date we had that never was a slip gets its own one-off, archived slip (see `keep_custom`).
-- Marking it lets deleting that date remove the one-off too, instead of leaving it in the archive.
ALTER TABLE joy_activities ADD COLUMN custom INTEGER NOT NULL DEFAULT 0;
