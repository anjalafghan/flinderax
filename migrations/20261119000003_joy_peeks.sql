-- Putting a slip back while drawing is a peek, not a date: it no longer writes into joy_history.
-- `reason` is the optional one-tap why. Only 'not_my_thing' teaches the taste model.
CREATE TABLE joy_peeks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    couple_id TEXT NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    activity_id INTEGER NOT NULL REFERENCES joy_activities (id) ON DELETE CASCADE,
    who INTEGER CHECK (who IN (0, 1)),
    reason TEXT NOT NULL DEFAULT '' CHECK (reason IN ('', 'not_tonight', 'too_far', 'too_pricey', 'not_my_thing')),
    at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX joy_peeks_couple ON joy_peeks (couple_id, at DESC);
CREATE INDEX joy_peeks_activity ON joy_peeks (activity_id);
