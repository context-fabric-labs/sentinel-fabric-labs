#!/usr/bin/env bash
set -euo pipefail

cargo run -q -p upstream_stub &
UP_PID=$!

cargo run -q -p sentinel &
SEN_PID=$!

echo "upstream_stub pid=$UP_PID (4000), sentinel pid=$SEN_PID (3000)"
echo "ctrl-c to stop"

cleanup() {
  kill $SEN_PID >/dev/null 2>&1 || true
  kill $UP_PID  >/dev/null 2>&1 || true
}
trap cleanup INT TERM EXIT

wait
