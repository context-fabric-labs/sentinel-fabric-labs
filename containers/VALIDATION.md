# Container Validation Guide

This guide provides commands to validate container builds locally and in Kubernetes.

## Quick Validation Checklist

- [ ] Docker images build successfully
- [ ] Containers run locally
- [ ] Health checks pass
- [ ] Images pushed to ECR
- [ ] Kubernetes pods start successfully
- [ ] Services accessible via port-forward

## 1. Build Validation

### Build All Images

```bash
# From repo root
cd /Users/shaileshpilare/Documents/sentinel-fabric

# Build sentinel
docker build -f containers/sentinel/Dockerfile -t sentinel:latest .

# Build upstream_stub
docker build -f containers/upstream_stub/Dockerfile -t upstream_stub:latest .

# Build bench
docker build -f containers/bench/Dockerfile -t bench:latest .
```

### Verify Image Details

```bash
# List images
docker images | grep -E "sentinel|upstream_stub|bench"

# Inspect sentinel
docker inspect sentinel:latest | grep -A 5 "Config"

# Check user (should be 1000)
docker inspect sentinel:latest --format='{{.Config.User}}'

# Check exposed ports
docker inspect sentinel:latest --format='{{.Config.ExposedPorts}}'
```

## 2. Local Runtime Validation

### Test Sentinel

```bash
# Run sentinel container
docker run --rm -d --name sentinel-test -p 8080:8080 sentinel:latest

# Wait for startup
sleep 3

# Test health endpoint
curl -f http://localhost:8080/health

# Test ready endpoint
curl -f http://localhost:8080/ready

# Check logs
docker logs sentinel-test

# Stop container
docker stop sentinel-test
```

### Test Upstream Stub

```bash
# Run upstream_stub container
docker run --rm -d --name upstream-stub-test -p 4000:4000 upstream_stub:latest

# Wait for startup
sleep 3

# Test health endpoint
curl -f http://localhost:4000/health

# Test ready endpoint
curl -f http://localhost:4000/ready

# Test chat endpoint (example)
curl -X POST http://localhost:4000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "test-model",
    "messages": [
      {"role": "user", "content": "Hello"}
    ]
  }'

# Check logs
docker logs upstream-stub-test

# Stop container
docker stop upstream-stub-test
```

### Test Bench

```bash
# First, start sentinel
docker run --rm -d --name sentinel-test -p 8080:8080 sentinel:latest
sleep 3

# Run bench (one-shot)
docker run --rm \
  --network host \
  -e SENTINEL_URL=http://localhost:8080 \
  bench:latest \
  --help

# Run actual benchmark
docker run --rm \
  --network host \
  -e SENTINEL_URL=http://localhost:8080 \
  bench:latest

# Stop sentinel
docker stop sentinel-test
```

## 3. Docker Compose Validation

### Test with Docker Compose

```bash
# Test sentinel
cd containers/sentinel
docker-compose up --build
# In another terminal: curl http://localhost:8080/health
docker-compose down

# Test upstream_stub
cd containers/upstream_stub
docker-compose up --build
# In another terminal: curl http://localhost:4000/health
docker-compose down

# Test bench
cd containers/bench
docker-compose run --rm bench
```

## 4. ECR Push Validation

### Push to ECR

```bash
# Get account ID
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

# Run build and push script
cd /Users/shaileshpilare/Documents/sentinel-fabric
./infra/build_push.sh

# Verify images in ECR
aws ecr list-images --repository-name sentinel-fabric/sentinel --region us-east-1
aws ecr list-images --repository-name sentinel-fabric/upstream_stub --region us-east-1
aws ecr list-images --repository-name sentinel-fabric/bench --region us-east-1
```

### Verify ECR Images

```bash
# Describe sentinel repository
aws ecr describe-repositories \
  --repository-names sentinel-fabric/sentinel \
  --region us-east-1

# Get image details
aws ecr describe-images \
  --repository-name sentinel-fabric/sentinel \
  --image-ids imageTag=latest \
  --region us-east-1
```

## 5. Kubernetes Deployment Validation

### Update Image References

```bash
# Get ECR URI
ECR_URI="${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric"

# Update kustomize images
cd k8s/overlays/dev
kustomize edit set image sentinel=${ECR_URI}/sentinel:latest
kustomize edit set image upstream_stub=${ECR_URI}/upstream_stub:latest
kustomize edit set image bench=${ECR_URI}/bench:latest
```

### Deploy to Kubernetes

```bash
# Deploy base
kubectl apply -k k8s/base/

# Deploy dev overlay
kubectl apply -k k8s/overlays/dev/

# Watch pods come up
kubectl get pods -n sentinel-fabric -w
```

### Verify Kubernetes Deployment

```bash
# Check deployments
kubectl get deployments -n sentinel-fabric

# Check pods
kubectl get pods -n sentinel-fabric

# Check services
kubectl get services -n sentinel-fabric

# Verify image used
kubectl get deployment sentinel -n sentinel-fabric -o yaml | grep image:
kubectl get deployment upstream-stub -n sentinel-fabric -o yaml | grep image:
```

### Test Kubernetes Services

```bash
# Port-forward sentinel
kubectl port-forward svc/sentinel 8080:8080 -n sentinel-fabric &

# Port-forward upstream-stub
kubectl port-forward svc/upstream-stub 4000:4000 -n sentinel-fabric &

# Test sentinel
curl http://localhost:8080/health

# Test upstream-stub
curl http://localhost:4000/health

# Kill port-forward processes
pkill -f "kubectl port-forward"
```

### Check Pod Logs

```bash
# Sentinel logs
kubectl logs -l app.kubernetes.io/name=sentinel -n sentinel-fabric

# Upstream stub logs
kubectl logs -l app.kubernetes.io/name=upstream-stub -n sentinel-fabric

# Follow logs
kubectl logs -f -l app.kubernetes.io/name=sentinel -n sentinel-fabric
```

### Describe Pods (for debugging)

```bash
# Describe sentinel pod
kubectl describe pod -l app.kubernetes.io/name=sentinel -n sentinel-fabric

# Describe upstream-stub pod
kubectl describe pod -l app.kubernetes.io/name=upstream-stub -n sentinel-fabric

# Check for image pull errors
kubectl get events -n sentinel-fabric --sort-by='.lastTimestamp'
```

## 6. Security Validation

### Verify Non-Root User

```bash
# Check sentinel user
docker run --rm --entrypoint /bin/bash sentinel:latest -c "whoami && id"
# Should output: app uid=1000(app) gid=1000(app)

# Check upstream_stub user
docker run --rm --entrypoint /bin/bash upstream_stub:latest -c "whoami && id"

# Check bench user
docker run --rm --entrypoint /bin/bash bench:latest -c "whoami && id"
```

### Verify No Root Access

```bash
# Try to install package (should fail)
docker run --rm sentinel:latest apt-get update
# Should fail with permission denied

# Try to write to system directory (should fail)
docker run --rm sentinel:latest touch /etc/test
# Should fail with permission denied
```

### Check Image Vulnerabilities

```bash
# ECR scan (if enabled)
aws ecr describe-image-scan-findings \
  --repository-name sentinel-fabric/sentinel \
  --image-id imageTag=latest \
  --region us-east-1
```

## 7. Performance Validation

### Check Image Size

```bash
# List image sizes
docker images | grep -E "sentinel|upstream_stub|bench"

# Expected sizes:
# sentinel: ~150-200 MB
# upstream_stub: ~150-200 MB
# bench: ~300-400 MB (includes AWS CLI)
```

### Check Startup Time

```bash
# Time sentinel startup
time docker run --rm sentinel:latest curl -f http://localhost:8080/health || true

# Should start within 5-10 seconds
```

## 8. Troubleshooting

### Build Issues

```bash
# Clean build cache
docker builder prune -a

# Rebuild without cache
docker build --no-cache -f containers/sentinel/Dockerfile -t sentinel:latest .
```

### Runtime Issues

```bash
# Run with debug output
docker run --rm -e RUST_LOG=debug sentinel:latest

# Run with shell access
docker run -it --rm --entrypoint /bin/bash sentinel:latest
```

### Kubernetes Issues

```bash
# Check pod status
kubectl get pods -n sentinel-fabric

# If ImagePullBackOff, check image name and credentials
kubectl describe pod <pod-name> -n sentinel-fabric

# Check image pull secrets
kubectl get secrets -n sentinel-fabric

# Create ECR pull secret if needed
kubectl create secret docker-registry ecr-secret \
  --docker-server=${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com \
  --docker-username=AWS \
  --docker-password=$(aws ecr get-login-password --region us-east-1) \
  -n sentinel-fabric
```

## 9. Cleanup

### Local Cleanup

```bash
# Stop all test containers
docker stop $(docker ps -q --filter ancestor=sentinel:latest)
docker stop $(docker ps -q --filter ancestor=upstream_stub:latest)
docker stop $(docker ps -q --filter ancestor=bench:latest)

# Remove images
docker rmi sentinel:latest upstream_stub:latest bench:latest

# Remove dangling images
docker image prune -f
```

### Kubernetes Cleanup

```bash
# Delete dev overlay
kubectl delete -k k8s/overlays/dev/

# Delete base
kubectl delete -k k8s/base/

# Or delete namespace
kubectl delete namespace sentinel-fabric
```

## Validation Checklist

Run through this checklist:

```bash
# ✅ Build
docker build -f containers/sentinel/Dockerfile -t sentinel:latest .
docker build -f containers/upstream_stub/Dockerfile -t upstream_stub:latest .
docker build -f containers/bench/Dockerfile -t bench:latest .

# ✅ Local test
docker run --rm -d -p 8080:8080 sentinel:latest && sleep 3 && curl -f http://localhost:8080/health
docker run --rm -d -p 4000:4000 upstream_stub:latest && sleep 3 && curl -f http://localhost:4000/health

# ✅ Push to ECR
./infra/build_push.sh

# ✅ Deploy to K8s
kubectl apply -k k8s/base/
kubectl apply -k k8s/overlays/dev/

# ✅ Verify
kubectl get pods -n sentinel-fabric
kubectl get services -n sentinel-fabric

# ✅ Test
kubectl port-forward svc/sentinel 8080:8080 -n sentinel-fabric &
curl http://localhost:8080/health
pkill -f "kubectl port-forward"
```

All validations complete! 🎉
