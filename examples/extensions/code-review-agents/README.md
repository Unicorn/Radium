# Code Review Agents Extension

A collection of specialized AI agents for code review across multiple programming languages.

## Installation

```bash
rad extension install ./examples/extensions/code-review-agents
```

## Components

### Agents

This extension provides three language-specific code review agents:

- **rust-reviewer** - Reviews Rust code for safety, performance, and idiomatic patterns
- **typescript-reviewer** - Reviews TypeScript code for type safety and modern patterns
- **python-reviewer** - Reviews Python code for Pythonic idioms and best practices

## Usage

After installation, the agents will be available via the agent discovery system:

```bash
# List available agents
rad agents list

# Use a specific reviewer
rad agents info rust-reviewer
```

## Agent Prompts

Each agent has a specialized prompt template in `prompts/review/`:

- `rust-reviewer.md` - Rust-specific review guidelines
- `typescript-reviewer.md` - TypeScript-specific review guidelines
- `python-reviewer.md` - Python-specific review guidelines

## Example

```bash
# The agents will be discoverable and can be used in workflows
# or directly via the agent system
```

## See Also

- [Extension System Guide](../../../docs/extensions/README.md)
- [Creating Extensions](../../../docs/extensions/creating-extensions.md)

