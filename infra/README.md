# AWS Platform Cell + Burst Cells Infrastructure

This infrastructure setup creates a scalable Kubernetes cluster with different node groups for different workloads:

## What is Created

1. **EKS Platform Cell** - Core platform with CPU spot instances for general workloads
2. **GPU Burst Cells** - Dedicated GPU node groups for GPU-intensive workloads
3. **EFA Burst Cells** - Dedicated EFA node groups for high-performance networking

## Cost Considerations

- **Platform Cell**: Uses spot instances for cost savings (typically 70-90% cheaper than on-demand)
- **GPU Cells**: On-demand instances for consistent GPU availability
- **EFA Cells**: On-demand instances for high-performance networking
- **S3 Bucket**: For storing cluster state and logs (minimal cost)

## Teardown Safety

All teardown operations are designed to be safe:
- `platform_down.sh`: Scales nodegroups to 0 but keeps the cluster
- `teardown_all.sh`: Requires explicit confirmation before destroying everything
- All scripts include validation checks to prevent accidental operations

## Required Environment Variables

See `env.example` for required variables.