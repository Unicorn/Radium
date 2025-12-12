---
id: "migration"
title: "Cloud-to-Self-Hosted Migration Guide"
sidebar_label: "Cloud-to-Self-Hosted Migrat..."
---

# Cloud-to-Self-Hosted Migration Guide

## Overview

This guide walks you through migrating your Radium workspace from cloud-based AI providers (Gemini, OpenAI, Claude) to self-hosted model infrastructure. The migration can be done gradually, allowing you to test and validate at each step.

## Pre-Migration Assessment

### Checklist

Before starting migration, assess your current setup:

- [ ] **Inventory Agents**: List all agents using cloud providers
- [ ] **Assess Hardware**: Verify you have sufficient resources for self-hosted models
- [ ] **Plan Timeline**: Schedule migration during low-usage periods
- [ ] **Backup Configurations**: Save current agent configurations
- [ ] **Test Environment**: Set up a test environment if possible
- [ ] **Identify Critical Agents**: Determine which agents can be migrated first

### Agent Configuration Audit

**Find all cloud-based agents:**
```bash
# Find agents using cloud providers
grep -r "engine = \"gemini\"" agents/
grep -r "engine = \"openai\"" agents/
grep -r "engine = \"claude\"" agents/

# List all agent files
find agents/ -name "*.toml" -type f
```

**Document current configuration:**
```bash
# Create backup
cp -r agents/ agents-backup-$(date +%Y%m%d)/

# Export current configuration
find agents/ -name "*.toml" > agent-list.txt
```

### Hardware Assessment

**Minimum Requirements:**
- **Ollama**: 8GB RAM (16GB recommended)
- **vLLM**: NVIDIA GPU with 16GB+ VRAM
- **LocalAI**: 8GB RAM (16GB recommended)

**Check your system:**
```bash
# System memory
free -h

# GPU (if available)
nvidia-smi

# Disk space (for models)
df -h
```

## Model Equivalency Mapping

### Cloud to Self-Hosted Equivalents

| Cloud Model | Self-Hosted Equivalent | Quality Match | Notes |
|-------------|------------------------|---------------|-------|
| **GPT-4** | Llama-3-70B (vLLM) | ~85-90% | Best quality match |
| **GPT-4** | Mixtral 8x7B | ~80-85% | Good alternative |
| **GPT-3.5-turbo** | Llama-3-8B | ~80-85% | Good match for most tasks |
| **GPT-3.5-turbo** | Mistral 7B | ~75-80% | Faster alternative |
| **Claude 3 Opus** | Llama-3-70B | ~80-85% | Close match |
| **Claude 3 Sonnet** | Llama-3-13B | ~75-80% | Good match |
| **Gemini Pro** | Llama-3-8B | ~75-80% | Reasonable match |
| **Gemini Flash** | Llama-3-3B | ~70-75% | Faster, lower quality |

### Quality vs Performance Trade-offs

**High Quality (Slower):**
- Llama-3-70B (vLLM) - Best quality, requires GPU
- Mixtral 8x7B - High quality, requires significant VRAM

**Balanced:**
- Llama-3-13B - Good balance of quality and speed
- Llama-3-8B - Fast and capable

**Fast (Lower Quality):**
- Llama-3-3B - Very fast, good for simple tasks
- Mistral 7B - Fast and efficient

## Migration Strategy

### Gradual Migration Approach

**Phase 1: Low-Risk Agents (Week 1)**
- Migrate non-critical agents first
- Test agents with simple tasks
- Validate output quality

**Phase 2: Medium-Risk Agents (Week 2-3)**
- Migrate agents with moderate importance
- Use multi-tier strategy (local primary, cloud fallback)
- Monitor performance and quality

**Phase 3: Critical Agents (Week 4+)**
- Migrate high-importance agents
- Full testing and validation
- Keep cloud as premium tier

### Multi-Tier Safety Net

Use Radium's multi-tier model strategy during migration:

```toml
[agent.persona.models]
primary = "llama3.2"           # Self-hosted (new)
fallback = "gpt-4o-mini"       # Cloud (safety net)
premium = "gpt-4o"             # Cloud (when needed)
```

This allows:
- Testing self-hosted models as primary
- Automatic fallback to cloud if issues occur
- Gradual confidence building

## Step-by-Step Migration

### Step 1: Set Up Self-Hosted Model Server

**Choose a provider:**
- **Ollama**: Easiest setup, good for testing
- **vLLM**: Best performance, requires GPU
- **LocalAI**: Most flexible, good for experimentation

**Follow setup guide:**
- [Ollama Setup](setup/ollama.md)
- [vLLM Setup](setup/vllm.md)
- [LocalAI Setup](setup/localai.md)

**Verify server is running:**
```bash
# Ollama
curl http://localhost:11434/api/tags

# vLLM
curl http://localhost:8000/v1/models

# LocalAI
curl http://localhost:8080/v1/models
```

### Step 2: Configure Environment

**Set environment variables:**
```bash
# Ollama
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"

# vLLM
export UNIVERSAL_BASE_URL="http://localhost:8000/v1"

# LocalAI
export UNIVERSAL_BASE_URL="http://localhost:8080/v1"
```

### Step 3: Migrate Single Agent

**Example: Migrating a code agent**

**Before (Cloud):**
```toml
[agent]
id = "code-agent"
name = "Code Agent"
description = "Code implementation agent"
prompt_path = "prompts/agents/core/code-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"

[agent.persona.models]
primary = "gemini-2.0-flash-exp"
fallback = "gpt-4o-mini"
```

**After (Self-Hosted with Fallback):**
```toml
[agent]
id = "code-agent"
name = "Code Agent"
description = "Code implementation agent"
prompt_path = "prompts/agents/core/code-agent.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Self-hosted (new)
fallback = "gemini-2.0-flash-exp"  # Cloud (safety net)
premium = "gpt-4o"             # Cloud (when needed)
```

**Steps:**
1. Update agent TOML file
2. Set environment variables
3. Test agent execution
4. Compare outputs with cloud version

### Step 4: Test and Validate

**Functional Testing:**
```bash
# Test agent with same prompts as before
rad run code-agent "Implement a function to sort a list"

# Compare outputs
# - Quality: Is the output acceptable?
# - Completeness: Does it meet requirements?
# - Style: Is it consistent with previous outputs?
```

**Performance Testing:**
```bash
# Measure response time
time rad run code-agent "Test prompt"

# Compare with cloud version
# - Latency: Acceptable delay?
# - Throughput: Can handle load?
```

### Step 5: Monitor and Adjust

**Monitor for issues:**
- Check error rates
- Monitor response times
- Review output quality
- Track fallback usage

**Adjust configuration:**
- Tune model parameters
- Adjust reasoning effort
- Optimize hardware usage
- Fine-tune model selection

## Testing Methodology

### Output Comparison

**Side-by-Side Testing:**
1. Run same prompt on both cloud and self-hosted
2. Compare outputs for:
   - Accuracy
   - Completeness
   - Style consistency
   - Code quality (if applicable)

**Example Test:**
```bash
# Cloud version
rad run code-agent-cloud "Implement quicksort"

# Self-hosted version
rad run code-agent "Implement quicksort"

# Compare outputs
```

### Performance Comparison

**Metrics to Track:**
- **Latency**: Time to first token, total response time
- **Throughput**: Requests per second
- **Resource Usage**: CPU, memory, GPU utilization
- **Cost**: Compare cloud costs vs hardware costs

**Measurement:**
```bash
# Time execution
time rad run <agent> "<prompt>"

# Monitor resources
htop  # CPU/memory
nvidia-smi  # GPU (if applicable)
```

### Quality Assessment

**Criteria:**
- **Correctness**: Is the output correct?
- **Completeness**: Does it address all requirements?
- **Quality**: Is it production-ready?
- **Consistency**: Similar quality to cloud version?

## Rollback Procedure

### Quick Rollback

**Revert agent configuration:**
```bash
# Restore from backup
cp agents-backup-YYYYMMDD/<agent>.toml agents/<agent>.toml

# Or manually edit
# Change engine back to cloud provider
# Change model back to cloud model
```

**Restart Radium:**
```bash
# Restart server if needed
# Or just re-run agent
rad run <agent> "<prompt>"
```

### Full Rollback

**If migration fails completely:**
1. Restore all agent configurations from backup
2. Remove environment variables
3. Restart Radium services
4. Verify cloud agents work

## Post-Migration Optimization

### Performance Tuning

**Optimize model selection:**
- Use smaller models for simple tasks
- Use larger models for complex tasks
- Balance quality vs speed

**Tune parameters:**
```toml
[agent]
reasoning_effort = "medium"  # Adjust based on needs

[agent.persona.performance]
profile = "balanced"  # speed, balanced, thinking, expert
```

### Cost Optimization

**Maximize local usage:**
- Use self-hosted as primary for all agents
- Keep cloud only as premium tier
- Monitor cloud usage to minimize costs

**Hardware optimization:**
- Right-size hardware for workload
- Use GPU when available
- Optimize model selection

### Monitoring Setup

**Set up monitoring:**
- Track model server health
- Monitor agent performance
- Alert on failures
- Track cost savings

## Success Metrics

### Migration Success Indicators

- **Functionality**: All agents work as before
- **Quality**: Output quality maintained or improved
- **Performance**: Acceptable latency and throughput
- **Cost**: Reduced cloud costs
- **Reliability**: Stable operation

### Measuring Success

**Before Migration:**
- Document current cloud costs
- Measure current performance
- Capture sample outputs

**After Migration:**
- Compare costs (cloud vs hardware)
- Compare performance metrics
- Compare output quality
- Track error rates

## Common Migration Issues

### Quality Degradation

**Problem**: Self-hosted model output quality is lower

**Solutions:**
1. Use a larger/better model
2. Adjust reasoning effort
3. Fine-tune prompts
4. Use cloud as fallback for critical tasks

### Performance Issues

**Problem**: Self-hosted model is too slow

**Solutions:**
1. Use a smaller/faster model
2. Optimize hardware (GPU, more RAM)
3. Reduce context length
4. Use cloud for time-sensitive tasks

### Compatibility Issues

**Problem**: Some features don't work with self-hosted models

**Solutions:**
1. Check model capabilities
2. Use cloud for unsupported features
3. Update to newer model version
4. Report issues for future support

## Best Practices

1. **Start Small**: Migrate one agent at a time
2. **Test Thoroughly**: Validate before full migration
3. **Keep Fallback**: Use multi-tier strategy
4. **Monitor Closely**: Watch for issues early
5. **Document Changes**: Keep track of what works
6. **Optimize Gradually**: Fine-tune over time

## Next Steps

- Review [Setup Guides](setup/) for provider installation
- Check [Configuration Guide](configuration/agent-config.md) for agent setup
- See [Troubleshooting Guide](troubleshooting.md) for issues
- Explore [Advanced Configuration](configuration/advanced.md) for optimization

