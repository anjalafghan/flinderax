# Backups

Two copies of the database, both made by the first request of each day (the machine sleeps when
idle, so there is no scheduler):

1. **Local**: `VACUUM INTO` writes `backups/joy-YYYY-MM-DD.db` next to the database. The last 7 are kept.
2. **Off-site** (optional): the same copy is gzipped, encrypted with XChaCha20-Poly1305 and uploaded
   in the background to an S3-compatible bucket as `joy-YYYY-MM-DD.db.gz.enc`. The last 30 are kept.
   Upload failures are logged and never fail the request.

Off-site backups turn on when `BACKUP_KEY` and the S3 settings are both set. Without `BACKUP_KEY`
the app keeps only the local copies and logs once that off-site backup is disabled.

## Set up (Tigris on Fly)

1. Create the bucket. This also sets the `AWS_*` and `BUCKET_NAME` secrets on the app:

   ```sh
   fly storage create
   ```

2. Generate a key with the same build you deploy, and **save it in your password manager before
   going on**. Without this key the backups cannot be decrypted, and nothing can recover them:

   ```sh
   cargo run -q --bin joy -- gen-backup-key
   ```

3. Set it as a secret (the key is not printed to logs):

   ```sh
   fly secrets set BACKUP_KEY="<the key from step 2>"
   ```

   Setting a secret restarts the machine. The next request after the restart makes that day's copy
   and uploads it.

Settings the app reads (Tigris sets the Fly ones):

| Setting | Fly/Tigris name | Override |
| --- | --- | --- |
| Endpoint | `AWS_ENDPOINT_URL_S3` | `BACKUP_S3_ENDPOINT` |
| Bucket | `BUCKET_NAME` | `BACKUP_S3_BUCKET` |
| Region | `AWS_REGION` (`auto`) | `BACKUP_S3_REGION` (default `auto`) |
| Access key | `AWS_ACCESS_KEY_ID` | none |
| Secret key | `AWS_SECRET_ACCESS_KEY` | none |

Check that it works: `fly logs | grep "off-site"` should show `off-site backup joy-YYYY-MM-DD.db.gz.enc
uploaded`.

## Restore

Each backup is a single file. To read one, download it with any S3 client pointed at the endpoint
(for example `aws s3 cp s3://<bucket>/joy-YYYY-MM-DD.db.gz.enc . --endpoint-url https://fly.storage.tigris.dev`),
then decrypt and check it with the same build:

```sh
BACKUP_KEY="<the key>" cargo run -q --bin joy -- restore-backup joy-YYYY-MM-DD.db.gz.enc restored.db
```

It prints `ok` when the file decrypts (the magic, version and authentication tag are checked) and
`PRAGMA integrity_check` passes. Otherwise it prints the error and exits 1. It never overwrites an
existing file.

To put a restored copy into service, stop the app first. Move the current database file aside, and
also move any `<db>-wal` and `<db>-shm` sidecar files, since the app runs in WAL mode and a leftover
WAL would be replayed onto the restored file. Then copy `restored.db` to the `DATABASE_URL` path and
start the app again.

## Notes

- Lose `BACKUP_KEY` and the off-site copies are unreadable. Rotating it means a new key: old files
  still need the old key, so keep every key you have used.
- `joy gen-backup-key` prints only the key, so `$(...)` captures it cleanly.
- Local copies on the volume protect against bad writes, not against losing the volume. The
  off-site copy is the one that protects against that.
