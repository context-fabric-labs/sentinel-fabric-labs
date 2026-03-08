# NVIDIA GPU Operator Installation Guide

This guide covers installing and managing NVIDIA GPU Operator on Amazon EKS using Helm.

## Overview

NVIDIA GPU Operator automates the management of NVIDIA GPUs on Kubernetes clusters by:

- Installing NVIDIA drivers (if needed)
- Deploying container toolkit
- Installing device plugin
- Enabling DCGM exporter for metrics
- Managing GPU resources across nodes

## Prerequisites

### 1. EKS Cluster with GPU Nodes

You need an EKS cluster with GPU-enabled nodes (g4dn, g5, p3, p4 instances):

```bash
# Check for GPU nodes
kubectl get nodes -l nvidia.com/gpu.present=true

# If no GPU nodes exist, create a GPU nodegroup
./infra/scripts/gpu_on.sh
```

### 2. Required Tools

- **kubectl** configured with EKS cluster access
- **Helm v3** installed
- **AWS CLI** configured

```bash
# Verify tools
helm version --short
kubectl version --client --short
aws --version
```

### 3. IAM Permissions

Ensure your IAM user/role has permissions to:
- Create/manage namespaces
- Deploy Helm charts
- Create DaemonSets and Pods

### 4. Node Requirements

- **Amazon Linux 2** or **Ubuntu 20.04+** on GPU nodes
- **containerd** or **Docker** runtime
- Kernel version compatible with NVIDIA drivers

## Installation

### Quick Install

```bash
# Install GPU Operator
./infra/gpu_operator_install.sh
```

The script will:
1. ✅ Validate prerequisites
2. ✅ Check cluster context
3. ✅ Create `gpu-operator` namespace
4. ✅ Add NVIDIA Helm repository
5. ✅ Install GPU Operator with DCGM exporter
6. ✅ Wait for components to be ready
7. ✅ Verify installation

### Manual Installation

If you prefer manual installation:

```bash
# Create namespace
kubectl create namespace gpu-operator

# Add NVIDIA Helm repo
helm repo add nvidia https://helm.ngc.nvidia.com/nvidia
helm repo update

# Install GPU Operator with DCGM exporter
helm install gpu-operator nvidia/gpu-operator \
  --namespace gpu-operator \
  --create-namespace \
  --version v24.9.1 \
  --set operator.defaultRuntime=containerd \
  --set dcgmExporter.enabled=true \
  --set dcgmExporter.serviceMonitor.enabled=false
```

### Custom Configuration

For advanced configurations, create a values file:

```yaml
# gpu-operator-values.yaml
operator:
  defaultRuntime: containerd

dcgmExporter:
  enabled: true
  serviceMonitor:
    enabled: false  # Enable if Prometheus Operator is installed

devicePlugin:
  enabled: true
  config:
    name: ""  # Optional: reference to ConfigMap with device plugin config

driver:
  enabled: true  # Set false if drivers are pre-installed
  upgradePolicy:
    autoUpgrade: true
```

Install with custom values:

```bash
helm install gpu-operator nvidia/gpu-operator \
  --namespace gpu-operator \
  --create-namespace \
  -f gpu-operator-values.yaml
```

## Verification

### 1. Check GPU Operator Pods

All pods should be in `Running` state:

```bash
kubectl get pods -n gpu-operator
```

Expected output:

```
NAME                                         READY   STATUS      RESTARTS   AGE
gpu-operator-5d8c9f7b6-xk2m3                1/1     Running     0          5m
nvidia-container-toolkit-daemonset-abc12    1/1     Running     0          5m
nvidia-dcgm-exporter-xyz89                   1/1     Running     0          5m
nvidia-device-plugin-daemonset-def45        1/1     Running     0          5m
nvidia-driver-daemonset-ghi78                1/1     Running     0          5m
nvidia-operator-validator-jkl01              1/1     Running     0          5m
```

### 2. Verify GPU Node Labels

GPU nodes should have NVIDIA labels:

```bash
kubectl get nodes -l nvidia.com/gpu.present=true -o json | jq '.items[].metadata.labels' | grep nvidia
```

Expected labels:
- `nvidia.com/gpu.present=true`
- `nvidia.com/gpu.count=<number>`
- `nvidia.com/gpu.product=<gpu-model>`

### 3. Test GPU Access with nvidia-smi

Run a test pod to verify GPU access:

```bash
# Run nvidia-smi in a debug pod
kubectl run nvidia-smi \
  --image=nvidia/cuda:12.4.1-base-ubuntu22.04 \
  --rm -it \
  --restart=Never \
  --overrides='{
    "spec": {
      "containers": [{
        "name": "nvidia-smi",
        "image": "nvidia/cuda:12.4.1-base-ubuntu22.04",
        "command": ["nvidia-smi"],
        "resources": {
          "limits": {
            "nvidia.com/gpu": 1
          }
        }
      }]
    }
  }'
```

Expected output shows GPU details:

```
+-----------------------------------------------------------------------------+
| NVIDIA-SMI 550.54.15    Driver Version: 550.54.15    CUDA Version: 12.4     |
|-------------------------------+----------------------+----------------------+
| GPU  Name        Persistence-M| Bus-Id        Disp.A | Volatile Uncorr. ECC |
| Fan  Temp  Perf  Pwr:Usage/Cap|         Memory-Usage | GPU-Util  Compute M. |
|===============================+======================+======================|
|   0  Tesla T4            On   | 00000000:00:1E.0 Off |                    0 |
| N/A   45C    P0    27W /  70W |      0MiB / 15360MiB |      0%      Default |
+-------------------------------+----------------------+----------------------+
```

### 4. Deploy Sample GPU Workload

Create a test deployment:

```yaml
# gpu-test.yaml
apiVersion: v1
kind: Pod
metadata:
  name: gpu-test
spec:
  restartPolicy: Never
  containers:
  - name: cuda-vectoradd
    image: nvidia/samples:vectoradd-cuda12.4.1
    resources:
      limits:
        nvidia.com/gpu: 1
```

```bash
kubectl apply -f gpu-test.yaml
kubectl logs gpu-test
```

## DCGM Exporter and Metrics

### DCGM Exporter Status

The DCGM exporter is enabled by default. Check its status:

```bash
# Check exporter pod
kubectl get pods -n gpu-operator -l app=nvidia-dcgm-exporter

# Check exporter service
kubectl get svc -n gpu-operator nvidia-dcgm-exporter
```

### Access DCGM Metrics

DCGM exporter exposes metrics on port 9400:

```bash
# Port-forward to access metrics locally
kubectl port-forward svc/nvidia-dcgm-exporter -n gpu-operator 9400:9400

# In another terminal, query metrics
curl http://localhost:9400/metrics
```

### Sample DCGM Metrics

Expected metrics include:

```
# HELP DCGM_FI_DEV_GPU_TEMP GPU temperature in degrees C
# TYPE DCGM_FI_DEV_GPU_TEMP gauge
DCGM_FI_DEV_GPU_TEMP{gpu="0",UUID="GPU-abc123"} 45.0

# HELP DCGM_FI_DEV_POWER_USAGE Power draw in W
# TYPE DCGM_FI_DEV_POWER_USAGE gauge
DCGM_FI_DEV_POWER_USAGE{gpu="0",UUID="GPU-abc123"} 27.0

# HELP DCGM_FI_DEV_UTILIZATION_GPU GPU utilization %
# TYPE DCGM_FI_DEV_UTILIZATION_GPU gauge
DCGM_FI_DEV_UTILIZATION_GPU{gpu="0",UUID="GPU-abc123"} 0.0
```

### Prometheus Integration

**Note:** This guide does NOT automatically install Prometheus Operator. Below are manual scraping options.

#### Option 1: Enable ServiceMonitor (if Prometheus Operator exists)

If you have Prometheus Operator installed:

```bash
helm upgrade gpu-operator nvidia/gpu-operator \
  --namespace gpu-operator \
  --set dcgmExporter.serviceMonitor.enabled=true \
  --set dcgmExporter.serviceMonitor.additionalLabels.release=prometheus
```

#### Option 2: Manual Prometheus Configuration

Add scrape config to your Prometheus:

```yaml
# prometheus-config.yaml
scrape_configs:
  - job_name: 'nvidia-dcgm-exporter'
    kubernetes_sd_configs:
      - role: endpoints
        namespaces:
          names:
            - gpu-operator
    relabel_configs:
      - source_labels: [__meta_kubernetes_service_name]
        regex: nvidia-dcgm-exporter
        action: keep
      - source_labels: [__meta_kubernetes_endpoint_port_name]
        regex: gpu-metrics
        action: keep
```

#### Option 3: Static Scrape Config

For standalone Prometheus:

```yaml
scrape_configs:
  - job_name: 'nvidia-gpu'
    static_configs:
      - targets:
        - nvidia-dcgm-exporter.gpu-operator.svc.cluster.local:9400
```

### Verify Metrics in Prometheus

Access Prometheus UI and query:

```
# Count of GPUs
count(DCGM_FI_DEV_GPU_TEMP)

# Average GPU temperature
avg(DCGM_FI_DEV_GPU_TEMP)

# GPU utilization by GPU
DCGM_FI_DEV_UTILIZATION_GPU
```

## Troubleshooting

### GPU Operator Pods Not Ready

```bash
# Check pod logs
kubectl logs -n gpu-operator -l app=nvidia-driver-daemonset
kubectl logs -n gpu-operator -l app=nvidia-device-plugin-daemonset

# Describe pod for events
kubectl describe pod -n gpu-operator -l app=nvidia-driver-daemonset
```

### GPU Not Detected

```bash
# Check node labels
kubectl describe node <node-name> | grep -A 5 "Capacity"

# Verify driver installation
kubectl debug -it node/<node-name> --image=nvidia/cuda:12.4.1-base-ubuntu22.04 -- nvidia-smi
```

### DCGM Metrics Not Appearing

```bash
# Check exporter logs
kubectl logs -n gpu-operator -l app=nvidia-dcgm-exporter

# Verify service endpoints
kubectl get endpoints -n gpu-operator nvidia-dcgm-exporter

# Test metrics endpoint
kubectl run test --rm -it --image=curlimages/curl --restart=Never -- \
  http://nvidia-dcgm-exporter.gpu-operator.svc.cluster.local:9400/metrics
```

### Driver Installation Fails

```bash
# Check if drivers are already installed
kubectl run check-drivers --image=nvidia/cuda:12.4.1-base-ubuntu22.04 --rm -it --restart=Never -- nvidia-smi

# If pre-installed, disable driver installation
helm upgrade gpu-operator nvidia/gpu-operator \
  --namespace gpu-operator \
  --set driver.enabled=false
```

## Uninstallation

### Quick Uninstall

```bash
# Uninstall GPU Operator
./infra/gpu_operator_uninstall.sh
```

### Manual Uninstall

```bash
# Remove Helm release
helm uninstall gpu-operator -n gpu-operator

# Delete namespace
kubectl delete namespace gpu-operator

# Optional: Remove GPU labels from nodes
kubectl label nodes -l nvidia.com/gpu.present=true nvidia.com/gpu.present-
kubectl label nodes -l nvidia.com/gpu.present=true nvidia.com/gpu.count-
```

## Upgrade

### Upgrade GPU Operator

```bash
# Update Helm repo
helm repo update

# Upgrade to latest
helm upgrade gpu-operator nvidia/gpu-operator \
  --namespace gpu-operator \
  --set operator.defaultRuntime=containerd \
  --set dcgmExporter.enabled=true

# Or upgrade to specific version
helm upgrade gpu-operator nvidia/gpu-operator \
  --namespace gpu-operator \
  --version v24.9.1
```

## Best Practices

1. **Production Configuration**
   - Pin GPU Operator version for stability
   - Enable driver auto-upgrade with caution
   - Monitor DCGM metrics continuously

2. **Resource Allocation**
   - Use GPU time-slicing for better utilization
   - Set appropriate resource limits
   - Monitor GPU memory usage

3. **Security**
   - Run containers as non-root where possible
   - Use Pod Security Policies/Standards
   - Limit GPU access with RBAC

4. **Monitoring**
   - Set up alerts for GPU temperature
   - Monitor GPU utilization trends
   - Track GPU memory pressure

## References

- [NVIDIA GPU Operator Documentation](https://docs.nvidia.com/datacenter/cloud-native/gpu-operator/latest/)
- [NVIDIA Helm Charts](https://helm.ngc.nvidia.com/nvidia)
- [DCGM Documentation](https://github.com/NVIDIA/dcgm-exporter)
- [EKS GPU Guide](https://docs.aws.amazon.com/eks/latest/userguide/gpu-support.html)

## Support

For issues:
1. Check GPU Operator logs
2. Verify node compatibility
3. Review NVIDIA documentation
4. Check AWS EKS GPU support matrix
