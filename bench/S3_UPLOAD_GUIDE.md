# Bench S3 Upload Guide

This guide covers uploading benchmark results to S3 using the bench tool.

## Features

- ✅ Optional S3 upload after benchmark runs
- ✅ AWS CLI v2 for reliable uploads in Kubernetes
- ✅ Configurable S3 bucket and prefix
- ✅ IRSA (IAM Roles for Service Accounts) support
- ✅ Automatic timestamped result filenames

## CLI Arguments

### New S3-related Flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--s3-bucket` | optional | - | S3 bucket name for results upload |
| `--s3-prefix` | optional | `sentinel-fabric/bench/` | S3 key prefix for results |
| `--upload-with-aws-cli` | optional | `true` | Use AWS CLI v2 for upload (recommended) |

### All Run Flags

```bash
./bench run \
  --scenario <name> \
  --url <url> \
  --concurrency <n> \
  --requests <n> \
  --timeout-s <n> \
  --warmup-requests <n> \
  --body-file <path> \
  --body-json <json> \
  --out <path> \
  --s3-bucket <bucket> \
  --s3-prefix <prefix> \
  --upload-with-aws-cli <true|false>
```

## Usage Examples

### 1. Local Run Without S3

```bash
# Basic health check benchmark
cargo run -p bench -- run \
  --scenario health \
  --url http://127.0.0.1:8080/health \
  --concurrency 32 \
  --requests 2000

# Chat completion benchmark with custom body
cargo run -p bench -- run \
  --scenario chat \
  --url http://127.0.0.1:8080/v1/chat/completions \
  --concurrency 16 \
  --requests 500 \
  --body-json '{"model":"stub","messages":[{"role":"user","content":"hello"}],"max_tokens":16}'

# Save results to custom path (no S3)
cargo run -p bench -- run \
  --scenario health \
  --url http://127.0.0.1:8080/health \
  --out ./my-results/health-test.json
```

### 2. Local Run With S3 Upload

**Prerequisites:**
- AWS CLI v2 installed and configured
- AWS credentials configured (`aws configure` or environment variables)
- S3 bucket exists with write permissions

```bash
# Upload to S3 with default prefix
cargo run -p bench -- run \
  --scenario health \
  --url http://127.0.0.1:8080/health \
  --s3-bucket my-bench-results

# Upload to S3 with custom prefix
cargo run -p bench -- run \
  --scenario chat \
  --url http://127.0.0.1:8080/v1/chat/completions \
  --s3-bucket my-bench-results \
  --s3-prefix benchmarks/chat/

# Upload with custom region (via AWS CLI env)
AWS_REGION=us-east-1 cargo run -p bench -- run \
  --scenario health \
  --url http://127.0.0.1:8080/health \
  --s3-bucket my-bench-results \
  --s3-prefix us-east-1-results/
```

### 3. Docker Run With S3 Upload

```bash
# Build bench image
docker build -f containers/bench/Dockerfile -t bench:latest .

# Run with S3 upload (requires AWS credentials)
docker run --rm \
  -e AWS_ACCESS_KEY_ID \
  -e AWS_SECRET_ACCESS_KEY \
  -e AWS_REGION=us-east-1 \
  -e SENTINEL_URL=http://host.docker.internal:8080 \
  bench:latest \
  run \
  --scenario health \
  --url http://host.docker.internal:8080/health \
  --s3-bucket my-bench-results \
  --s3-prefix docker-runs/
```

### 4. Kubernetes Job With S3 Upload (IRSA)

**Prerequisites:**
1. IRSA configured for bench service account
2. IAM role with S3 write permissions
3. S3 bucket exists

#### Step 1: Create IAM Policy

```bash
# Create IAM policy for S3 access
cat > /tmp/bench-s3-policy.json <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "s3:PutObject",
                "s3:PutObjectAcl"
            ],
            "Resource": "arn:aws:s3:::my-bench-results-bucket/*"
        }
    ]
}
EOF

aws iam create-policy \
  --policy-name BenchS3Upload \
  --policy-document file:///tmp/bench-s3-policy.json
```

#### Step 2: Create IAM Role for Service Account

```bash
# Get cluster OIDC issuer
OIDC_ISSUER=$(aws eks describe-cluster \
  --name sentinel-fabric-platform-cell \
  --region us-east-1 \
  --query "cluster.identity.oidc.issuer" \
  --output text)

# Create trust policy
cat > /tmp/bench-trust-policy.json <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": {
                "Federated": "arn:aws:iam::ACCOUNT_ID:oidc-provider/${OIDC_ISSUER#*//}"
            },
            "Action": "sts:AssumeRoleWithWebIdentity",
            "Condition": {
                "StringEquals": {
                    "${OIDC_ISSUER#*//}:sub": "system:serviceaccount:sentinel-fabric:bench-sa"
                }
            }
        }
    ]
}
EOF

# Create IAM role
aws iam create-role \
  --role-name bench-s3-role \
  --assume-role-policy-document file:///tmp/bench-trust-policy.json \
  --description "IAM role for bench service account with S3 access"

# Attach S3 policy
aws iam attach-role-policy \
  --role-name bench-s3-role \
  --policy-arn arn:aws:iam::ACCOUNT_ID:policy/BenchS3Upload
```

#### Step 3: Annotate Service Account

```bash
# Get IAM role ARN
ROLE_ARN=$(aws iam get-role \
  --role-name bench-s3-role \
  --query 'Role.Arn' \
  --output text)

# Annotate service account
kubectl annotate serviceaccount bench-sa \
  -n sentinel-fabric \
  eks.amazonaws.com/role-arn=${ROLE_ARN} \
  --overwrite
```

#### Step 4: Run Bench Job

```bash
# Update job.yaml with your bucket name
# Edit k8s/bench/job.yaml:
#   - Uncomment --s3-bucket line
#   - Set your bucket name
#   - Uncomment eks.amazonaws.com/role-arn annotation

# Apply the job
kubectl apply -f k8s/bench/job.yaml

# Watch job progress
kubectl get job bench -n sentinel-fabric -w

# View logs
kubectl logs -l app.kubernetes.io/name=bench -n sentinel-fabric -f
```

#### Example Job Output

```bash
# Sample job execution
kubectl apply -f k8s/bench/job.yaml

# Check job status
kubectl get job bench -n sentinel-fabric
# NAME   COMPLETIONS   DURATION   AGE
# bench  1/1           45s        2m

# View logs
kubectl logs job/bench -n sentinel-fabric
# Starting benchmark...
# wrote /tmp/bench/results.json
# scenario=health conc=32 req=2000 rps=1234.5 errors=0 p50=1200us p95=2500us p99=5000us max=15000us wall=1620ms
# Uploading /tmp/bench/results.json to s3://my-bench-results-bucket/bench-results/results-20260307-123456.json using AWS CLI...
# Successfully uploaded to s3://my-bench-results-bucket/bench-results/results-20260307-123456.json
```

## Result File Structure

### Local Path
```
bench/results/
  health-20260307-123456.json
  chat-20260307-123500.json
```

### S3 Path
```
s3://my-bucket/sentinel-fabric/bench/
  health-20260307-123456.json
  chat-20260307-123500.json

s3://my-bucket/custom-prefix/
  health-20260307-123456.json
```

## Result JSON Format

```json
{
  "scenario": "health",
  "url": "http://sentinel:8080/health",
  "concurrency": 32,
  "requests": 2000,
  "warmup_requests": 200,
  "timestamp_utc": "2026-03-07T12:34:56.789Z",
  "wall_time_ms": 1620,
  "rps": 1234.5,
  "errors": 0,
  "status_counts": {
    "200": 2000
  },
  "latency_us": {
    "p50": 1200,
    "p95": 2500,
    "p99": 5000,
    "max": 15000
  },
  "env": {
    "os": "linux",
    "arch": "x86_64",
    "cpu_count": 4,
    "git_commit": "a3f5c2b"
  }
}
```

## Troubleshooting

### AWS CLI Not Found

```bash
# Verify AWS CLI is installed
aws --version

# If running locally, install AWS CLI v2
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install
```

### S3 Upload Fails

```bash
# Check AWS credentials
aws sts get-caller-identity

# Verify S3 bucket permissions
aws s3 ls s3://my-bucket/

# Test upload manually
aws s3 cp /tmp/test.json s3://my-bucket/test.json
```

### Kubernetes IRSA Issues

```bash
# Check service account annotation
kubectl get sa bench-sa -n sentinel-fabric -o yaml | grep eks.amazonaws.com

# Verify pod has IAM role
kubectl get pod -l app.kubernetes.io/name=bench -n sentinel-fabric -o jsonpath='{.items[0].metadata.annotations}'

# Check AWS environment in pod
kubectl exec -it job/bench -n sentinel-fabric -- env | grep AWS
```

### Permission Denied

```bash
# Check IAM role permissions
aws iam get-role-policy --role-name bench-s3-role --policy-name BenchS3Upload

# Verify bucket policy allows IAM role
aws s3api get-bucket-policy --bucket my-bench-results-bucket
```

## Security Best Practices

1. **Use IRSA in EKS** - Never hardcode AWS credentials in Kubernetes
2. **Least privilege** - Grant only `s3:PutObject` to specific bucket/prefix
3. **Bucket policies** - Restrict access to specific IAM roles
4. **Encryption** - Enable S3 bucket encryption at rest
5. **Lifecycle policies** - Clean up old results automatically

## Cost Optimization

- Enable S3 Intelligent-Tiering for infrequent access
- Set lifecycle rules to transition to Glacier after 30 days
- Use compression for large result sets (future enhancement)

## Next Steps

- [ ] Add result compression before upload
- [ ] Implement Rust SDK upload as alternative to AWS CLI
- [ ] Add S3 upload metrics and logging
- [ ] Support for S3 batch operations for multiple files
