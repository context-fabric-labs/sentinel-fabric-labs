#!/bin/bash

# Fail fast settings
set -euo pipefail

# Print what we're about to do
echo "Scaling up GPU nodegroup..."

# Check if DRY_RUN is enabled
if [[ "${DRY_RUN:-0}" == "1" ]]; then
    echo "DRY_RUN enabled - only printing commands"
fi

# Validate prerequisites
echo "Validating prerequisites..."
if ! command -v aws &> /dev/null; then
    echo "ERROR: aws command not found"
    exit 1
fi

if ! command -v eksctl &> /dev/null; then
    echo "ERROR: eksctl command not found"
    exit 1
fi

if ! command -v kubectl &> /dev/null; then
    echo "ERROR: kubectl command not found"
    exit 1
fi

# Validate AWS credentials
echo "Validating AWS credentials..."
if ! aws sts get-caller-identity &> /dev/null; then
    echo "ERROR: AWS credentials validation failed"
    exit 1
fi

# Validate eksctl version
echo "Checking eksctl version..."
eksctl version

# Validate kubectl version
echo "Checking kubectl version..."
kubectl version --client

# Source environment variables
if [[ -f .env ]]; then
    source .env
fi

# Set defaults if not set
: "${AWS_REGION:=us-east-1}"
: "${CLUSTER_NAME:=sentinel-fabric-platform-cell}"
# Use g4dn.xlarge with launch template (tested and working in us-east-1)
: "${GPU_CELL_NODE_TYPE:=g4dn.xlarge}"
: "${GPU_CELL_MIN_NODES:=0}"
: "${GPU_CELL_MAX_NODES:=5}"
: "${GPU_CELL_DESIRED_NODES:=1}"
: "${GPU_LAUNCH_TEMPLATE:=lt-0e62b524e76a3ecfe}"

# Validate required variables
if [[ -z "${AWS_REGION}" ]]; then
    echo "ERROR: AWS_REGION is not set"
    exit 1
fi

if [[ -z "${CLUSTER_NAME}" ]]; then
    echo "ERROR: CLUSTER_NAME is not set"
    exit 1
fi

# Create or scale GPU nodegroup
if [[ "${DRY_RUN:-0}" == "1" ]]; then
    echo "Would create/enable GPU nodegroup with ${GPU_CELL_MIN_NODES}-${GPU_CELL_MAX_NODES} nodes"
    echo "Using launch template: ${GPU_LAUNCH_TEMPLATE}"
else
    echo "Creating/enabling GPU nodegroup..."
    eksctl create nodegroup \
        --name=gpu-cell \
        --cluster="${CLUSTER_NAME}" \
        --region="${AWS_REGION}" \
        --node-type="${GPU_CELL_NODE_TYPE}" \
        --nodes-min="${GPU_CELL_MIN_NODES}" \
        --nodes-max="${GPU_CELL_MAX_NODES}" \
        --nodes="${GPU_CELL_DESIRED_NODES}" \
        --node-labels="role=gpu-cell,accelerator=nvidia" \
        --node-zones="${AWS_REGION}a,${AWS_REGION}b,${AWS_REGION}c" \
        --managed=true \
        --spot
fi

echo "GPU cell scaling up complete!"