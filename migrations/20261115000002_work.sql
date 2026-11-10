-- Work: a small job-search tracker. One row per lead (a company and role being pursued).
-- `updated_on` moves whenever the lead changes, so a stalled search can be noticed.
CREATE TABLE job_leads (
    lead_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    company TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT '',
    stage TEXT NOT NULL DEFAULT 'idea' CHECK (stage IN ('idea', 'applied', 'interview', 'offer', 'closed')),
    take_home_paise INTEGER CHECK (take_home_paise > 0),   -- monthly, expected or offered
    next_step TEXT NOT NULL DEFAULT '',
    next_on TEXT,                                          -- 'YYYY-MM-DD'
    note TEXT NOT NULL DEFAULT '',
    updated_on TEXT NOT NULL,                              -- 'YYYY-MM-DD'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_job_leads_user ON job_leads (user_id, stage);

-- Hours of job preparation in a week, in tenths, logged on the weekly screen.
ALTER TABLE diet_weeks ADD COLUMN prep_hours_x10 INTEGER CHECK (prep_hours_x10 BETWEEN 0 AND 1680);
