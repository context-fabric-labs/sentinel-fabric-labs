# Container Build and Push Guide

## Overview

This guide covers building, tagging, and pushing Docker images to Amazon ECR for the Sentinel Fabric platform.

## Prerequisites

- Docker installed and running
- AWS CLI v2 configured
- Git installed
- ECR permissions (ecr:CreateRepository, ecr:BatchCheckLayerAvailability, ecr:PutImage, ecr:InitiateLayerUpload, ecr:UploadLayerPart, ecr:CompleteLayerUpload)

## Quick Start

### Build and Push All Images

```bash
# From repo root
./infra/build_push.sh

# With custom region
AWS_REGION=us-east-1 ./infra/build_push.sh

# With custom ECR prefix
ECR_PREFIX=my-prefix ./infra/build_push.sh
```

### What It Does

1. ✅ Validates prerequisites (Docker, AWS CLI, Git)
2. ✅ Gets git short SHA for tagging
3. ✅ Logs in to ECR
4. ✅ Creates ECR repositories if missing
5. ✅ Builds images with multi-stage Dockerfiles
6. ✅ Tags with git SHA and `latest`
7. ✅ Pushes to ECR

## Manual Build Commands

### Build Individual Images

```bash
# Sentinel
docker build -f containers/sentinel/Dockerfile -t sentinel:latest .

# Upstream Stub
docker build -f containers/upstream_stub/Dockerfile -t upstream_stub:latest .

# Bench
docker build -f containers/bench/Dockerfile -t bench:latest .
```

### Tag for ECR

```bash
# Get your account ID
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

# Tag sentinel
docker tag sentinel:latest ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:latest

# Tag upstream_stub
docker tag upstream_stub:latest ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/upstream_stub:latest

# Tag bench
docker tag bench:latest ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/bench:latest
```

### Push to ECR

```bash
# Login to ECR
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com

# Push images
docker push ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:latest
docker push ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/upstream_stub:latest
docker push ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/bench:latest
```

## Create ECR Repositories

```bash
# Create all repositories
aws ecr create-repository --repository-name sentinel-fabric/sentinel --region us-east-1
aws ecr create-repository --repository-name sentinel-fabric/upstream_stub --region us-east-1
aws ecr create-repository --repository-name sentinel-fabric/bench --region us-east-1
```

## Image Tagging Strategy

Images are tagged with:

1. **Git SHA** (e.g., `a3f5c2b`) - Unique identifier for each commit
2. **latest** - Points to most recent build

Example:
```
sentinel-fabric/sentinel:a3f5c2b
sentinel-fabric/sentinel:latest
```

## Update Kubernetes Manifests

After pushing images, update your Kubernetes manifests:

### Option 1: Update kustomization.yaml

Edit `k8s/overlays/dev/kustomization.yaml`:

```yaml
images:
  - name: sentinel
    newName: <ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel
    newTag: latest
  - name: upstream_stub
    newName: <ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/upstream_stub
    newTag: latest
  - name: bench
    newName: <ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/bench
    newTag: latest
```

### Option 2: Use kustomize edit

```bash
cd k8s/overlays/dev

# Set sentinel image
kustomize edit set image sentinel=<ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:latest

# Set upstream_stub image
kustomize edit set image upstream_stub=<ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/upstream_stub:latest

# Set bench image
kustomize edit set image bench=<ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/bench:latest
```

### Option 3: Apply with kustomize

```bash
# Build and apply with image overrides
kubectl apply -k k8s/overlays/dev/ \
  --image sentinel=<ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:latest \
  --image upstream_stub=<ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/upstream_stub:latest \
  --image bench=<ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/bench:latest
```

## Local Testing

### Test Images Locally

```bash
# Run sentinel
docker run --rm -p 8080:8080 sentinel:latest

# Test health endpoint
curl http://localhost:8080/health

# Run upstream_stub
docker run --rm -p 4000:4000 upstream_stub:latest

# Test health endpoint
curl http://localhost:4000/health

# Run bench (requires sentinel running)
docker run --rm \
  -e SENTINEL_URL=http://host.docker.internal:8080 \
  bench:latest
```

### Docker Compose

```bash
# Test sentinel
cd containers/sentinel
docker-compose up

# Test upstream_stub
cd containers/upstream_stub
docker-compose up

# Test bench
cd containers/bench
docker-compose run --rm bench
```

## Security Features

All images include:

- ✅ **Non-root user**: Runs as `app` user (UID 1000)
- ✅ **Multi-stage builds**: No build tools in production images
- ✅ **Minimal base images**: Debian bookworm-slim or Amazon Linux 2023
- ✅ **Health checks**: Built-in Docker HEALTHCHECK
- ✅ **Image scanning**: ECR scan on push enabled

## Troubleshooting

### Docker Build Fails

```bash
# Clean build cache
docker builder prune -a

# Rebuild
docker build --no-cache -f containers/sentinel/Dockerfile -t sentinel:latest .
```

### ECR Push Fails

```bash
# Check ECR login
aws ecr get-login-password --region us-west-2 | \
  docker login --username AWS --password-stdin ${ACCOUNT_ID}.dkr.ecr.us-west-2.amazonaws.com

# Check repository exists
aws ecr describe-repositories --repository-names sentinel-fabric/sentinel --region us-west-2

# Check permissions
aws ecr get-repository-policy --repository-name sentinel-fabric/sentinel --region us-west-2
```

### Image Not Found in Kubernetes

```bash
# Check image pull policy
kubectl get deployment sentinel -n sentinel-fabric -o yaml | grep imagePullPolicy

# Should be: IfNotPresent or Always

# Check image name
kubectl get deployment sentinel -n sentinel-fabric -o yaml | grep image:

# Check pod events
kubectl describe pod -l app.kubernetes.io/name=sentinel -n sentinel-fabric
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build and Push

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v2
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-west-2
      
      - name: Login to Amazon ECR
        id: login-ecr
        uses: aws-actions/amazon-ecr-login@v1
      
      - name: Build and push
        env:
          ECR_REGISTRY: ${{ steps.login-ecr.outputs.registry }}
          ECR_REPOSITORY: sentinel-fabric
          IMAGE_TAG: ${{ github.sha }}
        run: |
          ./infra/build_push.sh
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `AWS_REGION` | `us-west-2` | AWS region for ECR |
| `ECR_PREFIX` | `sentinel-fabric` | ECR repository prefix |

## Cost Optimization

- Enable **ECR lifecycle policies** to clean up old images
- Use **immutable tags** for production (git SHA)
- Enable **scan on push** for security
- Consider **ECR pull-through cache** for base images

## Next Steps

After building and pushing images:

1. ✅ Update Kubernetes manifests with ECR image URIs
2. ✅ Deploy to EKS: `kubectl apply -k k8s/overlays/dev/`
3. ✅ Verify pods are running: `kubectl get pods -n sentinel-fabric`
4. ✅ Test services: `kubectl port-forward svc/sentinel 8080:8080 -n sentinel-fabric`

## Resources

- [ECR Documentation](https://docs.aws.amazon.com/AmazonECR/latest/userguide/)
- [Docker Best Practices](https://docs.docker.com/develop/develop-images/dockerfile_best-practices/)
- [Kubernetes Image Pulling](https://kubernetes.io/docs/concepts/containers/images/)
