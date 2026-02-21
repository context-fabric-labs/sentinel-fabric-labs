#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "flamegraph.sh: Linux-only (perf). Skipping."
  exit 0
fi

OUT="${1:-bench/results/flame.svg}"
shift || true

mkdir -p "$(dirname "$OUT")"

perf record -F 99 -g -- "$@"
perf script | inferno-collapse-perf | inferno-flamegraph > "$OUT"
echo "wrote $OUT"
