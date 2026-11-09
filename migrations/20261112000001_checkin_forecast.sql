-- The forecast as it stood at each finished check-in, for the next one to compare with.
ALTER TABLE checkins ADD COLUMN forecast_json TEXT;
