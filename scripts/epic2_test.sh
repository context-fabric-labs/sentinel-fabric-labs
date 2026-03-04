#!/bin/bash

# Epic 2 - Quick test script
# This script runs a quick test of all the labs in Epic 2

set -e  # Exit on any error

echo "Running quick Epic 2 tests..."
echo "============================"

# Create results directory if it doesn't exist
mkdir -p labs/results

# Get current timestamp for unique filenames
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

echo "Running CFS Cliff Lab (quick test)..."
cargo run -p cfs_cliff \
  -- --mode contended \
  --requests 100 \
  --warmup-requests 10 \
  --out labs/results/s21-quick-${TIMESTAMP}.json

echo "Running TLB Cliff Lab (quick test)..."
cargo run -p tlb_cliff \
  -- --access-pattern pointer-chasing \
  --requests 100 \
  --warmup-requests 10 \
  --out labs/results/s22-quick-${TIMESTAMP}.json

echo "Running NUMA Cliff Lab (quick test)..."
cargo run -p numa_cliff \
  -- --access-type local \
  --requests 100 \
  --warmup-requests 10 \
  --out labs/results/s23-quick-${TIMESTAMP}.json

echo "Running Perf Stat Gate (quick test)..."
cargo run -p perf_gate \
  -- --run-type bad \
  --requests 100 \
  --warmup-requests 10 \
  --out labs/results/s24-quick-${TIMESTAMP}.json

echo ""
echo "Quick tests completed!"
echo "Results saved to:"
echo "  labs/results/s21-quick-${TIMESTAMP}.json"
echo "  labs/results/s22-quick-${TIMESTAMP}.json"
echo "  labs/results/s23-quick-${TIMESTAMP}.json"
echo "  labs/results/s24-quick-${TIMESTAMP}.json"