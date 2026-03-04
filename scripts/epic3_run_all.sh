#!/bin/bash

# Epic 3 - Run all labs
# This script runs all the labs in Epic 3

set -e  # Exit on any error

echo "Running all Epic 3 labs..."
echo "=========================="

# Create results directory if it doesn't exist
mkdir -p labs/results

# Get current timestamp for unique filenames
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

echo "Running GPU Exporter..."
cargo run -p gpu_exporter \
  -- --gpu-id 0 \
  --requests 1000 \
  --warmup-requests 100 \
  --out labs/results/s31-gpu-metrics-${TIMESTAMP}.json

echo "Running GPU Guardrail..."
cargo run -p gpu_guardrail \
  -- --gpu-id 0 \
  --requests 1000 \
  --warmup-requests 100 \
  --min-free-memory-percent 10 \
  --out labs/results/s32-headroom-policy-${TIMESTAMP}.json

echo "Running KV Pressure..."
cargo run -p kv_pressure \
  -- --gpu-id 0 \
  --requests 1000 \
  --warmup-requests 100 \
  --avg-seq-len 512 \
  --concurrency 8 \
  --out labs/results/s33-kv-pressure-${TIMESTAMP}.json

echo "Running Profiling Playbook..."
cargo run -p profiling_playbook \
  -- --tool nsight \
  --requests 1000 \
  --warmup-requests 100 \
  --out labs/results/s34-profiling-methodology-${TIMESTAMP}.json

echo ""
echo "All experiments completed!"
echo "Results saved to:"
echo "  labs/results/s31-gpu-metrics-${TIMESTAMP}.json"
echo "  labs/results/s32-headroom-policy-${TIMESTAMP}.json"
echo "  labs/results/s33-kv-pressure-${TIMESTAMP}.json"
echo "  labs/results/s34-profiling-methodology-${TIMESTAMP}.json"