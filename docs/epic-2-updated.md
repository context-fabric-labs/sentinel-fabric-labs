# Epic 2 — Systems Physics Lab

## Objective
Be able to explain p99 spikes with OS counters.

## Stories

### S2.1 CFS Cliff Lab
**Oversubscription + contended mutex → context switches/runnable inflation → p99 spike; fix via sharding/less contention; record perf counters**

#### Implementation
- Created `labs/cfs_cliff` crate with CLI interface
- Implemented contended and sharded modes with lock contention
- Added busy-spin workloads to simulate CPU contention
- Implemented HDR histogram for latency measurements
- Added JSON output with standardized schema

#### Test Tasks
- Unit tests for busy_spin function
- Sharding logic validation
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s21-contended.json` - Contended mode results
- `labs/results/s21-sharded.json` - Sharded mode results
- `labs/results/s21-contended-perf-*.txt` - Perf stats for contended mode (Linux)
- `labs/results/s21-sharded-perf-*.txt` - Perf stats for sharded mode (Linux)

#### Required Readings
- Linux scheduling basics: CFS, runqueue, context switches, CPU migrations
- perf stat interpretation: context-switches, cpu-migrations, task-clock
- Mutex contention and scalability patterns: sharding, reducing critical sections

#### Quick Test:
```bash
# Run a quick test with small parameters
cargo run -p cfs_cliff -- --mode contended --requests 100 --warmup-requests 10
```

#### Full Test:
```bash
# Run with default parameters (2000 requests each)
cargo run -p cfs_cliff -- --mode contended
cargo run -p cfs_cliff -- --mode sharded --shards 64
```

### S2.2 TLB Cliff Lab
**Pointer-chasing vs contiguous layout; dTLB misses/page-walks; demonstrate improvement with contiguous buffers/arena-style layout**

#### Implementation
- Created `labs/tlb_cliff` crate with CLI interface
- Implemented pointer-chasing and contiguous memory access patterns
- Added TLB miss measurement using performance counters
- Implemented memory allocation strategies for comparison
- Added JSON output with standardized schema

#### Test Tasks
- Unit tests for memory access patterns
- TLB miss measurement validation
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s22-pointer-chasing.json` - Pointer-chasing results
- `labs/results/s22-contiguous.json` - Contiguous access results

#### Required Readings
- Memory management: virtual memory, page tables, TLB, page walks
- Cache behavior: dTLB, iTLB, cache line alignment
- Performance counter usage for memory access patterns

#### Quick Test:
```bash
# Run a quick test with small parameters
cargo run -p tlb_cliff -- --access-pattern pointer-chasing --requests 100 --warmup-requests 10
```

#### Full Test:
```bash
# Run with default parameters (2000 requests each)
cargo run -p tlb_cliff -- --access-pattern pointer-chasing
cargo run -p tlb_cliff -- --access-pattern contiguous
```

### S2.3 NUMA Cliff Lab
**NUMA topology effects; remote memory access vs local; demonstrate improvement with NUMA-aware allocation**

#### Implementation
- Created `labs/numa_cliff` crate with CLI interface
- Implemented local and remote memory access patterns
- Added NUMA topology awareness for memory allocation
- Implemented performance metrics for remote access detection
- Added JSON output with standardized schema

#### Test Tasks
- Unit tests for memory access patterns
- NUMA topology validation
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s23-local.json` - Local memory access results
- `labs/results/s23-remote.json` - Remote memory access results

#### Required Readings
- NUMA architecture: memory domains, interconnects, latency differences
- Memory allocation strategies: NUMA-aware allocation
- Performance counter usage for NUMA effects

#### Quick Test:
```bash
# Run a quick test with small parameters
cargo run -p numa_cliff -- --access-type local --requests 100 --warmup-requests 10
```

#### Full Test:
```bash
# Run with default parameters (2000 requests each)
cargo run -p numa_cliff -- --access-type local
cargo run -p numa_cliff -- --access-type remote
```

### S2.4 Perf Stat Gate
**Performance regression testing framework; baseline vs fixed performance**

#### Implementation
- Created `labs/perf_gate` crate with CLI interface
- Implemented performance regression testing framework
- Added performance counter collection for baseline comparison
- Implemented standardized output format for regression analysis
- Added JSON output with standardized schema

#### Test Tasks
- Unit tests for performance measurement
- Regression testing validation
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s24-bad.json` - Bad performance run results
- `labs/results/s24-fixed.json` - Fixed performance run results

#### Required Readings
- Performance testing methodologies
- Regression testing frameworks
- Performance counter usage: context switches, CPU migrations, cycles

#### Quick Test:
```bash
# Run a quick test with small parameters
cargo run -p perf_gate -- --run-type bad --requests 100 --warmup-requests 10
```

#### Full Test:
```bash
# Run with default parameters (10000 requests each)
cargo run -p perf_gate -- --run-type bad
cargo run -p perf_gate -- --run-type fixed
```

## Running All Labs

### Quick Test Script
```bash
# Run all labs with small parameters for quick testing
./scripts/epic2_test.sh
```

### Full Test Script
```bash
# Run all labs with default parameters
./scripts/epic2_run_all.sh
```

## Test Checklist

- [ ] All labs compile successfully with `cargo check`
- [ ] Quick test scripts run without errors
- [ ] All labs produce JSON output files in `labs/results/`
- [ ] Results are properly formatted and contain expected metrics
- [ ] Performance characteristics are visible in the results