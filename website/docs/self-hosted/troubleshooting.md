---
id: "troubleshooting"
title: "Troubleshooting Guide"
sidebar_label: "Troubleshooting Guide"
---

# Troubleshooting Guide

## Overview

This guide helps you diagnose and resolve common issues when using self-hosted models with Radium. Issues are organized by symptom to help you quickly find solutions.

## Quick Diagnostic Commands

### Check Model Server Status

```bash
# Ollama
curl http://localhost:11434/api/tags

# vLLM
curl http://localhost:8000/v1/models

# LocalAI
curl http://localhost:8080/v1/models
```

### Check Network Connectivity

```bash
# Test port accessibility
telnet localhost 11434  # Ollama
telnet localhost 8000   # vLLM
telnet localhost 8080   # LocalAI

# Check if port is in use
netstat -an | grep 11434
lsof -i :11434
```

### Check Radium Configuration

```bash
# List agents
rad agents list

# Test agent execution
rad run <agent-id> "test"
```

## Common Errors

### Connection Refused

**Error Message:**
```
RequestError: Connection refused
RequestError: Network error: ... connection refused
```

**Diagnosis:**
1. Check if model server is running:
   ```bash
   # Ollama
   ps aux | grep ollama
   docker ps | grep ollama
   
   # vLLM
   docker ps | grep vllm
   
   # LocalAI
   docker ps | grep localai
   ```

2. Verify the port is correct:
   ```bash
   # Ollama: 11434
   # vLLM: 8000
   # LocalAI: 8080
   ```

3. Check firewall settings:
   ```bash
   # Linux
   sudo ufw status
   sudo iptables -L
   
   # macOS
   # Check System Preferences → Security & Privacy → Firewall
   ```

**Solutions:**
1. **Start the model server:**
   ```bash
   # Ollama
   ollama serve
   
   # vLLM (Docker)
   docker run --gpus all -p 8000:8000 vllm/vllm-openai:latest --model <model>
   
   # LocalAI (Docker)
   docker-compose up -d
   ```

2. **Verify environment variables:**
   ```bash
   echo $UNIVERSAL_BASE_URL
   # Should be: http://localhost:11434/v1 (Ollama)
   #            http://localhost:8000/v1 (vLLM)
   #            http://localhost:8080/v1 (LocalAI)
   ```

3. **Check base URL includes `/v1`:**
   ```bash
   # Correct
   export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
   
   # Incorrect (missing /v1)
   export UNIVERSAL_BASE_URL="http://localhost:11434"
   ```

4. **For remote servers, check network access:**
   ```bash
   ping <server-ip>
   curl http://<server-ip>:11434/api/tags
   ```

### Model Not Found

**Error Message:**
```
ModelResponseError: Model not found
UnsupportedModelProvider: Model 'xxx' not available
```

**Diagnosis:**
1. **Ollama - List available models:**
   ```bash
   ollama list
   ```

2. **vLLM - Check loaded models:**
   ```bash
   curl http://localhost:8000/v1/models
   ```

3. **LocalAI - Check configured models:**
   ```bash
   curl http://localhost:8080/v1/models
   ls -la config/  # Check model config files
   ```

**Solutions:**
1. **Download the model:**
   ```bash
   # Ollama
   ollama pull llama3.2
   
   # vLLM - Model loads automatically from Hugging Face
   # Check server logs for download progress
   
   # LocalAI - Install via gallery or manually
   curl http://localhost:8080/models/apply -d '{"id": "ggml-gpt4all-j"}'
   ```

2. **Verify model name matches exactly:**
   ```bash
   # Case-sensitive, must match exactly
   # Correct: llama3.2
   # Incorrect: Llama3.2, llama-3.2, llama3
   ```

3. **Check agent configuration:**
   ```toml
   [agent]
   model = "llama3.2"  # Must match model name on server
   ```

### Timeout Errors

**Error Message:**
```
RequestError: Request timeout
RequestError: Operation timed out
```

**Diagnosis:**
1. Check server response time:
   ```bash
   time curl http://localhost:11434/v1/models
   ```

2. Check hardware resources:
   ```bash
   # CPU usage
   top
   htop
   
   # Memory usage
   free -h
   
   # GPU usage (if applicable)
   nvidia-smi
   ```

3. Check model server logs for errors

**Solutions:**
1. **Increase timeout (if configurable):**
   ```bash
   export UNIVERSAL_TIMEOUT=120  # Increase from default 60s
   ```

2. **Reduce request complexity:**
   - Use smaller `max_tokens`
   - Reduce context length
   - Use a faster/smaller model

3. **Optimize hardware:**
   - Ensure sufficient RAM/VRAM
   - Use GPU if available
   - Close other applications

4. **Check for resource constraints:**
   ```bash
   # Check swap usage (indicates memory pressure)
   swapon --show
   free -h
   ```

### Out of Memory

**Error Message:**
```
ModelResponseError: Out of memory
RequestError: Insufficient memory
```

**Diagnosis:**
1. **Check available memory:**
   ```bash
   # System memory
   free -h
   
   # GPU memory (if using GPU)
   nvidia-smi
   ```

2. **Check model size vs available memory:**
   - 7B model: ~14GB RAM/VRAM
   - 13B model: ~26GB RAM/VRAM
   - 30B+ model: 40GB+ VRAM

**Solutions:**
1. **Use a smaller model:**
   ```bash
   # Ollama - Use quantized model
   ollama pull llama3.2        # ~2GB
   # Instead of
   ollama pull llama3.2:13b    # ~7GB
   ```

2. **Reduce memory usage:**
   ```bash
   # vLLM - Reduce GPU memory utilization
   vllm serve <model> --gpu-memory-utilization 0.7
   
   # LocalAI - Reduce context size
   # Edit config YAML: context_size: 2048
   ```

3. **Close other applications** to free memory

4. **Use CPU-only inference** (slower but uses less VRAM)

### API Compatibility Issues

**Error Message:**
```
SerializationError: Failed to parse response
ModelResponseError: Invalid API response format
```

**Diagnosis:**
1. Test API endpoint directly:
   ```bash
   curl http://localhost:11434/v1/chat/completions \
     -H "Content-Type: application/json" \
     -d '{
       "model": "llama3.2",
       "messages": [{"role": "user", "content": "test"}]
     }'
   ```

2. Check response format matches OpenAI spec

**Solutions:**
1. **Verify endpoint path:**
   ```bash
   # Must include /v1
   export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
   ```

2. **Check server supports OpenAI API:**
   - Ollama: Requires OpenAI-compatible endpoint (available by default)
   - vLLM: Native OpenAI compatibility
   - LocalAI: Configured via model YAML

3. **Update server version** if using outdated software

### Authentication Errors

**Error Message:**
```
UnsupportedModelProvider: Authentication failed
RequestError: 401 Unauthorized
```

**Diagnosis:**
1. Check if server requires authentication:
   ```bash
   # Most local servers don't require auth
   curl http://localhost:11434/v1/models
   ```

2. Verify API key if required:
   ```bash
   echo $UNIVERSAL_API_KEY
   ```

**Solutions:**
1. **Remove API key for local servers:**
   ```bash
   unset UNIVERSAL_API_KEY
   # Or use without_auth() constructor
   ```

2. **Set correct API key if required:**
   ```bash
   export UNIVERSAL_API_KEY="your-api-key"
   ```

3. **Check server authentication settings**

## Performance Issues

### Slow Inference

**Symptoms:**
- Long response times
- Low tokens/second

**Diagnosis:**
1. Check hardware utilization:
   ```bash
   # CPU
   top
   
   # GPU
   nvidia-smi -l 1
   ```

2. Check model server logs for warnings

**Solutions:**
1. **Use GPU if available:**
   ```bash
   # Ollama - Automatically uses GPU if detected
   # vLLM - Requires GPU
   # LocalAI - Configure gpu_layers in model config
   ```

2. **Use a faster model:**
   - Smaller models are faster
   - Quantized models (Q4, Q8) are faster

3. **Optimize server settings:**
   ```bash
   # vLLM - Increase batch size
   vllm serve <model> --max-num-seqs 512
   
   # LocalAI - Increase threads
   # Edit config: threads: 8
   ```

4. **Reduce context length** if not needed

### High Latency

**Symptoms:**
- Long time to first token
- Slow initial response

**Solutions:**
1. **Pre-warm the model:**
   ```bash
   # Make a test request to load model
   curl http://localhost:11434/v1/chat/completions ...
   ```

2. **Use streaming** for better perceived performance

3. **Reduce model size** for faster loading

## Network Issues

### Cannot Access Remote Server

**Symptoms:**
- Connection works locally but not remotely
- Timeout when accessing from another machine

**Solutions:**
1. **Check server binding:**
   ```bash
   # Ollama - Bind to 0.0.0.0
   OLLAMA_HOST=0.0.0.0:11434 ollama serve
   ```

2. **Configure firewall:**
   ```bash
   # Allow port in firewall
   sudo ufw allow 11434
   ```

3. **Check network connectivity:**
   ```bash
   ping <server-ip>
   telnet <server-ip> 11434
   ```

4. **Verify base URL:**
   ```bash
   # Use server IP or hostname
   export UNIVERSAL_BASE_URL="http://192.168.1.100:11434/v1"
   ```

## Diagnostic Decision Tree

```
Is the model server running?
├─ No → Start the server
└─ Yes → Can you connect to the endpoint?
    ├─ No → Check firewall/network
    └─ Yes → Is the model available?
        ├─ No → Download/configure the model
        └─ Yes → Check agent configuration
            ├─ Wrong model name → Fix model name
            ├─ Wrong endpoint → Fix base URL
            └─ Other → Check logs
```

## Log Analysis

### Radium Logs

**Location:**
- Default: `logs/radium-core.log`
- Or check console output

**What to look for:**
- Connection errors
- Model creation failures
- Request/response details

### Model Server Logs

**Ollama:**
```bash
# Check service logs
journalctl -u ollama -f

# Docker logs
docker logs ollama -f
```

**vLLM:**
```bash
# Docker logs
docker logs vllm -f
```

**LocalAI:**
```bash
# Docker logs
docker logs localai -f
```

### Common Log Patterns

**Connection Refused:**
```
ERROR: Connection refused
ERROR: Failed to connect to http://localhost:11434
```

**Model Not Found:**
```
ERROR: Model 'xxx' not found
ERROR: Model not available
```

**Timeout:**
```
ERROR: Request timeout
ERROR: Operation timed out
```

## Still Stuck?

### Additional Resources

1. **Check Provider Documentation:**
   - [Ollama Docs](https://ollama.com/docs)
   - [vLLM Docs](https://docs.vllm.ai/)
   - [LocalAI Docs](https://localai.io/)

2. **Review Setup Guides:**
   - [Ollama Setup](setup/ollama.md)
   - [vLLM Setup](setup/vllm.md)
   - [LocalAI Setup](setup/localai.md)

3. **Check Configuration:**
   - [Agent Configuration](configuration/agent-config.md)
   - [Configuration Examples](configuration/examples.md)

4. **Community Support:**
   - GitHub Issues
   - Discord/Slack (if available)
   - Stack Overflow

### Collecting Debug Information

When seeking help, provide:

1. **Error message** (exact text)
2. **Model server logs** (last 50 lines)
3. **Radium logs** (relevant sections)
4. **Configuration:**
   ```bash
   # Agent config
   cat agents/my-agents/<agent>.toml
   
   # Environment variables
   env | grep UNIVERSAL
   ```
5. **System information:**
   ```bash
   # OS
   uname -a
   
   # Docker (if used)
   docker version
   
   # GPU (if applicable)
   nvidia-smi
   ```

## Next Steps

- Review [Setup Guides](setup/) for installation issues
- Check [Configuration Guide](configuration/agent-config.md) for config problems
- See [Advanced Configuration](configuration/advanced.md) for optimization
- Explore [Migration Guide](migration.md) for transition issues

