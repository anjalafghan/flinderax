-- Several photos per kept date. Each row is one stored file in the couple's photos folder.
-- The single photo a date could hold before (<history id>.jpg) becomes its first row.
CREATE TABLE joy_history_photos (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    history_id INTEGER NOT NULL REFERENCES joy_history(id) ON DELETE CASCADE,
    file       TEXT    NOT NULL,
    added_at   INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX joy_history_photos_by_history ON joy_history_photos (history_id, id);

INSERT INTO joy_history_photos (history_id, file, added_at)
SELECT id, id || '.jpg', picked_at FROM joy_history WHERE has_photo = 1;
