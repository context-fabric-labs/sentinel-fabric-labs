#!/bin/bash

# Uninstall NVIDIA GPU Operator from EKS
# Removes Helm release and gpu-operator namespace

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
: "${GPU_OPERATOR_NAMESPACE:=gpu-operator}"

# Validate prerequisites
validate_prerequisites() {
    print_info "Validating prerequisites..."
    
    if ! command -v helm &> /dev/null; then
        print_error "Helm is not installed"
        exit 1
    fi
    
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl is not installed"
        exit 1
    fi
}

# Check cluster context
check_cluster_context() {
    print_info "Checking cluster context..."
    
    CURRENT_CONTEXT=$(kubectl config current-context 2>/dev/null || echo "")
    
    if [[ -z "${CURRENT_CONTEXT}" ]]; then
        print_error "No kubectl context selected"
        exit 1
    fi
    
    print_info "Current context: ${CURRENT_CONTEXT}"
    
    # Verify cluster connectivity
    if ! kubectl cluster-info &> /dev/null; then
        print_error "Cannot connect to cluster"
        exit 1
    fi
}

# Check if GPU Operator is installed
check_installation() {
    print_info "Checking for GPU Operator installation..."
    
    if ! kubectl get namespace ${GPU_OPERATOR_NAMESPACE} &> /dev/null; then
        print_warn "Namespace ${GPU_OPERATOR_NAMESPACE} does not exist"
        print_info "Nothing to uninstall"
        exit 0
    fi
    
    if ! helm list -n ${GPU_OPERATOR_NAMESPACE} --short | grep -q "gpu-operator"; then
        print_warn "GPU Operator Helm release not found in namespace ${GPU_OPERATOR_NAMESPACE}"
        print_info "Namespace exists but Helm release not found. Manual cleanup may be required."
        
        read -p "Do you want to delete the namespace anyway? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            kubectl delete namespace ${GPU_OPERATOR_NAMESPACE}
            print_info "Namespace deleted"
            exit 0
        else
            print_info "Aborting"
            exit 1
        fi
    fi
    
    print_info "GPU Operator installation found"
}

# Uninstall Helm release
uninstall_helm_release() {
    print_info "Uninstalling GPU Operator Helm release..."
    
    helm uninstall gpu-operator -n ${GPU_OPERATOR_NAMESPACE}
    
    print_info "Helm release uninstalled"
}

# Wait for resources to be deleted
wait_for_deletion() {
    print_info "Waiting for GPU Operator resources to be deleted..."
    
    # Wait for daemonsets to be deleted
    TIMEOUT=120
    COUNTER=0
    
    while kubectl get daemonset -n ${GPU_OPERATOR_NAMESPACE} --no-headers &> /dev/null; do
        sleep 5
        COUNTER=$((COUNTER + 5))
        if [[ ${COUNTER} -ge ${TIMEOUT} ]]; then
            print_warn "Timeout waiting for daemonsets to be deleted"
            break
        fi
    done
    
    # Wait for pods to terminate
    REMAINING_PODS=$(kubectl get pods -n ${GPU_OPERATOR_NAMESPACE} --no-headers 2>/dev/null | wc -l || echo "0")
    
    if [[ "${REMAINING_PODS}" -gt 0 ]]; then
        print_warn "${REMAINING_PODS} pods still terminating..."
        sleep 10
    fi
    
    print_info "Resource cleanup completed"
}

# Delete namespace
delete_namespace() {
    print_info "Deleting namespace: ${GPU_OPERATOR_NAMESPACE}..."
    
    kubectl delete namespace ${GPU_OPERATOR_NAMESPACE} --ignore-not-found=true
    
    print_info "Namespace deleted"
}

# Clean up GPU labels from nodes (optional)
cleanup_node_labels() {
    print_info "Checking for GPU-labeled nodes..."
    
    GPU_NODES=$(kubectl get nodes -l nvidia.com/gpu.present=true --no-headers 2>/dev/null | wc -l || echo "0")
    
    if [[ "${GPU_NODES}" -gt 0 ]]; then
        print_warn "Found ${GPU_NODES} GPU-labeled node(s)"
        print_warn "GPU labels will remain on nodes. To remove them manually:"
        print_warn "  kubectl label nodes -l nvidia.com/gpu.present=true nvidia.com/gpu.present-"
        print_warn "  kubectl label nodes -l nvidia.com/gpu.present=true nvidia.com/gpu.count-"
        print_warn ""
        print_warn "Or keep them if you plan to reinstall GPU Operator later"
    fi
}

# Verify uninstallation
verify_uninstallation() {
    print_info "Verifying uninstallation..."
    
    echo ""
    print_info "Checking for remaining GPU Operator resources:"
    
    # Check namespace
    if kubectl get namespace ${GPU_OPERATOR_NAMESPACE} &> /dev/null; then
        print_warn "Namespace ${GPU_OPERATOR_NAMESPACE} still exists (may be terminating)"
    else
        print_info "✓ Namespace ${GPU_OPERATOR_NAMESPACE} deleted"
    fi
    
    # Check Helm release
    if helm list --all-namespaces --short | grep -q "gpu-operator"; then
        print_warn "GPU Operator Helm release still found"
    else
        print_info "✓ Helm release removed"
    fi
    
    # Check pods
    REMAINING_PODS=$(kubectl get pods -n ${GPU_OPERATOR_NAMESPACE} --no-headers 2>/dev/null | wc -l || echo "0")
    if [[ "${REMAINING_PODS}" -eq 0 ]]; then
        print_info "✓ All pods terminated"
    else
        print_warn "${REMAINING_PODS} pods still running in ${GPU_OPERATOR_NAMESPACE}"
    fi
    
    echo ""
    print_info "Uninstallation verification complete"
}

# Main execution
main() {
    print_info "NVIDIA GPU Operator Uninstallation Script"
    echo "=============================================="
    echo ""
    
    validate_prerequisites
    check_cluster_context
    check_installation
    
    print_warn ""
    print_warn "This will uninstall NVIDIA GPU Operator from your cluster."
    print_warn "All GPU workloads will stop functioning."
    print_warn ""
    
    read -p "Are you sure you want to proceed? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Aborting uninstallation"
        exit 0
    fi
    
    uninstall_helm_release
    wait_for_deletion
    delete_namespace
    cleanup_node_labels
    verify_uninstallation
    
    print_info ""
    print_info "✅ GPU Operator uninstallation completed!"
    print_info ""
    print_info "Note: GPU node labels remain on nodes. Remove manually if needed."
}

# Run main
main "$@"
