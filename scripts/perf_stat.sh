#!/usr/bin/env bash
set -euo pipefail

CMD="${1:-./bench/load.sh}"
echo "Running perf stat over: $CMD"

perf stat -e \
  task-clock,context-switches,cpu-migrations,page-faults,cycles,instructions,branches,branch-misses,cache-misses \
  -- bash -lc "$CMD"