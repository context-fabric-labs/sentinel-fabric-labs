#!/usr/bin/env bash
set -euo pipefail
URL=${1:-http://127.0.0.1:8080/v1/chat/completions}
CONC_LIST=(1 2 4 8 16 32 64)
REQS=${REQS:-2000}
WARMUP=${WARMUP:-200}
BODY='{"model":"stub","messages":[{"role":"user","content":"hello"}],"max_tokens":16,"stream":false}'

mkdir -p bench/results
for c in "${CONC_LIST[@]}"; do
  echo "Running chat sweep: c=$c"
  cargo run -q -p bench -- run --scenario chat --url "$URL" \
    --concurrency "$c" --requests "$REQS" --warmup-requests "$WARMUP" \
    --body-json "$BODY" \
    --out "bench/results/chat-c${c}-$(date +%Y%m%d-%H%M%S).json"
done
