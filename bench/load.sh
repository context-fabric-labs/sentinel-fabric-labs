#!/usr/bin/env bash
set -euo pipefail

URL="${1:-http://127.0.0.1:8080/v1/chat/completions}"
oha -z 20s -c 50 -m POST \
  -H "content-type: application/json" \
  -d '{"model":"stub","messages":[{"role":"user","content":"hello"}]}' \
  "$URL"