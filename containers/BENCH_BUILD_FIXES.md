# Bench Docker Build - Issues Fixed

## Issue 1: Curl Package Conflict

**Error:**
```
Problem: problem with installed package curl-minimal-8.17.0-1.amzn2023.0.1.aarch64
  - package curl-minimal conflicts with curl
```

**Root Cause:**
Amazon Linux 2023 comes with `curl-minimal` installed, which conflicts with the regular `curl` package.

**Solution:**
Used `--allowerasing` flag to allow yum to replace conflicting packages:

```dockerfile
yum install -y --allowerasing curl
```

## Issue 2: Missing groupadd/useradd Commands

**Error:**
```
/bin/sh: line 1: groupadd: command not found
```

**Root Cause:**
Amazon Linux 2023 minimal doesn't include `shadow-utils` package which provides `groupadd` and `useradd` commands.

**Solution:**
Added `shadow-utils` to the packages being installed:

```dockerfile
yum install -y \
    python3 \
    python3-pip \
    unzip \
    shadow-utils
```

## Issue 3: AWS CLI Architecture Mismatch

**Error:**
```
rosetta error: failed to open elf at /lib64/ld-linux-x86-64.so.2
```

**Root Cause:**
The Dockerfile was downloading the x86_64 version of AWS CLI, but the container is running on ARM64 (aarch64) architecture.

**Solution:**
Changed the AWS CLI download URL to use the ARM64 version:

```dockerfile
curl "https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip" -o "awscliv2.zip"
```

## Final Working Dockerfile

```dockerfile
# Runtime stage - based on Amazon Linux for better AWS CLI compatibility
FROM amazonlinux:2023

# Install AWS CLI v2 and runtime dependencies
# Use --allowerasing to handle curl-minimal conflicts
# Install shadow-utils for groupadd/useradd commands
RUN yum update -y && \
    yum install -y \
    python3 \
    python3-pip \
    unzip \
    shadow-utils \
    && yum install -y --allowerasing curl \
    && curl "https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip" -o "awscliv2.zip" && \
    unzip awscliv2.zip && \
    ./aws/install && \
    rm -rf aws awscliv2.zip && \
    yum clean all && \
    groupadd --gid 1000 app && \
    useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash app
```

## Verification

All checks pass:

```bash
# Build
docker build -f containers/bench/Dockerfile -t bench:latest .

# Verify user
docker run --rm --entrypoint /bin/bash bench:latest -c "whoami && id"
# Output: app uid=1000(app) gid=1000(app)

# Verify AWS CLI
docker run --rm --entrypoint /bin/bash bench:latest -c "aws --version"
# Output: aws-cli/2.34.4 Python/3.13.11 Linux/6.5.11-linuxkit exe/aarch64.amzn.2023

# Verify bench command
docker run --rm bench:latest --help
# Output: Usage: bench <COMMAND>
```

## Image Details

- **Size**: 444 MB
- **Base**: Amazon Linux 2023
- **User**: app (UID 1000, GID 1000)
- **AWS CLI**: v2.34.4 (ARM64)
- **Bench**: Latest release build

## All Images Status

✅ **sentinel**: 116 MB - Debian bookworm-slim, Rust 1.93, port 8080  
✅ **upstream_stub**: 110 MB - Debian bookworm-slim, Rust 1.93, port 4000  
✅ **bench**: 444 MB - Amazon Linux 2023, Rust 1.93, AWS CLI v2

All images:
- Run as non-root user (app:app, UID 1000)
- Multi-stage builds
- Successfully built and tested
- Ready for ECR push
