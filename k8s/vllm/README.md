# vLLM Deployment Guide

Deploy vLLM OpenAI-compatible inference server on Kubernetes with GPU support.

## Overview

vLLM provides a high-throughput, memory-efficient LLM inference server with an OpenAI-compatible API.

**Features:**
- ✅ OpenAI-compatible `/v1/chat/completions` endpoint
- ✅ GPU-accelerated inference
- ✅ PagedAttention for efficient memory usage
- ✅ Continuous batching for higher throughput

## Quick Start

### 1. Create Hugging Face Secret (Optional)

For gated models like Llama 3.1:

```bash
# Get your Hugging Face token from https://huggingface.co/settings/tokens
kubectl create secret generic huggingface-secret \
  --from-literal=token=hf_your_token_here \
  -n sentinel-fabric
```

### 2. Configure Model

Edit `deployment.yaml` to set your desired model:

```yaml
env:
  - name: MODEL_ID
    value: "meta-llama/Llama-3.1-8B-Instruct"  # Change this
```

**Popular models:**
- `meta-llama/Llama-3.1-8B-Instruct` - Llama 3.1 8B Instruct
- `meta-llama/Llama-3.1-70B-Instruct` - Llama 3.1 70B Instruct
- `mistralai/Mistral-7B-Instruct-v0.3` - Mistral 7B Instruct
- `Qwen/Qwen2.5-7B-Instruct` - Qwen 2.5 7B Instruct

### 3. Deploy

```bash
# Apply manifests
kubectl apply -f k8s/vllm/

# Watch deployment
kubectl get pods -n sentinel-fabric -l app.kubernetes.io/name=vllm -w
```

### 4. Verify Deployment

```bash
# Check pod status
kubectl get pods -n sentinel-fabric -l app.kubernetes.io/name=vllm

# Check logs
kubectl logs -l app.kubernetes.io/name=vllm -n sentinel-fabric -f

# Verify GPU access
kubectl exec -it deploy/vllm -n sentinel-fabric -- nvidia-smi
```

## Usage

### Port Forward

```bash
# Forward vLLM service to localhost
kubectl port-forward svc/vllm -n sentinel-fabric 8000:8000
```

### Test Health Endpoint

```bash
curl http://localhost:8000/health
```

### Sample Chat Completion Request

```bash
curl http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "meta-llama/Llama-3.1-8B-Instruct",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Hello, how are you?"}
    ],
    "max_tokens": 100,
    "temperature": 0.7
  }'
```

### Sample Response

```json
{
  "id": "cmpl-abc123",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "meta-llama/Llama-3.1-8B-Instruct",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm doing well, thank you for asking. How can I help you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 25,
    "completion_tokens": 18,
    "total_tokens": 43
  }
}
```

### Streaming Chat Completion

```bash
curl http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "meta-llama/Llama-3.1-8B-Instruct",
    "messages": [
      {"role": "user", "content": "Count from 1 to 5"}
    ],
    "max_tokens": 50,
    "stream": true
  }'
```

## Configuration

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `MODEL_ID` | Hugging Face model ID | `meta-llama/Llama-3.1-8B-Instruct` |
| `HUGGING_FACE_HUB_TOKEN` | HF token for gated models | (from secret) |

### Resource Allocation

Default resources (adjust based on model size):

**For 7B-8B models:**
```yaml
resources:
  requests:
    cpu: "4"
    memory: "16Gi"
    nvidia.com/gpu: "1"
  limits:
    cpu: "6"
    memory: "32Gi"
    nvidia.com/gpu: "1"
```

**For 70B models:**
```yaml
resources:
  requests:
    cpu: "8"
    memory: "64Gi"
    nvidia.com/gpu: "1"
  limits:
    cpu: "16"
    memory: "128Gi"
    nvidia.com/gpu: "1"
```

### Advanced vLLM Flags

Modify the `command` section in `deployment.yaml`:

```yaml
command:
  - "python3"
  - "-m"
  - "vllm.entrypoints.openai.api_server"
  - "--model"
  - "$(MODEL_ID)"
  - "--host"
  - "0.0.0.0"
  - "--port"
  - "8000"
  - "--tensor-parallel-size"
  - "1"                    # Increase for multi-GPU
  - "--max-model-len"
  - "8192"                 # Max context length
  - "--gpu-memory-utilization"
  - "0.9"                  # GPU memory utilization
  - "--kv-cache-dtype"
  - "float16"              # KV cache dtype
```

## Troubleshooting

### Pod Not Starting

```bash
# Check pod events
kubectl describe pod -l app.kubernetes.io/name=vllm -n sentinel-fabric

# Check logs
kubectl logs -l app.kubernetes.io/name=vllm -n sentinel-fabric
```

### GPU Not Found

```bash
# Verify GPU node selector
kubectl get nodes -l nvidia.com/gpu.present=true

# Check GPU Operator is installed
kubectl get pods -n gpu-operator
```

### Model Download Fails

```bash
# Check Hugging Face secret
kubectl get secret huggingface-secret -n sentinel-fabric -o yaml

# Verify token is valid
kubectl exec -it deploy/vllm -n sentinel-fabric -- env | grep HUGGING
```

### Out of Memory

- Reduce `--max-model-len`
- Reduce `--gpu-memory-utilization`
- Use a smaller model
- Increase GPU memory limits

## Undeploy

```bash
# Remove vLLM deployment
kubectl delete -f k8s/vllm/

# Remove Hugging Face secret (optional)
kubectl delete secret huggingface-secret -n sentinel-fabric
```

## API Reference

vLLM provides OpenAI-compatible endpoints:

- `GET /health` - Health check
- `GET /v1/models` - List models
- `POST /v1/chat/completions` - Chat completions
- `POST /v1/completions` - Text completions

Full API docs: http://localhost:8000/docs

## Resources

- [vLLM Documentation](https://docs.vllm.ai/)
- [vLLM GitHub](https://github.com/vllm-project/vllm)
- [Hugging Face Models](https://huggingface.co/models)
- [OpenAI API Reference](https://platform.openai.com/docs/api-reference)
