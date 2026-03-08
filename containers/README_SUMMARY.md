# Containerization Summary

## ✅ What Was Created

### Dockerfiles (Multi-stage, Non-root)

1. **`containers/sentinel/Dockerfile`**
   - Multi-stage build (Rust builder → Debian runtime)
   - Runs as non-root user `app` (UID 1000)
   - Exposes port 8080
   - Health checks included
   - Final size: ~150-200 MB

2. **`containers/upstream_stub/Dockerfile`**
   - Multi-stage build (Rust builder → Debian runtime)
   - Runs as non-root user `app` (UID 1000)
   - Exposes port 4000
   - Health checks included
   - Final size: ~150-200 MB

3. **`containers/bench/Dockerfile`**
   - Multi-stage build (Rust builder → Amazon Linux 2023)
   - Includes AWS CLI v2 for S3 uploads
   - Runs as non-root user `app` (UID 1000)
   - Volume mount for output files
   - Final size: ~300-400 MB

### Supporting Files

**Per Component:**
- `docker-compose.yaml` - Local testing configuration
- `README.md` - Quick reference for build/run commands

**Infrastructure:**
- `infra/build_push.sh` - Automated ECR build and push script
- `infra/BUILD_PUSH_GUIDE.md` - Comprehensive build documentation
- `containers/VALIDATION.md` - Validation and testing guide

### Kubernetes Integration

**Updated:**
- `k8s/overlays/dev/kustomization.yaml` - Added `images:` section for easy image override

## 📁 File Structure

```
containers/
├── sentinel/
│   ├── Dockerfile           # Multi-stage, non-root
│   ├── docker-compose.yaml  # Local testing
│   └── README.md            # Quick reference
├── upstream_stub/
│   ├── Dockerfile           # Multi-stage, non-root
│   ├── docker-compose.yaml  # Local testing
│   └── README.md            # Quick reference
├── bench/
│   ├── Dockerfile           # Multi-stage + AWS CLI
│   ├── docker-compose.yaml  # Local testing
│   └── README.md            # Quick reference
└── VALIDATION.md            # Comprehensive validation guide

infra/
├── build_push.sh            # ECR build/push automation
└── BUILD_PUSH_GUIDE.md      # Build documentation

k8s/overlays/dev/
└── kustomization.yaml       # Updated with images: section
```

## 🚀 Quick Start Commands

### 1. Build Images Locally

```bash
cd /Users/shaileshpilare/Documents/sentinel-fabric

# Build all
docker build -f containers/sentinel/Dockerfile -t sentinel:latest .
docker build -f containers/upstream_stub/Dockerfile -t upstream_stub:latest .
docker build -f containers/bench/Dockerfile -t bench:latest .
```

### 2. Test Locally

```bash
# Test sentinel
docker run --rm -d -p 8080:8080 sentinel:latest
curl http://localhost:8080/health
docker stop sentinel

# Test upstream_stub
docker run --rm -d -p 4000:4000 upstream_stub:latest
curl http://localhost:4000/health
docker stop upstream-stub
```

### 3. Build and Push to ECR

```bash
# Automated (creates repos, builds, tags, pushes)
./infra/build_push.sh

# Manual
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com

docker tag sentinel:latest ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:latest
docker push ${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:latest
# (repeat for upstream_stub and bench)
```

### 4. Update Kubernetes Manifests

```bash
cd k8s/overlays/dev

# Set ECR images
kustomize edit set image sentinel=${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:latest
kustomize edit set image upstream_stub=${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/upstream_stub:latest
kustomize edit set image bench=${ACCOUNT_ID}.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/bench:latest
```

### 5. Deploy to Kubernetes

```bash
# Deploy
kubectl apply -k k8s/base/
kubectl apply -k k8s/overlays/dev/

# Verify
kubectl get pods -n sentinel-fabric
kubectl get services -n sentinel-fabric

# Test
kubectl port-forward svc/sentinel 8080:8080 -n sentinel-fabric
curl http://localhost:8080/health
```

## 🏷️ Image Tagging Strategy

Images are tagged with:

1. **Git SHA** (e.g., `a3f5c2b`) - Unique per commit
2. **latest** - Most recent build

Example ECR URIs:
```
<ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:a3f5c2b
<ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/sentinel-fabric/sentinel:latest
```

## 🔒 Security Features

All images include:

- ✅ **Non-root user**: Runs as `app` (UID 1000, GID 1000)
- ✅ **Multi-stage builds**: No build tools in production
- ✅ **Minimal base images**: Debian bookworm-slim / Amazon Linux 2023
- ✅ **Health checks**: Docker HEALTHCHECK instructions
- ✅ **Image scanning**: ECR scan on push enabled
- ✅ **No secrets**: No hardcoded credentials

## 📊 Image Details

| Component | Base Image | Port | User | Size | Features |
|-----------|-----------|------|------|------|----------|
| sentinel | Debian bookworm-slim | 8080 | app:app (1000) | ~150-200 MB | Health checks |
| upstream_stub | Debian bookworm-slim | 4000 | app:app (1000) | ~150-200 MB | Health checks |
| bench | Amazon Linux 2023 | N/A | app:app (1000) | ~300-400 MB | AWS CLI v2 |

## 🛠️ Build Script Features

`infra/build_push.sh` provides:

- ✅ Prerequisites validation (Docker, AWS CLI, Git)
- ✅ Automatic ECR repository creation
- ✅ ECR login automation
- ✅ Multi-tag builds (git SHA + latest)
- ✅ Parallel-friendly (can build components independently)
- ✅ Color-coded output
- ✅ Fail-fast on errors
- ✅ DRY_RUN support (print commands only)

Usage:
```bash
# Standard
./infra/build_push.sh

# Dry run (print commands only)
DRY_RUN=1 ./infra/build_push.sh

# Custom region
AWS_REGION=us-east-1 ./infra/build_push.sh

# Custom prefix
ECR_PREFIX=my-app ./infra/build_push.sh
```

## 🧪 Validation Commands

### Quick Validation

```bash
# Build
docker build -f containers/sentinel/Dockerfile -t sentinel:latest .

# Run
docker run --rm -p 8080:8080 sentinel:latest

# Test
curl http://localhost:8080/health

# Verify non-root
docker run --rm sentinel:latest whoami
# Output: app
```

### Full Validation

See [`containers/VALIDATION.md`](containers/VALIDATION.md) for comprehensive validation checklist.

## 📖 Documentation

- **Build Guide**: [`infra/BUILD_PUSH_GUIDE.md`](infra/BUILD_PUSH_GUIDE.md)
- **Validation**: [`containers/VALIDATION.md`](containers/VALIDATION.md)
- **Component READMEs**:
  - [`containers/sentinel/README.md`](containers/sentinel/README.md)
  - [`containers/upstream_stub/README.md`](containers/upstream_stub/README.md)
  - [`containers/bench/README.md`](containers/bench/README.md)

## 🎯 Next Steps

1. ✅ Build images: `./infra/build_push.sh`
2. ✅ Update K8s manifests with ECR URIs
3. ✅ Deploy to EKS: `kubectl apply -k k8s/overlays/dev/`
4. ✅ Validate: `kubectl get pods -n sentinel-fabric`
5. ✅ Test: Port-forward and curl health endpoints

## 📝 Environment Variables

### Build Script

| Variable | Default | Description |
|----------|---------|-------------|
| `AWS_REGION` | `us-east-1` | AWS region |
| `ECR_PREFIX` | `sentinel-fabric` | ECR repo prefix |
| `DRY_RUN` | `0` | Print commands only |

### Runtime (Sentinel)

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level |
| `UPSTREAM_URL` | `http://upstream-stub:4000` | Backend URL |

### Runtime (Upstream Stub)

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level |

### Runtime (Bench)

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level |
| `SENTINEL_URL` | `http://sentinel:8080` | Sentinel URL |
| `S3_BUCKET` | optional | S3 bucket for results |
| `S3_PREFIX` | `sentinel-fabric/bench/` | S3 prefix |

## ✅ Checklist

- [x] Dockerfiles created (multi-stage, non-root)
- [x] Docker Compose files for local testing
- [x] README files with quick reference
- [x] Build and push script (`infra/build_push.sh`)
- [x] Build guide documentation
- [x] Validation guide
- [x] Kubernetes manifests updated with `images:` section
- [x] Git SHA tagging implemented
- [x] ECR repository auto-creation
- [x] Security best practices (non-root, minimal images)

All containerization requirements complete! 🎉
