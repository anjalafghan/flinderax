-- Spending allowance can be a percentage of income instead of a fixed amount. Stored in basis
-- points (2500 = 25%) so it follows the income if the salary changes. At most one of
-- spending_paise / spending_bps is set; neither = the 20% default.
ALTER TABLE user_settings ADD COLUMN spending_bps INTEGER CHECK (spending_bps BETWEEN 0 AND 10000);
