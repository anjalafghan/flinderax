-- One row for every slip the jar shows, with the chance it had of being shown. Putting a slip
-- back draws again, so a draw that took three tries leaves three rows. This is what lets a later
-- taste model be judged fairly (inverse-propensity replay): without the chance a slip had, there is
-- no telling "liked because it suits us" from "liked because it was drawn a lot".
CREATE TABLE joy_draws (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    couple_id TEXT NOT NULL REFERENCES couples (couple_id) ON DELETE CASCADE,
    activity_id INTEGER NOT NULL REFERENCES joy_activities (id) ON DELETE CASCADE,
    who INTEGER CHECK (who IN (0, 1)),
    probability REAL NOT NULL CHECK (probability > 0 AND probability <= 1),
    pool_size INTEGER NOT NULL CHECK (pool_size >= 1),
    at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX joy_draws_couple ON joy_draws (couple_id, at DESC);
CREATE INDEX joy_draws_activity ON joy_draws (activity_id);
