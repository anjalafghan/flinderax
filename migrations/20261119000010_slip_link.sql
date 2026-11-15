-- A slip can carry a link (booking page, menu, the post that gave the idea), and remembers when a
-- person last saved it, so its card can say how old its hours and cost might be.
ALTER TABLE joy_activities ADD COLUMN url TEXT NOT NULL DEFAULT '';
ALTER TABLE joy_activities ADD COLUMN verified_at INTEGER;
