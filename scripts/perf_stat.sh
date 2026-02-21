#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "perf_stat.sh: Linux-only (perf). Skipping."
  exit 0
fi

OUT="${1:-bench/results/perf_stat.txt}"
shift || true

mkdir -p "$(dirname "$OUT")"

EVENTS="context-switches,cpu-migrations,page-faults,cycles,instructions,cache-misses,dTLB-load-misses,LLC-load-misses"

echo "running: perf stat -e $EVENTS -o $OUT -- $*"
perf stat -e "$EVENTS" -o "$OUT" -- "$@"
echo "wrote $OUT"
