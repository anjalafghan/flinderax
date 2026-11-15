# Backups

The first request of each day (the machine sleeps when idle, so there is no scheduler) makes a copy
of the database: `VACUUM INTO` writes `backups/joy-YYYY-MM-DD.db` next to the database, and the last
7 are kept. A failure is logged and never fails the request.

The copies live on the same Fly volume as the database, so they protect against a bad write or a
bad deploy, not against losing the volume. For that, pull a copy to your own machine now and then.

## Pull a backup to this machine

```sh
fly ssh console -a joy -C "ls /data/backups"
fly ssh sftp get -a joy /data/backups/joy-YYYY-MM-DD.db ./joy-YYYY-MM-DD.db
```

Check the copy:

```sh
sqlite3 joy-YYYY-MM-DD.db "PRAGMA integrity_check;"
```

## Restore

Stop the app (`fly scale count 0 -a joy`), put the copy at `/data/joy-app.db` with `fly ssh sftp`,
delete `/data/joy-app.db-wal` and `/data/joy-app.db-shm`, then `fly scale count 1 -a joy`.

The old off-site (S3) backup is gone. The `BACKUP_KEY`, `AWS_*` and `BUCKET_NAME` secrets on the app
are no longer read; remove them with `fly secrets unset`.
