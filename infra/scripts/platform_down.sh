#!/bin/bash

# Fail fast settings
set -euo pipefail

# Print what we're about to do
echo "Scaling down Platform Cell nodegroups..."

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

# Validate required variables
if [[ -z "${AWS_REGION}" ]]; then
    echo "ERROR: AWS_REGION is not set"
    exit 1
fi

if [[ -z "${CLUSTER_NAME}" ]]; then
    echo "ERROR: CLUSTER_NAME is not set"
    exit 1
fi

# Scale down platform nodegroup
if [[ "${DRY_RUN:-0}" == "1" ]]; then
    echo "Would scale platform nodegroup to 0 nodes"
else
    echo "Scaling platform nodegroup to 0 nodes..."
    eksctl scale nodegroup \
        --name=platform-cell \
        --cluster="${CLUSTER_NAME}" \
        --region="${AWS_REGION}" \
        --nodes=0
fi

echo "Platform cell scaling down complete!"