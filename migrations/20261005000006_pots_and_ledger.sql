-- HDFC/Kotak "pots", FD details, yearly bills, the transfer ledger and amount-based settings.

-- Account roles: 'spending' (salary lands here, SIP debits, the spending pot), 'reserve' (Kotak:
-- savings, card bills, needs), 'other'. FDs/RDs are 'locked' via the existing flag.
ALTER TABLE accounts ADD COLUMN role TEXT NOT NULL DEFAULT 'other' CHECK (role IN ('spending', 'reserve', 'other'));
ALTER TABLE accounts ADD COLUMN institution TEXT;  -- where an FD/RD is held
-- What an FD does at maturity: pay principal + interest out, renew principal only (interest
-- paid out), or renew principal + interest (compound).
ALTER TABLE accounts ADD COLUMN renewal TEXT NOT NULL DEFAULT 'payout' CHECK (renewal IN ('payout', 'renew_principal', 'renew_all'));
ALTER TABLE accounts ADD COLUMN payout_account_id TEXT REFERENCES accounts (account_id) ON DELETE SET NULL;

-- Yearly bills (phone, gym, internet) are set aside monthly and due once a year.
ALTER TABLE commitments ADD COLUMN frequency TEXT NOT NULL DEFAULT 'monthly' CHECK (frequency IN ('monthly', 'yearly'));
ALTER TABLE commitments ADD COLUMN due_month INTEGER CHECK (due_month BETWEEN 1 AND 12);
-- Where the money goes when it is a transfer to savings (an RD, an FD top-up...).
ALTER TABLE commitments ADD COLUMN to_account_id TEXT REFERENCES accounts (account_id) ON DELETE SET NULL;

-- Budget in amounts, not percentages: needs = your listed bills, spending = this amount,
-- savings = whatever is left.
ALTER TABLE user_settings ADD COLUMN spending_paise INTEGER;
ALTER TABLE user_settings ADD COLUMN allocation_day INTEGER NOT NULL DEFAULT 10 CHECK (allocation_day BETWEEN 1 AND 31);
ALTER TABLE user_settings ADD COLUMN savings_floor_paise INTEGER;
ALTER TABLE user_settings DROP COLUMN needs_pct;
ALTER TABLE user_settings DROP COLUMN savings_pct;
ALTER TABLE user_settings DROP COLUMN wants_pct;
ALTER TABLE user_settings DROP COLUMN everyday_paise;
ALTER TABLE user_settings DROP COLUMN cushion_paise;

-- Every "I moved the money" / "already covered" / "skip this month" click.
CREATE TABLE moves (
    move_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('card_bill', 'allocation')),
    -- card_bill: '<card_id>:<bill due date>'; allocation: the pay period's start date
    move_key TEXT NOT NULL,
    amount_paise INTEGER NOT NULL CHECK (amount_paise >= 0),
    mode TEXT NOT NULL CHECK (mode IN ('moved', 'already_covered', 'skipped')),
    from_account_id TEXT REFERENCES accounts (account_id) ON DELETE SET NULL,
    to_account_id TEXT REFERENCES accounts (account_id) ON DELETE SET NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
CREATE INDEX idx_moves_user_key ON moves (user_id, move_key);
