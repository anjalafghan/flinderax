-- Bumped on password change and "sign out everywhere"; a token carries the value it was issued
-- with, so raising it ends every older session at once.
ALTER TABLE users ADD COLUMN token_version INTEGER NOT NULL DEFAULT 0;
