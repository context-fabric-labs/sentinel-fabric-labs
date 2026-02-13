#!/usr/bin/env bash
set -euo pipefail

# Upstream stub
cargo run -q -p upstream_stub -- --bind 127.0.0.1:8000 &
UP_PID=$!

# Sentinel
UPSTREAM_URL="http://127.0.0.1:8000" \
cargo run -q -p sentinel -- --bind 127.0.0.1:8080 &
SENT_PID=$!

echo "Upstream PID=$UP_PID Sentinel PID=$SENT_PID"
echo "Try: curl -s http://127.0.0.1:8080/health"
wait

