# Epic 2 S2.1: CFS Cliff Lab

## Objective

This lab demonstrates how CPU scheduler effects under oversubscription and lock contention cause tail latency degradation (p99 spikes), and how sharding can improve performance by reducing contention.

## How It Works

The lab simulates a producer-consumer system with:
- **Producers**: Generate tasks and enqueue them
- **Workers**: Dequeue tasks and process them with busy work
- **Critical Section**: Lock contention that causes scheduling delays

### Contended Mode
- Single global mutex protects all critical sections
- High contention leads to increased scheduling overhead and p99 latency spikes

### Sharded Mode
- Multiple mutexes (shards) with tasks distributed by ID
- Reduced contention per shard leads to better scalability

## Running the Lab

### Basic Usage

```bash
# Run contended mode
cargo run -p cfs_cliff -- --mode contended --requests 20000

# Run sharded mode  
cargo run -p cfs_cliff -- --mode sharded --shards 64 --requests 20000
```

### Running Experiments

```bash
# Run both modes with standard parameters
scripts/epic2_s21_run.sh
```

### Capturing Perf Statistics (Linux only)

```bash
# Capture perf statistics for both modes
scripts/epic2_s21_perf_stat.sh
```

## Metrics to Monitor

### Performance Metrics
- **p50/p95/p99/max latency**: Latency percentiles in microseconds
- **RPS**: Requests per second
- **Wall time**: Total execution time in milliseconds

### Linux Perf Metrics (when available)
- **context-switches**: Number of context switches
- **cpu-migrations**: Number of CPU migrations
- **task-clock**: CPU time consumed
- **cycles**: CPU cycles
- **instructions**: Instructions executed

## Expected Outcome

The sharded mode should show:
- Lower p99 latency (typically 2-10x better)
- Lower context switches
- Better RPS performance
- Reduced CPU migrations

The contended mode will show:
- Higher p99 latency spikes
- More context switches
- Higher CPU contention
- Reduced throughput

## Troubleshooting

### Increase contention
- Increase `--lock-hold-us` to amplify contention effects
- Increase `--workers` to increase oversubscription
- Increase `--producers` to increase load

### Adjust parameters
- `--shards`: Increase for better sharding (default 64)
- `--queue`: Adjust queue size if needed
- `--requests`: Increase for more stable measurements

## Test Checklist

```bash
# Run unit tests
cargo test -p cfs_cliff

# Run experiments
scripts/epic2_s21_run.sh

# Optional: capture perf stats (Linux)
scripts/epic2_s21_perf_stat.sh
```

## Required Readings

### Linux Scheduling Basics
- **CFS (Completely Fair Scheduler)**: How Linux schedules processes
- **Runqueue**: The queue of runnable processes
- **Context switches**: Cost of switching between processes
- **CPU migrations**: Cost of moving processes between cores

### Perf Stat Interpretation
- **context-switches**: High numbers indicate scheduling overhead
- **cpu-migrations**: High numbers indicate load imbalance
- **task-clock**: CPU time consumed by the process
- **cycles/instructions**: Performance counters for CPU utilization

### Mutex Contention and Scalability
- **Sharding**: Distributing locks to reduce contention
- **Reducing critical sections**: Minimizing time spent in locks
- **Lock-free programming**: Alternative approaches to reduce contention