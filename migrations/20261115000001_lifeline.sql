-- Life line choices: things the user could do (switch job, buy a home, retire earlier), kept so
-- they can be switched on and off against the plan. `choice_json` is a planner::lifeline::Choice.
-- `in_plan = 1` means the choice is part of the plan the life line is drawn from.
CREATE TABLE life_choices (
    choice_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    choice_json TEXT NOT NULL,
    in_plan INTEGER NOT NULL DEFAULT 0 CHECK (in_plan IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_life_choices_user ON life_choices (user_id);
