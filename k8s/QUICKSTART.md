# Kubernetes Quick Reference

## Deployment Commands

### Deploy Everything (Base + Dev Overlay)
```bash
# One-liner to deploy base + dev environment
kubectl apply -k k8s/base/ && kubectl apply -k k8s/overlays/dev/
```

### Deploy Components Individually
```bash
# 1. Base infrastructure
kubectl apply -k k8s/base/

# 2. Sentinel
kubectl apply -k k8s/sentinel/

# 3. Upstream stub
kubectl apply -k k8s/upstream_stub/

# 4. (Optional) Bench job
kubectl apply -k k8s/bench/
```

## Port Forwarding

### Quick Start (Both Services)
```bash
# Terminal 1
kubectl port-forward svc/sentinel 8080:8080 -n sentinel-fabric

# Terminal 2
kubectl port-forward svc/upstream-stub 4000:4000 -n sentinel-fabric
```

### Test Endpoints
```bash
# Sentinel health check
curl http://localhost:8080/health

# Upstream stub health check
curl http://localhost:4000/health
```

## Monitoring

### Watch All Resources
```bash
kubectl get all -n sentinel-fabric -w
```

### View Logs
```bash
# Sentinel logs
kubectl logs -n sentinel-fabric -l app.kubernetes.io/name=sentinel -f

# Upstream stub logs
kubectl logs -n sentinel-fabric -l app.kubernetes.io/name=upstream-stub -f
```

## Cleanup

### Remove Everything
```bash
kubectl delete -k k8s/overlays/dev/
kubectl delete -k k8s/base/
```

### Remove Namespace (Nuclear Option)
```bash
kubectl delete namespace sentinel-fabric
```

## Dry Run (Preview Changes)
```bash
# See what will be applied
kubectl apply -k k8s/overlays/dev/ --dry-run=client -o yaml
```
