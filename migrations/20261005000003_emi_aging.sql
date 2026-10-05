-- EMIs age with the calendar: `months_left` / `remaining_principal_paise` are true as of
-- `as_of_month` ('YYYY-MM'); the app derives today's values from how many months have passed.
ALTER TABLE card_emis ADD COLUMN as_of_month TEXT;
UPDATE card_emis SET as_of_month = strftime('%Y-%m', created_at);
