-- Dates is part of this app, so there is no outside Joy to link to. Drops the table that held a
-- remote address and its API token in plaintext. `transactions.category_locked` stays: it is used
-- by categorisation, not by the link.
DROP TABLE IF EXISTS joy_links;
