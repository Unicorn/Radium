---
id: "advanced"
title: "Advanced Configuration Guide"
sidebar_label: "Advanced Config Guide"
---

# Advanced Configuration Guide

## Overview

This guide covers advanced deployment patterns for self-hosted models, including load balancing, high availability, health checks, and performance optimization.

## Load Balancing

### nginx Load Balancer for Ollama

**Configuration: `nginx.conf`**
```nginx
upstream ollama_backend {
    least_conn;
    server localhost:11434;
    server localhost:11435;
    server localhost:11436;
}

server {
    listen 11434;
    
    location / {
        proxy_pass http://ollama_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
}
```

**Docker Compose:**
```yaml
version: '3.8'

services:
  ollama1:
    image: ollama/ollama:latest
    ports:
      - "11434:11434"
    volumes:
      - ollama1-data:/root/.ollama

  ollama2:
    image: ollama/ollama:latest
    ports:
      - "11435:11434"
    volumes:
      - ollama2-data:/root/.ollama

  ollama3:
    image: ollama/ollama:latest
    ports:
      - "11436:11434"
    volumes:
      - ollama3-data:/root/.ollama

  nginx:
    image: nginx:alpine
    ports:
      - "11434:11434"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - ollama1
      - ollama2
      - ollama3

volumes:
  ollama1-data:
  ollama2-data:
  ollama3-data:
```

### HAProxy Load Balancer for vLLM

**Configuration: `haproxy.cfg`**
```haproxy
global
    log stdout format raw local0
    maxconn 4096

defaults
    mode http
    timeout connect 60s
    timeout client 60s
    timeout server 60s

frontend vllm_frontend
    bind *:8000
    default_backend vllm_backend

backend vllm_backend
    balance roundrobin
    option httpchk GET /health
    http-check expect status 200
    server vllm1 localhost:8000 check
    server vllm2 localhost:8001 check
    server vllm3 localhost:8002 check
```

## High Availability

### Active-Passive Failover

**Agent Configuration:**
```toml
[agent]
id = "ha-agent"
name = "High Availability Agent"
description = "Agent with automatic failover"
prompt_path = "prompts/agents/my-agents/ha-agent.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Primary endpoint
fallback = "llama3.2"          # Secondary endpoint (same model, different server)
premium = "gpt-4o-mini"        # Cloud fallback
```

**Environment Setup:**
```bash
# Primary endpoint
export UNIVERSAL_BASE_URL="http://ollama-primary:11434/v1"

# Fallback endpoint (configured in engine system)
# Note: This may require custom engine configuration
```

### Health Checks

**Simple Health Check Script: `health-check.sh`**
```bash
#!/bin/bash

ENDPOINT="${1:-http://localhost:11434/v1/models}"
TIMEOUT=5

response=$(curl -s -o /dev/null -w "%{http_code}" --max-time $TIMEOUT "$ENDPOINT")

if [ "$response" -eq 200 ]; then
    echo "Healthy"
    exit 0
else
    echo "Unhealthy (HTTP $response)"
    exit 1
fi
```

**Cron Job for Monitoring:**
```bash
# Check every minute
* * * * * /path/to/health-check.sh http://localhost:11434/v1/models
```

### Kubernetes Health Checks

**Deployment with Liveness/Readiness Probes:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ollama
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ollama
  template:
    metadata:
      labels:
        app: ollama
    spec:
      containers:
      - name: ollama
        image: ollama/ollama:latest
        ports:
        - containerPort: 11434
        livenessProbe:
          httpGet:
            path: /api/tags
            port: 11434
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/tags
            port: 11434
          initialDelaySeconds: 10
          periodSeconds: 5
```

## Performance Optimization

### Connection Pooling

**Optimized HTTP Client Configuration:**
```rust
// Example: Configure connection pool for UniversalModel
// This is handled internally, but you can optimize timeouts:

// Environment variables for tuning:
export UNIVERSAL_TIMEOUT=120        # Request timeout (seconds)
export UNIVERSAL_MAX_RETRIES=3      # Retry attempts
export UNIVERSAL_RETRY_DELAY=1     # Retry delay (seconds)
```

### Batch Processing

**Agent Configuration for Batch:**
```toml
[agent]
id = "batch-agent"
name = "Batch Processing Agent"
description = "Optimized for batch processing"
prompt_path = "prompts/agents/my-agents/batch-agent.md"
engine = "universal"
model = "llama3.2"
reasoning_effort = "low"

[agent.persona.performance]
profile = "speed"
estimated_tokens = 1000
```

### Model-Specific Tuning

**vLLM Performance Tuning:**
```bash
# High throughput
vllm serve meta-llama/Llama-3-8B-Instruct \
  --max-num-seqs 512 \
  --gpu-memory-utilization 0.95 \
  --tensor-parallel-size 1

# Low latency
vllm serve meta-llama/Llama-3-8B-Instruct \
  --max-num-seqs 64 \
  --gpu-memory-utilization 0.8 \
  --tensor-parallel-size 1
```

**Ollama Performance Tuning:**
```bash
# Use GPU layers
OLLAMA_NUM_GPU=1 ollama serve

# Set thread count
OLLAMA_NUM_THREAD=8 ollama serve
```

## Cost Optimization

### Local-First Strategy

**Agent Configuration:**
```toml
[agent]
id = "cost-optimized"
name = "Cost-Optimized Agent"
description = "Maximize local usage"
prompt_path = "prompts/agents/my-agents/cost-optimized.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Free local
fallback = "llama3.2:13b"      # Free local (better)
premium = "gpt-4o-mini"        # Cheap cloud ($0.15/1M tokens)
```

### Hybrid Cost Strategy

**Agent Configuration:**
```toml
[agent]
id = "hybrid-cost"
name = "Hybrid Cost Agent"
description = "Balance cost and quality"
prompt_path = "prompts/agents/my-agents/hybrid-cost.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Free local (most requests)
fallback = "gemini-2.0-flash-exp"  # Cheap cloud ($0.075/1M tokens)
premium = "gpt-4o"             # Expensive cloud (only when needed)
```

## Monitoring and Observability

### Prometheus Metrics

**Example Metrics to Track:**
- Request latency (p50, p95, p99)
- Request rate (requests/second)
- Error rate (errors/requests)
- Model availability (uptime)
- Token usage (tokens/second)

### Grafana Dashboard

**Key Metrics to Display:**
1. **Request Rate**: Requests per second by model
2. **Latency**: Response time percentiles
3. **Error Rate**: Failed requests percentage
4. **Model Health**: Uptime and availability
5. **Resource Usage**: CPU, memory, GPU utilization

### Logging

**Structured Logging:**
```bash
# Enable debug logging
export RUST_LOG=radium_core::engines=debug

# Log to file
export RUST_LOG=radium_core::engines=debug,file=radium.log
```

## Security

### Network Isolation

**Docker Network Configuration:**
```yaml
version: '3.8'

networks:
  internal:
    driver: bridge
    internal: true  # No external access

services:
  ollama:
    image: ollama/ollama:latest
    networks:
      - internal
    # No ports exposed externally

  radium:
    image: radium:latest
    networks:
      - internal
    # Can access ollama via internal network
```

### Authentication

**API Key Protection:**
```bash
# Use secrets management
export UNIVERSAL_API_KEY=$(cat /run/secrets/universal_api_key)

# Or use environment file with restricted permissions
chmod 600 .env
```

### Firewall Rules

**iptables Example:**
```bash
# Allow only localhost access
iptables -A INPUT -p tcp --dport 11434 -s 127.0.0.1 -j ACCEPT
iptables -A INPUT -p tcp --dport 11434 -j DROP
```

## Scaling Strategies

### Horizontal Scaling

**Multiple Model Instances:**
```yaml
version: '3.8'

services:
  ollama-1:
    image: ollama/ollama:latest
    ports:
      - "11434:11434"
  
  ollama-2:
    image: ollama/ollama:latest
    ports:
      - "11435:11434"
  
  ollama-3:
    image: ollama/ollama:latest
    ports:
      - "11436:11434"
  
  nginx:
    image: nginx:alpine
    # Load balance across instances
```

### Vertical Scaling

**Resource Allocation:**
```yaml
services:
  vllm:
    image: vllm/vllm-openai:latest
    deploy:
      resources:
        limits:
          cpus: '8'
          memory: 32G
          nvidia.com/gpu: 1
        reservations:
          cpus: '4'
          memory: 16G
          nvidia.com/gpu: 1
```

## Best Practices

1. **Start Simple**: Begin with a single instance, then scale as needed
2. **Monitor First**: Set up monitoring before optimizing
3. **Test Failover**: Regularly test failover scenarios
4. **Document Configurations**: Keep track of what works
5. **Version Control**: Store configurations in version control
6. **Backup Models**: Regularly backup model files
7. **Security First**: Apply security best practices from the start

## Next Steps

- Review [Configuration Examples](examples.md) for more patterns
- Check [Troubleshooting Guide](../troubleshooting.md) for issues
- See [Setup Guides](../setup/) for provider-specific details
- Explore [Migration Guide](../migration.md) for transitioning strategies

