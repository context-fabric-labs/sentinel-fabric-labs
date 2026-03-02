#!/bin/bash

# Epic 2 S2.1 CFS Cliff Lab - Perf stat capture
# This script captures perf statistics for the CFS Cliff Lab experiments

set -e  # Exit on any error

echo "Capturing perf statistics for CFS Cliff Lab..."
echo "=============================================="

# Check if perf is available
if ! command -v perf &> /dev/null; then
    echo "perf not available. Please install perf to capture statistics."
    echo "On Ubuntu/Debian: sudo apt install linux-tools-common linux-tools-generic"
    echo "On CentOS/RHEL: sudo yum install perf"
    exit 0
fi

# Create results directory if it doesn't exist
mkdir -p labs/results

# Get current timestamp for unique filenames
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

echo "Capturing perf stats for contended mode..."
perf stat -e context-switches,cpu-migrations,task-clock,cycles,instructions \
  -o labs/results/s21-contended-perf-${TIMESTAMP}.txt \
  --append \
  cargo run -p cfs_cliff \
    -- --mode contended \
    --requests 10000 \
    --warmup-requests 500 \
    --workers 0 \
    --producers 0 \
    --work-us 100 \
    --lock-hold-us 1000 \
    --queue 4096 \
    --out labs/results/s21-contended-${TIMESTAMP}.json

echo "Capturing perf stats for sharded mode..."
perf stat -e context-switches,cpu-migrations,task-clock,cycles,instructions \
  -o labs/results/s21-sharded-perf-${TIMESTAMP}.txt \
  --append \
  cargo run -p cfs_cliff \
    -- --mode sharded \
    --requests 10000 \
    --warmup-requests 500 \
    --workers 0 \
    --producers 0 \
    --work-us 100 \
    --lock-hold-us 1000 \
    --queue 4096 \
    --shards 64 \
    --out labs/results/s21-sharded-${TIMESTAMP}.json

echo ""
echo "Perf statistics captured:"
echo "  labs/results/s21-contended-perf-${TIMESTAMP}.txt"
echo "  labs/results/s21-sharded-perf-${TIMESTAMP}.txt"
echo ""
echo "Results:"
echo "  labs/results/s21-contended-${TIMESTAMP}.json"
echo "  labs/results/s21-sharded-${TIMESTAMP}.json"