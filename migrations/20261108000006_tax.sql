-- India tax tracker: manual deduction entries per financial year, and a tag on fixed payments
-- (an ELSS SIP -> 80C) so their instalments count towards a section.
CREATE TABLE tax_entries (
    entry_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    fy TEXT NOT NULL,                                -- '2026-27' (April to March)
    section TEXT NOT NULL CHECK (section IN ('80C', '80CCD1B', '80D', 'HRA', '24B', 'other')),
    label TEXT NOT NULL,
    amount_paise INTEGER NOT NULL CHECK (amount_paise >= 0),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
CREATE INDEX idx_tax_entries_user_fy ON tax_entries (user_id, fy);

ALTER TABLE commitments ADD COLUMN tax_section TEXT
    CHECK (tax_section IS NULL OR tax_section IN ('80C', '80CCD1B', '80D', 'HRA', '24B', 'other'));
