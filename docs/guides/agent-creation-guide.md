# Agent Creation Guide

Complete guide for creating and porting agents in the Radium platform.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Agent Structure](#agent-structure)
3. [Configuration File Format](#configuration-file-format)
4. [Prompt File Guidelines](#prompt-file-guidelines)
5. [Agent Categories](#agent-categories)
6. [Creating an Agent](#creating-an-agent)
7. [Testing and Validation](#testing-and-validation)
8. [Best Practices](#best-practices)

## Quick Start

Create a new agent in 3 steps:

```bash
# 1. Generate agent template
rad agents create my-agent "My Agent" \
  --description "Brief description" \
  --category "custom" \
  --engine "gemini" \
  --model "gemini-2.0-flash-exp"

# 2. Edit the prompt file
vim prompts/agents/custom/my-agent.md

# 3. Validate and test
rad agents validate
rad agents info my-agent
```

## Agent Structure

Every agent consists of two files:

```
project-root/
├── agents/                    # Agent configurations
│   └── {category}/           # Organized by category
│       └── {agent-id}.toml   # TOML configuration file
└── prompts/                   # Agent prompts
    └── agents/
        └── {category}/
            └── {agent-id}.md  # Markdown prompt file
```

### Example Structure

```
.
├── agents/
│   ├── core/
│   │   ├── arch-agent.toml
│   │   ├── plan-agent.toml
│   │   └── code-agent.toml
│   └── custom/
│       └── my-agent.toml
└── prompts/
    └── agents/
        ├── core/
        │   ├── arch-agent.md
        │   ├── plan-agent.md
        │   └── code-agent.md
        └── custom/
            └── my-agent.md
```

## Configuration File Format

Agent configuration files use TOML format:

### Basic Configuration

```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "A brief description of what this agent does"
prompt_path = "prompts/agents/custom/my-agent.md"
```

### Full Configuration

```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "A comprehensive agent for X task"
prompt_path = "prompts/agents/custom/my-agent.md"

# Optional: Default engine and model
engine = "gemini"                    # Options: gemini, openai, claude
model = "gemini-2.0-flash-exp"      # Model name
reasoning_effort = "medium"          # Options: low, medium, high

# Optional: Loop behavior (allows agent to request looping back)
[agent.loop_behavior]
steps = 2                            # Number of steps to loop back
max_iterations = 5                   # Maximum loop iterations
skip = ["step-1", "step-3"]         # Steps to skip during loop

# Optional: Trigger behavior (allows agent to trigger other agents)
[agent.trigger_behavior]
trigger_agent_id = "fallback-agent"  # Default agent to trigger
```

### Field Descriptions

| Field | Required | Description |
|-------|----------|-------------|
| `id` | ✅ Yes | Unique identifier (lowercase, hyphens, no spaces) |
| `name` | ✅ Yes | Human-readable agent name |
| `description` | ✅ Yes | Brief description of agent's purpose |
| `prompt_path` | ✅ Yes | Relative path to prompt markdown file |
| `engine` | ⚪ Optional | Default AI engine (gemini, openai, claude) |
| `model` | ⚪ Optional | Default model name |
| `reasoning_effort` | ⚪ Optional | Reasoning level (low, medium, high) |
| `loop_behavior` | ⚪ Optional | Loop configuration for iterative workflows |
| `trigger_behavior` | ⚪ Optional | Trigger configuration for dynamic workflows |

## Prompt File Guidelines

Prompt files define the agent's behavior, instructions, and output format.

### Recommended Structure

```markdown
# Agent Name

Brief one-line description.

## Role

Clear description of the agent's role and purpose.

## Capabilities

- List core capabilities
- What can this agent do?
- What are its strengths?

## Input

Describe what inputs this agent expects:
- Context from previous steps
- Required parameters
- Optional configuration

## Output

Describe what this agent produces:
- Output format (markdown, JSON, code, etc.)
- Key deliverables
- Success criteria

## Instructions

Step-by-step instructions for the agent:

1. **Step Name** - Description of what to do
2. **Next Step** - Continue the process
3. **Final Step** - Wrap up

## Examples

### Example 1: [Scenario Name]

**Input:**
```
Sample input here
```

**Expected Output:**
```
Sample output here
```

## Notes

- Important considerations
- Edge cases to handle
- Best practices to follow
```

### Prompt Writing Best Practices

1. **Be Specific**: Clear, unambiguous instructions
2. **Use Examples**: Show desired output format with real examples
3. **Structure Output**: Define clear output format templates
4. **Handle Errors**: Instruct agent on error handling
5. **Set Constraints**: Define boundaries and limitations
6. **Include Context**: Explain why certain approaches are preferred

## Agent Categories

Organize agents into logical categories:

### Core Categories

- **core**: Platform-essential agents (arch, plan, code, review, doc)
- **dev**: Development-specific agents
- **ops**: Operations and deployment agents
- **test**: Testing and QA agents
- **data**: Data processing and analysis agents
- **custom**: User-defined custom agents

### Creating New Categories

Simply use `--category` flag when creating an agent:

```bash
rad agents create ml-agent "ML Model Agent" --category "ml"
```

This automatically creates the `agents/ml/` and `prompts/agents/ml/` directories.

## Creating an Agent

### Method 1: Using CLI Generator (Recommended)

```bash
rad agents create {agent-id} "{Agent Name}" \
  --description "What this agent does" \
  --category "category-name" \
  --engine "gemini" \
  --model "gemini-2.0-flash-exp" \
  --reasoning "medium"
```

**All flags:**
- `--description, -d`: Agent description (optional)
- `--category, -c`: Category name (default: "custom")
- `--engine, -e`: AI engine (gemini, openai, claude)
- `--model, -m`: Model name
- `--reasoning, -r`: Reasoning effort (low, medium, high)
- `--output, -o`: Output directory (default: "./agents")

### Method 2: Manual Creation

1. **Create TOML file**: `agents/category/agent-id.toml`
2. **Create prompt file**: `prompts/agents/category/agent-id.md`
3. **Validate**: `rad agents validate`

## Testing and Validation

### Validation

```bash
# Validate all agents
rad agents validate

# Verbose validation with details
rad agents validate --verbose
```

**Validation checks:**
- ✅ Agent ID is not empty
- ✅ Agent name is not empty
- ✅ Prompt path is not empty
- ✅ Prompt file exists (if path is set)
- ✅ TOML syntax is valid

### Listing Agents

```bash
# List all agents (table format)
rad agents list

# List with details
rad agents list --verbose

# JSON output
rad agents list --json
```

### Agent Information

```bash
# Show detailed agent info
rad agents info agent-id

# JSON output
rad agents info agent-id --json
```

### Search Agents

```bash
# Search by name, description, or category
rad agents search "keyword"

# JSON output
rad agents search "keyword" --json
```

## Best Practices

### Naming Conventions

**Agent IDs:**
- Use lowercase
- Use hyphens to separate words
- Be descriptive but concise
- Examples: `arch-agent`, `plan-agent`, `api-doc-agent`

**Agent Names:**
- Use Title Case
- Be descriptive
- Examples: "Architecture Agent", "API Documentation Agent"

### Organization

**By Function:**
```
agents/
├── architecture/     # Architecture-related agents
├── planning/         # Planning and task breakdown
├── implementation/   # Code implementation
├── review/          # Code and design review
└── documentation/   # Documentation generation
```

**By Project Phase:**
```
agents/
├── discovery/    # Requirements and discovery
├── design/       # Design and architecture
├── development/  # Implementation
├── testing/      # Testing and QA
└── deployment/   # Deployment and ops
```

### Prompt Engineering

1. **Start with Role**: Clearly define what the agent is
2. **List Capabilities**: Be specific about what it can do
3. **Provide Context**: Explain inputs and outputs
4. **Give Instructions**: Step-by-step process
5. **Show Examples**: Real-world input/output pairs
6. **Add Constraints**: Define boundaries and limitations

### Versioning

When updating agents:

1. **Minor changes**: Update prompt directly
2. **Major changes**: Consider creating v2 agent (agent-name-v2)
3. **Breaking changes**: Create new agent with clear migration path

### Testing Strategy

1. **Validate**: Run `rad agents validate` after creation
2. **Review**: Check prompt clarity and completeness
3. **Test**: Run agent with sample inputs
4. **Iterate**: Refine based on results
5. **Document**: Update examples with real outputs

## Example: Creating a Test Agent

Complete walkthrough of creating a test generation agent:

```bash
# 1. Create agent
rad agents create test-gen-agent "Test Generation Agent" \
  --description "Generates unit and integration tests" \
  --category "testing" \
  --engine "gemini" \
  --reasoning "medium"

# 2. Edit prompt file
vim prompts/agents/testing/test-gen-agent.md
```

**Prompt content:**
```markdown
# Test Generation Agent

Generates comprehensive unit and integration tests for code.

## Role

You are an expert test engineer who writes thorough, maintainable tests
following TDD best practices.

## Instructions

1. **Analyze Code** - Read and understand the code to be tested
2. **Identify Test Cases** - List all scenarios, edge cases, errors
3. **Write Tests** - Generate test code with clear names
4. **Verify Coverage** - Ensure all paths and cases are tested

## Output Format

```
### Test File: {filename}_test.rs

```rust
// Test code here
```
```

**Validation:**
```bash
# 3. Validate
rad agents validate

# 4. Test
rad agents info test-gen-agent

# 5. Use in workflow
# (agent is now available for workflow execution)
```

## Troubleshooting

### Agent Not Found

```bash
rad agents list  # Check if agent is listed
rad agents validate --verbose  # Check for errors
```

**Solution**: Ensure TOML file is in correct directory with correct structure.

### Prompt File Not Found

**Error**: "Prompt file not found"

**Solution**: Check `prompt_path` in TOML matches actual file location.

### Validation Errors

```bash
rad agents validate --verbose  # See detailed errors
```

Common issues:
- Missing required fields (id, name, description, prompt_path)
- Invalid TOML syntax
- Incorrect file paths

## Advanced Topics

### Loop Behavior

Allows agents to request looping back to previous steps:

```toml
[agent.loop_behavior]
steps = 2                    # Loop back 2 steps
max_iterations = 5           # Max 5 iterations
skip = ["init-step"]        # Skip these steps in loop
```

**Use case**: Iterative refinement agents that improve output over multiple passes.

### Trigger Behavior

Allows agents to dynamically trigger other agents:

```toml
[agent.trigger_behavior]
trigger_agent_id = "specialist-agent"
```

**Use case**: General agents that delegate to specialized agents based on context.

## Resources

- **Example Agents**: See `agents/core/` for well-structured examples
- **CLI Help**: Run `rad agents --help` for command reference
- **Validation**: Always run `rad agents validate` before committing

## Next Steps

1. ✅ Create your first agent with `rad agents create`
2. ✅ Customize the prompt for your specific use case
3. ✅ Validate with `rad agents validate`
4. ✅ Test with sample inputs
5. ✅ Integrate into your workflow
