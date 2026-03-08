#!/bin/bash

# Fail fast settings
set -euo pipefail

# Print what we're about to do
echo "Destroying all infrastructure..."

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

# Confirm destruction
echo "WARNING: This will completely destroy the cluster and all associated resources!"
echo "Cluster name: ${CLUSTER_NAME}"
read -p "Type the cluster name to confirm destruction: " confirm

if [[ "${confirm}" != "${CLUSTER_NAME}" ]]; then
    echo "Confirmation failed. Aborting destruction."
    exit 1
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

# Validate AWS credentials
echo "Validating AWS credentials..."
if ! aws sts get-caller-identity &> /dev/null; then
    echo "ERROR: AWS credentials validation failed"
    exit 1
fi

# Validate eksctl version
echo "Checking eksctl version..."
eksctl version

# Destroy the cluster
echo "Destroying EKS cluster ${CLUSTER_NAME}..."
eksctl delete cluster \
    --name="${CLUSTER_NAME}" \
    --region="${AWS_REGION}"

echo "All infrastructure destruction complete!"