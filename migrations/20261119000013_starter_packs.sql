-- Starter slips come in packs a couple chooses, instead of every couple getting the same 333
-- Mumbai slips. `starter` marks slips that came from a pack (the couple's own slips are drawn a
-- little more often than ones nobody wrote).
ALTER TABLE joy_seed_activities ADD COLUMN pack TEXT NOT NULL DEFAULT 'mumbai';
UPDATE joy_seed_activities SET pack = CASE setting WHEN 'home' THEN 'home' WHEN 'apart' THEN 'apart' ELSE 'mumbai' END;
ALTER TABLE joy_activities ADD COLUMN starter INTEGER NOT NULL DEFAULT 0;
UPDATE joy_activities SET starter = 1
 WHERE added_by = 'list' AND title IN (SELECT title FROM joy_seed_activities);
