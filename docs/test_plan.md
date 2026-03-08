# Sentinel Fabric: Full AWS Plan (Dynamo + Sentinel)

Here's a stepwise, backend-aware, "real but light" plan that uses NVIDIA Dynamo (with vLLM/SGLang/TensorRT-LLM) and your Sentinel (Dynamo-lite in Rust)—with infra that you can bring up/down cheaply and still do real tuning (GPU placement, RDMA/EFA, NCCL, storage).

## 0) What We're Building



### Two Routers Side-by-Side

**NVIDIA Dynamo** (ground truth / reference) — supports vLLM, TensorRT-LLM, SGLang.

**Sentinel** (your Dynamo-lite) — Rust proxy/router focusing on low-latency routing + placement + SLO + instrumentation.

### Why This Is Powerful for Interviews

You can say: "I built a lightweight router and validated ideas against Dynamo in the same environment."

---

## 1) AWS Infra Design (Cost-Optimized)

### A) Always-on "Platform Cell" (cheap baseline)

**Purpose:** Keep EKS + observability + Sentinel up continuously on tiny Spot CPU nodes.

**Components:**
- EKS cluster
- CPU managed node group (Spot)
- Prometheus + Grafana
- ECR + S3 artifact bucket

**Reason to choose EKS:** Dynamo has an AWS/EKS deployment path (operator/blueprint).

### B) Burst Cell 1: Single-GPU Serving (short sessions)

**Purpose:** Run vLLM/SGLang/TensorRT-LLM backends and your router placement logic.

**Bring up only when needed:**
- GPU node group (Spot when possible)
- NVIDIA GPU Operator (drivers + device plugin + DCGM exporter managed)

### C) Burst Cell 2: Multi-Node RDMA/EFA (very short sessions)

**Purpose:** Real NCCL + RDMA/EFA tuning and (later) Dynamo-like distributed serving experiments.

- Add an EFA-enabled nodegroup to EKS (`efaEnabled: true`), which also auto-deploys the EFA device plugin.
- Use P4d (400 Gbps) or P5/P5e/P5en (up to 3,200 Gbps) depending on budget/quota.
- Follow AWS's "Get started with EFA and NCCL" steps for validation and AMI/tooling if you do non-K8s runs.

### Spot Strategy (Key to Stay Under $1,000)

- For Auto Scaling / fleets, use `price-capacity-optimized` (recommended by AWS) to reduce interruptions.
- Make everything scripted so GPU/EFA nodes can't be left running accidentally.

---

## 2) Stepwise Plan with Backend Infra (the "Backend Lane")

### Step 1 — Platform Cell Bootstrap (no GPU yet)

**Infra:**
- EKS + CPU Spot node group
- `kube-prometheus-stack` (Prom/Grafana)
- S3 bucket for artifacts (bench JSON, perf logs)
- ECR repos for: sentinel, upstream_stub, bench-job

**Deploy:**
- Sentinel + upstream_stub into EKS
- bench as a Kubernetes Job (writes JSON to S3)

**Acceptance tests:**
- `curl /health, /metrics, /v1/chat/completions` (via port-forward or ingress)
- bench chat baseline stored in S3
- regression gate run in CI

**Optional accelerator:** Use AI on EKS blueprints as your base IaC; AWS uses this path for Dynamo on EKS.

### Step 2 — Burst Cell 1: vLLM Backend (single GPU)

**Infra:**
- Add GPU node group (Spot if available)
- Install NVIDIA GPU Operator (drivers + toolkit + DCGM exporter lifecycle-managed)

**Backend:**
- Deploy vLLM OpenAI-compatible server (so Sentinel can proxy without changing API)

**Sentinel upgrades (Dynamo-lite v0):**
- "backend registry" (even static YAML is fine)
- per-backend health checking + metrics
- routing: start with round-robin

**Acceptance tests:**
- bench chat against vLLM through Sentinel
- Grafana panels: p99 latency, error rate, GPU util/mem

### Step 3 — Add SGLang Backend (still single-node, multi-backend routing)

**Infra:**
- Same GPU node group

**Backend:**
- Deploy SGLang as a second backend; it's compatible with OpenAI-style APIs (and is a Dynamo-supported backend)

**Sentinel upgrades (Dynamo-lite v1):**
- multi-backend routing: P2C (power-of-two choices) using inflight/latency
- routing decision metrics: `route_decisions_total{backend,reason}`

**Acceptance tests:**
- Run bench with 2 backends:
  - compare RR vs P2C under load
- Failure drill:
  - kill one backend → Sentinel routes away quickly (breaker/health)

### Step 4 — Add TensorRT-LLM via Triton + OpenAI Frontend (backend realism)

This is where you get "production-ish" backend components.

**Backend:**
- Deploy Triton Inference Server with TensorRT-LLM backend support
- Use Triton's OpenAI-compatible frontend, which explicitly supports vLLM backend and guided decoding in TensorRT-LLM backend

**Why this matters:**
You can swap backends under the same OpenAI API and test router behavior + latency/throughput tradeoffs.

**Acceptance tests:**
Same chat scenario through:
- Sentinel → vLLM
- Sentinel → SGLang
- Sentinel → Triton(OpenAI) → TensorRT-LLM
- artifact: compare p50/p95/p99 + GPU utilization

### Step 5 — Deploy NVIDIA Dynamo on the Same EKS Cluster (reference baseline)

**Infra:**
- Install Dynamo operator using AWS's Dynamo blueprint path (AI on EKS)
- Dynamo is open-source and explicitly supports vLLM/TRT-LLM/SGLang

**Goal:**
Run Dynamo with the same backends and compare:
- p99, throughput, scaling behavior, routing semantics
- This gives you a "control group" for your Dynamo-lite

**Acceptance tests:**
- same bench scenarios against Dynamo endpoint
- side-by-side dashboard snapshots: Dynamo vs Sentinel

### Step 6 — Burst Cell 2: Multi-Node RDMA/EFA + NCCL Tuning (Epic 5 realism)

**Infra:**
- Add EFA-enabled EKS node group (`efaEnabled: true`), which handles security group/placement group/device plugin prerequisites via eksctl workflows

**Run on:**
- P4d for 400 Gbps EFA + GPUDirect RDMA baseline
- or P5/P5e/P5en for up to 3,200 Gbps EFA + GPUDirect RDMA (capstone, expensive)

**Work:**
- run nccl-tests sweeps (bandwidth/latency)
- tune:
  - placement (cluster placement)
  - NCCL env knobs
  - validate RDMA path
- Use AWS's EFA+NCCL bring-up checklist as the canonical validation steps

**Acceptance tests:**
- store NCCL curves as JSON + plots
- document a "knob matrix" and "hang triage" runbook

### Step 7 — Storage Lane (Epic 7) Tied to Serving Realism

If you want a Dynamo-like "tiering / offload" story later:
- Add FSx for Lustre (and optionally tune it with EFA-capable nodes in EKS)
- Use it for checkpoint storm simulations and "burst buffer then flush" experiments

### Step 8 — Epic 8 Acceptance Suite: "Production-Lite Bring-Up"

This becomes your "I can run an HPC inference cell" proof.

**Acceptance script checks (run on any cluster):**
- GPU health + telemetry present (DCGM exporter / metrics)
- GPU↔NIC↔NUMA topology checks (for locality decisions)
- EFA present on RDMA clusters (EKS EFA integration)
- backend health endpoints reachable (vLLM/SGLang/Triton/Dynamo)
- end-to-end SLO check: bench p99 within budget + regression gate

---

## 3) What Scripts/IaC You'll Want (to stay under $1,000)

Create an `infra/` folder with these one-command flows:

### Always-on

- **`infra/platform_up.sh`** — Creates/updates EKS + CPU Spot node group + observability
- **`infra/platform_down.sh`** — Scales node groups to 0 (keeps EKS control plane if you want)

### Burst Controls

- **`infra/gpu_on.sh` / `infra/gpu_off.sh`** — Scales GPU node group up/down
- **`infra/efa_on.sh` / `infra/efa_off.sh`** — Creates/scales EFA nodegroup only for NCCL/RDMA sessions

### Hard Kill (cost safety)

- **`infra/teardown_all.sh`** — Destroys everything (cluster + nodegroups + LB + PVCs if desired)

### Spot Settings

- Always use `price-capacity-optimized` where supported (ASG/fleet)

---

## 4) How Sentinel (Dynamo-lite) Evolves in This Plan

You'll build Sentinel features only when the backend infra makes them real:

- **After vLLM (Step 2):** backend health + basic routing metrics
- **After multi-backend (Step 3–4):** P2C routing + placement policies
- **With GPU Operator metrics (Step 2+):** GPU headroom admission (deny/degrade)
- **With EFA/NCCL (Step 6):** topology-aware routing and "network is a first-class SLO input"
- **With Dynamo deployed (Step 5):** validate features and choose the next 1–2 Dynamo behaviors to clone (KV-affinity routing, prefill/decode split, etc.)

---

## 5) Suggested Execution Order (Max Value Early)

1. Platform Cell + Sentinel + stub + bench + dashboards
2. vLLM on GPU nodegroup
3. Multi-backend: add SGLang + P2C routing
4. Add Triton(OpenAI) + TensorRT-LLM
5. Deploy Dynamo and benchmark side-by-side
6. Short EFA/NCCL burst sessions (2 nodes) for real tuning artifacts
7. Storage lane if/when you want tiering/offload realism
8. Acceptance suite that proves the cell is "ready"
