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
- `labs/results/s22-contiguous.json` - Contiguous layout results
- `labs/results/s22-tlb-perf-*.txt` - TLB perf stats (Linux)

#### Required Readings
- TLB behavior and page walks
- Memory layout and cache behavior
- Performance counter usage for TLB metrics

### S2.3 NUMA Cliff Lab
**First-touch placement; local vs remote bandwidth/latency; pinning + numactl; measure migrations/remote accesses**

#### Implementation
- Created `labs/numa_cliff` crate with CLI interface
- Implemented NUMA-aware memory allocation
- Added memory access patterns for local vs remote access
- Implemented CPU pinning and NUMA topology detection
- Added JSON output with standardized schema

#### Test Tasks
- Unit tests for NUMA topology detection
- Memory access pattern validation
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s23-local-access.json` - Local memory access results
- `labs/results/s23-remote-access.json` - Remote memory access results
- `labs/results/s23-numa-perf-*.txt` - NUMA perf stats (Linux)

#### Required Readings
- NUMA topology and memory placement
- CPU pinning and process affinity
- Memory access patterns and performance impact

### S2.4 Perf Stat Gate
**One-command "bad vs fixed" runs; store perf stat artifacts; add regression thresholds (cs/migrations/TLB)**

#### Implementation
- Created `labs/perf_gate` crate with CLI interface
- Implemented performance regression testing framework
- Added support for perf stat capture with multiple counters
- Implemented threshold checking and reporting
- Added JSON output with standardized schema

#### Test Tasks
- Unit tests for perf stat capture
- Threshold validation
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s24-bad-run.json` - Bad run results
- `labs/results/s24-fixed-run.json` - Fixed run results
- `labs/results/s24-perf-stats-*.txt` - Perf stats for both runs

#### Required Readings
- Performance regression testing methodology
- perf stat usage and interpretation
- Threshold setting and validation

## How To Run & Test

### Prerequisites
- Rust toolchain
- macOS/Linux shell

### Running Individual Labs

```bash
# CFS Cliff Lab
cargo run -p cfs_cliff -- --mode contended --requests 20000

# TLB Cliff Lab  
cargo run -p tlb_cliff -- --access-pattern pointer-chasing --requests 10000

# NUMA Cliff Lab
cargo run -p numa_cliff -- --access-type local --requests 10000

# Perf Stat Gate
cargo run -p perf_gate -- --bad-run --requests 5000
```

### Running All Experiments
```bash
# Run all labs with standard parameters
scripts/epic2_run_all.sh
```

### Capturing Perf Statistics (Linux only)
```bash
# Capture perf stats for all labs
scripts/epic2_perf_stat.sh
```

## Metrics & Signals

### CFS Cliff Lab
- p50/p95/p99/max latency (microseconds)
- RPS (requests per second)
- Context switches
- CPU migrations

### TLB Cliff Lab
- TLB miss rate
- Page walk count
- Memory access latency
- Cache hit ratio

### NUMA Cliff Lab
- Memory access latency
- Remote memory access count
- CPU migration count
- NUMA node affinity

### Perf Stat Gate
- Context switches
- CPU migrations
- Task-clock
- Cycles and instructions

## Interview Focus Areas

### CFS Cliff Lab
- How does oversubscription cause p99 spikes?
- Why does sharding reduce contention?
- What perf counters indicate scheduling overhead?

### TLB Cliff Lab
- How does memory layout affect TLB behavior?
- What causes page walks?
- How to measure TLB efficiency?

### NUMA Cliff Lab
- How does NUMA topology affect memory access?
- What are the costs of remote memory access?
- How to pin processes to specific CPUs?

### Perf Stat Gate
- How to set up regression testing?
- What perf counters are most useful?
- How to interpret performance changes?

## Key Decisions / Trade-offs

- Simplicity over completeness for performance measurements
- Prom-first metrics; logs are structured but lightweight
- Rust `tokio::spawn` for concurrency control
- Standardized JSON output for easy comparison

## Test Checklist

```bash
# Run all unit tests
cargo test --workspace

# Run individual crate tests
cargo test -p cfs_cliff
cargo test -p tlb_cliff
cargo test -p numa_cliff
cargo test -p perf_gate

# Run experiments
scripts/epic2_run_all.sh

# Optional: capture perf stats (Linux)
scripts/epic2_perf_stat.sh
```

## Required Readings

### Linux Scheduling Basics
- CFS (Completely Fair Scheduler): How Linux schedules processes
- Runqueue: The queue of runnable processes
- Context switches: Cost of switching between processes
- CPU migrations: Cost of moving processes between cores

### Perf Stat Interpretation
- context-switches: High numbers indicate scheduling overhead
- cpu-migrations: High numbers indicate load imbalance
- task-clock: CPU time consumed by the process
- cycles/instructions: Performance counters for CPU utilization

### Memory Access Patterns
- TLB behavior and page walks
- NUMA topology and memory placement
- Memory layout and cache behavior

### Performance Testing
- Performance regression testing methodology
- Threshold setting and validation
- Perf stat usage and interpretation

## Code Map

This epic implements four systems physics labs that demonstrate various aspects of system behavior under different conditions. The following files and directories were created/modified:

### New Crates Created:
- `labs/cfs_cliff/` - CFS Cliff Lab implementation
  - `Cargo.toml` - Crate configuration
  - `src/lib.rs` - Core implementation with CLI parsing, memory access patterns, and performance metrics
  - `src/main.rs` - CLI entry point for the CFS Cliff Lab

- `labs/tlb_cliff/` - TLB Cliff Lab implementation
  - `Cargo.toml` - Crate configuration
  - `src/lib.rs` - Core implementation with CLI parsing, memory access patterns, and performance metrics
  - `src/main.rs` - CLI entry point for the TLB Cliff Lab

- `labs/numa_cliff/` - NUMA Cliff Lab implementation
  - `Cargo.toml` - Crate configuration
  - `src/lib.rs` - Core implementation with CLI parsing, memory access patterns, and performance metrics
  - `src/main.rs` - CLI entry point for the NUMA Cliff Lab

- `labs/perf_gate/` - Perf Stat Gate implementation
  - `Cargo.toml` - Crate configuration
  - `src/lib.rs` - Core implementation with CLI parsing and performance metrics
  - `src/main.rs` - CLI entry point for the Perf Stat Gate
 
  - ===
Exp-1
San Jose, CA
•	Leading design of a real-time MEV defense and arbitrage platform for digital-asset and payment-adjacent liquidity workflows, using agentic AI patterns to monitor mempool shifts, propose execution actions, and enforce deterministic trade-protection policies.
•	Built a low-latency ingest and decision pipeline that collects pending-transaction telemetry from distributed blockchain nodes and routes signals to simulation, policy, and execution services under strict p99 latency budgets.
•	Implemented a Helios / Myelin style execution fabric that runs parallel threat simulations and execution-quality scoring to detect sandwich-attack and frontrun risk, then automatically cancels, reroutes to private execution paths, or hedges before public mempool exposure.
•	Added benchmark-driven reliability controls including admission limits, queue bounds, tracing, replayable test harnesses, failure injection, and SLO-aware routing to harden mission-critical flows.
•	Introduced agentic operations workflows that combine market telemetry, execution policies, fraud / abuse heuristics, and post-trade analysis to reduce manual intervention and accelerate response during volatile conditions. Used CUDA for parallel simulation and threat scoring
Technical Stack: Rust, Python, streaming data services, GPU-oriented simulation concepts, Kubernetes, Docker, Prometheus/Grafana, OpenTelemetry, Linux, AWS, private RPC / protected execution routing
<img width="468" height="230" alt="image" src="https://github.com/user-attachments/assets/56d0e4ee-fa0b-4bd9-9fbb-40bbe08ac601" />

Exp-2
San Jose, CA
•	Architected fraud decisioning and risk-scoring platforms for credit-card authorization, transaction enrichment, account takeover detection, and investigator workflows across high-volume payment flows.
•	Designed and built Project Sentinel, an internal distributed inference-serving and control fabric for fraud scoring—combining a Rust/Python gateway, benchmark harness, policy engine, and health-aware routing layer to coordinate model endpoints, feature services, caches, and fallback paths under regulated low-latency constraints.
•	Built custom CUDA kernels for fraud-scoring primitives, KV-cache or memory-layout tuning, GPU batching/stream concurrency, profiling p95/p99 bottlenecks, reducing host-device copies, optimizing utilization under bursty traffic..
•	Implemented admission control, concurrency limits, queue backpressure, circuit breaking, shadow traffic, canary releases, and fail-open / fail-safe behaviors so model and rules changes could be introduced without destabilizing card-transaction flows.
•	Built observability and performance tooling around p50/p95/p99 latency, throughput, dependency health, error budgets, and queue depth, creating an evidence-driven workflow for scaling fraud models in Kubernetes and cloud environments.
•	Advanced the platform toward NVIDIA Dynamo-like patterns for enterprise scope, including intelligent routing, tiered caching, warm pools, autoscaling, and policy-aware orchestration for inference services used by fraud and investigation teams.
•	Partnered with fraud analytics, platform, data engineering, and security teams to align model serving, compliance, operational resilience, and cost controls across production and pre-production environments.
Technical Stack: Rust, C++, Python, Go, Kubernetes / EKS, Docker, Kafka, Redis, RocksDB, gRPC/Protobuf, Apache Arrow, Prometheus/Grafana, OpenTelemetry, Linux, cloud-native inference patterns and benchmark-driven operations
<img width="468" height="336" alt="image" src="https://github.com/user-attachments/assets/431713d9-5d39-442e-8294-cd3f5776b885" />

### Scripts Created:
- `scripts/epic2_test.sh` - Quick test script for all labs with minimal parameters
- `scripts/epic2_run_all.sh` - Full test script for all labs with default parameters

### Workspace Integration:
- `Cargo.toml` - Updated to include all new crates in the workspace
