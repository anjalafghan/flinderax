-- Add migration script here
ALTER TABLE cards RENAME card_default_color TO card_primary_color;
ALTER TABLE cards ADD COLUMN card_secondary_color TEXT DEFAULT '';
