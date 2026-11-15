-- A client-generated key makes a retried or double-sent "I moved it" a no-op, however late it arrives.
ALTER TABLE moves ADD COLUMN idempotency_key TEXT;
CREATE UNIQUE INDEX moves_idempotency ON moves (user_id, idempotency_key) WHERE idempotency_key IS NOT NULL;
