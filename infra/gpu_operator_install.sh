#!/bin/bash

# Install NVIDIA GPU Operator on EKS using Helm
# Installs into gpu-operator namespace with DCGM exporter enabled

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration
: "${AWS_REGION:=us-east-1}"
: "${CLUSTER_NAME:=sentinel-fabric-platform-cell}"
: "${GPU_OPERATOR_NAMESPACE:=gpu-operator}"
: "${GPU_OPERATOR_VERSION:=v25.10.1}"  # Latest stable version
: "${DCGM_EXPORTER_ENABLED:=true}"

# Validate prerequisites
validate_prerequisites() {
    print_info "Validating prerequisites..."
    
    if ! command -v helm &> /dev/null; then
        print_error "Helm is not installed. Install from https://helm.sh/docs/intro/install/"
        exit 1
    fi
    
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl is not installed"
        exit 1
    fi
    
    if ! command -v aws &> /dev/null; then
        print_error "AWS CLI is not installed"
        exit 1
    fi
    
    # Validate AWS credentials
    if ! aws sts get-caller-identity &> /dev/null; then
        print_error "AWS credentials validation failed"
        exit 1
    fi
    
    print_info "Helm version: $(helm version --short)"
    print_info "kubectl version: $(kubectl version --client --short)"
}

# Check cluster context
check_cluster_context() {
    print_info "Checking cluster context..."
    
    # Get current context
    CURRENT_CONTEXT=$(kubectl config current-context 2>/dev/null || echo "")
    
    if [[ -z "${CURRENT_CONTEXT}" ]]; then
        print_error "No kubectl context selected"
        exit 1
    fi
    
    print_info "Current context: ${CURRENT_CONTEXT}"
    
    # Check if context matches our cluster
    if [[ "${CURRENT_CONTEXT}" != *"${CLUSTER_NAME}"* ]]; then
        print_warn "Current context does not match target cluster: ${CLUSTER_NAME}"
        print_warn "Switching to cluster context..."
        
        # Try to find and switch to the correct context
        TARGET_CONTEXT=$(kubectl config get-contexts --no-headers | grep "${CLUSTER_NAME}" | awk '{print $1}' | head -1)
        
        if [[ -n "${TARGET_CONTEXT}" ]]; then
            kubectl config use-context "${TARGET_CONTEXT}"
            print_info "Switched to context: ${TARGET_CONTEXT}"
        else
            print_error "Could not find context for cluster: ${CLUSTER_NAME}"
            print_error "Please manually switch to the correct context and re-run"
            exit 1
        fi
    fi
    
    # Verify cluster connectivity
    if ! kubectl cluster-info &> /dev/null; then
        print_error "Cannot connect to cluster"
        exit 1
    fi
    
    print_info "Cluster connectivity verified"
}

# Check for existing GPU nodes
check_gpu_nodes() {
    print_info "Checking for GPU nodes..."
    
    GPU_NODES=$(kubectl get nodes -l nvidia.com/gpu.present=true --no-headers 2>/dev/null | wc -l || echo "0")
    
    if [[ "${GPU_NODES}" -eq 0 ]]; then
        print_warn "No GPU-labeled nodes found in cluster"
        print_warn "GPU Operator will install drivers, but you need GPU nodes (e.g., g4dn, g5, p3, p4 instances)"
        print_warn "To create GPU nodegroup, run: ./infra/scripts/gpu_on.sh"
    else
        print_info "Found ${GPU_NODES} GPU node(s)"
    fi
}

# Check if GPU Operator is already installed
check_existing_installation() {
    print_info "Checking for existing GPU Operator installation..."
    
    if kubectl get namespace ${GPU_OPERATOR_NAMESPACE} &> /dev/null; then
        print_warn "Namespace ${GPU_OPERATOR_NAMESPACE} already exists"
        
        # Check if GPU Operator is already deployed
        if helm list -n ${GPU_OPERATOR_NAMESPACE} --short | grep -q "gpu-operator"; then
            print_warn "GPU Operator is already installed in namespace ${GPU_OPERATOR_NAMESPACE}"
            print_info "To upgrade, run: helm upgrade gpu-operator nvidia/gpu-operator -n ${GPU_OPERATOR_NAMESPACE}"
            print_info "To uninstall first, run: ./infra/gpu_operator_uninstall.sh"
            
            read -p "Do you want to proceed with upgrade? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                print_info "Aborting installation"
                exit 0
            fi
        fi
    fi
}

# Create namespace
create_namespace() {
    print_info "Creating namespace: ${GPU_OPERATOR_NAMESPACE}..."
    
    kubectl create namespace ${GPU_OPERATOR_NAMESPACE} --dry-run=client -o yaml | kubectl apply -f -
    
    print_info "Namespace created/verified"
}

# Add NVIDIA Helm repository
add_helm_repo() {
    print_info "Adding NVIDIA Helm repository..."
    
    helm repo add nvidia https://helm.ngc.nvidia.com/nvidia --force-update
    
    helm repo update
    
    print_info "Helm repository added and updated"
}

# Install GPU Operator
install_gpu_operator() {
    print_info "Installing NVIDIA GPU Operator ${GPU_OPERATOR_VERSION}..."
    
    # Build Helm values
    HELM_VALUES=(
        "-n" "${GPU_OPERATOR_NAMESPACE}"
        "--create-namespace"
        "--version" "${GPU_OPERATOR_VERSION}"
        "--set" "operator.defaultRuntime=containerd"
    )
    
    # Enable DCGM exporter
    if [[ "${DCGM_EXPORTER_ENABLED}" == "true" ]]; then
        HELM_VALUES+=(
            "--set" "dcgmExporter.enabled=true"
            "--set" "dcgmExporter.serviceMonitor.enabled=false"
        )
        print_info "DCGM exporter enabled"
    fi
    
    # Install or upgrade
    if helm list -n ${GPU_OPERATOR_NAMESPACE} 2>/dev/null | grep -q "gpu-operator"; then
        print_info "Upgrading existing installation..."
        helm upgrade gpu-operator nvidia/gpu-operator "${HELM_VALUES[@]}"
    else
        print_info "Installing new deployment..."
        helm install gpu-operator nvidia/gpu-operator "${HELM_VALUES[@]}"
    fi
    
    print_info "GPU Operator installation initiated"
}

# Wait for installation
wait_for_installation() {
    print_info "Waiting for GPU Operator components to be ready..."
    
    # Wait for daemonsets
    print_info "Waiting for nvidia-device-plugin-daemonset..."
    kubectl rollout status daemonset/nvidia-device-plugin-daemonset -n ${GPU_OPERATOR_NAMESPACE} --timeout=300s || print_warn "Device plugin rollout timed out"
    
    print_info "Waiting for nvidia-driver-daemonset..."
    kubectl rollout status daemonset/nvidia-driver-daemonset -n ${GPU_OPERATOR_NAMESPACE} --timeout=600s || print_warn "Driver daemonset rollout timed out"
    
    print_info "Waiting for nvidia-container-toolkit-daemonset..."
    kubectl rollout status daemonset/nvidia-container-toolkit-daemonset -n ${GPU_OPERATOR_NAMESPACE} --timeout=300s || print_warn "Container toolkit rollout timed out"
    
    if [[ "${DCGM_EXPORTER_ENABLED}" == "true" ]]; then
        print_info "Waiting for nvidia-dcgm-exporter..."
        kubectl rollout status daemonset/nvidia-dcgm-exporter -n ${GPU_OPERATOR_NAMESPACE} --timeout=300s || print_warn "DCGM exporter rollout timed out"
    fi
}

# Verify installation
verify_installation() {
    print_info "Verifying installation..."
    
    echo ""
    print_info "GPU Operator pods:"
    kubectl get pods -n ${GPU_OPERATOR_NAMESPACE}
    
    echo ""
    print_info "GPU Operator services:"
    kubectl get services -n ${GPU_OPERATOR_NAMESPACE}
    
    echo ""
    print_info "GPU-labeled nodes:"
    kubectl get nodes -l nvidia.com/gpu.present=true
    
    echo ""
    if [[ "${DCGM_EXPORTER_ENABLED}" == "true" ]]; then
        print_info "DCGM exporter pods:"
        kubectl get pods -n ${GPU_OPERATOR_NAMESPACE} -l app=nvidia-dcgm-exporter
    fi
    
    echo ""
    print_info "Installation complete!"
    print_info ""
    print_info "Next steps:"
    print_info "1. Verify GPU access: kubectl run nvidia-smi --image=nvidia/cuda:12.4.1-base-ubuntu22.04 --rm -it --restart=Never -- nvidia-smi"
    print_info "2. Check DCGM metrics: kubectl port-forward svc/nvidia-dcgm-exporter -n ${GPU_OPERATOR_NAMESPACE} 9400:9400"
    print_info "3. View docs/gpu_operator.md for detailed verification and Prometheus integration"
}

# Main execution
main() {
    print_info "NVIDIA GPU Operator Installation Script"
    echo "=========================================="
    echo ""
    
    validate_prerequisites
    check_cluster_context
    check_gpu_nodes
    check_existing_installation
    create_namespace
    add_helm_repo
    install_gpu_operator
    wait_for_installation
    verify_installation
    
    print_info ""
    print_info "✅ GPU Operator installation completed successfully!"
}

# Run main
main "$@"
