-- A child's education plan: the stages (school, college...) with their cost in today's money, the
-- assumptions for how fast fees rise and what savings earn, and the money already set aside.
-- One row per person, like the life profile it sits beside. Money is integer paise and rates are
-- basis points. `stages_json` is a list of planner::education::Stage.
CREATE TABLE education_plan (
    user_id TEXT PRIMARY KEY REFERENCES users (user_id) ON DELETE CASCADE,
    child_birth_year INTEGER NOT NULL,
    stages_json TEXT NOT NULL,
    inflation_bps INTEGER NOT NULL CHECK (inflation_bps BETWEEN 0 AND 2000),
    return_bps INTEGER NOT NULL CHECK (return_bps BETWEEN 0 AND 3000),
    corpus_paise INTEGER NOT NULL DEFAULT 0 CHECK (corpus_paise >= 0),
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
