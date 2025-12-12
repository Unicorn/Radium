---
id: "reasoning-configuration"
title: "Reasoning Configuration Guide"
sidebar_label: "Reasoning Configuration Guide"
---

# Reasoning Configuration Guide

This guide explains how to configure reasoning effort for AI models in Radium.

## Configuration Methods

### 1. CLI Flag (Highest Priority)

Override reasoning effort for a single execution:

```bash
rad step my-agent "Prompt" --reasoning high
```

**Options:**
- `--reasoning low`: Minimal reasoning
- `--reasoning medium`: Standard reasoning (default)
- `--reasoning high`: Maximum reasoning

### 2. Agent Configuration

Set default reasoning effort in agent TOML file:

```toml
[agent]
id = "my-agent"
name = "My Agent"
reasoning_effort = "high"  # low, medium, or high
```

This becomes the default for all executions unless overridden by CLI flag.

### 3. Persona Configuration

Configure reasoning through performance profiles:

```toml
[agent.persona]
performance.profile = "thinking"  # speed, balanced, thinking, or expert
```

Performance profiles map to reasoning capabilities:
- **speed**: Fast models, lower reasoning
- **balanced**: Balanced speed and quality (default)
- **thinking**: Optimized for deep reasoning
- **expert**: Expert-level reasoning, highest cost

## Precedence Chain

Reasoning effort is resolved in this order:

1. CLI flag (`--reasoning`) - **Highest priority**
2. Agent config (`reasoning_effort` in TOML)
3. Default (`medium`) - **Lowest priority**

## Examples

### Example 1: Agent with High Reasoning

```toml
[agent]
id = "math-solver"
name = "Math Problem Solver"
reasoning_effort = "high"
engine = "gemini"
model = "gemini-2.0-flash-thinking"
```

Usage:
```bash
rad step math-solver "Solve: x^2 + 5x + 6 = 0"
# Uses high reasoning from config
```

### Example 2: CLI Override

```toml
[agent]
id = "general-agent"
reasoning_effort = "low"  # Default to low
```

Usage:
```bash
rad step general-agent "Complex problem" --reasoning high
# CLI flag overrides config, uses high reasoning
```

### Example 3: Default Behavior

```toml
[agent]
id = "simple-agent"
# No reasoning_effort specified
```

Usage:
```bash
rad step simple-agent "Simple question"
# Uses default (medium) reasoning
```

## Provider-Specific Configuration

### Gemini

Gemini thinking models (e.g., `gemini-2.0-flash-thinking`) support thinking mode:

```toml
[agent]
id = "gemini-thinking-agent"
engine = "gemini"
model = "gemini-2.0-flash-thinking"
reasoning_effort = "high"
```

The reasoning effort maps to Gemini's `thinkingConfig.thinking_budget`:
- Low: 0.3 (minimal thinking)
- Medium: 0.6 (standard thinking)
- High: 1.0 (maximum thinking)

### Claude

Claude models support extended thinking:

```toml
[agent]
id = "claude-thinking-agent"
engine = "claude"
model = "claude-3-opus"
reasoning_effort = "high"
```

The reasoning effort maps to Claude's `thinking.thinking_budget`:
- Low: 0.3 (minimal extended thinking)
- Medium: 0.6 (standard extended thinking)
- High: 1.0 (maximum extended thinking)

## Cost Considerations

Reasoning effort directly impacts cost:

| Model | Standard | Thinking (High) | Multiplier |
|-------|----------|-----------------|------------|
| Gemini Flash Exp | $0.075/$0.30 | - | - |
| Gemini Flash Thinking | - | $0.20/$0.80 | ~2.7x |
| Claude Sonnet | $3.00/$15.00 | $3.00/$15.00* | ~1.0x* |
| Claude Opus | $15.00/$75.00 | $15.00/$75.00* | ~1.0x* |

*Claude models use extended thinking which increases token usage, effectively increasing cost per request.

## Best Practices

1. **Use appropriate reasoning levels**:
   - Simple tasks: `low` or `medium`
   - Complex problems: `high`

2. **Monitor costs**:
   - Check token usage with `--show-metadata`
   - Use thinking models only when needed

3. **Combine with model selection**:
   - Use thinking models for complex tasks
   - Use standard models for simple tasks

4. **Test reasoning levels**:
   - Start with `medium` and adjust based on results
   - Use `high` only when necessary

## Troubleshooting

### Reasoning effort not taking effect

- Verify model supports thinking mode (check model name)
- Check precedence: CLI flag overrides config
- Ensure reasoning effort is spelled correctly (`low`, `medium`, `high`)

### Unexpected costs

- Reduce reasoning effort level
- Use standard models for simple tasks
- Monitor token usage in metadata

### Slow performance

- Lower reasoning effort for faster responses
- Use streaming mode for real-time output
- Consider using faster models for time-sensitive tasks

## See Also

- [Thinking Mode Feature](../features/thinking-mode.md)
- [Agent Configuration](../user-guide/agent-configuration.md)
- [CLI Reference](../cli-reference.md)

