# Epic 1 — Serving Cell, Measurement, Safe Overload

## Objective
- Build a minimal but realistic serving “cell” that:
  - Proxies a chat endpoint to an upstream.
  - Exposes per-route metrics and request tracing.
  - Handles overload gracefully via admission control, bounded queue, degrade mode, and a circuit breaker with health probing.
  - Is easy to benchmark and reason about (p50/p99 vs QPS, tail stabilization under faults).

## Implementation Overview
- Axum-based reverse proxy (`sentinel`) forwards requests to `upstream_stub`.
- Observability: request IDs, per-route counters, latency histograms, inflight gauges, upstream error counters, structured logs, Prometheus scrape.
- Reliability:
  - Admission control with a small bounded queue; token estimate per request; optional degrade by clamping `max_tokens`.
  - Circuit breaker with background health probe; states CLOSED/OPEN/HALF_OPEN; limited trial slots in half-open.
- Bench harness (`bench`) drives both health GET and chat POST scenarios with concurrency controls and JSON result artifacts.

## Code Map
- Sentinel (proxy):
  - [sentinel/src/main.rs](../sentinel/src/main.rs): App wiring, routes (`/health`, `/metrics`, `/v1/chat/completions`), proxy pipeline, breaker fast-fail, admission + degrade, upstream response classification.
  - [sentinel/src/observability.rs](../sentinel/src/observability.rs): `RequestId`, `request_id_layer`, `http_metrics_layer`, helper for response logging.
  - [sentinel/src/admission.rs](../sentinel/src/admission.rs): `AdmissionController`, `AdmissionConfig`, `AdmissionGuard`; `estimate_tokens()`; `clamp_max_tokens()` to apply degrade.
  - [sentinel/src/breaker.rs](../sentinel/src/breaker.rs): `Breaker`, `BreakerConfig`, `State`, `HalfOpenGuard`, `run_health_probe()` background task.
- Upstream stub (test double):
  - [tools/upstream_stub](../tools/upstream_stub): Simple `/health` and chat handler to exercise proxy.
- Bench:
  - [bench/src/main.rs](../bench/src/main.rs): `bench run` and `bench report` subcommands; drives health/chat scenarios; writes JSON results.
- Scripts:
  - [scripts/sweep_chat.sh](../scripts/sweep_chat.sh): Concurrency sweep for chat scenario, stores artifacts under `bench/results/`.

## How To Run & Test
Prereqs: Rust toolchain, macOS/Linux shell.

- Unit tests (workspace):
  ```bash
  cargo test --workspace
  ```

- Lints (fail on warnings):
  ```bash
  cargo clippy --workspace --all-targets -- -D warnings
  ```

- Start local stack (two terminals):
  ```bash
  # Terminal A: upstream stub (port 4000)
  RUST_LOG=info cargo run -p upstream_stub
  ```
  ```bash
  # Terminal B: sentinel (port 8080, proxies to 4000)
  RUST_LOG=info cargo run -p sentinel -- --bind 127.0.0.1:8080 --upstream http://127.0.0.1:4000
  ```

- Smoke checks:
  ```bash
  curl -sS http://127.0.0.1:8080/health
  curl -sS http://127.0.0.1:8080/metrics | head
  ```

- Bench — health scenario:
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

- Bench — chat scenario (file body):
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

- Bench — chat scenario (inline JSON):
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

- Concurrency sweep:
  ```bash
  bash scripts/sweep_chat.sh
  ```

- Pretty-print results:
  ```bash
  cargo run -p bench -- report bench/results/*.json
  ```

## Metrics & Signals
- Request/route metrics: counters per route and status; latency histograms; inflight gauges.
- Upstream signals: error counters by kind (timeout, connect, HTTP 5xx), success/failure feeding breaker.
- Breaker state: a gauge for `CLOSED`/`OPEN`/`HALF_OPEN` and transition counters.
- Admission/queue: accepted vs queued vs rejected; degrade applied (clamped tokens) indicator.
- All exposed via Prometheus at `/metrics` on sentinel.

## Interview Focus Areas
- Measurement:
  - How do we produce a trustworthy p50/p99 vs QPS curve? Warm-up, steady-state windows, and how bench collects latencies.
  - Minimizing measurement perturbation; what’s the overhead of metrics and tracing layers?
- Overload strategy rationale:
  - Why admission control before work is started; why a small bounded queue vs unbounded; signals to flip from admit → reject.
  - Degrade mode: clamping `max_tokens` as a controlled quality reduction under pressure; risks and guardrails.
- Circuit breaker design:
  - Failure thresholding, OPEN/HALF_OPEN trial slots, background health probe cadence; avoiding thundering herds on recovery.
  - What constitutes a failure vs a success; edge cases (timeouts vs 4xx vs 5xx).
- Failure modes & tail stabilization:
  - How breaker + admission prevent amplification, and expected metrics changes during an upstream brownout.
- Extensibility:
  - Where to add rate limiting, per-tenant quotas, or token bucket shaping; layering with the current tower stack.

## Key Decisions / Trade-offs
- Simplicity over completeness for token estimation (character-based heuristic).
- Prom-first metrics; logs are structured but light-weight.
- Rust `tokio::Semaphore` for inflight and half-open slot control via `Arc<Semaphore>`.
- Axum + tower-http middleware for clarity and composability.

---
For quick links to code:
- Proxy entry: [sentinel/src/main.rs](../sentinel/src/main.rs)
- Admission: [sentinel/src/admission.rs](../sentinel/src/admission.rs)
- Breaker: [sentinel/src/breaker.rs](../sentinel/src/breaker.rs)
- Observability: [sentinel/src/observability.rs](../sentinel/src/observability.rs)
- Bench: [bench/src/main.rs](../bench/src/main.rs)
- Stub: [tools/upstream_stub](../tools/upstream_stub)
- Sweep: [scripts/sweep_chat.sh](../scripts/sweep_chat.sh)