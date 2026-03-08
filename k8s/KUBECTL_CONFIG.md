# kubectl Configuration Guide

## ✅ Current Status

Your `kubectl` is **already configured** and connected to your EKS cluster!

**Current Cluster:** `sentinel-fabric-platform-cell`  
**Region:** `us-east-1`  
**Account:** `545009852657`  
**Nodes:** 2 ready nodes (c5.large spot instances)

## Quick Configuration Commands

### Configure kubectl for Your EKS Cluster

```bash
# Add/update cluster configuration
aws eks update-kubeconfig --name sentinel-fabric-platform-cell --region us-east-1

# Switch to this context
kubectl config use-context arn:aws:eks:us-east-1:545009852657:cluster/sentinel-fabric-platform-cell
```

### Verify Configuration

```bash
# Check current context
kubectl config current-context

# View cluster info
kubectl cluster-info

# List nodes
kubectl get nodes

# Test connectivity
kubectl get namespace sentinel-fabric
```

## Step-by-Step Configuration

### Step 1: Install Prerequisites

```bash
# Verify AWS CLI is installed
aws --version

# Verify kubectl is installed
kubectl version --client

# Install kubectl (if needed)
brew install kubectl  # macOS
```

### Step 2: Configure AWS Credentials

```bash
# Check AWS credentials
aws sts get-caller-identity

# If not configured, run:
aws configure
# Enter:
# - AWS Access Key ID
# - AWS Secret Access Key
# - Default region name: us-east-1
# - Default output format: json
```

### Step 3: Update kubeconfig

```bash
# For sentinel-fabric-platform-cell cluster
aws eks update-kubeconfig \
  --name sentinel-fabric-platform-cell \
  --region us-east-1

# This adds the cluster to ~/.kube/config
```

### Step 4: Verify and Test

```bash
# Check context
kubectl config current-context

# Should output:
# arn:aws:eks:us-east-1:545009852657:cluster/sentinel-fabric-platform-cell

# List nodes
kubectl get nodes

# Should show your 2 platform-cell nodes
```

## Managing Multiple Clusters

### View All Contexts

```bash
kubectl config get-contexts
```

### Switch Between Clusters

```bash
# List all contexts
kubectl config get-contexts

# Switch to sentinel-fabric-platform-cell
kubectl config use-context arn:aws:eks:us-east-1:545009852657:cluster/sentinel-fabric-platform-cell

# Switch to another cluster (example)
kubectl config use-context arn:aws:eks:us-east-1:604508870856:cluster/wss-eks-105
```

### Set Default Context

```bash
# Set sentinel-fabric-platform-cell as default
kubectl config use-context arn:aws:eks:us-east-1:545009852657:cluster/sentinel-fabric-platform-cell
```

## Common kubectl Commands

### Cluster Information

```bash
# Cluster info
kubectl cluster-info

# Kubernetes version
kubectl version

# API resources
kubectl api-resources
```

### Node Management

```bash
# List all nodes
kubectl get nodes

# Detailed node info
kubectl describe nodes

# Node capacity
kubectl top nodes  # Requires metrics-server
```

### Namespace Operations

```bash
# List namespaces
kubectl get namespaces

# Create namespace
kubectl create namespace sentinel-fabric

# Delete namespace
kubectl delete namespace sentinel-fabric
```

### Pod Management

```bash
# List pods in namespace
kubectl get pods -n sentinel-fabric

# Watch pods
kubectl get pods -n sentinel-fabric -w

# Pod details
kubectl describe pod <pod-name> -n sentinel-fabric

# Pod logs
kubectl logs <pod-name> -n sentinel-fabric

# Follow logs
kubectl logs -f <pod-name> -n sentinel-fabric
```

### Service Management

```bash
# List services
kubectl get services -n sentinel-fabric

# Service details
kubectl describe service sentinel -n sentinel-fabric

# Get endpoints
kubectl get endpoints sentinel -n sentinel-fabric
```

### Deployment Management

```bash
# List deployments
kubectl get deployments -n sentinel-fabric

# Scale deployment
kubectl scale deployment sentinel --replicas=3 -n sentinel-fabric

# Rolling restart
kubectl rollout restart deployment sentinel -n sentinel-fabric

# Check rollout status
kubectl rollout status deployment sentinel -n sentinel-fabric
```

## Troubleshooting

### Connection Issues

```bash
# Check AWS credentials
aws sts get-caller-identity

# Verify cluster exists
aws eks list-clusters --region us-east-1

# Re-update kubeconfig
aws eks update-kubeconfig \
  --name sentinel-fabric-platform-cell \
  --region us-east-1 \
  --force
```

### Permission Denied

```bash
# Check IAM user/role permissions
aws eks describe-cluster --name sentinel-fabric-platform-cell --region us-east-1

# Ensure you have eks:DescribeCluster and eks:AccessKubernetesApi permissions
```

### Context Not Found

```bash
# List available contexts
kubectl config get-contexts

# If cluster context is missing, re-run:
aws eks update-kubeconfig \
  --name sentinel-fabric-platform-cell \
  --region us-east-1
```

### Certificate Issues

```bash
# Delete and regenerate kubeconfig
rm ~/.kube/config
aws eks update-kubeconfig \
  --name sentinel-fabric-platform-cell \
  --region us-east-1
```

## Advanced Configuration

### Custom kubeconfig Location

```bash
# Use custom config file
export KUBECONFIG=/path/to/custom/kubeconfig

# Or specify per command
kubectl --kubeconfig=/path/to/custom/kubeconfig get pods
```

### Merge Multiple kubeconfig Files

```bash
# Merge configs
export KUBECONFIG=~/.kube/config:~/.kube/config-secondary

# View merged contexts
kubectl config get-contexts
```

### View Raw kubeconfig

```bash
# View current config
kubectl config view

# View with secrets
kubectl config view --raw

# View minified (current context only)
kubectl config view --minify
```

### Modify kubeconfig

```bash
# Set cluster entry
kubectl config set-cluster my-cluster \
  --server=https://my-cluster.example.com \
  --certificate-authority=/path/to/ca.crt

# Set credentials
kubectl config set-credentials my-user \
  --client-certificate=/path/to/client.crt \
  --client-key=/path/to/client.key

# Set context
kubectl config set-context my-context \
  --cluster=my-cluster \
  --user=my-user \
  --namespace=my-namespace
```

## Security Best Practices

### 1. Use IAM Roles (Recommended)

Instead of static credentials, use IAM roles:

```bash
# Configure AWS to use IAM role
export AWS_PROFILE=your-profile

# Or use instance profile on EC2
```

### 2. Limit kubeconfig Access

```bash
# Restrict permissions on kubeconfig
chmod 600 ~/.kube/config
```

### 3. Use Short-Lived Tokens

AWS EKS uses temporary credentials via `aws eks get-token`, which expire after 15 minutes. kubectl automatically refreshes them.

### 4. Namespace Isolation

```bash
# Set default namespace for context
kubectl config set-context \
  arn:aws:eks:us-east-1:545009852657:cluster/sentinel-fabric-platform-cell \
  --namespace=sentinel-fabric

# Verify
kubectl config view --minify | grep namespace
```

## Integration with IDE/Editor

### VS Code Kubernetes Extension

1. Install "Kubernetes" extension by Microsoft
2. It automatically detects `~/.kube/config`
3. Browse clusters, deploy, view logs from sidebar

### IntelliJ IDEA

1. Go to **Settings** → **Kubernetes**
2. Click **+** to add cluster
3. Select kubeconfig file: `~/.kube/config`
4. Choose context: `sentinel-fabric-platform-cell`

## Quick Reference Card

```bash
# Configure
aws eks update-kubeconfig --name sentinel-fabric-platform-cell --region us-east-1

# Verify
kubectl cluster-info
kubectl get nodes

# Deploy
kubectl apply -k k8s/base/
kubectl apply -k k8s/overlays/dev/

# Monitor
kubectl get all -n sentinel-fabric
kubectl logs -f -l app.kubernetes.io/name=sentinel -n sentinel-fabric

# Cleanup
kubectl delete -k k8s/overlays/dev/
kubectl delete -k k8s/base/
```

## Useful Aliases

Add these to your `~/.zshrc` or `~/.bashrc`:

```bash
# kubectl aliases
alias k='kubectl'
alias kg='kubectl get'
alias kd='kubectl describe'
alias ka='kubectl apply'
alias kdel='kubectl delete'
alias kl='kubectl logs'
alias kef='kubectl exec -it'

# Namespace-specific aliases
alias kgs='kubectl get services -n sentinel-fabric'
alias kgp='kubectl get pods -n sentinel-fabric'
alias kgd='kubectl get deployments -n sentinel-fabric'
alias kgn='kubectl get nodes'

# Context switching
alias kctx-sentinel='kubectl config use-context arn:aws:eks:us-east-1:545009852657:cluster/sentinel-fabric-platform-cell'
```

## Resources

- [kubectl Documentation](https://kubernetes.io/docs/reference/kubectl/)
- [EKS kubectl Configuration](https://docs.aws.amazon.com/eks/latest/userguide/create-kubeconfig.html)
- [Kubernetes Cheat Sheet](https://kubernetes.io/docs/reference/kubectl/cheatsheet/)
