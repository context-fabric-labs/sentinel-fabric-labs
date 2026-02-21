#!/usr/bin/env bash
set -euo pipefail

URL="${1:-http://127.0.0.1:3000/health}"
CONC="${CONC:-32}"
REQ="${REQ:-2000}"

cargo run -q -p bench -- run --scenario health --url "$URL" --concurrency "$CONC" --requests "$REQ"
