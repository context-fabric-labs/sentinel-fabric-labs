#!/bin/bash

# Fail fast settings
set -euo pipefail

# Print what we're about to do
echo "Creating/updating EKS Platform Cell cluster..."

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
: "${S3_BUCKET:=sentinel-fabric-cluster-state}"
: "${VPC_CIDR:=10.0.0.0/16}"
: "${PLATFORM_CELL_NODE_TYPE:=c5.large}"
: "${PLATFORM_CELL_MIN_NODES:=2}"
: "${PLATFORM_CELL_MAX_NODES:=10}"
: "${PLATFORM_CELL_DESIRED_NODES:=2}"

# Validate required variables
if [[ -z "${AWS_REGION}" ]]; then
    echo "ERROR: AWS_REGION is not set"
    exit 1
fi

if [[ -z "${CLUSTER_NAME}" ]]; then
    echo "ERROR: CLUSTER_NAME is not set"
    exit 1
fi

if [[ -z "${S3_BUCKET}" ]]; then
    echo "ERROR: S3_BUCKET is not set"
    exit 1
fi

# Create the cluster if it doesn't exist
if [[ "${DRY_RUN:-0}" == "1" ]]; then
    echo "Would create cluster: ${CLUSTER_NAME} in region ${AWS_REGION}"
    echo "Would create VPC with CIDR: ${VPC_CIDR}"
    echo "Would create platform nodegroup with ${PLATFORM_CELL_MIN_NODES}-${PLATFORM_CELL_MAX_NODES} spot nodes"
else
    echo "Creating EKS cluster ${CLUSTER_NAME} with spot instances..."
    eksctl create cluster \
        --name="${CLUSTER_NAME}" \
        --region="${AWS_REGION}" \
        --vpc-cidr="${VPC_CIDR}" \
        --zones="${AWS_REGION}a,${AWS_REGION}b,${AWS_REGION}c" \
        --nodegroup-name=platform-cell \
        --node-type="${PLATFORM_CELL_NODE_TYPE}" \
        --nodes-min="${PLATFORM_CELL_MIN_NODES}" \
        --nodes-max="${PLATFORM_CELL_MAX_NODES}" \
        --nodes="${PLATFORM_CELL_DESIRED_NODES}" \
        --node-labels="role=platform-cell" \
        --node-zones="${AWS_REGION}a,${AWS_REGION}b,${AWS_REGION}c" \
        --managed=true \
        --ssh-public-key="~/.ssh/id_rsa.pub" \
        --version=1.31 \
        --spot
fi

echo "Platform cell infrastructure creation complete!"