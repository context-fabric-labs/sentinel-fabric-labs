# Roadmap (Interview-First, Hands-On Artifacts)

## Epic 1 — Serving Cell + Measurement + Safe Overload
**Merge: E0 + E1 + E2**

**Goal:** A real serving "cell" you can measure and protect.

### Stories (tight)
- Harness + baselines + regression gate (already done)
- Proxy /chat, /metrics, /health (done)
- Hard limits (done)
- Add minimal v1 reliability: admission (token budget), breaker, bounded queue, degrade mode

**Artifacts:** p50/p99 vs QPS curve, overload behavior report, dashboards.

**Sections hit:** (3) profiling/bench, (9) inference serving, (11) reliability, (12) systems practice.

#### Epic 1 — Verification & Tests

- Unit tests:
  ```bash
  cargo test --workspace
  ```

- Lints (fail on warnings):
  ```bash
  cargo clippy --workspace --all-targets -- -D warnings
  ```

- Start local stack:
  ```bash
  # Upstream stub (port 4000)
  RUST_LOG=info cargo run -p upstream_stub

  # Sentinel (port 8080, proxy -> 4000)
  RUST_LOG=info cargo run -p sentinel -- --bind 127.0.0.1:8080 --upstream http://127.0.0.1:4000
  ```

- Quick smoke:
  ```bash
  curl -sS http://127.0.0.1:8080/health
  # optional
  curl -sS http://127.0.0.1:8080/metrics | head
  ```

-- Bench — health scenario:
  ```bash
  cargo run -p bench -- run \
    --scenario health \
    --url http://127.0.0.1:8080 \
    --concurrency 16 \
    --requests 500 \
    --warmup-requests 50 \
    --timeout-s 5 \
    --out bench/results/health-$(date +%Y%m%d-%H%M%S).json
  ```

-- Bench — chat scenario (from file):
  ```bash
  cargo run -p bench -- run \
    --scenario chat \
    --url http://127.0.0.1:8080/v1/chat/completions \
    --concurrency 16 \
    --requests 500 \
    --warmup-requests 50 \
    --timeout-s 10 \
    --body-file bench/baselines/local_chat.json \
    --out bench/results/chat-$(date +%Y%m%d-%H%M%S).json
  ```

-- Bench — chat scenario (inline JSON):
  ```bash
  cargo run -p bench -- run \
    --scenario chat \
    --url http://127.0.0.1:8080/v1/chat/completions \
    --concurrency 8 \
    --requests 200 \
    --warmup-requests 20 \
    --timeout-s 10 \
    --body-json '{"model":"stub","messages":[{"role":"user","content":"hello"}],"max_tokens":16,"stream":false}' \
    --out bench/results/chat-$(date +%Y%m%d-%H%M%S).json
  ```

- Concurrency sweep (chat):
  ```bash
  bash scripts/sweep_chat.sh
  ```


## Epic 2 — Systems Physics Lab
**Merge: E3 + Myelin S9.1**

**Goal:** Be able to explain p99 spikes with OS counters.

### Stories
- CFS cliff demo (oversubscription + contention) + perf counters
- TLB cliff demo (pointer chasing vs contiguous) + perf counters
- NUMA cliff demo (first-touch / remote access) + perf counters
- Arena allocator (C++ bump/free-all) + fragmentation/locality tests (ties to TLB)
- "Perf Stat Gate" one-command bad vs fixed

**Artifacts:** Cliff lab report + perf stat outputs, "what counter proves what" cheat sheet.

**Sections hit:** (12) Linux/NUMA, (3) perf methodology, supports (1)(2).

## Epic 3 — GPU Guardrails + Profiling Mastery
**Merge: E5 + E10 + Module 1 + part of M5**

**Goal:** You can profile and enforce GPU SLOs.

### Stories
- NVML/DCGM exporter → Prometheus
- Headroom policy (deny/throttle pre-OOM) + metrics
- KV pressure proxy (or scrape vLLM) + policy input
- Profiling playbook: Nsight Systems/Compute (or rocprof), roofline basics, "don't perturb" rules
- One kernel micro-lab: coalesced vs strided access + occupancy vs registers (tiny CUDA/Triton)

**Artifacts:** GPU dashboard pack + profiling report ("why low FLOPs", "why p99 spikes").

**Sections hit:** (1) GPU microarch, (2) kernel optimization thinking, (3) profiling, (11).

## Epic 4 — Kernel / Op / Compiler Sprint
**Merge: Myelin S9.3 + Modules 5–7**

**Goal:** Demonstrate you can cross the boundary from theory → code.

### Stories
- Triton/CUDA kernel toy + profile + optimize (real counters)
- One custom op (PyTorch extension) OR a fused op comparison
- torch.compile analysis: eager vs compile, fusion evidence

**Artifacts:** "kernel beats baseline" repo section + short writeup + before/after numbers.

**Sections hit:** (2) kernel/framework opt, (3) profiling, supports (1).

## Epic 5 — Distributed Training + NCCL Ops
**Merge: Epic 11 + Module 8 + part of NetLab**

**Goal:** You can talk distributed parallelism and prove comm behavior.

### Stories
- DDP/FSDP mini run + scaling notes (even 2 GPUs is enough)
- nccl-tests harness + bandwidth/latency curves (or RCCL equivalent)
- Knob matrix + "how to tell latency vs bandwidth vs sync limited"
- NCCL hang/runbook (logs, env vars, isolation)

**Artifacts:** Scaling efficiency report + NCCL tuning matrix + hang triage playbook.

**Sections hit:** (4) dist training, (5) NCCL/RCCL, (3) profiling.

## Epic 6 — Fabric Networking + eBPF
**Merge: E7 + E12 + NetLab incast**

**Goal:** Correlate tail latency to network symptoms.

### Stories
- TCP retransmit eBPF → Prometheus + dashboard
- Incast/jitter experiment + correlation to p99
- If hardware exists: RDMA perftest baseline + validation checklist
- RoCE pitfalls drill (simulated loss/microburst using tc)

**Artifacts:** "network ↔ p99" dashboard + incident drill runbook.

**Sections hit:** (7) RDMA/IB/RoCE, (11) ops, (5) comm symptoms.

## Epic 7 — Storage & Checkpoint Storms + Object Basics
**Merge: E8A + critical parts of E14**

**Goal:** Reproduce training-killers in storage and fix them.

### Stories
- Checkpoint shard writer simulator
- Directory sharding + async/stagger
- NVMe burst buffer then flush
- Measure metadata ceiling vs data ceiling (p99 checkpoint time)
- Add object storage mini-lab (MinIO): realistic S3 bench + "when object vs POSIX"

**Artifacts:** Checkpoint storm report + "storage design memo" + S3 bench results.

**Sections hit:** (8) HPC storage, (9) object storage, (11).

## Epic 8 — Platform + Bring-up Hygiene
**Merge: E6 + E13 + light Module 11**

**Goal:** Show platform-engineer competence without boiling the ocean.

### Stories
- Minimal K8s operator skeleton OR "deployment template + guardrails" (keep scope small)
- Acceptance suite scripts: topology, GPU health, NCCL smoke, network smoke, storage smoke
- Rollout strategy doc (driver/kernel/fabric upgrades)
- Multi-tenant knobs: quotas/limits/readiness/PDB basics

**Artifacts:** Cluster bring-up checklist + acceptance suite + rollout playbook.

**Sections hit:** (6) bring-up/DC, (10) Kubernetes/platform, (11) reliability.

## 8-Week (2-Month) Schedule

You've already finished Epic 1 S1.2, so you're ahead. Here's a realistic cadence:

- **Week 1:** Finish Epic 1 (S1.3 + minimal S2 pieces) → "serve safely under overload"
- **Week 2:** Epic 2 (CFS/TLB/NUMA + perf gate)
- **Week 3:** Epic 3 (NVML/DCGM + profiling playbook + one kernel lab)
- **Week 4:** Epic 4 (Triton/CUDA + custom op + torch.compile evidence)
- **Week 5:** Epic 5 (DDP/FSDP mini + nccl-tests + tuning + runbook)
- **Week 6:** Epic 6 (eBPF retransmits + incast/jitter + correlation dashboard)
- **Week 7:** Epic 7 (checkpoint storms + NVMe tiering + MinIO S3 bench)
- **Week 8:** Epic 8 (bring-up acceptance suite + K8s deployment primitive + rollout playbook)

This puts the “GPU/NUMA/profiling/comm” content in the first half instead of late.

## 8-Epic Plan Table (Interview-First + Hands-On)

| Epic | Story | Area Covered (what you're covering) | Expected Time |
|------|-------|-----|---|
| **Epic 1** — Serving cell + measurement + safe overload | **S1.3** Chat-path metrics + tracing | Add request-id; per-route counters; latency histogram for /v1/chat/completions; inflight gauge; upstream error counters; structured logs; dashboards starter | 2 days |
| | **S1.4** Chat benchmark scenario | Extend bench to POST JSON to /v1/chat/completions; baseline + regression gate for chat; knee-of-curve sweep script | 2 days |
| | **S1.5** Overload behavior v1 | Token estimate (rough) + admission cap; bounded queue (small); degrade mode (max_tokens clamp); explicit 429/503 policies | 3 days |
| | **S1.6** Circuit breaker + upstream health | Health probing; breaker open/half-open/close; fast-fail tail stabilization; metrics for breaker state | 3 days |
| **Epic 2** — Systems Physics lab (CFS/TLB/NUMA) + Perf Gate | **S2.1** CFS cliff lab | Oversubscription + contended mutex → context switches/runnable inflation → p99 spike; fix via sharding/less contention; record perf counters | 3 days |
| | **S2.2** TLB cliff lab | Pointer-chasing vs contiguous layout; dTLB misses/page-walks; demonstrate improvement with contiguous buffers/arena-style layout | 3 days |
| | **S2.3** NUMA cliff lab | First-touch placement; local vs remote bandwidth/latency; pinning + numactl; measure migrations/remote accesses | 2 days |
| | **S2.4** Perf Stat Gate | One-command "bad vs fixed" runs; store perf stat artifacts; add regression thresholds (cs/migrations/TLB) | 2 days |
| **Epic 3** — GPU guardrails + profiling mastery | **S3.1** NVML/DCGM exporter + dashboards | GPU util/mem/temp/power; Prom scrape; Grafana dashboard; alert thresholds | 2 days |
| | **S3.2** Headroom policy (pre-OOM) | Enforce min free mem; throttle/deny requests; couple with Sentinel admission; metrics + logs | 2 days |
| | **S3.3** KV-pressure proxy | Estimate KV/cache pressure from seq len + concurrency (or scrape vLLM); feed into guardrails | 2 days |
| | **S3.4** Profiling playbook | Nsight Systems vs Compute (or ROCm equivalents); what each answers; roofline basics; "profiling without perturbation"; cross-run comparability checklist | 3 days |
| **Epic 4** — Kernel / op / compiler sprint | **S4.1** Kernel micro-lab (coalescing/occupancy) | Write tiny CUDA/Triton kernels; coalesced vs strided; register pressure vs occupancy; measure via profiler/counters | 4 days |
| | **S4.2** Custom op or fused op demo | PyTorch extension or fused op; correctness checks; perf before/after; ABI/FFI basics | 3 days |
| | **S4.3** torch.compile analysis | Eager vs compile; fusion evidence; compiler logs; latency deltas; pitfalls (dynamic shapes) | 3 days |
| **Epic 5** — Distributed training + NCCL ops | **S5.1** DDP/FSDP mini-run | Run small training; throughput, step time breakdown; gradient accumulation; memory tradeoffs; determinism notes | 3 days |
| | **S5.2** NCCL tests harness + curves | nccl-tests (or RCCL) bandwidth/latency sweeps; scaling across nodes if possible; store results | 3 days |
| | **S5.3** Knob matrix + topology awareness | Channels/topology envs; IB vs TCP validation; NUMA/NIC locality checks; interpret symptoms | 3 days |
| | **S5.4** Hang/debug runbook | NCCL hang triage: logs, env vars, rank isolation, timeouts; "works on 1 node not 64" comm angle | 2 days |
| **Epic 6** — Fabric networking + eBPF + incast | **S6.1** eBPF retransmits → Prom | Kernel telemetry (retransmits); exporter; dashboards; correlate to p99 | 3 days |
| | **S6.2** Incast/jitter lab (TCP) | Traffic generator; induce incast; observe cwnd/retrans/jitter; plots + analysis | 3 days |
| | **S6.3** Loss/microburst drill | tc netem drills; validate detection/alerts; incident-style report | 2 days |
| | **S6.4** RDMA baseline (optional) | If hardware: perftest/ib tools; validate RDMA path; CPU + latency comparisons | 2 days (optional) |
Epic 7 — Storage storms + object basics	S7.1 Checkpoint sharding simulator	N-rank writer; contention patterns; baseline performance model	2 days
    S7.2 Directory sharding + async/stagger	Reduce metadata herd; bounded IO concurrency; measure p99 checkpoint time	3 days
    S7.3 NVMe burst buffer then flush	Local tier absorb spikes; flush pipeline; tail latency improvements	3 days
    S7.4 Metadata vs data ceiling report	Ops/sec vs bandwidth ceilings; step impact model; regression thresholds	2 days
    S7.5 MinIO S3 benchmark mini-lab	Multipart/concurrency; small vs large objects; when object vs POSIX; hot-object intuition	3 days
Epic 8 — Platform + bring-up hygiene	S8.1 Acceptance suite scripts	Topology checks (GPU↔NIC↔NUMA); GPU health; comm smoke; storage smoke; “production-ready” checklist	4 days
    S8.2 Rollout + risk playbook	Driver/firmware/kernel/CNI/CSI upgrades; canary rings; rollback criteria; comms template	2 days
    S8.3 Minimal K8s deployment primitive	“Model deployment” template (doesn’t need full operator); readiness/liveness; resource limits; PDB/priority basics	4 days
    S8.4 Multi-tenant guardrails	Quotas/limits; namespace patterns; basic network/storage class separation; observability hooks	2 days
