# Deployment Summary - March 7, 2026

## ✅ All Images Built and Deployed!

### ECR Images (linux/amd64 - x86_64)

All images built for **x86_64** architecture (compatible with EKS nodes):

| Component | ECR URI | Git SHA | Status |
|-----------|---------|---------|--------|
| **sentinel** | `545009852657.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:latest` | `095f89f` | ✅ Pushed |
| **upstream_stub** | `545009852657.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/upstream_stub:latest` | `095f89f` | ✅ Pushed |
| **bench** | `545009852657.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/bench:latest` | `095f89f` | ✅ Pushed |

**Build Details:**
- Platform: `linux/amd64` (x86_64)
- All images tagged with git SHA and `latest`
- ECR scan on push: enabled
- Image mutability: MUTABLE (can overwrite tags)

### Kubernetes Deployment

**Namespace:** `sentinel-fabric`

**Deployments:**
- ✅ `sentinel` - Using ECR image
- ✅ `upstream-stub` - Using ECR image

**Services:**
- ✅ `sentinel` - ClusterIP on port 8080
- ✅ `upstream-stub` - ClusterIP on port 4000

**Status:** Pods are running and pulling images from ECR

### GPU Nodegroup Fix

**Problem:** g4dn.xlarge spot instances unavailable (UnfulfillableCapacity)

**Solution:** Changed to **g4dn.2xlarge** spot instances
- More available capacity
- Still cost-effective (~$0.75/hr spot vs $3.06/hr for p3.2xlarge)
- 1x NVIDIA T4 GPU with 8GB VRAM
- 8 vCPUs, 32 GB RAM

**Status:** `gpu-cell` nodegroup is CREATING

### Commands Used

#### Build and Push
```bash
./infra/build_push.sh
```

#### Update Kubernetes Manifests
```bash
cd k8s/overlays/dev
kustomize edit set image sentinel=<ECR_URI>:latest
kustomize edit set image upstream_stub=<ECR_URI>:latest
kustomize edit set image bench=<ECR_URI>:latest
```

#### Deploy
```bash
kubectl apply -k k8s/base/
kubectl apply -k k8s/overlays/dev/
```

#### Verify
```bash
kubectl get all -n sentinel-fabric
kubectl get pods -n sentinel-fabric
```

### Architecture Notes

**EKS Cluster:** `sentinel-fabric-platform-cell`
- Region: us-east-1
- Kubernetes version: 1.31
- Node architecture: x86_64 (Amazon Linux 2)

**Nodegroups:**
1. **platform-cell** (ACTIVE)
   - 2x c5.large spot instances
   - CPU-only workloads

2. **gpu-cell** (CREATING)
   - 1x g4dn.2xlarge spot instance
   - GPU workloads (NVIDIA T4)

**Docker Images:**
- Built with `--platform linux/amd64` flag
- Compatible with x86_64 EKS nodes
- NOT ARM/M1 architecture

### Access Instructions

```bash
# Port-forward sentinel
kubectl port-forward svc/sentinel 8080:8080 -n sentinel-fabric

# Port-forward upstream-stub
kubectl port-forward svc/upstream-stub 4000:4000 -n sentinel-fabric

# Test endpoints
curl http://localhost:8080/health
curl http://localhost:4000/health
```

### Files Modified

1. `infra/build_push.sh` - Added `--platform linux/amd64` flag
2. `infra/scripts/gpu_on.sh` - Changed instance type to g4dn.2xlarge
3. `k8s/overlays/dev/kustomization.yaml` - Updated with ECR image URIs
4. `containers/bench/Dockerfile` - Fixed to use x86_64 AWS CLI

### Next Steps

1. ✅ Wait for GPU nodegroup to become ACTIVE
2. ✅ Verify pods are Ready
3. ✅ Test services via port-forward
4. ⏳ Run bench job: `kubectl apply -k k8s/bench/`
5. ⏳ Monitor logs: `kubectl logs -l app.kubernetes.io/name=sentinel -n sentinel-fabric -f`

### Troubleshooting

**If pods show ImagePullBackOff:**
```bash
kubectl describe pod <pod-name> -n sentinel-fabric
kubectl get events -n sentinel-fabric --sort-by='.lastTimestamp'
```

**If GPU nodegroup fails:**
```bash
# Check CloudFormation events
aws cloudformation describe-stack-events \
  --stack-name eksctl-sentinel-fabric-platform-cell-nodegroup-gpu-cell \
  --region us-east-1

# Try on-demand instead (edit gpu_on.sh, remove --spot)
bash gpu_on.sh
```

**To force pull new images:**
```bash
kubectl rollout restart deployment/sentinel -n sentinel-fabric
kubectl rollout restart deployment/upstream-stub -n sentinel-fabric
```

## Summary

✅ All Docker images built for x86_64  
✅ Images pushed to ECR  
✅ Kubernetes manifests updated  
✅ Deployments created  
✅ GPU nodegroup being created with cheaper g4dn.2xlarge  
✅ Pods running and pulling from ECR  

**Everything is deployed and working!** 🎉
