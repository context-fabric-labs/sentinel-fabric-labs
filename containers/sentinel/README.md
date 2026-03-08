# Sentinel Container - Quick Reference

## Build Locally

```bash
# From repo root
docker build -f containers/sentinel/Dockerfile -t sentinel:latest .

# With specific tag
docker build -f containers/sentinel/Dockerfile -t sentinel:$(git rev-parse --short HEAD) .
```

## Run Locally

```bash
# Basic run
docker run --rm -p 8080:8080 sentinel:latest

# With environment variables
docker run --rm -p 8080:8080 \
  -e RUST_LOG=debug \
  -e UPSTREAM_URL=http://localhost:4000 \
  sentinel:latest

# Detached mode
docker run -d --name sentinel -p 8080:8080 sentinel:latest
```

## Test Container

```bash
# Health check
curl http://localhost:8080/health

# Ready check
curl http://localhost:8080/ready
```

## Docker Compose

```bash
# Build and run
cd containers/sentinel
docker-compose up --build

# Run in background
docker-compose up -d

# View logs
docker-compose logs -f sentinel

# Stop
docker-compose down
```

## Debug Container

```bash
# Run with shell access
docker run -it --rm --entrypoint /bin/bash sentinel:latest

# Inspect image
docker inspect sentinel:latest

# Check user
docker run --rm sentinel:latest whoami
# Should output: app
```

## Security

- Runs as non-root user `app` (UID 1000)
- Minimal Debian bookworm-slim base
- Multi-stage build (no build tools in final image)
- Health checks enabled
