-- A regular night is the same thing as any kept date: going to one creates a kept date (a
-- history row), and the visit remembers which. Visits without one (older ones) are given one at
-- start-up. The visit still records the habit (which day, skipped or went).
ALTER TABLE joy_regular_visits ADD COLUMN history_id INTEGER REFERENCES joy_history (id) ON DELETE SET NULL;
CREATE INDEX joy_regular_visits_history ON joy_regular_visits (history_id);
