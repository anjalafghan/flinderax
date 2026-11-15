#!/usr/bin/env bash
# Starts a throw-away joy server for the browser tests: a fresh database, the built web app, port 3123.
set -euo pipefail
cd "$(dirname "$0")/.."
rm -rf e2e/.tmp && mkdir -p e2e/.tmp
bun run build >/dev/null
SQLX_OFFLINE=true cargo build --bin joy
export PORT=3123
export DATABASE_URL="sqlite://$PWD/e2e/.tmp/e2e.db?mode=rwc"
export JOY_PHOTOS_DIR="$PWD/e2e/.tmp/photos"
export PASETO_KEY="$(head -c 32 /dev/zero | base64)"
export SIGNUP_CODE="e2e-code"
export TZ="Asia/Kolkata"
export STATIC_DIR="$PWD/web/dist"
exec target/debug/joy
