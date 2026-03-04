#!/bin/bash

# Epic 3 - Quick test script
# This script runs a quick test of all the labs in Epic 3

set -e  # Exit on any error

echo "Running quick Epic 3 tests..."
echo "============================"

# Create results directory if it doesn't exist
mkdir -p labs/results

# Get current timestamp for unique filenames
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

echo "Running GPU Exporter (quick test)..."
cargo run -p gpu_exporter \
  -- --gpu-id 0 \
  --requests 100 \
  --warmup-requests 10 \
  --out labs/results/s31-gpu-metrics-${TIMESTAMP}.json

echo "Running GPU Guardrail (quick test)..."
cargo run -p gpu_guardrail \
  -- --gpu-id 0 \
  --requests 100 \
  --warmup-requests 10 \
  --min-free-memory-percent 10 \
  --out labs/results/s32-headroom-policy-${TIMESTAMP}.json

echo "Running KV Pressure (quick test)..."
cargo run -p kv_pressure \
  -- --gpu-id 0 \
  --requests 100 \
  --warmup-requests 10 \
  --avg-seq-len 512 \
  --concurrency 8 \
  --out labs/results/s33-kv-pressure-${TIMESTAMP}.json

echo "Running Profiling Playbook (quick test)..."
cargo run -p profiling_playbook \
  -- --tool nsight \
  --requests 100 \
  --warmup-requests 10 \
  --out labs/results/s34-profiling-methodology-${TIMESTAMP}.json

echo ""
echo "Quick tests completed!"
echo "Results saved to:"
echo "  labs/results/s31-gpu-metrics-${TIMESTAMP}.json"
echo "  labs/results/s32-headroom-policy-${TIMESTAMP}.json"
echo "  labs/results/s33-kv-pressure-${TIMESTAMP}.json"
echo "  labs/results/s34-profiling-methodology-${TIMESTAMP}.json"