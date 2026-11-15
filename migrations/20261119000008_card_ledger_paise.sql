-- The original card ledger kept rupees as REAL. Everything else is integer paise, and so is this now.
ALTER TABLE card_events ADD COLUMN total_due_input_paise INTEGER NOT NULL DEFAULT 0;
UPDATE card_events SET total_due_input_paise = CAST(ROUND(total_due_input * 100) AS INTEGER);
ALTER TABLE card_events DROP COLUMN total_due_input;

ALTER TABLE card_running_state ADD COLUMN last_total_due_paise INTEGER NOT NULL DEFAULT 0;
ALTER TABLE card_running_state ADD COLUMN last_delta_paise INTEGER NOT NULL DEFAULT 0;
UPDATE card_running_state SET last_total_due_paise = CAST(ROUND(last_total_due * 100) AS INTEGER),
                              last_delta_paise = CAST(ROUND(last_delta * 100) AS INTEGER);
ALTER TABLE card_running_state DROP COLUMN last_total_due;
ALTER TABLE card_running_state DROP COLUMN last_delta;
