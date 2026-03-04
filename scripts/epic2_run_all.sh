#!/bin/bash

# Epic 2 - Run all labs
# This script runs all the labs in Epic 2

set -e  # Exit on any error

echo "Running all Epic 2 labs..."
echo "=========================="

# Create results directory if it doesn't exist
mkdir -p labs/results

# Get current timestamp for unique filenames
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

echo "Running CFS Cliff Lab..."
cargo run -p cfs_cliff \
  -- --mode contended \
  --requests 2000 \
  --warmup-requests 1000 \
  --out labs/results/s21-contended-${TIMESTAMP}.json

cargo run -p cfs_cliff \
  -- --mode sharded \
  --shards 64 \
  --requests 2000 \
  --warmup-requests 1000 \
  --out labs/results/s21-sharded-${TIMESTAMP}.json

echo "Running TLB Cliff Lab..."
cargo run -p tlb_cliff \
  -- --access-pattern pointer-chasing \
  --requests 2000 \
  --warmup-requests 1000 \
  --out labs/results/s22-pointer-chasing-${TIMESTAMP}.json

cargo run -p tlb_cliff \
  -- --access-pattern contiguous \
  --requests 2000 \
  --warmup-requests 1000 \
  --out labs/results/s22-contiguous-${TIMESTAMP}.json

echo "Running NUMA Cliff Lab..."
cargo run -p numa_cliff \
  -- --access-type local \
  --requests 2000 \
  --warmup-requests 1000 \
  --out labs/results/s23-local-${TIMESTAMP}.json

cargo run -p numa_cliff \
  -- --access-type remote \
  --requests 2000 \
  --warmup-requests 1000 \
  --out labs/results/s23-remote-${TIMESTAMP}.json

echo "Running Perf Stat Gate..."
cargo run -p perf_gate \
  -- --run-type bad \
  --requests 10000 \
  --warmup-requests 500 \
  --out labs/results/s24-bad-${TIMESTAMP}.json

cargo run -p perf_gate \
  -- --run-type fixed \
  --requests 10000 \
  --warmup-requests 500 \
  --out labs/results/s24-fixed-${TIMESTAMP}.json

echo ""
echo "All experiments completed!"
echo "Results saved to:"
echo "  labs/results/s21-contended-${TIMESTAMP}.json"
echo "  labs/results/s21-sharded-${TIMESTAMP}.json"
echo "  labs/results/s22-pointer-chasing-${TIMESTAMP}.json"
echo "  labs/results/s22-contiguous-${TIMESTAMP}.json"
echo "  labs/results/s23-local-${TIMESTAMP}.json"
echo "  labs/results/s23-remote-${TIMESTAMP}.json"
echo "  labs/results/s24-bad-${TIMESTAMP}.json"
echo "  labs/results/s24-fixed-${TIMESTAMP}.json"