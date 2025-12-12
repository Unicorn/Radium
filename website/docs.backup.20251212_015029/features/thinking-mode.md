# Thinking Mode for Complex Reasoning

Thinking mode enables AI models to show their reasoning process before providing final answers. This feature is particularly useful for complex problems that require deep analysis, mathematical reasoning, or multi-step problem solving.

## Overview

When thinking mode is enabled, models perform internal reasoning before generating their final response. This reasoning process is captured and can be displayed to help users understand how the model arrived at its answer.

## Benefits

- **Better Answers**: Models take more time to think through complex problems
- **Transparency**: See the reasoning steps the model used
- **Debugging**: Understand model behavior and decision-making process
- **Learning**: Learn problem-solving approaches from the model's reasoning

## Supported Models

### Gemini Models
- `gemini-2.0-flash-thinking`: Optimized for deep reasoning with thinking mode

### Claude Models
- `claude-3-opus`: Highest capability with extended thinking support
- `claude-3-sonnet`: Balanced performance with extended thinking support

## Configuration Methods

Thinking mode can be configured in three ways, with the following precedence:

1. **CLI Flag** (highest priority): `--reasoning <level>`
2. **Agent Configuration**: `reasoning_effort` in agent TOML file
3. **Default**: `medium` (if not specified)

### Reasoning Effort Levels

- **Low**: Minimal reasoning effort for simple tasks
- **Medium**: Moderate reasoning for balanced performance (default)
- **High**: Maximum reasoning effort for complex problems

## Usage Examples

### CLI Configuration

```bash
# Use high reasoning effort for complex problem
rad step my-agent "Solve this complex math problem" --reasoning high

# Use low reasoning for simple tasks
rad step my-agent "What is 2+2?" --reasoning low
```

### Agent Configuration

```toml
[agent]
id = "math-agent"
name = "Math Problem Solver"
reasoning_effort = "high"  # Always use high reasoning for this agent
```

### Viewing Thinking Process

To see the model's thinking process, use the `--show-metadata` flag:

```bash
rad step my-agent "Complex problem" --reasoning high --show-metadata
```

The thinking process will appear in the metadata section, displayed before other metadata like token usage.

## Cost Implications

Thinking models typically cost 2-3x more than standard models due to increased token usage:

- **Gemini Flash Thinking**: $0.20/$0.80 per 1M tokens (vs $0.075/$0.30 for Flash Exp)
- **Claude Opus**: $15.00/$75.00 per 1M tokens (supports extended thinking)
- **Claude Sonnet**: $3.00/$15.00 per 1M tokens (supports extended thinking)

Higher reasoning effort levels result in more thinking tokens, increasing overall cost.

## When to Use Thinking Mode

**Use thinking mode when:**
- Solving complex mathematical problems
- Performing multi-step reasoning
- Analyzing complex code or systems
- Making important decisions that require careful consideration
- Debugging or understanding model behavior

**Don't use thinking mode when:**
- Simple questions or straightforward tasks
- High-volume processing where cost matters
- Real-time applications requiring fast responses
- Tasks that don't benefit from deep reasoning

## Troubleshooting

### Thinking process not appearing

- **Check model support**: Ensure you're using a thinking model (e.g., `gemini-2.0-flash-thinking`)
- **Verify reasoning effort**: Make sure reasoning effort is set (CLI flag or agent config)
- **Check metadata flag**: Use `--show-metadata` to display thinking process
- **Non-thinking models**: Regular models won't show thinking process even with reasoning effort set

### High costs

- **Reduce reasoning effort**: Use `low` or `medium` instead of `high`
- **Use standard models**: Switch to non-thinking models for simple tasks
- **Monitor token usage**: Check metadata to see actual token consumption

### Slow responses

- **Expected behavior**: Thinking mode takes longer as models perform internal reasoning
- **Reduce reasoning effort**: Lower reasoning effort levels are faster
- **Use streaming**: Consider using `--stream` flag for real-time output

## Technical Details

### How It Works

1. User specifies reasoning effort (CLI, config, or default)
2. Reasoning effort is resolved through precedence chain
3. Value is passed to `ModelParameters.reasoning_effort`
4. Provider-specific APIs map reasoning effort to thinking configuration:
   - Gemini: Maps to `thinkingConfig.thinking_budget` (0.3/0.6/1.0)
   - Claude: Maps to `thinking.thinking_budget` (0.3/0.6/1.0)
5. Model performs thinking process
6. Thinking process is extracted from response and stored in metadata
7. CLI displays thinking process when `--show-metadata` is used

### Provider-Specific Implementation

**Gemini**: Uses `thinkingConfig` field in generation config. Thinking process is returned in response `thinking` field.

**Claude**: Uses `thinking` field in request. Thinking process is returned in response `thinking` field.

## See Also

- [Reasoning Configuration Guide](reasoning-configuration.md)
- [Agent Configuration](../user-guide/agent-configuration.md)
- [CLI Reference](../cli-reference.md)

