# Kubernetes Deployment for Sentinel Fabric

This directory contains Kubernetes manifests and Kustomize configurations for deploying the Sentinel Fabric platform on EKS.

## Directory Structure

```
k8s/
├── base/                    # Common resources
│   ├── namespace.yaml       # sentinel-fabric namespace
│   ├── serviceaccounts.yaml # Service accounts for all components
│   └── kustomization.yaml
├── sentinel/                # Sentinel gateway
│   ├── deployment.yaml
│   ├── service.yaml
│   └── kustomization.yaml
├── upstream_stub/           # Upstream stub service
│   ├── deployment.yaml
│   ├── service.yaml
│   └── kustomization.yaml
├── bench/                   # Benchmark job
│   ├── job.yaml
│   └── kustomization.yaml
├── overlays/
│   └── dev/                 # Development overlay
│       └── kustomization.yaml
└── README.md
```

## Prerequisites

- EKS cluster with kubectl configured
- Kustomize v5.0+ (included in kubectl v1.14+)
- Docker images built and available:
  - `sentinel:dev` or `sentinel:latest`
  - `upstream_stub:dev` or `upstream_stub:latest`
  - `bench:latest` (optional, for benchmarking)

## Quick Start

### 1. Deploy Base Infrastructure (Namespace + Service Accounts)

```bash
# Apply base resources
kubectl apply -k k8s/base/

# Verify
kubectl get namespace sentinel-fabric
kubectl get serviceaccounts -n sentinel-fabric
```

### 2. Deploy Development Environment

```bash
# Apply dev overlay (deploys sentinel + upstream_stub)
kubectl apply -k k8s/overlays/dev/

# Verify deployments
kubectl get deployments -n sentinel-fabric
kubectl get pods -n sentinel-fabric
kubectl get services -n sentinel-fabric
```

### 3. Verify Services

```bash
# Check sentinel service
kubectl get service sentinel -n sentinel-fabric

# Check upstream-stub service
kubectl get service upstream-stub -n sentinel-fabric

# Check pod status
kubectl get pods -n sentinel-fabric -w
```

## Port Forwarding

Since services are ClusterIP (internal only), use port-forwarding to access them locally:

### Forward Sentinel (Port 8080)

```bash
# In terminal 1 - Forward sentinel
kubectl port-forward svc/sentinel 8080:8080 -n sentinel-fabric

# Test sentinel
curl http://localhost:8080/health
curl http://localhost:8080/ready
```

### Forward Upstream Stub (Port 4000)

```bash
# In terminal 2 - Forward upstream-stub
kubectl port-forward svc/upstream-stub 4000:4000 -n sentinel-fabric

# Test upstream-stub
curl http://localhost:4000/health
curl http://localhost:4000/ready
```

### Forward Both Simultaneously

```bash
# Use two separate terminals or use background processes
kubectl port-forward svc/sentinel 8080:8080 -n sentinel-fabric &
kubectl port-forward svc/upstream-stub 4000:4000 -n sentinel-fabric &

# Test both
curl http://localhost:8080/health
curl http://localhost:4000/health
```

## Running Benchmarks

The benchmark job is **not** deployed by default. Run it manually when needed:

### Option 1: Apply Bench Job

```bash
# Create and run bench job
kubectl apply -k k8s/bench/

# Watch job progress
kubectl get jobs -n sentinel-fabric
kubectl get pods -n sentinel-fabric -w

# View logs
kubectl logs -l app.kubernetes.io/name=bench -n sentinel-fabric -f

# Clean up after completion
kubectl delete -k k8s/bench/
```

### Option 2: Run Bench Job Once

```bash
# Create job from template
kubectl create job bench-manual --from=job/bench -n sentinel-fabric

# Or run ad-hoc pod
kubectl run bench-manual \
  --image=bench:latest \
  --namespace=sentinel-fabric \
  --restart=Never \
  --env="SENTINEL_URL=http://sentinel:8080"
```

## Service Architecture

```
┌─────────────────┐
│   Localhost     │
│   8080          │
└────────┬────────┘
         │ port-forward
         ▼
┌─────────────────┐      ┌─────────────────┐
│   Sentinel      │─────▶│  Upstream Stub  │
│   :8080         │      │  :4000          │
│  ClusterIP      │      │  ClusterIP      │
└─────────────────┘      └─────────────────┘
```

- **Sentinel**: Gateway service on port 8080
- **Upstream Stub**: Backend stub service on port 4000
- **Communication**: Sentinel → Upstream Stub (internal cluster DNS)

## Common Commands

### View All Resources

```bash
kubectl get all -n sentinel-fabric
```

### Check Logs

```bash
# Sentinel logs
kubectl logs -l app.kubernetes.io/name=sentinel -n sentinel-fabric -f

# Upstream stub logs
kubectl logs -l app.kubernetes.io/name=upstream-stub -n sentinel-fabric -f
```

### Scale Deployments

```bash
# Scale sentinel
kubectl scale deployment sentinel --replicas=3 -n sentinel-fabric

# Scale upstream-stub
kubectl scale deployment upstream-stub --replicas=2 -n sentinel-fabric
```

### Debug Pods

```bash
# Exec into sentinel pod
kubectl exec -it -l app.kubernetes.io/name=sentinel -n sentinel-fabric -- /bin/sh

# Exec into upstream-stub pod
kubectl exec -it -l app.kubernetes.io/name=upstream-stub -n sentinel-fabric -- /bin/sh
```

## Cleanup

### Remove Development Environment

```bash
# Delete dev overlay
kubectl delete -k k8s/overlays/dev/

# Delete bench job if running
kubectl delete -k k8s/bench/ 2>/dev/null || true

# Delete base (namespace + service accounts)
kubectl delete -k k8s/base/
```

### Force Cleanup

```bash
# Delete entire namespace (removes everything)
kubectl delete namespace sentinel-fabric
```

## Troubleshooting

### Pods Not Starting

```bash
# Check pod status
kubectl describe pod -l app.kubernetes.io/name=sentinel -n sentinel-fabric

# Check events
kubectl get events -n sentinel-fabric --sort-by='.lastTimestamp'
```

### Service Not Accessible

```bash
# Check service endpoints
kubectl get endpoints sentinel -n sentinel-fabric
kubectl get endpoints upstream-stub -n sentinel-fabric

# Test DNS resolution from within cluster
kubectl run -it --rm debug --image=curlimages/curl -n sentinel-fabric --restart=Never -- \
  curl http://sentinel:8080/health
```

### Port Forward Issues

```bash
# Check if port is already in use
lsof -i :8080
lsof -i :4000

# Kill existing port-forward processes
pkill -f "kubectl port-forward"
```

## Next Steps

- Add Ingress configuration for external access
- Configure Horizontal Pod Autoscaler (HPA)
- Add monitoring with Prometheus/Grafana
- Set up CI/CD pipeline for automated deployments
- Configure network policies for security

## Environment Variables

### Sentinel

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level |
| `UPSTREAM_URL` | `http://upstream-stub:4000` | Upstream service URL |

### Upstream Stub

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level |

### Bench

| Variable | Default | Description |
|----------|---------|-------------|
| `SENTINEL_URL` | `http://sentinel:8080` | Sentinel service URL |
| `RUST_LOG` | `info` | Log level |

## Resources

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Kustomize Documentation](https://kustomize.io/)
- [EKS Best Practices](https://aws.github.io/aws-eks-best-practices/)
