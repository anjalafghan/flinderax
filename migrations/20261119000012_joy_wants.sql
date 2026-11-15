-- Each of us can quietly mark a slip "want", "fine" or "no". Nobody sees the other's answer; the
-- jar only shows "you both want this" when both said want, and the draw leans on the overlap.
CREATE TABLE joy_wants (
    activity_id INTEGER NOT NULL REFERENCES joy_activities (id) ON DELETE CASCADE,
    person INTEGER NOT NULL CHECK (person IN (0, 1)),
    level TEXT NOT NULL CHECK (level IN ('want', 'fine', 'no')),
    PRIMARY KEY (activity_id, person)
);
