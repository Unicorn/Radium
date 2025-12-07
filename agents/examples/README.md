# Agent Configuration Examples

This directory contains example agent configurations demonstrating various use cases and configuration patterns for Radium agents.

## Overview

Each example includes:
- A TOML configuration file (`.toml`)
- A corresponding prompt template (`.md` in `prompts/agents/examples/`)

## Example Agents

### 1. code-gen-agent
**Purpose:** Fast code generation optimized for speed and iteration  
**Use Case:** When you need to generate code quickly for rapid prototyping  
**Features:** Fast model class, low cost tier, high concurrency  
**Configuration:** `code-gen-agent.toml`

### 2. test-gen-agent
**Purpose:** Generates comprehensive tests with balanced quality and speed  
**Use Case:** Creating test suites that balance coverage and execution time  
**Features:** Balanced model class, medium cost tier  
**Configuration:** `test-gen-agent.toml`

### 3. security-agent
**Purpose:** Deep security analysis with high reasoning effort  
**Use Case:** Identifying security vulnerabilities and threats  
**Features:** Reasoning model class, high cost tier, low concurrency  
**Configuration:** `security-agent.toml`

### 4. doc-gen-agent
**Purpose:** Generates comprehensive documentation with high output volume  
**Use Case:** Creating detailed documentation for APIs, libraries, and systems  
**Features:** Balanced model class, medium cost tier, high concurrency  
**Configuration:** `doc-gen-agent.toml`

### 5. refactor-agent
**Purpose:** Refactors code with loop behavior for iterative improvement  
**Use Case:** Improving code quality through incremental refinement  
**Features:** Loop behavior (steps=2, max_iterations=3)  
**Configuration:** `refactor-agent.toml`

### 6. integration-agent
**Purpose:** Handles integrations with trigger behavior for dynamic agent delegation  
**Use Case:** Coordinating work across systems and triggering specialized agents  
**Features:** Trigger behavior for dynamic agent delegation  
**Configuration:** `integration-agent.toml`

### 7. data-analysis-agent
**Purpose:** Analyzes data with rich metadata recommendations  
**Use Case:** Deep data analysis with advanced reasoning  
**Features:** YAML frontmatter with model recommendations  
**Configuration:** `data-analysis-agent.toml`

### 8. api-design-agent
**Purpose:** Designs APIs with sandbox configuration for safe testing  
**Use Case:** Creating and testing APIs in isolated environments  
**Features:** Docker sandbox configuration  
**Configuration:** `api-design-agent.toml`

### 9. perf-opt-agent
**Purpose:** Optimizes code and system performance  
**Use Case:** Identifying and fixing performance bottlenecks  
**Features:** Reasoning model class, high cost tier  
**Configuration:** `perf-opt-agent.toml`

### 10. strict-review-agent
**Purpose:** Performs strict code review with comprehensive validation  
**Use Case:** Ensuring high code quality through thorough reviews  
**Features:** High reasoning effort, strict validation  
**Configuration:** `strict-review-agent.toml`

## Using Examples

### Copy an Example

To use an example as a starting point:

```bash
# Copy the configuration
cp agents/examples/code-gen-agent.toml agents/my-category/my-agent.toml

# Copy the prompt template
cp prompts/agents/examples/code-gen-agent.md prompts/agents/my-category/my-agent.md

# Edit both files to customize for your needs
```

### Validate Examples

All examples should pass validation:

```bash
rad agents validate
```

### Customize for Your Needs

1. **Update the agent ID and name** to match your use case
2. **Modify the prompt template** with your specific instructions
3. **Adjust capabilities** (model_class, cost_tier, max_concurrent_tasks) based on your requirements
4. **Add or remove behaviors** (loop, trigger) as needed
5. **Configure sandbox** if you need isolated execution

## Configuration Patterns

### Fast Iteration Pattern
Use `code-gen-agent` as a template for agents that need to iterate quickly:
- `model_class = "fast"`
- `cost_tier = "low"`
- `max_concurrent_tasks = 20`

### Deep Reasoning Pattern
Use `security-agent` or `perf-opt-agent` as templates for complex analysis:
- `model_class = "reasoning"`
- `cost_tier = "high"`
- `reasoning_effort = "high"`
- `max_concurrent_tasks = 3`

### Balanced Pattern
Use `test-gen-agent` or `doc-gen-agent` for balanced quality and speed:
- `model_class = "balanced"`
- `cost_tier = "medium"`
- `reasoning_effort = "medium"`

### Iterative Refinement Pattern
Use `refactor-agent` for agents that need to loop back:
- Add `[agent.loop_behavior]` section
- Set `steps` and `max_iterations` appropriately

### Multi-Agent Coordination Pattern
Use `integration-agent` for agents that trigger others:
- Add `[agent.trigger_behavior]` section
- Set `trigger_agent_id` for default delegation

## Best Practices

1. **Start Simple:** Begin with `basic-agent.toml` template and add features as needed
2. **Match Model to Task:** Use fast models for simple tasks, reasoning models for complex analysis
3. **Set Appropriate Concurrency:** Higher concurrency for fast agents, lower for reasoning agents
4. **Test Thoroughly:** Validate your agent configuration before using in production
5. **Document Your Agents:** Add clear descriptions and examples in prompt templates

## Further Reading

- [Agent Configuration Guide](../../docs/user-guide/agent-configuration.md)
- [Agent Creation Guide](../../docs/guides/agent-creation-guide.md)
- [Agent System Architecture](../../docs/developer-guide/agent-system-architecture.md)

