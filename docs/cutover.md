# Cutover: two apps → one (`joy-jar`)

Moves production from two Fly apps (Joy on `joy-jar`, Flinderax on `flinderax`) to **one machine**:
the merged `joy` build on `joy-jar`. The old Joy database (`/data/joy.db`) and photos (`/data/photos`)
are **never modified**. The merged app uses a new file (`/data/joy-app.db`) and copies photos to
`/data/joy-photos`, so rolling back is just redeploying Joy's last image.

Rehearsed on copies of both production databases (8 Nov 2026):
- every migration applied cleanly and `PRAGMA foreign_key_check` came back clean
- the Joy import matched field for field: 333 slips, 50 dates, 22 pairs, the memory line, the regular, the budget, ₹5,670 for October, envelope tokens, and a byte-identical photo
- the existing cards survived

Do every step from a clean checkout of the commit being deployed (`git worktree add`), never the
working folder: `fly deploy` uploads the folder as it is.

## 0. Before

| Check | Command |
|---|---|
| Local copy of Joy's data | `joy/data/prod-backup-2026-11-08/` (sha256 of joy.db `9f6787…`) |
| Fresh Fly snapshots | `fly volumes snapshots create <vol> -a joy-jar` and the same for `-a flinderax` |
| You know which Joy person you are | `PERSON_A` → `--as a`, `PERSON_B` → `--as b` |
| Nobody uses Joy or Flinderax during the move | ~10 minutes |

## 1. Put Flinderax's database on Joy's volume

```bash
fly ssh sftp get -a flinderax /data/flinderax.db ./flinderax-live.db
fly ssh console -a joy-jar -C "ls -la /data"          # joy-app.db must NOT exist yet
fly ssh sftp put -a joy-jar ./flinderax-live.db /data/joy-app.db
```

Joy's current build ignores this file, so nothing changes yet.

## 2. Secrets

Copy the login key, so Flinderax logins stay valid. It goes through a pipe and is never printed:

```bash
fly ssh console -a flinderax -C "sh -c 'echo PASETO_KEY=\$PASETO_KEY'" | grep '^PASETO_KEY=.' | tr -d '\r' \
  | fly secrets import -a joy-jar --stage
```

`fly secrets list -a joy-jar` must show the same `PASETO_KEY` digest as on `flinderax`.

Joy's settings secrets (`HOME_A`, `DATE_DAYS`, `OFFICE`…) stay set for step 4, which copies them into the database.

## 3. Deploy the merged build

```bash
git worktree add --detach ../joy-deploy <commit>
cd ../joy-deploy && fly deploy --depot=false      # fly.toml: app joy-jar, one machine, 512 MB
fly scale count 1 -a joy-jar                      # exactly one machine
```

On boot, the migrations bring `/data/joy-app.db` up to date. Check:

```bash
curl -s -o /dev/null -w "%{http_code}\n" https://joy-jar.fly.dev/                          # 200
curl -s -o /dev/null -w "%{http_code}\n" https://joy-jar.fly.dev/api/dashboard/summary     # 401
```

## 4. Import Joy into your account

```bash
fly ssh console -a joy-jar -C "joy import-joy /data/joy.db anjalafghan --as a --photos /data/photos --dry-run"
fly ssh console -a joy-jar -C "joy import-joy /data/joy.db anjalafghan --as a --photos /data/photos"
```

The dry run must report 333 slips, 50 dates, 22 pairs, 1 memory line, 1 regular, 1 night and 1 photo, plus about 20 settings. Run the import once only; a second run is refused.

## 5. Check in the app

1. Log in as `anjalafghan`. Dates shows the jar with your slips, Kept shows the October date with its photo, and Year shows 2026.
2. Old links redirect: `https://joy-jar.fly.dev/e/<token>` opens the envelope, and `/wishlist`, `/history` and `/year/…` land in Dates.
3. Dates → Settings → set "Your partner covers ₹6,000 a month".
4. Make an invite code there. Your partner registers on `https://joy-jar.fly.dev`, then joins with the code.
5. Both phones: delete the old Joy home-screen icon and add the new one (Share → Add to Home Screen).

## 6. Clean up (only after a few days of normal use)

```bash
fly secrets unset -a joy-jar APP_PASSCODE_HASH SESSION_SECRET   # Joy's old login, unused now
fly apps destroy flinderax          # permanent: its volume and snapshots go too
fly apps destroy flinderax-backend  # permanent
```

Keep `/data/joy.db` and `/data/photos` on the volume as the archive; they're small.

## Rollback

Joy's data was never touched, so going back is a single deploy of Joy's last release:

```bash
fly releases -a joy-jar --image          # note the image of the last release before the merge (v27)
fly deploy -a joy-jar --image <that image>
```

Flinderax keeps running on `flinderax` until step 6, so it needs no rollback.
