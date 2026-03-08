# Bench Container - Quick Reference

## Build Locally

```bash
# From repo root
docker build -f containers/bench/Dockerfile -t bench:latest .

# With specific tag
docker build -f containers/bench/Dockerfile -t bench:$(git rev-parse --short HEAD) .
```

## Run Locally

```bash
# Basic run (will fail without sentinel)
docker run --rm bench:latest

# With custom sentinel URL
docker run --rm \
  -e SENTINEL_URL=http://localhost:8080 \
  bench:latest

# With output volume
docker run --rm \
  -v $(pwd)/output:/app/output \
  -e SENTINEL_URL=http://localhost:8080 \
  bench:latest

# Run specific benchmark
docker run --rm \
  -e SENTINEL_URL=http://localhost:8080 \
  bench:latest --help
```

## Test Container

```bash
# Check AWS CLI is available
docker run --rm bench:latest aws --version

# Check bench help
docker run --rm bench:latest --help
```

## Docker Compose

```bash
# Build and run
cd containers/bench
docker-compose up --build

# Run once
docker-compose run --rm bench

# View output
ls -la containers/bench/output/
```

## Debug Container

```bash
# Run with shell access
docker run -it --rm --entrypoint /bin/bash bench:latest

# Inspect image
docker inspect bench:latest

# Check user
docker run --rm bench:latest whoami
# Should output: app

# Test AWS CLI
docker run --rm bench:latest aws --version
```

## S3 Upload (Kubernetes)

In Kubernetes, the bench job uses IRSA (IAM Roles for Service Accounts) for S3 access:

```bash
# Run with S3 upload
docker run --rm \
  -e SENTINEL_URL=http://sentinel:8080 \
  -e AWS_REGION=us-east-1 \
  bench:latest \
  --s3-bucket my-bench-results \
  --s3-prefix benchmarks/
```

## Security

- Runs as non-root user `app` (UID 1000)
- Amazon Linux 2023 base (security updates)
- Multi-stage build (no build tools in final image)
- AWS CLI v2 included for S3 uploads
