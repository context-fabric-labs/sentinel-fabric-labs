PHASE 1 — Infra skeleton + guardrails (repo-level)
Prompt 1 — Create infra folder + scripts + cost guardrails

(Open repo root in Copilot Chat)

Implement the AWS “Platform Cell + Burst Cells” scaffold in the repo.
Create an infra/ directory with:

README.md describing: what is created, costs, and teardown safety

env.example containing required vars (AWS_REGION, CLUSTER_NAME, ACCOUNT_ID, S3_BUCKET, etc.)

scripts (bash, POSIX-compatible):

platform_up.sh (create/update EKS platform cell with CPU spot nodegroup)

platform_down.sh (scale nodegroups to 0; keep cluster)

teardown_all.sh (destroy everything; must require typing the cluster name to proceed)

gpu_on.sh / gpu_off.sh (scale GPU nodegroup)

efa_on.sh / efa_off.sh (create/scale EFA nodegroup)

all scripts must:

fail fast (set -euo pipefail)

print what they will do

support a DRY_RUN=1 mode that only prints commands
Use eksctl as the primary tool (do not introduce Terraform).
Do not run commands. Just write scripts and docs.
Include validation section: aws sts get-caller-identity, eksctl version, kubectl version --client.
After changes: output the list of files created and the exact commands a user would run to bring up and tear down the cluster.

Prompt 2 — Add minimal K8s manifests folder

(Open repo root)

Create a k8s/ folder with subfolders:

k8s/base/ for common namespace + service accounts

k8s/sentinel/ for Sentinel Deployment/Service

k8s/upstream_stub/ for upstream stub Deployment/Service

k8s/bench/ for bench Job template
Provide Kustomize files (kustomization.yaml) for base and overlays:

k8s/overlays/dev/ that deploys sentinel + upstream_stub + bench job (manual trigger) into namespace sentinel-fabric.
Use port 8080 for sentinel and 4000 for upstream.
Do not add Ingress yet—just ClusterIP services + port-forward instructions in docs.
Add a k8s/README.md with exact apply commands and port-forward commands.

PHASE 2 — Deploy Sentinel + stub on EKS (Epic1 integration)
Prompt 3 — Add Dockerfiles + ECR build scripts

(Open repo root)

Add containerization for sentinel, upstream_stub, and bench:

Create containers/sentinel/Dockerfile, containers/upstream_stub/Dockerfile, containers/bench/Dockerfile

Use multi-stage builds where appropriate

Images must run as non-root if feasible

Add infra/build_push.sh script that builds and pushes images to ECR (create repos if missing)

Tag images with git short SHA and latest
Update k8s manifests to reference image names via kustomize images: so they’re easy to override.
Add validation commands for building locally and running docker run for each image.

Prompt 4 — Make bench job write artifacts to S3

(Open bench/src/main.rs and repo root)

Extend the bench tool to optionally upload results JSON to S3 after a run.
Requirements:

Add CLI flags to bench run:

--s3-bucket <name> optional

--s3-prefix <prefix> default sentinel-fabric/bench/

If bucket is provided, upload the output JSON file after writing it locally.

Use AWS SDK for Rust (minimal features) or invoke AWS CLI from the job (choose simplest that is reliable in k8s).

For k8s job, prefer AWS CLI approach to avoid heavy Rust deps:

Add option --upload-with-aws-cli (default true in container) which shells out aws s3 cp.

Update containers/bench/Dockerfile to include AWS CLI v2.

Update k8s/bench job to:

mount an IRSA service account (documented but keep YAML generic)

write output to a known path and upload to S3.
Add tests for the new CLI args parsing (no network calls).
Provide exact commands for:

local run without S3

k8s job run with S3 upload (assuming IRSA is set up)

PHASE 3 — GPU backend lane (vLLM + SGLang)
Prompt 5 — Add GPU Operator install guide + Helm scripts

(Open repo root)

Add infra/gpu_operator_install.sh to install NVIDIA GPU Operator on EKS using Helm.
Requirements:

Script must check cluster context and namespace

Install into namespace gpu-operator

Enable DCGM exporter

Provide uninstall script infra/gpu_operator_uninstall.sh
Add docs/gpu_operator.md with:

prerequisites

verification commands (kubectl get pods -n gpu-operator, nvidia-smi via debug pod)

how to confirm DCGM metrics appear in Prometheus
Do not add Prometheus operator integration automatically—just document scraping options.

Prompt 6 — Add vLLM deployment (OpenAI server) behind a service

(Open repo root)

Add Kubernetes manifests under k8s/vllm/ to deploy vLLM OpenAI-compatible server.
Requirements:

Deployment + Service in namespace sentinel-fabric

Use GPU requests/limits (nvidia.com/gpu: 1)

Provide config via env vars (MODEL_ID placeholder, e.g. meta-llama/Llama-3.1-8B-Instruct)

Add a k8s/vllm/README.md with:

how to set model + HF token (K8s secret)

port-forward command

sample curl to /v1/chat/completions

Keep it minimal and avoid advanced persistence.
Also add docs/backend_lane.md describing B0 stub → B1 vLLM.
Do not modify Sentinel yet.

Prompt 7 — Sentinel multi-backend routing v0 (Round-robin + health)

(Open sentinel/src/main.rs)

Evolve Sentinel from single upstream to multi-backend routing.
Requirements:

Introduce a config file sentinel/backends.yaml (or JSON) listing backends:

name, base_url, weight(optional), enabled(true)

Sentinel loads this at startup (path via env BACKENDS_CONFIG).

Implement simple round-robin selection among healthy enabled backends for /v1/chat/completions.

Add per-backend health checks:

background task probes <base_url>/health every 2s

store health state with last_ok timestamp

Metrics:

sentinel_backend_healthy{backend} gauge

sentinel_route_decisions_total{backend,reason} counter

Logs include selected backend + request_id.

Keep existing overload/timeout/body-limit behavior.
Add unit tests for:

parsing backends config

round-robin selection stability given a set of healthy backends
Provide example YAML with 2 backends: upstream_stub and vLLM service.
Ensure cargo test -p sentinel passes.

Prompt 8 — Add SGLang manifests + Sentinel P2C routing

(Open repo root + sentinel/src/main.rs)

Add k8s/sglang/ manifests to deploy SGLang as a second backend (OpenAI-compatible if possible; document endpoint differences if not).
Then upgrade Sentinel routing from round-robin to Power-of-Two-Choices (P2C):

pick 2 random healthy backends and choose lower observed load.
Load signal:

track per-backend inflight count and EWMA latency (maintained in memory).
Metrics:

sentinel_backend_inflight{backend} gauge

sentinel_backend_ewma_latency_ms{backend} gauge

sentinel_route_decisions_total{backend,reason} should include reason like p2c.
Add tests:

P2C chooses lower inflight backend deterministically when one has much higher inflight.
Keep all other behavior unchanged.

PHASE 4 — TensorRT-LLM backend via Triton OpenAI frontend
Prompt 9 — Add Triton OpenAI frontend + TRT-LLM backend manifests (skeleton + docs)

(Open repo root)

Add k8s/triton/ manifests and docs for deploying Triton Inference Server with:

OpenAI-compatible frontend enabled

TensorRT-LLM backend (document model repo requirements and leave placeholders)
Provide:

Deployment + Service

GPU request/limit

Volume mount placeholder for model repository

k8s/triton/README.md detailing:

required images

how to structure model repo

how to curl OpenAI endpoints
Do not attempt to fully build TRT engines automatically. Keep it as a realistic scaffold with clear steps.
Add to docs/backend_lane.md: B2 Triton/TRT-LLM.

PHASE 5 — NVIDIA Dynamo as reference baseline on same EKS
Prompt 10 — Dynamo deployment guide + manifests folder

(Open repo root)

Create k8s/dynamo/ folder with a deployment guide for NVIDIA Dynamo on EKS.
Requirements:

Do not vendor Dynamo source code.

Provide a README that points to the official Dynamo repo and the AWS blog about Dynamo on EKS.

Include a minimal Helm/Kustomize placeholder layout:

namespace, service accounts, and where Dynamo chart/manifests would be applied

Add a section: “How to benchmark Dynamo vs Sentinel” with bench commands and expected artifacts.
Add a doc docs/dynamo_vs_sentinel.md describing:

What metrics we compare (p99, rps, error rate)

How to keep backend constant while swapping routers.

PHASE 6 — EFA/RDMA + NCCL tuning burst cell
Prompt 11 — EFA nodegroup + NCCL tests harness (K8s job)

(Open repo root)

Add support for an EFA-enabled burst nodegroup and NCCL tests on EKS.
Requirements:

Add infra/efa_on.sh that:

creates or scales an EKS managed nodegroup with EFA enabled (eksctl config)

uses cluster placement group if supported

prints verification steps

Add k8s/nccl-tests/:

Kubernetes Job or MPIJob-like approach (keep simple)

runs nccl-tests all_reduce_perf

writes results to JSON in labs/results/ schema or bench-like schema

uploads to S3 using aws cli (reuse bench upload approach)

Add docs docs/nccl_efa.md:

how to verify EFA present

how to interpret bandwidth/latency curves

knob matrix placeholders (env vars, topology checks)
Do not assume RDMA works by default; include troubleshooting checklist.
Ensure manifests do not require proprietary controllers beyond EKS basics.

PHASE 7 — Plan + validation checklist updates
Prompt 12 — Create a single validation checklist doc

(Open repo root)

Create docs/validation_checklist.md that lists, in order, how to validate each phase:

Platform cell

Sentinel+stub

vLLM backend

multi-backend routing

Triton/TRT-LLM scaffold

Dynamo baseline

EFA/NCCL burst
Include exact commands and expected outputs/metrics names.
Also update docs/full_plan.md to reference this checklist.