# Docker Build Fixes

## Issue Encountered

When building Docker images, you encountered this error:

```
error: failed to load manifest for workspace member `/app/tools/upstream_stub`
Caused by: failed to read `/app/tools/upstream_stub/Cargo.toml`
Caused by: No such file or directory (os error 2)
```

## Root Cause

The workspace `Cargo.toml` references multiple crates:
```toml
[workspace]
members = [
  "sentinel",
  "tools/upstream_stub",
  "bench",
  ...
]
```

The original Dockerfiles tried to copy only specific files, but Cargo needs the entire workspace structure.

## Solution Applied

### 1. Updated Dockerfiles to Copy Entire Workspace

**Before:**
```dockerfile
COPY Cargo.toml Cargo.lock ./
COPY sentinel/Cargo.toml ./sentinel/Cargo.toml
COPY sentinel/src ./sentinel/src
```

**After:**
```dockerfile
COPY . .
```

This ensures Cargo can resolve all workspace members.

### 2. Fixed Binary Path

Cargo builds all binaries to the workspace root `target/` directory, not in each crate's folder.

**Before:**
```dockerfile
WORKDIR /app/sentinel
RUN cargo build --release
COPY --from=builder /app/sentinel/target/release/sentinel ./
```

**After:**
```dockerfile
RUN cargo build --release --bin sentinel
COPY --from=builder /app/target/release/sentinel ./
```

### 3. Updated Rust Version

The workspace uses Rust 1.93.0, so Dockerfiles were updated:

**Before:**
```dockerfile
FROM rust:1.75-slim-bookworm as builder
```

**After:**
```dockerfile
FROM rust:1.93-slim-bookworm as builder
```

### 4. Fixed Network Binding

Sentinel was binding to `127.0.0.1:8080` instead of `0.0.0.0:8080`, making it inaccessible from outside the container.

**Added to Dockerfile:**
```dockerfile
ENTRYPOINT ["./sentinel", "--bind", "0.0.0.0:8080"]
```

### 5. Added .dockerignore

Created `.dockerignore` to speed up builds by excluding unnecessary files:
- `target/` directories
- `docs/`, `scripts/`, `k8s/`
- IDE files
- Git files

## Build Commands

All images now build successfully:

```bash
# Build sentinel
docker build -f containers/sentinel/Dockerfile -t sentinel:latest .

# Build upstream_stub
docker build -f containers/upstream_stub/Dockerfile -t upstream_stub:latest .

# Build bench
docker build -f containers/bench/Dockerfile -t bench:latest .
```

## Test Commands

```bash
# Test sentinel
docker run --rm -d --name sentinel-test -p 8080:8080 sentinel:latest
sleep 3
curl http://localhost:8080/health
docker stop sentinel-test

# Test upstream_stub
docker run --rm -d --name upstream-test -p 4000:4000 upstream_stub:latest
sleep 3
curl http://localhost:4000/health
docker stop upstream-test

# Verify non-root user
docker run --rm --entrypoint /bin/bash sentinel:latest -c "whoami && id"
# Output: app uid=1000(app) gid=1000(app)
```

## Files Modified

1. `containers/sentinel/Dockerfile` - Fixed workspace copy, binary path, Rust version, bind address
2. `containers/upstream_stub/Dockerfile` - Fixed workspace copy, binary path, Rust version
3. `containers/bench/Dockerfile` - Fixed workspace copy, binary path, Rust version
4. `.dockerignore` - Created to optimize build context

## Verification

✅ All Dockerfiles build successfully  
✅ Containers run as non-root user (app:app, UID 1000)  
✅ Sentinel binds to 0.0.0.0:8080 (accessible externally)  
✅ Upstream stub binds to 0.0.0.0:4000 (accessible externally)  
✅ Health checks work  
✅ Multi-stage builds minimize image size  

## Next Steps

Images are ready to:
1. Push to ECR: `./infra/build_push.sh`
2. Deploy to Kubernetes: `kubectl apply -k k8s/overlays/dev/`
3. Test via port-forward: `kubectl port-forward svc/sentinel 8080:8080 -n sentinel-fabric`
