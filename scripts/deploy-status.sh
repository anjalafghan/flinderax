#!/usr/bin/env bash
# Where is the latest release?  bun run deploy:status       one line: deployed / running / failed, and when
#                               bun run deploy:status -f    the same, then follow the live log
#                               bun run deploy:status -a    the last 10 releases
set -euo pipefail
ROOT="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel)"
DIR="$ROOT/logs/deploys"
S="$DIR/latest.status"

ago() { # seconds -> "2 min ago" / "3 h ago" / "2 days ago"
  local s=$1
  if   (( s < 90 ));     then echo "${s}s ago"
  elif (( s < 5400 ));   then echo "$(( s / 60 )) min ago"
  elif (( s < 172800 )); then echo "$(( s / 3600 )) h ago"
  else echo "$(( s / 86400 )) days ago"; fi
}
dur() { local s=$1; (( s < 90 )) && echo "${s}s" || echo "$(( s / 60 )) min"; }
at()  { date -r "$1" '+%a %d %b %H:%M' 2>/dev/null || date -d "@$1" '+%a %d %b %H:%M'; }

if [[ "${1:-}" == "-a" ]]; then
  for f in $(ls -1t "$DIR"/20*.log 2>/dev/null | head -10); do
    r="$({ grep -h '^==> RESULT' "$f" || true; } | tail -1)"
    echo "$(basename "$f" .log)  ${r:-==> (running or stopped)}"
  done
  exit 0
fi

[[ -f "$S" ]] || { echo "No releases yet. Start one with: bun run release"; exit 0; }
get() { { grep "^$1=" "$S" || true; } | tail -1 | cut -d= -f2-; }
now=$(date +%s); state=$(get state); step=$(get step); started=$(get started); finished=$(get finished)
commit=$(get commit); commit=${commit:-$(get develop)}

case "$state" in
  deployed) echo "✅ Deployed $commit $(ago $((now - finished))) ($(at "$finished"), took $(dur $((finished - started))))" ;;
  dry-run)  echo "🧪 Dry run passed $(ago $((now - finished))) ($(at "$finished")); nothing was pushed or deployed" ;;
  failed)   echo "❌ Failed at \"$step\" $(ago $((now - finished))) ($(at "$finished")). Log: $(get log)" ;;
  running)
    pid=$(get pid)
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      echo "⏳ Running for $(dur $((now - started))), now at: $step"
    else
      echo "⚠️  Stopped without finishing, last step \"$step\" (started $(ago $((now - started)))). Log: $(get log)"
    fi ;;
  *) echo "Unknown state; see $DIR/latest.log" ;;
esac

[[ "${1:-}" == "-f" ]] && exec tail -n 40 -f "$DIR/latest.log"
exit 0
