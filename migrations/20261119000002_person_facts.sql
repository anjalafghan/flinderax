-- Things to remember about a person (allergies, kids' names, what they love). Private to each user.
CREATE TABLE person_facts (
    fact_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES people (person_id) ON DELETE CASCADE,
    text TEXT NOT NULL CHECK (length(text) BETWEEN 1 AND 400),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_person_facts_person ON person_facts (person_id);
CREATE INDEX idx_person_facts_user ON person_facts (user_id);
