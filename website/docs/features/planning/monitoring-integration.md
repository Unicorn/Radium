---
id: "monitoring-integration"
title: "Monitoring and Telemetry Integration"
sidebar_label: "Monitoring and Telemetry In..."
---

# Monitoring and Telemetry Integration

Radium automatically tracks token usage, costs, and execution metrics during plan execution. This data helps you understand resource consumption and optimize your workflows.

## Overview

Monitoring integration provides:

- **Token Usage Tracking**: Prompt and completion tokens per task
- **Cost Calculation**: Estimated costs based on engine and model pricing
- **Session Analytics**: Aggregated metrics across all tasks
- **Agent Tracking**: Per-agent execution metrics

## Automatic Telemetry

Telemetry is recorded automatically for each task execution:

```bash
# Execute plan with monitoring
rad craft

# Output includes token usage:
# → Executing I1.T1...
#   • Tokens: 1500 prompt, 800 completion
#   • Progress: 20%
```

## Token Usage

Token usage is tracked from model responses:

- **Input Tokens**: Tokens in the prompt sent to the model
- **Output Tokens**: Tokens in the model's response
- **Total Tokens**: Sum of input and output tokens

### Example

```
Task: I1.T1
  Input Tokens: 1,500
  Output Tokens: 800
  Total Tokens: 2,300
```

## Cost Calculation

Costs are calculated automatically based on:

- **Engine**: OpenAI, Claude, Gemini, etc.
- **Model**: Specific model used (gpt-4, claude-3-sonnet, etc.)
- **Token Usage**: Input and output tokens

### Pricing Examples

**OpenAI GPT-4**:
- Input: $30 per 1M tokens
- Output: $60 per 1M tokens

**Claude 3 Sonnet**:
- Input: $3 per 1M tokens
- Output: $15 per 1M tokens

**Claude 3 Haiku**:
- Input: $0.25 per 1M tokens
- Output: $1.25 per 1M tokens

### Cost Calculation Example

```bash
# Task uses Claude 3 Sonnet
# Input: 1,000 tokens
# Output: 500 tokens

# Cost calculation:
# Input: (1,000 / 1,000,000) * $3 = $0.003
# Output: (500 / 1,000,000) * $15 = $0.0075
# Total: $0.0105
```

## Session Analytics

After plan execution, session analytics are generated:

```bash
# Execute plan
rad craft

# Session report (if monitoring available):
# Session: REQ-123
# Total Tokens: 45,000
# Total Cost: $0.45
# Execution Time: 0:15:30
# Agents Used: code-agent, review-agent
```

## Engine and Model Tracking

Telemetry records which engine and model were used:

```bash
# Telemetry includes:
# - Engine ID: claude
# - Model: claude-3-sonnet
# - Provider: claude
```

This helps track costs and performance across different providers.

## Monitoring Service

Monitoring data is stored in `.radium/monitoring.db`:

```bash
# Monitoring database location
.radium/monitoring.db
```

The monitoring service:
- Stores telemetry records per task
- Tracks agent execution status
- Generates session reports
- Provides historical analytics

## Graceful Degradation

If monitoring is unavailable, execution continues without telemetry:

```bash
# If monitoring.db cannot be created:
rad craft

# Output:
# ⚠ Warning: Failed to record telemetry: <error>
# → Execution continues normally
```

## Viewing Telemetry

Use the monitoring commands to view telemetry:

```bash
# View session metrics
rad monitor session REQ-123

# View agent metrics
rad monitor agent code-agent

# View cost summary
rad stats
```

## Best Practices

1. **Monitor Costs**: Regularly check token usage and costs
2. **Optimize Models**: Use cheaper models when appropriate
3. **Track Trends**: Monitor token usage trends over time
4. **Set Budgets**: Use budget tracking to control spending
5. **Review Analytics**: Review session analytics to optimize workflows

## Cost Optimization

### Use Efficient Models

```bash
# For simple tasks, use cheaper models
rad craft --engine claude --model claude-3-haiku

# For complex tasks, use more capable models
rad craft --engine claude --model claude-3-opus
```

### Monitor Token Usage

```bash
# Check token usage after execution
rad stats

# Output:
# Total Tokens: 45,000
# Total Cost: $0.45
# Average per Task: 2,250 tokens
```

### Set Budget Limits

```bash
# Set daily budget limit
rad budget set --daily 10.00

# Check budget status
rad budget status
```

## Troubleshooting

### Telemetry Not Recording

- Check `.radium/monitoring.db` exists and is writable
- Verify monitoring service initialized successfully
- Check for permission errors in workspace directory

### Incorrect Costs

- Verify engine and model are correctly identified
- Check pricing is up-to-date for your model
- Review token counts for accuracy

### Missing Session Data

- Ensure monitoring service is available
- Check that execution completed successfully
- Verify session ID is correctly tracked

## Telemetry Record Structure

Each telemetry record includes:

```rust
{
  "agent_id": "code-agent",
  "timestamp": 1234567890,
  "input_tokens": 1500,
  "output_tokens": 800,
  "total_tokens": 2300,
  "estimated_cost": 0.0105,
  "model": "claude-3-sonnet",
  "provider": "claude",
  "engine_id": "claude"
}
```

## See Also

- [Execution Modes](./execution-modes.md) - Execution configuration
- [Error Handling](./error-handling.md) - Error tracking
- [Best Practices](./best-practices.md) - Cost optimization

