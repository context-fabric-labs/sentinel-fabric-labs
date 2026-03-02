#!/bin/bash

# Epic 2 S2.1 CFS Cliff Lab - Run experiments
# This script runs the CFS Cliff Lab in both contended and sharded modes

set -e  # Exit on any error

echo "Running CFS Cliff Lab experiments..."
echo "====================================="

# Create results directory if it doesn't exist
mkdir -p labs/results

# Get current timestamp for unique filenames
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Run contended mode
echo "Running contended mode..."
cargo run -p cfs_cliff \
  -- --mode contended \
  --requests 20000 \
  --warmup-requests 1000 \
  --workers 0 \
  --producers 0 \
  --work-us 100 \
  --lock-hold-us 1000 \
  --queue 4096 \
  --out labs/results/s21-contended-${TIMESTAMP}.json

# Run sharded mode
echo "Running sharded mode..."
cargo run -p cfs_cliff \
  -- --mode sharded \
  --requests 20000 \
  --warmup-requests 1000 \
  --workers 0 \
  --producers 0 \
  --work-us 100 \
  --lock-hold-us 1000 \
  --queue 4096 \
  --shards 64 \
  --out labs/results/s21-sharded-${TIMESTAMP}.json

echo ""
echo "Experiments completed!"
echo "Results saved to:"
echo "  labs/results/s21-contended-${TIMESTAMP}.json"
echo "  labs/results/s21-sharded-${TIMESTAMP}.json"

# Show comparison
echo ""
echo "Comparison:"
echo "----------"
echo "Contended mode:"
cat labs/results/s21-contended-${TIMESTAMP}.json | jq '.p99_us, .rps'
echo "Sharded mode:"
cat labs/results/s21-sharded-${TIMESTAMP}.json | jq '.p99_us, .rps'