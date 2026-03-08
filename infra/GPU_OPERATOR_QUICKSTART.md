# GPU Operator Quick Start

## Installation

```bash
# Install NVIDIA GPU Operator on EKS
./infra/gpu_operator_install.sh
```

**What it does:**
- ✅ Checks cluster context and prerequisites
- ✅ Creates `gpu-operator` namespace
- ✅ Installs GPU Operator via Helm
- ✅ Enables DCGM exporter for metrics
- ✅ Waits for all components to be ready

## Verification

```bash
# Check GPU Operator pods
kubectl get pods -n gpu-operator

# Test GPU access with nvidia-smi
kubectl run nvidia-smi \
  --image=nvidia/cuda:12.4.1-base-ubuntu22.04 \
  --rm -it --restart=Never -- nvidia-smi

# Check DCGM metrics
kubectl port-forward svc/nvidia-dcgm-exporter -n gpu-operator 9400:9400
curl http://localhost:9400/metrics
```

## Uninstallation

```bash
# Remove GPU Operator
./infra/gpu_operator_uninstall.sh
```

## Prometheus Integration

**Not enabled by default.** See [docs/gpu_operator.md](../docs/gpu_operator.md) for:
- Enabling ServiceMonitor (if Prometheus Operator installed)
- Manual scrape configuration
- Sample metrics queries

## Key Commands

| Task | Command |
|------|---------|
| Install | `./infra/gpu_operator_install.sh` |
| Uninstall | `./infra/gpu_operator_uninstall.sh` |
| Check pods | `kubectl get pods -n gpu-operator` |
| Test GPU | `kubectl run nvidia-smi --image=nvidia/cuda:12.4.1-base-ubuntu22.04 --rm -it --restart=Never -- nvidia-smi` |
| View metrics | `kubectl port-forward svc/nvidia-dcgm-exporter -n gpu-operator 9400:9400` |
| Upgrade | `helm upgrade gpu-operator nvidia/gpu-operator -n gpu-operator` |

## Configuration

Default settings:
- **Namespace:** `gpu-operator`
- **DCGM Exporter:** Enabled
- **ServiceMonitor:** Disabled (enable manually if needed)
- **Runtime:** containerd

Override with environment variables:
```bash
GPU_OPERATOR_NAMESPACE=gpu \
DCGM_EXPORTER_ENABLED=true \
./infra/gpu_operator_install.sh
```

## Prerequisites

- EKS cluster with GPU nodes (g4dn, g5, p3, p4)
- Helm v3 installed
- kubectl configured
- AWS CLI configured

Create GPU nodegroup if needed:
```bash
./infra/scripts/gpu_on.sh
```

## Documentation

Full guide: [docs/gpu_operator.md](../docs/gpu_operator.md)
