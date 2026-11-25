# Backups

## What exists

- **Daily copy, on the volume.** The first request of each day (the machine sleeps when idle, so
  there is no scheduler) makes a copy of the database: `VACUUM INTO` writes
  `backups/joy-YYYY-MM-DD.db` next to the database (`/data/backups/`), and the last 7 are kept
  (`BACKUP_KEEP` changes the count, `BACKUP_DIR` the folder). A failure is logged and never fails the
  request. These protect against a bad write or a bad deploy, **not** against losing the volume.
- **Off-volume pull.** `POST /api/internal/backup/run` takes a fresh `VACUUM INTO` snapshot and
  returns it as the response body (`application/octet-stream`), so a scheduled job anywhere else can
  store it. It uses the same kind of secret as the push round: `Authorization: Bearer <token>`,
  where the token is `BACKUP_RUN_TOKEN` (or, if that is unset, `PUSH_RUN_TOKEN`; 8+ characters).
  With neither set it answers 404; a wrong token answers 401. Waking the sleeping machine is part
  of the point, like the push round.

  ```sh
  curl -fsS -X POST -H "Authorization: Bearer $BACKUP_RUN_TOKEN" \
    https://joy.fly.dev/api/internal/backup/run -o "joy-$(date +%F).db"
  sqlite3 "joy-$(date +%F).db" "PRAGMA integrity_check;"
  ```

  Run that from a scheduler outside Fly (a cron on your own machine, or a scheduled CI job writing to
  storage you control) and keep your own retention there. The token lives in that scheduler's secret
  store and in `fly secrets`, never in the repo. The old S3 backup is gone; `BACKUP_KEY`, `AWS_*` and
  `BUCKET_NAME` are no longer read and can be removed with `fly secrets unset`.

- **By hand.**

  ```sh
  fly ssh console -a joy -C "ls /data/backups"
  fly ssh sftp get -a joy /data/backups/joy-YYYY-MM-DD.db ./joy-YYYY-MM-DD.db
  ```

## Check a copy (safe, scratch only)

```sh
scripts/restore-drill.sh ./joy-YYYY-MM-DD.db
```

It copies the file to a temp folder, then runs `integrity_check`, `SELECT 1` (what `/healthz`
runs), compares applied migrations with `migrations/`, prints row counts and checks foreign keys.
It never touches Fly or a running database. `cargo test --test backup_restore` does the same in
code: snapshot, open in a fresh pool, migrations no-op, row counts, `/healthz` answers 200.
Run the drill after any change to the backup code and now and then on a real pulled copy.

## Restoring production

Use this when the live database is damaged or a bad write has to be undone. The copy you restore
loses everything written after it was taken, so pick the newest good one.

1. **Pick a snapshot.** Either a daily copy on the volume (`fly ssh console -a joy -C "ls /data/backups"`)
   or an off-volume copy you pulled (see "Off-volume pull"). Bring it to your machine:
   `fly ssh sftp get -a joy /data/backups/joy-YYYY-MM-DD.db ./restore.db`.
2. **Check it with the drill.** `scripts/restore-drill.sh ./restore.db`. Stop here if it fails and try an
   older copy. The drill works on a scratch copy and never touches Fly.
3. **Stage it on the volume, beside the live file.** While a machine is up:
   `fly ssh sftp shell -a joy`, then `put ./restore.db /data/joy-restore.db`. This does not touch the
   running database. (Check the size with `ls -l /data` afterwards.)
4. **Stop the app so nothing writes.** `fly machine list -a joy` for the machine id, then
   `fly machine stop <id> -a joy`.
5. **Swap the file and remove the stale WAL.** With the app stopped, on the volume:
   `mv /data/joy-app.db /data/joy-app.db.bad`, `mv /data/joy-restore.db /data/joy-app.db`,
   `rm -f /data/joy-app.db-wal /data/joy-app.db-shm`. The old WAL must go: replayed over the restored
   file it would corrupt it. Keep the `.bad` file until you are sure.
   *Uncertain:* a stopped machine has no shell, so this step needs a way to reach the volume while the
   app is not running. The usual route is a one-off machine that mounts `joy_data`
   (`fly machine run <image> --volume joy_data:/data -a joy`, with an image that has a shell, then
   `fly ssh console` into it and stop it afterwards); check the current `fly machine run` flags before
   relying on this. If that is not workable, do steps 5 and 6 on the running machine and restart it
   immediately: `fly ssh console -a joy`, run the three commands, `exit`, then
   `fly machine restart <id> -a joy`.
6. **Start.** `fly machine start <id> -a joy` (after a one-off machine, stop that one first so two
   machines never share the volume). The app applies any migrations newer than the copy at start.
7. **Verify.** `curl -fsS https://joy.fly.dev/healthz` (it runs `SELECT 1`), then sign in and look at the
   dashboard and the Dates page. If they look right, delete `/data/joy-app.db.bad` later.

## After a restore: things to know

- **Secrets are not in the database.** `PASETO_KEY` (login tokens) and `DATES_URL_KEY` (photo,
  calendar and envelope links; without it the key derives from `PASETO_KEY`) are Fly secrets. If a
  secret is missing or changed, everyone is signed out (`PASETO_KEY`) or every signed link already
  sent stops working (`DATES_URL_KEY`). Set the same values as before; also `PUSH_RUN_TOKEN` /
  `BACKUP_RUN_TOKEN` for the scheduled jobs.
- **Photos are files, not rows.** `JOY_PHOTOS_DIR` (`/data/joy-photos`) is on the volume and is not
  part of the SQLite copy. Back it up separately if photos matter.
- **`unrecognized_alerts` keeps 30 days.** Bank texts the parser could not read are purged after 30
  days on the next ingest. A restored copy holds only those it had at snapshot time, and older rows
  in it are purged again once a text arrives. Anything received after the snapshot is lost; texts
  from that window may be re-sent from the phone if still needed.
- **Idempotency keys and offline outboxes.** Writes after the snapshot are gone; a client outbox
  replaying a request whose key predates the snapshot will simply be applied again.
- **Push subscriptions** are in the database; browsers keep their end, so notifications resume.
