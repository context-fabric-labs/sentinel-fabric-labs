#!/usr/bin/env bash
set -euo pipefail

cargo run -q -p upstream_stub &
UP_PID=$!

export UPSTREAM_URL="${UPSTREAM_URL:-http://127.0.0.1:4000}"
export SENTINEL_ADDR="${SENTINEL_ADDR:-127.0.0.1:8080}"

cargo run -q -p sentinel &
SEN_PID=$!

echo "upstream_stub pid=$UP_PID (4000), sentinel pid=$SEN_PID (8080)"
echo "ctrl-c to stop"

cleanup() {
  kill $SEN_PID >/dev/null 2>&1 || true
  kill $UP_PID  >/dev/null 2>&1 || true
}
trap cleanup INT TERM EXIT

wait