#!/bin/bash

# Build and push Docker images to ECR
# Creates ECR repositories if they don't exist
# Tags images with git SHA and latest

set -euo pipefail

# Configuration
: "${AWS_REGION:=us-east-1}"
: "${ECR_PREFIX:=sentinel-fabric}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print with color
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Validate prerequisites
validate_prerequisites() {
    print_info "Validating prerequisites..."
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        exit 1
    fi
    
    if ! command -v aws &> /dev/null; then
        print_error "AWS CLI is not installed"
        exit 1
    fi
    
    if ! command -v git &> /dev/null; then
        print_error "Git is not installed"
        exit 1
    fi
    
    # Validate AWS credentials
    if ! aws sts get-caller-identity &> /dev/null; then
        print_error "AWS credentials validation failed"
        exit 1
    fi
    
    print_info "All prerequisites validated"
}

# Get git short SHA
get_git_sha() {
    git rev-parse --short HEAD
}

# Get ECR account ID
get_account_id() {
    aws sts get-caller-identity --query Account --output text
}

# Create ECR repository if it doesn't exist
create_ecr_repo() {
    local repo_name=$1
    
    if aws ecr describe-repositories --repository-names "$repo_name" --region "$AWS_REGION" &> /dev/null; then
        print_info "ECR repository '$repo_name' already exists"
    else
        print_info "Creating ECR repository '$repo_name'..."
        aws ecr create-repository \
            --repository-name "$repo_name" \
            --region "$AWS_REGION" \
            --image-scanning-configuration scanOnPush=true \
            --image-tag-mutability MUTABLE \
            | jq -r '.repository.repositoryUri'
    fi
}

# Login to ECR
ecr_login() {
    local account_id=$1
    
    print_info "Logging in to ECR..."
    aws ecr get-login-password --region "$AWS_REGION" | \
        docker login --username AWS --password-stdin "${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com"
}

# Build and tag image
build_image() {
    local component=$1
    local git_sha=$2
    local account_id=$3
    local ecr_uri="${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ECR_PREFIX}/${component}"
    
    print_info "Building ${component}..."
    
    # Build with git SHA tag for linux/amd64 (x86_64) platform
    # This ensures compatibility with EKS nodes (not ARM/M1)
    docker build \
        --platform linux/amd64 \
        -f "containers/${component}/Dockerfile" \
        -t "${ecr_uri}:${git_sha}" \
        -t "${ecr_uri}:latest" \
        .
    
    print_info "Built ${component}:${git_sha} and ${component}:latest (linux/amd64)"
}

# Push image to ECR
push_image() {
    local component=$1
    local git_sha=$2
    local account_id=$3
    local ecr_uri="${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ECR_PREFIX}/${component}"
    
    print_info "Pushing ${component} to ECR..."
    
    # Push both tags
    docker push "${ecr_uri}:${git_sha}"
    docker push "${ecr_uri}:latest"
    
    print_info "Pushed ${component}:${git_sha} and ${component}:latest to ECR"
}

# Main execution
main() {
    print_info "Starting build and push to ECR..."
    
    # Validate
    validate_prerequisites
    
    # Get identifiers
    local git_sha
    git_sha=$(get_git_sha)
    print_info "Git SHA: ${git_sha}"
    
    local account_id
    account_id=$(get_account_id)
    print_info "AWS Account: ${account_id}"
    
    # Login to ECR
    ecr_login "$account_id"
    
    # Components to build
    local components=("sentinel" "upstream_stub" "bench")
    
    # Build and push each component
    for component in "${components[@]}"; do
        print_info "Processing ${component}..."
        
        # Create ECR repository
        local repo_name="${ECR_PREFIX}/${component}"
        create_ecr_repo "$repo_name"
        
        # Build image
        build_image "$component" "$git_sha" "$account_id"
        
        # Push image
        push_image "$component" "$git_sha" "$account_id"
        
        print_info "✅ ${component} complete"
        echo ""
    done
    
    print_info "========================================="
    print_info "All images built and pushed successfully!"
    print_info "========================================="
    echo ""
    print_info "Images available at:"
    for component in "${components[@]}"; do
        echo "  ${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ECR_PREFIX}/${component}:${git_sha}"
        echo "  ${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ECR_PREFIX}/${component}:latest"
    done
    echo ""
    print_info "To use in Kubernetes, update k8s/overlays/dev/kustomization.yaml"
}

# Run main
main "$@"
