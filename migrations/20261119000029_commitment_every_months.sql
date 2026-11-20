-- A commitment that repeats every few months (a perfume every 4 months, a service every 6).
-- `frequency` stays 'monthly' or 'yearly' (its CHECK cannot be changed in place); a monthly
-- commitment with every_months > 1 is due only in every n-th month, counting from first_month
-- ('YYYY-MM') and never before it. The planner sets aside a n-th of the amount each month.
ALTER TABLE commitments ADD COLUMN every_months INTEGER NOT NULL DEFAULT 1 CHECK (every_months BETWEEN 1 AND 11);
ALTER TABLE commitments ADD COLUMN first_month TEXT;
