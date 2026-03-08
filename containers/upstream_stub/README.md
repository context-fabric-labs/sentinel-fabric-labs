# Upstream Stub Container - Quick Reference

## Build Locally

```bash
# From repo root
docker build -f containers/upstream_stub/Dockerfile -t upstream_stub:latest .

# With specific tag
docker build -f containers/upstream_stub/Dockerfile -t upstream_stub:$(git rev-parse --short HEAD) .
```

## Run Locally

```bash
# Basic run
docker run --rm -p 4000:4000 upstream_stub:latest

# With environment variables
docker run --rm -p 4000:4000 \
  -e RUST_LOG=debug \
  upstream_stub:latest

# Detached mode
docker run -d --name upstream-stub -p 4000:4000 upstream_stub:latest
```

## Test Container

```bash
# Health check
curl http://localhost:4000/health

# Ready check
curl http://localhost:4000/ready

# Test endpoint (example)
curl http://localhost:4000/v1/chat/completions \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"model": "test", "messages": [{"role": "user", "content": "hello"}]}'
```

## Docker Compose

```bash
# Build and run
cd containers/upstream_stub
docker-compose up --build

# Run in background
docker-compose up -d

# View logs
docker-compose logs -f upstream-stub

# Stop
docker-compose down
```

## Debug Container

```bash
# Run with shell access
docker run -it --rm --entrypoint /bin/bash upstream_stub:latest

# Inspect image
docker inspect upstream_stub:latest

# Check user
docker run --rm upstream_stub:latest whoami
# Should output: app
```

## Security

- Runs as non-root user `app` (UID 1000)
- Minimal Debian bookworm-slim base
- Multi-stage build (no build tools in final image)
- Health checks enabled
