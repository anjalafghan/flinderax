#!/usr/bin/env bash
# Build what Dockerfile.prebuilt ships, natively on this Mac, into a staging folder:
#   bash scripts/build-prebuilt.sh <staging dir>
# Cross-compiles the server for x86_64 Linux (Fly's machines) with cargo-zigbuild, builds the web app,
# and copies both plus fly.toml and the Dockerfile into the folder, ready for `fly deploy` there.
# Needs once:  brew install zig && cargo install cargo-zigbuild --locked && rustup target add x86_64-unknown-linux-gnu
# Run from the repo (or worktree) root. CARGO_TARGET_DIR is honoured, so deps stay cached between releases.
set -Eeuo pipefail
OUT="${1:?usage: build-prebuilt.sh <staging dir>}"
TARGET=x86_64-unknown-linux-gnu
TDIR="${CARGO_TARGET_DIR:-target}"

command -v zig >/dev/null && command -v cargo-zigbuild >/dev/null || {
  echo "cargo-zigbuild or zig missing: brew install zig && cargo install cargo-zigbuild --locked"; exit 1; }

echo "--> web build"
bun run --cwd web build

echo "--> server build ($TARGET, profile deploy)"
SQLX_OFFLINE=true cargo zigbuild --profile deploy --bin joy --target "$TARGET"

mkdir -p "$OUT"
rm -rf "$OUT/dist"
cp -R web/dist "$OUT/dist"
cp "$TDIR/$TARGET/deploy/joy" "$OUT/joy"
cp "$(dirname "$0")/../Dockerfile.prebuilt" "$OUT/Dockerfile"
cp fly.toml "$OUT/fly.toml"
echo "--> staged in $OUT ($(du -sh "$OUT/joy" | cut -f1) binary)"
