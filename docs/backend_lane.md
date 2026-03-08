# Backend Lane Evolution: B0 Stub → B1 vLLM

This document describes the evolution of the backend lane from a simple stub service (B0) to a full LLM inference server (B1) using vLLM.

## Overview

The backend lane provides LLM inference capabilities to the Sentinel Fabric platform. It evolves through two stages:

- **B0 (Current)**: Upstream stub - mock responses for testing
- **B1 (Target)**: vLLM server - real LLM inference with OpenAI-compatible API

```
┌─────────────────────────────────────────────────────────────┐
│                     Sentinel Fabric                          │
│                                                              │
│  ┌──────────┐         ┌──────────┐         ┌──────────┐    │
│  │  Client  │ ──────> │ Sentinel │ ──────> │  Backend │    │
│  └──────────┘         └──────────┘         └──────────┘    │
│                          Gateway               Lane         │
│                                                              │
│  B0: Backend = Upstream Stub (mock)                         │
│  B1: Backend = vLLM Server (real LLM)                       │
└─────────────────────────────────────────────────────────────┘
```

## B0: Upstream Stub (Current)

### Characteristics

- **Purpose**: Development and testing
- **Implementation**: Rust HTTP server
- **Responses**: Mock/stubbed responses
- **Latency**: <10ms
- **GPU Required**: No
- **Cost**: Minimal

### Architecture

```yaml
Deployment: upstream-stub
Namespace: sentinel-fabric
Port: 4000
Endpoints:
  - GET /health
  - GET /ready
  - POST /v1/chat/completions (mock)
```

### Configuration

```yaml
# k8s/upstream_stub/deployment.yaml
containers:
  - name: upstream-stub
    image: upstream_stub:latest
    ports:
      - containerPort: 4000
    resources:
      requests:
        cpu: "100m"
        memory: "128Mi"
      limits:
        cpu: "500m"
        memory: "512Mi"
```

### Limitations

- ❌ No real LLM inference
- ❌ Cannot test actual model performance
- ❌ No GPU utilization
- ❌ Limited benchmarking capabilities

## B1: vLLM Server (Target)

### Characteristics

- **Purpose**: Production LLM inference
- **Implementation**: vLLM OpenAI-compatible server
- **Responses**: Real LLM-generated responses
- **Latency**: 100ms-5s (depends on model and prompt)
- **GPU Required**: Yes (NVIDIA GPU with 16GB+ VRAM)
- **Cost**: Higher (GPU instances)

### Architecture

```yaml
Deployment: vllm
Namespace: sentinel-fabric
Port: 8000
Endpoints:
  - GET /health
  - GET /v1/models
  - POST /v1/chat/completions
  - POST /v1/completions
```

### Configuration

```yaml
# k8s/vllm/deployment.yaml
containers:
  - name: vllm
    image: vllm/vllm-openai:latest
    ports:
      - containerPort: 8000
    resources:
      requests:
        cpu: "4"
        memory: "16Gi"
        nvidia.com/gpu: "1"
      limits:
        cpu: "8"
        memory: "32Gi"
        nvidia.com/gpu: "1"
    env:
      - name: MODEL_ID
        value: "meta-llama/Llama-3.1-8B-Instruct"
```

### Benefits

- ✅ Real LLM inference
- ✅ OpenAI-compatible API
- ✅ High throughput with PagedAttention
- ✅ Continuous batching
- ✅ GPU acceleration
- ✅ Production-ready

## Migration Path: B0 → B1

### Phase 1: Infrastructure Setup

1. **Install GPU Operator**
   ```bash
   ./infra/gpu_operator_install.sh
   ```

2. **Create GPU Nodegroup**
   ```bash
   ./infra/scripts/gpu_on.sh
   ```

3. **Verify GPU Access**
   ```bash
   kubectl run nvidia-smi --image=nvidia/cuda:12.4.1-base-ubuntu22.04 --rm -it --restart=Never -- nvidia-smi
   ```

### Phase 2: Deploy vLLM

1. **Create Hugging Face Secret**
   ```bash
   kubectl create secret generic huggingface-secret \
     --from-literal=token=hf_your_token \
     -n sentinel-fabric
   ```

2. **Deploy vLLM**
   ```bash
   kubectl apply -f k8s/vllm/
   ```

3. **Verify Deployment**
   ```bash
   kubectl get pods -n sentinel-fabric -l app.kubernetes.io/name=vllm
   kubectl logs -l app.kubernetes.io/name=vllm -n sentinel-fabric -f
   ```

### Phase 3: Testing

1. **Port Forward**
   ```bash
   kubectl port-forward svc/vllm -n sentinel-fabric 8000:8000
   ```

2. **Test Health**
   ```bash
   curl http://localhost:8000/health
   ```

3. **Test Chat Completion**
   ```bash
   curl http://localhost:8000/v1/chat/completions \
     -H "Content-Type: application/json" \
     -d '{
       "model": "meta-llama/Llama-3.1-8B-Instruct",
       "messages": [{"role": "user", "content": "Hello"}],
       "max_tokens": 50
     }'
   ```

### Phase 4: Cutover (Future - Not Implemented)

When Sentinel is ready to switch from B0 to B1:

1. Update Sentinel configuration to point to vLLM service
2. Update `UPSTREAM_URL` environment variable
3. Monitor latency and error rates
4. Gradually shift traffic (canary deployment)

## Comparison: B0 vs B1

| Aspect | B0 (Stub) | B1 (vLLM) |
|--------|-----------|-----------|
| **Implementation** | Rust HTTP server | vLLM + PyTorch |
| **Response Type** | Mock/Static | LLM-generated |
| **Latency** | <10ms | 100ms-5s |
| **GPU Required** | No | Yes (1x NVIDIA GPU) |
| **Memory** | 128Mi | 16-32Gi |
| **CPU** | 100m | 4-8 cores |
| **Cost/Hour** | ~$0.05 (spot) | ~$0.50-1.50 (g4dn/g5) |
| **Use Case** | Dev/Test | Production |
| **API Compatibility** | OpenAI-compatible | OpenAI-compatible |

## Resource Requirements

### B0: Upstream Stub

```yaml
requests:
  cpu: "100m"
  memory: "128Mi"
limits:
  cpu: "500m"
  memory: "512Mi"
```

**Node Requirements:**
- Any EC2 instance type
- No GPU needed
- Works on spot instances

### B1: vLLM (8B Model)

```yaml
requests:
  cpu: "4"
  memory: "16Gi"
  nvidia.com/gpu: "1"
limits:
  cpu: "8"
  memory: "32Gi"
  nvidia.com/gpu: "1"
```

**Node Requirements:**
- GPU instance: g4dn.xlarge (1x T4, 16GB VRAM) or g5.xlarge (1x A10G, 24GB VRAM)
- NVIDIA GPU Operator installed
- Container runtime: containerd

## Model Options

### Recommended Models for B1

| Model | Size | VRAM Required | Quality | Speed |
|-------|------|---------------|---------|-------|
| Llama-3.1-8B-Instruct | 8B | 16GB | High | Fast |
| Mistral-7B-Instruct-v0.3 | 7B | 14GB | High | Fast |
| Qwen2.5-7B-Instruct | 7B | 14GB | High | Fast |
| Llama-3.1-70B-Instruct | 70B | 140GB+ | Very High | Slow (needs multi-GPU) |

### Model Selection Criteria

- **VRAM**: Must fit in GPU memory (with room for KV cache)
- **License**: Check commercial use permissions
- **Quality vs Speed**: Larger models = better quality, slower inference
- **Context Length**: Longer context = more VRAM needed

## Performance Expectations

### B0: Upstream Stub

- **Throughput**: 1000+ RPS
- **Latency (p50)**: 5ms
- **Latency (p99)**: 10ms
- **Error Rate**: <0.1%

### B1: vLLM (Llama-3.1-8B on T4)

- **Throughput**: 50-100 RPS (depends on prompt length)
- **Latency (p50)**: 500ms (first token + decoding)
- **Latency (p99)**: 2s
- **Error Rate**: <1%
- **Tokens/sec**: 20-40 tokens/sec

## Monitoring

### B0 Metrics

```bash
# Check stub pod
kubectl top pod -l app.kubernetes.io/name=upstream-stub -n sentinel-fabric
```

### B1 Metrics

```bash
# Check vLLM pod
kubectl top pod -l app.kubernetes.io/name=vllm -n sentinel-fabric

# Check GPU utilization
kubectl exec -it deploy/vllm -n sentinel-fabric -- nvidia-smi

# Access DCGM metrics (if GPU Operator installed)
kubectl port-forward svc/nvidia-dcgm-exporter -n gpu-operator 9400:9400
curl http://localhost:9400/metrics | grep DCGM
```

## Rollback Plan

If B1 encounters issues:

1. **Keep B0 Running**
   - Both deployments can coexist
   - B0 remains available on port 4000

2. **Switch Back to B0**
   - Update Sentinel `UPSTREAM_URL` to `http://upstream-stub:4000`
   - Restart Sentinel pods

3. **Debug B1**
   - Check vLLM logs
   - Verify GPU access
   - Test model loading
   - Re-deploy when fixed

## Future Enhancements (Beyond B1)

- **B2**: Multi-model support with model routing
- **B3**: Multi-GPU tensor parallelism for 70B+ models
- **B4**: Auto-scaling based on queue depth
- **B5**: Model caching and hot-swapping
- **B6**: Quantization support (INT8, FP8) for efficiency

## References

- [vLLM Documentation](https://docs.vllm.ai/)
- [GPU Operator Guide](./gpu_operator.md)
- [vLLM Deployment](../k8s/vllm/README.md)
- [NVIDIA GPU Cloud](https://catalog.ngc.nvidia.com/containers)
