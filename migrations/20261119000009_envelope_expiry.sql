-- Envelope links expire. A slip's lasts 30 days from when it was last shared; a kept date's lasts a
-- week past the day (at least 30 days for links already out there). Sharing again renews it, and
-- "stop sharing" clears the token. NULL means the row has never been shared.
ALTER TABLE joy_activities ADD COLUMN share_expires_at INTEGER;
ALTER TABLE joy_history ADD COLUMN share_expires_at INTEGER;
UPDATE joy_activities SET share_expires_at = unixepoch() + 30 * 86400 WHERE share_token IS NOT NULL;
UPDATE joy_history SET share_expires_at = MAX(COALESCE(planned_at, picked_at) + 7 * 86400, unixepoch() + 30 * 86400)
 WHERE share_token IS NOT NULL;
