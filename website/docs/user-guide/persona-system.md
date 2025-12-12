---
id: "persona-system"
title: "Persona System User Guide"
sidebar_label: "Persona System User Guide"
---

# Persona System User Guide

The Persona System provides intelligent model selection, cost optimization, and automatic fallback chains for Radium agents. This guide explains how to use persona metadata to enhance your agents.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Persona Configuration](#persona-configuration)
4. [Performance Profiles](#performance-profiles)
5. [Model Recommendations](#model-recommendations)
6. [Cost Estimation](#cost-estimation)
7. [Budget Management](#budget-management)
8. [CLI Commands](#cli-commands)
9. [Troubleshooting](#troubleshooting)

## Overview

### What is the Persona System?

The Persona System extends agent configuration with metadata that enables:

- **Intelligent Model Selection**: Automatically choose the best model based on task requirements
- **Cost Optimization**: Track and estimate costs for agent executions
- **Fallback Chains**: Gracefully handle model unavailability with automatic fallbacks
- **Performance Profiles**: Match model capabilities to task complexity

### Benefits

- **Automatic Model Selection**: No need to manually specify models for each execution
- **Cost Transparency**: Understand and control AI model costs
- **Reliability**: Automatic fallback when primary models are unavailable
- **Optimization**: Match model performance to task requirements

## Quick Start

### Adding Persona to an Agent

The easiest way to add persona metadata is when creating a new agent:

```bash
rad agents create my-agent --with-persona
```

This generates an agent configuration with a persona template that you can customize.

### Adding Persona to Existing Agents

Edit your agent's TOML configuration file and add a `[agent.persona]` section:

```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "Does something useful"
prompt_path = "prompts/agents/my-category/my-agent.md"

[agent.persona]
[agent.persona.models]
primary = "gemini-2.0-flash-exp"
fallback = "gemini-2.0-flash-thinking"
premium = "gemini-1.5-pro"

[agent.persona.performance]
profile = "balanced"
estimated_tokens = 1500
```

## Persona Configuration

### TOML Format

Persona configuration is added to your agent's TOML file under the `[agent.persona]` section:

```toml
[agent.persona]
[agent.persona.models]
primary = "gemini-2.0-flash-exp"
fallback = "gemini-2.0-flash-thinking"  # Optional
premium = "gemini-1.5-pro"              # Optional

[agent.persona.performance]
profile = "balanced"                    # speed, balanced, thinking, or expert
estimated_tokens = 1500                 # Optional
```

### Model Format

Models can be specified in two formats:

1. **Simple format** (uses agent's engine):
   ```toml
   primary = "gemini-2.0-flash-exp"
   ```

2. **Full format** (explicit engine):
   ```toml
   primary = "gemini:gemini-2.0-flash-exp"
   ```

### Required Fields

- `primary`: The primary recommended model (required)

### Optional Fields

- `fallback`: Model to use if primary is unavailable
- `premium`: Premium model for critical tasks
- `profile`: Performance profile (defaults to "balanced")
- `estimated_tokens`: Estimated token usage per execution

## Performance Profiles

Performance profiles help match model capabilities to task requirements:

### Speed

Optimized for fast responses and lower costs. Best for:
- Simple tasks
- High-volume operations
- Cost-sensitive applications

**Example:**
```toml
[agent.persona.performance]
profile = "speed"
```

### Balanced

Balanced speed and quality. Best for:
- General-purpose tasks
- Code generation
- Documentation

**Example:**
```toml
[agent.persona.performance]
profile = "balanced"
```

### Thinking

Optimized for deep reasoning. Best for:
- Complex problem-solving
- Architecture design
- Planning and analysis

**Example:**
```toml
[agent.persona.performance]
profile = "thinking"
```

### Expert

Expert-level reasoning, highest cost. Best for:
- Critical decisions
- Complex analysis
- Premium features

**Example:**
```toml
[agent.persona.performance]
profile = "expert"
```

## Model Recommendations

### Choosing Models

When selecting models for your persona configuration:

1. **Primary Model**: Choose based on performance profile
   - Speed: Fast models (e.g., `gemini-2.0-flash-exp`)
   - Balanced: General models (e.g., `gemini-2.0-flash-exp`)
   - Thinking: Reasoning models (e.g., `gemini-2.0-flash-thinking`)
   - Expert: Premium models (e.g., `gemini-1.5-pro`)

2. **Fallback Model**: Choose a reliable alternative
   - Should be available when primary might not be
   - Can be a different performance tier

3. **Premium Model**: Choose for critical tasks
   - Highest quality option
   - Used when explicitly requested or primary/fallback unavailable

### Example Configurations

**Speed-Optimized Agent:**
```toml
[agent.persona]
[agent.persona.models]
primary = "gemini-2.0-flash-exp"
fallback = "gemini-2.0-flash-thinking"
premium = "gemini-1.5-pro"

[agent.persona.performance]
profile = "speed"
estimated_tokens = 1000
```

**Thinking Agent:**
```toml
[agent.persona]
[agent.persona.models]
primary = "gemini-2.0-flash-thinking"
fallback = "gemini-2.0-flash-exp"
premium = "gemini-1.5-pro"

[agent.persona.performance]
profile = "thinking"
estimated_tokens = 2000
```

## Cost Estimation

### Understanding Costs

Costs are calculated based on:
- **Input Tokens**: Tokens in the prompt
- **Output Tokens**: Tokens in the response
- **Model Pricing**: Per-token pricing from the model provider

### Viewing Cost Estimates

Use the `rad agents cost` command to see cost estimates:

```bash
rad agents cost my-agent
```

You can also specify expected token counts:

```bash
rad agents cost my-agent --input-tokens 2000 --output-tokens 1000
```

### Cost Breakdown

The cost command shows:
- Estimated costs for primary, fallback, and premium models
- Token estimates (input, output, total)
- Cost per model in the fallback chain

## Budget Management

### Setting a Budget

Set a budget limit to track spending:

```bash
rad budget set 100.00
```

This sets a budget of $100.00 USD.

### Viewing Budget Status

Check your current budget usage:

```bash
rad budget status
```

This shows:
- Budget limit
- Amount spent
- Remaining budget
- Usage percentage
- Status (active, warning, exceeded)

### Resetting Budget

Reset budget tracking (keeps the limit):

```bash
rad budget reset
```

## CLI Commands

### View Agent Persona

Show persona configuration for an agent:

```bash
rad agents persona <agent-id>
```

### View Agent Info (with Persona)

Show full agent information including persona:

```bash
rad agents info <agent-id>
```

### List Agents by Profile

Filter agents by performance profile:

```bash
rad agents list --profile thinking
```

Valid profiles: `speed`, `balanced`, `thinking`, `expert`

### Estimate Costs

Show cost estimates for an agent:

```bash
rad agents cost <agent-id>
rad agents cost <agent-id> --input-tokens 2000 --output-tokens 1000
```

### Budget Commands

```bash
rad budget set <amount>      # Set budget limit
rad budget status            # Show budget status
rad budget reset             # Reset budget tracking
```

## Troubleshooting

### "No persona configuration found"

**Problem**: Agent doesn't have persona metadata.

**Solution**: Add persona configuration to the agent's TOML file or use `rad agents create --with-persona` when creating new agents.

### "Model not available"

**Problem**: Selected model is unavailable.

**Solution**: The system will automatically try fallback models. Ensure your fallback chain is configured correctly.

### "Invalid performance profile"

**Problem**: Profile value is not recognized.

**Solution**: Use one of: `speed`, `balanced`, `thinking`, or `expert`.

### Cost Estimates Seem Incorrect

**Problem**: Cost estimates don't match actual costs.

**Solution**: 
- Check that `estimated_tokens` is set correctly
- Verify model pricing is up to date
- Use `--input-tokens` and `--output-tokens` flags for more accurate estimates

### Budget Not Tracking

**Problem**: Budget status shows no spending.

**Solution**: Budget tracking is currently in-memory. Future versions will add persistent tracking.

## Best Practices

1. **Set Appropriate Profiles**: Match performance profiles to task complexity
2. **Configure Fallbacks**: Always set a fallback model for reliability
3. **Estimate Tokens**: Set `estimated_tokens` for accurate cost estimates
4. **Use Budgets**: Set budget limits to control costs
5. **Test Fallback Chains**: Verify fallback models work correctly

## Examples

See the core agents for examples:
- `agents/core/arch-agent.toml` - Thinking profile
- `agents/core/plan-agent.toml` - Thinking profile
- `agents/core/code-agent.toml` - Balanced profile
- `agents/core/review-agent.toml` - Balanced profile
- `agents/core/doc-agent.toml` - Speed profile

## Further Reading

- [Agent Creation Guide](../guides/agent-creation-guide.md) - Complete guide to creating agents
- [Persona System Architecture](../../docs/design/persona-system-architecture.md) - Technical architecture details

