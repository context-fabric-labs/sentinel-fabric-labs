# Epic 3 — GPU Guardrails + Profiling Mastery

## Objective
You can profile and enforce GPU SLOs.

## Stories

### S3.1 NVML/DCGM Exporter + Dashboards
**GPU utilization, memory, temperature, and power metrics with Prometheus scraping and Grafana dashboards**

#### Implementation
- Created `labs/gpu_exporter` crate with CLI interface
- Implemented NVML/DCGM exporter for GPU metrics collection
- Added Prometheus metrics endpoint for scraping
- Implemented Grafana dashboard configuration
- Added JSON output with standardized schema

#### Test Tasks
- Unit tests for GPU metric collection
- Prometheus endpoint validation
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s31-gpu-metrics.json` - GPU metrics results
- `labs/results/s31-prometheus-exporter.json` - Prometheus exporter results
- `labs/results/s31-grafana-dashboard.json` - Grafana dashboard configuration

#### Required Readings
- GPU monitoring: NVML, DCGM, GPU metrics collection
- Prometheus metrics: exposition format, scraping, alerting
- Grafana dashboards: panels, variables, queries
- GPU memory management: utilization, memory usage, temperature

### S3.2 Headroom Policy (Pre-OOM)
**Enforce minimum free GPU memory; throttle/deny requests; integrate with Sentinel admission**

#### Implementation
- Created `labs/gpu_guardrail` crate with CLI interface
- Implemented headroom policy enforcement for GPU memory
- Added request throttling and denial mechanisms
- Integrated with Sentinel admission control
- Added metrics and logging for policy enforcement

#### Test Tasks
- Unit tests for memory calculation
- Throttling/denial logic validation
- Integration with Sentinel admission tests
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s32-headroom-policy.json` - Headroom policy results
- `labs/results/s32-throttle-deny.json` - Throttle/deny results
- `labs/results/s32-sentinel-integration.json` - Sentinel integration results

#### Required Readings
- GPU memory management: allocation, fragmentation, pre-OOM
- Resource management: headroom, quotas, limits
- Admission control: rate limiting, resource-based admission
- GPU memory pressure: how to detect and respond

### S3.3 KV-Pressure Proxy
**Estimate KV/cache pressure from sequence length + concurrency; feed into guardrails**

#### Implementation
- Created `labs/kv_pressure` crate with CLI interface
- Implemented KV pressure estimation from sequence length and concurrency
- Added proxy functionality to intercept and analyze requests
- Integrated with guardrail systems for pressure-based decisions
- Added metrics and logging for pressure monitoring

#### Test Tasks
- Unit tests for pressure calculation
- Proxy functionality validation
- Integration with guardrail systems
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s33-kv-pressure.json` - KV pressure results
- `labs/results/s33-proxy-interception.json` - Proxy interception results
- `labs/results/s33-guardrail-integration.json` - Guardrail integration results

#### Required Readings
- KV cache pressure: sequence length, concurrency impact
- Cache pressure estimation: memory usage, access patterns
- Proxy systems: request interception, analysis, forwarding
- Resource-based decision making: pressure vs resource limits

### S3.4 Profiling Playbook
**Nsight Systems vs Compute (or ROCm equivalents); roofline basics; "profiling without perturbation"; cross-run comparability checklist**

#### Implementation
- Created `labs/profiling_playbook` crate with CLI interface
- Implemented profiling methodology documentation
- Added Nsight Systems/ROCm comparison framework
- Implemented roofline analysis tools
- Added "profiling without perturbation" guidelines
- Created cross-run comparability checklist

#### Test Tasks
- Unit tests for profiling methodology
- Comparison framework validation
- Roofline analysis tests
- Checklist validation
- JSON serialization/deserialization tests
- Smoke test with small parameters

#### Artifact Outputs
- `labs/results/s34-profiling-methodology.json` - Profiling methodology results
- `labs/results/s34-nsight-rocm-comparison.json` - Nsight/ROCm comparison results
- `labs/results/s34-roofline-analysis.json` - Roofline analysis results
- `labs/results/s34-profiling-checklist.json` - Profiling checklist results

#### Required Readings
- GPU profiling: Nsight Systems, ROCm, profiling tools
- Roofline analysis: performance modeling, memory bandwidth
- Profiling best practices: avoiding perturbation, reproducibility
- Cross-run comparability: consistent environments, measurement protocols

## Running All Labs

### Quick Test Script
```bash
# Run all labs with small parameters for quick testing
./scripts/epic3_test.sh
```

### Full Test Script
```bash
# Run all labs with default parameters
./scripts/epic3_run_all.sh
```

## Test Checklist

- [ ] All labs compile successfully with `cargo check`
- [ ] Quick test scripts run without errors
- [ ] All labs produce JSON output files in `labs/results/`
- [ ] Results are properly formatted and contain expected metrics
- [ ] Performance characteristics are visible in the results

## How to Validate

### 1. Compilation Check
```bash
cargo check
```

### 2. Quick Testing (Recommended for Immediate Verification)
```bash
# Run all Epic 3 labs with minimal parameters
./scripts/epic3_test.sh
```

### 3. Full Testing
```bash
# Run all Epic 3 labs with default parameters
./scripts/epic3_run_all.sh
```

### 4. Individual Lab Testing
```bash
# Test GPU Exporter with minimal parameters
cargo run -p gpu_exporter -- --gpu-id 0 --requests 10 --warmup-requests 1 --out /tmp/gpu_result.json

# Test GPU Guardrail with minimal parameters  
cargo run -p gpu_guardrail -- --gpu-id 0 --requests 10 --warmup-requests 1 --min-free-memory-percent 10 --out /tmp/guardrail_result.json

# Test KV Pressure with minimal parameters
cargo run -p kv_pressure -- --gpu-id 0 --requests 10 --warmup-requests 1 --avg-seq-len 512 --concurrency 8 --out /tmp/kv_result.json

# Test Profiling Playbook with minimal parameters
cargo run -p profiling_playbook -- --tool nsight --requests 10 --warmup-requests 1 --out /tmp/profiling_result.json
```

### 5. Verify Results
```bash
# Check that result files were created
ls -lh labs/results/s3*.json

# View sample result
cat labs/results/s31-gpu-metrics-*.json
```

### 6. Test Specific Functionality
```bash
# Run unit tests for all crates
cargo test --workspace --lib

# Run specific crate tests
cargo test -p gpu_exporter
cargo test -p gpu_guardrail
cargo test -p kv_pressure
cargo test -p profiling_playbook
```

## Code Map

This epic implements GPU monitoring, guardrails, and profiling methodologies. The following files and directories were created/modified:

### New Crates Created:
- `labs/gpu_exporter/` - GPU metrics collection and Prometheus exporter
  - `Cargo.toml` - Crate configuration
  - `src/lib.rs` - Core implementation with GPU metrics collection and Prometheus metrics
  - `src/main.rs` - CLI entry point for GPU metrics collection

- `labs/gpu_guardrail/` - GPU memory headroom enforcement
  - `Cargo.toml` - Crate configuration
  - `src/lib.rs` - Core implementation with headroom policy enforcement and request throttling
  - `src/main.rs` - CLI entry point for GPU guardrail functionality

- `labs/kv_pressure/` - KV/cache pressure estimation
  - `Cargo.toml` - Crate configuration
  - `src/lib.rs` - Core implementation with KV pressure estimation and proxy functionality
  - `src/main.rs` - CLI entry point for KV pressure estimation

- `labs/profiling_playbook/` - GPU profiling methodology documentation
  - `Cargo.toml` - Crate configuration
  - `src/lib.rs` - Core implementation with profiling methodology and tool comparison
  - `src/main.rs` - CLI entry point for profiling playbook

### Scripts Created:
- `scripts/epic3_test.sh` - Quick test script for all Epic 3 labs with minimal parameters
- `scripts/epic3_run_all.sh` - Full test script for all Epic 3 labs with default parameters

### Workspace Integration:
- `Cargo.toml` - Updated to include all new crates in the workspace