-- The photo we chose to stand for a kept date: shown on the Month page, the Year page and as the
-- cover in Kept. NULL means "the oldest one", which is how every date looked before this.
ALTER TABLE joy_history ADD COLUMN hero_photo_id INTEGER;
