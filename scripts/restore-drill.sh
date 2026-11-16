#!/usr/bin/env bash
# Restore drill: proves a backup copy is a usable database. Works on a scratch copy only; it never
# connects to Fly or touches /data or any running database.
#
# Usage: scripts/restore-drill.sh path/to/joy-YYYY-MM-DD.db
# Needs: sqlite3. Optional: the repo's migrations/ folder is used for the migration count.
set -euo pipefail

SRC="${1:-}"
[ -n "$SRC" ] && [ -f "$SRC" ] || { echo "usage: $0 <backup.db>" >&2; exit 2; }

SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/joy-restore-drill.XXXXXX")"
trap 'rm -rf "$SCRATCH"' EXIT
step() { printf '\n== %s\n' "$*"; }

step "1/6 copy the backup to a scratch file ($SCRATCH/joy-app.db); the original is only read"
cp "$SRC" "$SCRATCH/joy-app.db"
# A restore must not bring a stale -wal/-shm along (see docs/backups.md).
rm -f "$SCRATCH/joy-app.db-wal" "$SCRATCH/joy-app.db-shm"
DB="$SCRATCH/joy-app.db"

step "2/6 integrity check"
r="$(sqlite3 "$DB" 'PRAGMA integrity_check;')"; echo "$r"
[ "$r" = "ok" ] || { echo "FAIL: integrity_check" >&2; exit 1; }

step "3/6 health check equivalent (SELECT 1, what /healthz runs)"
[ "$(sqlite3 "$DB" 'SELECT 1;')" = "1" ] || { echo "FAIL: SELECT 1" >&2; exit 1; }
echo ok

step "4/6 migrations: applied vs. shipped"
applied="$(sqlite3 "$DB" 'SELECT count(*) FROM _sqlx_migrations WHERE success = 1;')"
bad="$(sqlite3 "$DB" 'SELECT count(*) FROM _sqlx_migrations WHERE success = 0;')"
echo "applied: $applied, failed: $bad"
[ "$bad" = "0" ] || { echo "FAIL: a migration is marked failed" >&2; exit 1; }
if [ -d "$(dirname "$0")/../migrations" ]; then
  shipped="$(ls "$(dirname "$0")/../migrations"/*.sql | wc -l | tr -d ' ')"
  echo "shipped in this checkout: $shipped"
  [ "$applied" -le "$shipped" ] || echo "WARN: backup is newer than this checkout"
  [ "$applied" -ge "$shipped" ] || echo "note: backup is older; the app applies the remaining migrations at start"
fi

step "5/6 row counts"
for t in users unrecognized_alerts; do
  printf '%-22s %s\n' "$t" "$(sqlite3 "$DB" "SELECT count(*) FROM $t;")"
done
for t in $(sqlite3 "$DB" "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '\\_%' ESCAPE '\\' ORDER BY name;"); do
  printf '%s=%s ' "$t" "$(sqlite3 "$DB" "SELECT count(*) FROM \"$t\";")"
done; echo

step "6/6 foreign keys"
fk="$(sqlite3 "$DB" 'PRAGMA foreign_key_check;' | wc -l | tr -d ' ')"
echo "violations: $fk"
[ "$fk" = "0" ] || { echo "FAIL: foreign key violations" >&2; exit 1; }

printf '\nDrill passed on a scratch copy. Nothing else was touched; scratch removed.\n'
echo "Reminder: after a real restore you still need PASETO_KEY and DATES_URL_KEY set (docs/backups.md)."
