# Agent Creation Guide

This comprehensive guide will help you create custom AI agents for Radium. Agents are specialized AI assistants that perform specific tasks within your workflows. This guide covers everything from basic configuration to advanced patterns and best practices.

## Table of Contents

1. [Introduction](#introduction)
2. [Agent Configuration Format](#agent-configuration-format)
3. [Prompt Template Structure](#prompt-template-structure)
4. [Agent Discovery and Organization](#agent-discovery-and-organization)
5. [Creating Your First Agent](#creating-your-first-agent)
6. [Common Agent Patterns](#common-agent-patterns)
7. [Advanced Configuration](#advanced-configuration)
8. [Best Practices](#best-practices)
9. [Testing and Validation](#testing-and-validation)
10. [Troubleshooting](#troubleshooting)

## Introduction

### What is an Agent?

An agent in Radium is a specialized AI assistant configured to perform specific tasks. Each agent consists of:

- **Configuration file** (`.toml`): Defines the agent's identity, capabilities, and behavior
- **Prompt template** (`.md`): Contains the instructions that guide the agent's behavior

Agents are automatically discovered from configured directories and can be used in workflows, executed directly via CLI, or integrated into your development process.

### Why Create Custom Agents?

- **Specialization**: Create agents tailored to your specific domain or use case
- **Consistency**: Ensure consistent behavior across projects and teams
- **Reusability**: Share agents across multiple projects
- **Optimization**: Configure agents with optimal models and settings for their tasks

## Agent Configuration Format

### Basic Structure

Agent configurations are written in TOML format and stored in `.toml` files. The basic structure is:

```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "A description of what this agent does"
prompt_path = "prompts/agents/my-category/my-agent.md"
```

### Required Fields

#### `id` (string)

A unique identifier for the agent. Use kebab-case (lowercase with hyphens).

**Examples:**
- `arch-agent` ✅
- `code-review-agent` ✅
- `myAgent` ❌ (use kebab-case)
- `agent 1` ❌ (no spaces)

**Best Practices:**
- Use descriptive names that indicate the agent's purpose
- Keep IDs concise but clear
- Use consistent naming conventions across your agents

#### `name` (string)

A human-readable name for the agent displayed in CLI and UI.

**Examples:**
- `"Architecture Agent"`
- `"Code Review Agent"`
- `"Documentation Generator"`

#### `description` (string)

A brief description of what the agent does. This helps users discover and understand the agent's purpose.

**Examples:**
- `"Defines system architecture and technical design decisions"`
- `"Reviews code for quality, security, and best practices"`
- `"Generates comprehensive documentation and API references"`

#### `prompt_path` (PathBuf)

The path to the markdown file containing the agent's prompt template. Can be absolute or relative to the workspace root.

**Examples:**
- `"prompts/agents/core/arch-agent.md"` (relative)
- `"/absolute/path/to/prompt.md"` (absolute)

**Path Resolution:**
- Relative paths are resolved from the workspace root
- If the prompt file is in the same directory structure as the config, use relative paths
- The path must point to an existing markdown file

### Optional Fields

#### `engine` (string)

The default AI engine to use for this agent. If not specified, the engine must be provided at runtime.

**Supported Engines:**
- `"gemini"` - Google Gemini models
- `"openai"` - OpenAI models (GPT-4, GPT-3.5)
- `"claude"` - Anthropic Claude models
- `"codex"` - OpenAI Codex models

**Example:**
```toml
engine = "gemini"
```

#### `model` (string)

The specific model to use. Must be compatible with the specified engine.

**Examples:**
- `"gemini-2.0-flash-exp"` (Gemini)
- `"gpt-4"` (OpenAI)
- `"claude-3-opus-20240229"` (Anthropic)

**Example:**
```toml
model = "gemini-2.0-flash-exp"
```

#### `reasoning_effort` (string)

The default reasoning effort level. Controls how much computational effort the model should use for reasoning.

**Valid Values:**
- `"low"` - Minimal reasoning, faster responses
- `"medium"` - Balanced reasoning (default)
- `"high"` - Maximum reasoning, slower but more thorough

**Example:**
```toml
reasoning_effort = "high"
```

**When to Use:**
- **Low**: Simple tasks, code generation, quick responses
- **Medium**: General-purpose tasks, balanced performance
- **High**: Complex reasoning, architecture decisions, critical reviews

#### `mirror_path` (PathBuf)

Optional mirror path for RAD-agents. Used when agents are mirrored from another location.

**Example:**
```toml
mirror_path = "/path/to/original/agent"
```

### Advanced Configuration

#### Loop Behavior

Configure an agent to request looping back to previous steps in a workflow.

```toml
[agent.loop_behavior]
steps = 2              # Number of steps to go back
max_iterations = 5     # Maximum iterations before stopping (optional)
skip = ["step-1"]      # List of step IDs to skip during loop (optional)
```

**Fields:**
- `steps` (required): Number of steps to go back when looping (must be > 0)
- `max_iterations` (optional): Maximum number of loop iterations (must be > 0 if present)
- `skip` (optional): List of step IDs to skip during loop execution

**Use Cases:**
- Iterative refinement agents
- Agents that need to fix issues in previous steps
- Validation agents that may require multiple passes

**Example:**
```toml
[agent]
id = "refinement-agent"
name = "Refinement Agent"
description = "Refines output based on feedback"
prompt_path = "prompts/agents/core/refinement-agent.md"

[agent.loop_behavior]
steps = 1
max_iterations = 3
```

#### Trigger Behavior

Configure an agent to dynamically trigger other agents during workflow execution.

```toml
[agent.trigger_behavior]
trigger_agent_id = "fallback-agent"
```

**Fields:**
- `trigger_agent_id` (optional): Default agent ID to trigger (can be overridden in workflow)

**Use Cases:**
- Coordinator agents that delegate to specialized agents
- Fallback agents for error handling
- Multi-agent workflows

**Example:**
```toml
[agent]
id = "coordinator"
name = "Coordinator Agent"
description = "Coordinates multiple agents"
prompt_path = "prompts/agents/core/coordinator.md"

[agent.trigger_behavior]
trigger_agent_id = "worker-agent"
```

### Complete Configuration Example

```toml
[agent]
id = "arch-agent"
name = "Architecture Agent"
description = "Defines system architecture and technical design decisions"
prompt_path = "prompts/agents/core/arch-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "high"

[agent.loop_behavior]
steps = 1
max_iterations = 3
```

## Prompt Template Structure

### File Format

Prompt templates are markdown files (`.md`) that contain instructions for the agent. The structure is flexible, but following a consistent pattern improves maintainability.

### Recommended Structure

```markdown
# Agent Name

Brief description of what the agent does.

## Role

Define the agent's role and primary responsibilities here.

## Capabilities

- List the agent's core capabilities
- Include what tasks it can perform
- Specify any constraints or limitations

## Input

Describe what inputs this agent expects:
- Context from previous steps
- Required parameters
- Optional configuration

## Output

Describe what this agent produces:
- Expected output format
- Key deliverables
- Success criteria

## Instructions

Provide step-by-step instructions for the agent:

1. First step - explain what to do
2. Second step - detail the process
3. Third step - clarify expectations
4. Continue as needed...

## Examples

### Example 1: [Scenario Name]

**Input:**
```
Provide sample input
```

**Expected Output:**
```
Show expected result
```

### Example 2: [Another Scenario]

**Input:**
```
Different scenario input
```

**Expected Output:**
```
Corresponding output
```

## Notes

- Add any important notes
- Include edge cases to consider
- Document best practices
```

### Key Sections Explained

#### Role

Clearly define what the agent is and what it's responsible for. This helps the AI understand its identity and purpose.

**Example:**
```markdown
## Role

You are an expert software architect responsible for designing robust, scalable, and maintainable system architectures. You analyze requirements, evaluate trade-offs, and make informed technical decisions that align with project goals and constraints.
```

#### Capabilities

List what the agent can do. Be specific about capabilities and limitations.

**Example:**
```markdown
## Capabilities

- Design high-level system architecture and component interactions
- Select appropriate technologies, frameworks, and design patterns
- Define data models, APIs, and integration strategies
- Evaluate architectural trade-offs and document decisions
- Create architecture diagrams and technical specifications
```

#### Instructions

Provide clear, step-by-step instructions. Use numbered lists for sequential processes.

**Example:**
```markdown
## Instructions

1. **Analyze Requirements**
   - Review functional and non-functional requirements
   - Identify critical user flows and data flows
   - Clarify ambiguous requirements and constraints

2. **Design System Components**
   - Break system into logical components and services
   - Define component responsibilities and boundaries
   - Map component interactions and dependencies
```

#### Examples

Include concrete examples showing expected inputs and outputs. Examples help the AI understand the desired format and quality.

**Example:**
```markdown
## Examples

### Example 1: E-Commerce Platform

**Input:**
```
Requirements:
- Multi-tenant SaaS platform for online stores
- Support 10,000+ concurrent users
- Real-time inventory management
```

**Expected Output:**
```markdown
# E-Commerce Platform Architecture

## System Overview
- Microservices architecture with API Gateway
- Event-driven communication for inventory updates
```

## Agent Discovery and Organization

### Directory Structure

Agents are organized in a directory structure that reflects their categories:

```
agents/
├── core/              # Core agents (arch, plan, code, review, doc)
├── design/            # Design agents
├── testing/           # Testing agents
├── deployment/        # Deployment agents
└── custom/            # User-defined agents
```

### Search Path Hierarchy

Agents are discovered from multiple directories in this order (precedence from highest to lowest):

1. **Project-local agents**: `./agents/`
2. **User agents**: `~/.radium/agents/`
3. **Workspace agents**: `$RADIUM_WORKSPACE/agents/` (if `RADIUM_WORKSPACE` is set)
4. **Project-level extension agents**: `./.radium/extensions/*/agents/`
5. **User-level extension agents**: `~/.radium/extensions/*/agents/`

**Precedence Rules:**
- Agents from higher-precedence directories override agents with the same ID from lower-precedence directories
- This allows project-specific agents to override user-level or extension agents

### Category Derivation

The agent's category is automatically derived from the directory structure:

- `agents/core/arch-agent.toml` → category: `"core"`
- `agents/custom/my-agent.toml` → category: `"custom"`
- `agents/rad-agents/design/design-agent.toml` → category: `"rad-agents/design"`

The category is determined by the parent directory path relative to the agents root.

### File Naming Convention

- Configuration files: `{agent-id}.toml`
- Prompt files: `{agent-id}.md`
- Keep the agent ID consistent between config and prompt files

**Example:**
```
agents/core/
├── arch-agent.toml
└── prompts/agents/core/
    └── arch-agent.md
```

## Creating Your First Agent

### Step 1: Choose a Category

Decide which category your agent belongs to. If none fit, create a new category directory.

**Common Categories:**
- `core` - Essential agents used across projects
- `design` - Design and architecture agents
- `testing` - Testing and quality assurance agents
- `deployment` - Deployment and infrastructure agents
- `custom` - Project-specific agents

### Step 2: Create the Configuration File

Create a new TOML file in the appropriate directory:

```bash
mkdir -p agents/my-category
touch agents/my-category/my-agent.toml
```

Add the basic configuration:

```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "Does something useful"
prompt_path = "prompts/agents/my-category/my-agent.md"
```

### Step 3: Create the Prompt Template

Create the prompt file at the specified path:

```bash
mkdir -p prompts/agents/my-category
touch prompts/agents/my-category/my-agent.md
```

Write the prompt template following the recommended structure:

```markdown
# My Agent

Does something useful.

## Role

You are an expert in [domain] responsible for [primary responsibility].

## Capabilities

- Capability 1
- Capability 2
- Capability 3

## Instructions

1. First step
2. Second step
3. Third step

## Examples

### Example 1: Basic Use Case

**Input:**
```
Sample input
```

**Expected Output:**
```
Expected output
```
```

### Step 4: Validate Your Agent

Use the CLI to validate your agent:

```bash
rad agents validate
```

Or validate a specific agent:

```bash
rad agents info my-agent
```

### Step 5: Test Discovery

Verify your agent is discovered:

```bash
rad agents list
```

Your agent should appear in the list.

## Common Agent Patterns

### Pattern 1: Architecture Agent

Architecture agents design system architecture and make technical decisions.

**Configuration:**
```toml
[agent]
id = "arch-agent"
name = "Architecture Agent"
description = "Defines system architecture and technical design decisions"
prompt_path = "prompts/agents/core/arch-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "high"
```

**Prompt Structure:**
- Role: Software architect
- Capabilities: System design, technology selection, architecture decisions
- Instructions: Requirements analysis, component design, technology selection, documentation
- Examples: Different architecture scenarios

### Pattern 2: Code Generation Agent

Code generation agents implement features and write production code.

**Configuration:**
```toml
[agent]
id = "code-agent"
name = "Code Implementation Agent"
description = "Implements features and writes production-ready code"
prompt_path = "prompts/agents/core/code-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "medium"
```

**Prompt Structure:**
- Role: Software engineer
- Capabilities: Code implementation, testing, documentation
- Instructions: Specification reading, planning, TDD, implementation, refactoring
- Examples: Different implementation scenarios

### Pattern 3: Code Review Agent

Review agents analyze code for quality, security, and best practices.

**Configuration:**
```toml
[agent]
id = "review-agent"
name = "Code Review Agent"
description = "Reviews code for quality, security, and best practices"
prompt_path = "prompts/agents/core/review-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "high"
```

**Prompt Structure:**
- Role: Code reviewer
- Capabilities: Bug detection, security analysis, quality assessment
- Instructions: Review checklist, issue prioritization, feedback format
- Examples: Different review scenarios

### Pattern 4: Documentation Agent

Documentation agents generate comprehensive documentation.

**Configuration:**
```toml
[agent]
id = "doc-agent"
name = "Documentation Agent"
description = "Generates comprehensive documentation and API references"
prompt_path = "prompts/agents/core/doc-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "medium"
```

**Prompt Structure:**
- Role: Technical writer
- Capabilities: README generation, API documentation, tutorials
- Instructions: Documentation types, audience consideration, examples
- Examples: Different documentation types

### Pattern 5: Planning Agent

Planning agents break down requirements into structured tasks.

**Configuration:**
```toml
[agent]
id = "plan-agent"
name = "Planning Agent"
description = "Breaks down requirements into structured iterations and tasks"
prompt_path = "prompts/agents/core/plan-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "high"
```

**Prompt Structure:**
- Role: Project planner
- Capabilities: Task breakdown, dependency analysis, estimation
- Instructions: Requirements analysis, iteration planning, task definition
- Examples: Different planning scenarios

## Advanced Configuration

### Using the CLI to Create Agents

You can use the `rad agents create` command to generate agent templates:

```bash
rad agents create my-agent "My Agent" \
  --description "Agent description" \
  --category custom \
  --engine gemini \
  --model gemini-2.0-flash-exp \
  --reasoning medium
```

This command will:
1. Create the agent configuration file
2. Create a prompt template file with a basic structure
3. Set up the directory structure

### Environment-Specific Configuration

You can create environment-specific agents by organizing them in different directories:

```
agents/
├── development/
│   └── dev-agent.toml
├── staging/
│   └── staging-agent.toml
└── production/
    └── prod-agent.toml
```

### Agent Composition

Agents can reference and trigger other agents using trigger behavior:

```toml
[agent]
id = "coordinator"
name = "Coordinator Agent"
prompt_path = "prompts/agents/core/coordinator.md"

[agent.trigger_behavior]
trigger_agent_id = "worker-agent"
```

## Best Practices

### Prompt Engineering

1. **Be Specific**: Clearly define what the agent should do
2. **Provide Examples**: Include concrete examples of inputs and outputs
3. **Set Context**: Explain the agent's role and responsibilities
4. **Define Constraints**: Specify limitations and boundaries
5. **Use Structure**: Organize prompts with clear sections
6. **Iterate**: Refine prompts based on results

### Model Selection

Choose models based on task complexity:

- **Simple tasks** (code generation, formatting): Use faster models like `gemini-2.0-flash-exp`
- **Complex reasoning** (architecture, planning): Use more capable models like `gemini-1.5-pro`
- **Balanced tasks**: Use medium-capability models for general-purpose work

### Reasoning Effort

Match reasoning effort to task requirements:

- **Low**: Quick responses, simple tasks, code generation
- **Medium**: General-purpose tasks, balanced performance
- **High**: Complex reasoning, critical decisions, thorough analysis

### Naming Conventions

- Use descriptive IDs: `arch-agent` not `agent1`
- Use consistent naming: `*-agent` suffix for agents
- Keep names concise but clear
- Use kebab-case for IDs

### Organization

- Group related agents in the same category
- Use subdirectories for complex category hierarchies
- Keep agent IDs unique within your organization
- Document agent purposes in descriptions

### Testing

- Test agents with various inputs
- Validate output quality and format
- Test edge cases and error conditions
- Verify agent discovery and configuration

### Version Control

- Store agent configs and prompts in version control
- Use meaningful commit messages
- Tag agent versions if needed
- Document changes in agent descriptions

## Testing and Validation

### Validation Commands

Validate all agents:

```bash
rad agents validate
```

Validate with verbose output:

```bash
rad agents validate --verbose
```

### Common Validation Errors

1. **Missing prompt file**: Ensure the prompt_path points to an existing file
2. **Invalid ID**: Use kebab-case, no spaces or special characters
3. **Empty name or description**: Provide meaningful values
4. **Invalid engine/model**: Use supported engine and model combinations
5. **Invalid reasoning_effort**: Must be "low", "medium", or "high"

### Testing Agent Discovery

List all discovered agents:

```bash
rad agents list
```

List with verbose output:

```bash
rad agents list --verbose
```

Search for agents:

```bash
rad agents search "architecture"
```

Get agent information:

```bash
rad agents info arch-agent
```

### Manual Testing

1. Create test inputs matching your agent's expected format
2. Execute the agent with test inputs
3. Verify output quality and completeness
4. Check for edge cases and error handling
5. Validate output format matches expectations

## Troubleshooting

### Agent Not Discovered

**Problem**: Agent doesn't appear in `rad agents list`

**Solutions:**
1. Check file location: Ensure agent is in a valid search path
2. Verify file extension: Must be `.toml`
3. Check file permissions: Ensure file is readable
4. Validate TOML syntax: Use a TOML validator
5. Check category derivation: Verify directory structure

### Prompt File Not Found

**Problem**: Validation error: "Prompt file not found"

**Solutions:**
1. Verify prompt_path: Check the path is correct
2. Check file exists: Ensure the markdown file exists
3. Verify path resolution: Test with absolute path first
4. Check relative path: Ensure relative path is correct from workspace root

### Invalid Configuration

**Problem**: Validation errors in configuration

**Solutions:**
1. Check required fields: Ensure id, name, description, prompt_path are set
2. Validate TOML syntax: Use a TOML parser to check syntax
3. Check field types: Ensure values match expected types
4. Verify engine/model: Use supported combinations
5. Check reasoning_effort: Must be "low", "medium", or "high"

### Agent Not Executing Correctly

**Problem**: Agent produces unexpected output

**Solutions:**
1. Review prompt template: Ensure instructions are clear
2. Check examples: Verify examples match expected behavior
3. Test with different inputs: Identify patterns in failures
4. Refine prompt: Iterate on prompt based on results
5. Check model selection: Consider using a different model

### Path Resolution Issues

**Problem**: Relative paths not resolving correctly

**Solutions:**
1. Use absolute paths: Test with absolute paths first
2. Check workspace root: Verify current working directory
3. Use relative paths from workspace: Ensure paths are relative to project root
4. Check directory structure: Verify prompt files are in expected locations

## Additional Resources

- [Agent Configuration Guide](../user-guide/agent-configuration.md) - Detailed configuration reference
- [Agent System Architecture](../developer-guide/agent-system-architecture.md) - Technical architecture details
- [Example Agents](../../examples/agents/) - Example agent configurations
- [CLI Documentation](../../README.md) - Command-line interface reference

## Conclusion

Creating effective agents requires understanding both the configuration format and prompt engineering. Follow the patterns and best practices outlined in this guide, and iterate based on results. Well-designed agents can significantly improve productivity and consistency across your projects.

Remember:
- Start simple and iterate
- Use examples from existing agents
- Test thoroughly before deploying
- Document your agents clearly
- Share successful patterns with your team

Happy agent creating!

