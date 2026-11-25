-- "Not this month" for a payment every few months (a perfume, new clothes). Putting it off moves
-- first_month to the month after it was due, so the schedule counts on from when it is really
-- bought. waiting_from ('YYYY-MM') keeps the month it was first due: the money for it is already
-- set aside, so nothing more is set aside from then until it falls due. NULL when not put off.
ALTER TABLE commitments ADD COLUMN waiting_from TEXT;
