#!/usr/bin/env bash
# Ship develop: checks -> merge develop into master -> push both -> fly deploy -> /healthz.
#
#   bun run release                 start a release in the background and return at once
#   bun run release --skip-checks   same, without the test/lint suite (use when you just ran it)
#   bun run deploy:status           where the latest release is ("deployed 2 min ago", "failed at checks")
#   bun run deploy:log              follow the latest release's log live
#
# It works in its own temporary git worktree, so it never switches branches or touches uncommitted
# files in your checkout. Commit what you want shipped to develop first; only committed work ships.
# RELEASE_DRY_RUN=1 does everything except push, move master and fly deploy (it prints those instead).
# Logs: logs/deploys/<time>.log, with logs/deploys/latest.log and latest.status pointing at the newest.
set -Eeuo pipefail

ROOT="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel)"
DIR="$ROOT/logs/deploys"
APP_URL="${JOY_APP_URL:-https://joy.fly.dev}"

status_set() { # key=value pairs, rewritten whole so readers never see half a file
  local tmp="$DIR/latest.status.tmp"
  { cat "$DIR/latest.status" 2>/dev/null | grep -v -E "^($(printf '%s|' "${@%%=*}" | sed 's/|$//'))=" || true
    printf '%s\n' "$@"; } > "$tmp"
  mv "$tmp" "$DIR/latest.status"
}
act() { if [[ -n "${RELEASE_DRY_RUN:-}" ]]; then echo "(dry run) would run: $*"; else "$@"; fi; }
step() { echo; echo "==> [$(date '+%H:%M:%S')] $*"; status_set "step=$*"; }

# ---------------- background part ----------------
if [[ "${1:-}" == "--run" ]]; then
  shift
  SKIP_CHECKS="${1:-}"
  WT="$(mktemp -d "${TMPDIR:-/tmp}/joy-release.XXXXXX")"
  cleanup() { git -C "$ROOT" worktree remove --force "$WT" >/dev/null 2>&1 || rm -rf "$WT"; git -C "$ROOT" worktree prune; }
  fail() {
    echo; echo "==> RESULT FAILED at: $(grep '^step=' "$DIR/latest.status" | cut -d= -f2-)"
    status_set "state=failed" "finished=$(date +%s)"; cleanup; exit 1
  }
  trap fail ERR
  trap 'echo "==> stopped"; status_set "state=failed" "step=stopped" "finished=$(date +%s)"; cleanup; exit 1' INT TERM
  # Keep the Mac awake until this script exits: a sleeping Mac drops the connection to the builder and
  # the deploy hangs. -i stops idle sleep; -s also stops system sleep while on power.
  command -v caffeinate >/dev/null && { caffeinate -is -w $$ & }

  step "worktree"
  git -C "$ROOT" worktree add --detach "$WT" develop
  cd "$WT"
  ln -s "$ROOT/node_modules" node_modules
  ln -s "$ROOT/web/node_modules" web/node_modules
  export SQLX_OFFLINE=true CARGO_TARGET_DIR="$ROOT/target"

  if [[ "$SKIP_CHECKS" != "--skip-checks" ]]; then
    step "checks: cargo test";   cargo test --all-targets
    step "checks: clippy";       cargo clippy --all-targets -- -D warnings
    step "checks: typecheck";    bun run typecheck
    step "checks: lint";         bun run --cwd web lint
    step "checks: web tests";    bun run --cwd web test
  else
    echo "(checks skipped)"
  fi

  step "merge develop into master"
  git checkout --detach master
  if git merge-base --is-ancestor develop HEAD; then
    echo "master already has develop; nothing to merge"
  else
    git merge --no-ff develop -m "Merge develop: $(git log -1 --format=%s develop)"
  fi
  SHIP="$(git rev-parse --short HEAD)"
  status_set "commit=$SHIP"

  step "push develop and master"
  act git push origin develop "HEAD:refs/heads/master"
  # Move the local master branch too, unless a checkout is sitting on it (then: git pull there).
  if git -C "$ROOT" worktree list --porcelain | grep -qx 'branch refs/heads/master'; then
    echo "note: master is checked out in a worktree; run 'git pull --ff-only' there to catch up"
  else
    act git -C "$ROOT" branch -f master HEAD
  fi

  # Fast path: compile on this Mac (minutes, deps cached in target/) and ship a copy-only image.
  # Slow path, when cargo-zigbuild isn't installed: Fly's remote builder compiles from source (~35 min).
  # (`cargo zigbuild --version` is rejected by cargo-zigbuild 0.24, so look for the binary itself.)
  if command -v zig >/dev/null && command -v cargo-zigbuild >/dev/null; then
    step "build on this Mac"
    STAGE="$WT/.prebuilt"
    bash "$ROOT/scripts/build-prebuilt.sh" "$STAGE"
    # Build the tiny image with local Docker when it answers; otherwise Fly's builder (it only copies files).
    if docker info >/dev/null 2>&1; then BUILDER=--local-only; else BUILDER=--remote-only; fi
    step "fly deploy (prebuilt, $BUILDER)"
    (cd "$STAGE" && act fly deploy --depot=false "$BUILDER")
  else
    echo "note: cargo-zigbuild not found, so Fly's builder compiles from source (slow)."
    echo "      For fast deploys: brew install zig && cargo install cargo-zigbuild --locked"
    step "fly deploy"
    act fly deploy --depot=false
  fi

  step "health check"
  for i in $(seq 1 12); do
    if curl -fsS --max-time 10 "$APP_URL/healthz" >/dev/null; then ok=1; break; fi
    echo "healthz not ready yet ($i/12)"; sleep 5
  done
  [[ "${ok:-}" == 1 ]] || { echo "healthz never answered"; false; }

  trap - ERR
  cleanup
  if [[ -n "${RELEASE_DRY_RUN:-}" ]]; then
    status_set "state=dry-run" "step=done" "finished=$(date +%s)"
    echo; echo "==> RESULT OK (dry run, nothing pushed or deployed): $SHIP at $(date '+%Y-%m-%d %H:%M:%S')"
  else
    status_set "state=deployed" "step=done" "finished=$(date +%s)"
    echo; echo "==> RESULT OK: deployed $SHIP to $APP_URL at $(date '+%Y-%m-%d %H:%M:%S')"
  fi
  exit 0
fi

# ---------------- foreground part: start and return ----------------
mkdir -p "$DIR"
if [[ -f "$DIR/latest.status" ]] && grep -qx 'state=running' "$DIR/latest.status"; then
  pid="$(grep '^pid=' "$DIR/latest.status" | cut -d= -f2)"
  if kill -0 "$pid" 2>/dev/null; then
    echo "A release is already running (pid $pid). Watch it with: bun run deploy:log"; exit 1
  fi
fi
command -v fly >/dev/null || { echo "fly CLI not found"; exit 1; }
git -C "$ROOT" fetch -q origin
# Refuse to ship over work that only exists on GitHub.
for b in develop master; do
  if ! git -C "$ROOT" merge-base --is-ancestor "origin/$b" "$b"; then
    echo "origin/$b has commits your local $b doesn't. Pull them first (git fetch; merge into $b)."; exit 1
  fi
done

TS="$(date '+%Y-%m-%d_%H-%M-%S')"
LOG="$DIR/$TS.log"
ln -sfn "$TS.log" "$DIR/latest.log"
printf 'state=running\nstep=starting\nstarted=%s\nlog=%s\ndevelop=%s\n' \
  "$(date +%s)" "$LOG" "$(git -C "$ROOT" rev-parse --short develop)" > "$DIR/latest.status"
{ echo "joy release $TS"; echo "develop: $(git -C "$ROOT" log -1 --format='%h %s' develop)"; } > "$LOG"

RELEASE_DRY_RUN="${RELEASE_DRY_RUN:-}" JOY_APP_URL="$APP_URL" nohup bash "$0" --run "${1:-}" >> "$LOG" 2>&1 < /dev/null &
echo "pid=$!" >> "$DIR/latest.status"
disown
echo "Release started in the background (develop $(git -C "$ROOT" rev-parse --short develop))."
echo "  status: bun run deploy:status"
echo "  live:   bun run deploy:log"
