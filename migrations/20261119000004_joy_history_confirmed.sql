-- "Did it happen?" A kept date is only a plan until its day has passed and someone says so.
-- confirmed_at is set when they confirm it, rate it or enter what it cost. A date can also end as
-- outcome 'cancelled'. Dates already in the past when this ships count as confirmed.
ALTER TABLE joy_history ADD COLUMN confirmed_at INTEGER;
UPDATE joy_history SET confirmed_at = unixepoch() WHERE COALESCE(planned_at, picked_at) <= unixepoch();
