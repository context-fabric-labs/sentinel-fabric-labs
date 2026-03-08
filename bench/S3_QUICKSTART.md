# Bench S3 Upload - Quick Start

## ✅ What's New

The bench tool now supports optional S3 upload of benchmark results with these new flags:

- `--s3-bucket <name>` - Enable S3 upload to specified bucket
- `--s3-prefix <prefix>` - S3 key prefix (default: `sentinel-fabric/bench/`)
- `--upload-with-aws-cli <true|false>` - Use AWS CLI v2 (default: `true`)

## 🚀 Quick Commands

### Local Run Without S3

```bash
# Basic health benchmark
cargo run -p bench -- run \
  --scenario health \
  --url http://127.0.0.1:8080/health \
  --concurrency 32 \
  --requests 2000

# Results saved to: bench/results/health-YYYYMMDD-HHMMSS.json
```

### Local Run With S3 Upload

```bash
# Upload to S3 after benchmark
cargo run -p bench -- run \
  --scenario health \
  --url http://127.0.0.1:8080/health \
  --s3-bucket my-bench-results \
  --s3-prefix benchmarks/health/

# Results uploaded to: s3://my-bench-results/benchmarks/health/health-YYYYMMDD-HHMMSS.json
```

### Kubernetes Job With S3 (IRSA)

```bash
# 1. Configure service account with IRSA (see S3_UPLOAD_GUIDE.md)
kubectl annotate serviceaccount bench-sa -n sentinel-fabric \
  eks.amazonaws.com/role-arn=arn:aws:iam::ACCOUNT_ID:role/bench-s3-role

# 2. Edit k8s/bench/job.yaml to enable S3 upload
#    Uncomment: --s3-bucket and --s3-prefix lines

# 3. Run the job
kubectl apply -f k8s/bench/job.yaml

# 4. Watch progress
kubectl logs -l app.kubernetes.io/name=bench -n sentinel-fabric -f
```

## 📦 Docker Image

The bench container already includes AWS CLI v2:

```bash
# Build
docker build -f containers/bench/Dockerfile -t bench:latest .

# Run with S3
docker run --rm \
  -e AWS_REGION=us-east-1 \
  -e SENTINEL_URL=http://sentinel:8080 \
  bench:latest \
  run \
  --scenario health \
  --url http://sentinel:8080/health \
  --s3-bucket my-bucket
```

## 🧪 Tests

All CLI argument tests pass:

```bash
cargo test -p bench
# running 6 tests
# test tests::test_cli_with_s3_args ... ok
# test tests::test_cli_without_s3_args ... ok
# test tests::test_cli_s3_prefix_default ... ok
# test tests::test_cli_s3_prefix_custom ... ok
# test tests::test_cli_upload_with_aws_cli_default ... ok
# test tests::test_cli_upload_with_aws_cli_explicit_false ... ok
```

## 📖 Full Documentation

See [S3_UPLOAD_GUIDE.md](./S3_UPLOAD_GUIDE.md) for:
- Complete IRSA setup instructions
- IAM policy examples
- Troubleshooting guide
- Security best practices

## 🔍 Example Output

```bash
$ cargo run -p bench -- run \
    --scenario health \
    --url http://localhost:8080/health \
    --s3-bucket my-results

wrote "bench/results/health-20260307-123456.json"
scenario=health conc=32 req=2000 rps=1234.5 errors=0 p50=1200us p95=2500us p99=5000us max=15000us wall=1620ms
Uploading bench/results/health-20260307-123456.json to s3://my-results/sentinel-fabric/bench/health-20260307-123456.json using AWS CLI...
Successfully uploaded to s3://my-results/sentinel-fabric/bench/health-20260307-123456.json
```

## ✅ Checklist

Before using S3 upload:

- [ ] AWS CLI v2 installed (for local runs)
- [ ] AWS credentials configured (`aws configure`)
- [ ] S3 bucket exists
- [ ] IAM permissions for S3 PutObject
- [ ] For K8s: IRSA configured with IAM role
- [ ] For K8s: Service account annotated with role ARN
