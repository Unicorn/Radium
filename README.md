# Radium

> **Next-generation agentic orchestration tool for developers and power users**

Radium is a high-performance, Rust-based platform for creating, managing, and deploying autonomous agents. Built with safety and efficiency in mind, Radium provides a robust framework for orchestrating complex agent workflows.

## Features

- **🚀 High-Performance Backend**: Rust-based core with concurrent agent orchestration
- **🔌 Extensible Agent Framework**: Create custom agents and integrate them easily
- **📱 Multiple Interfaces**: CLI, TUI, Desktop app, and Web application
- **🤖 Flexible Model Support**: Works with major AI models (Gemini, OpenAI, etc.)
- **⚙️ Powerful Workflow Engine**: Define complex task chains and decision trees
- **📊 Comprehensive Monitoring**: Real-time tracking of agents and workflows
- **📈 Session Analytics**: Track costs, performance, and optimize agent sessions
- **🔧 Auto-Managed Server**: Embedded server lifecycle management for seamless deployment
- **🧠 Metacognitive Oversight (Vibe Check)**: Chain-Pattern Interrupt system for preventing reasoning lock-in (+27% success rate, -41% harmful actions)
- **📚 Learning System**: Track mistakes, preferences, and successes to build pattern recognition
- **📖 ACE Skillbook**: Learn and apply successful strategies from past work
- **🎭 Persona System**: Intelligent model selection, cost optimization, and automatic fallback chains
- **🔒 Policy Engine**: Fine-grained tool execution control with rule-based policies and approval modes

## Quick Start

### Prerequisites

- Rust (latest stable)
- Node.js and npm/bun (for frontend apps)

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/radium.git
cd radium

# Build the project
cargo build --release

# Or use npm scripts
npm run build
```

### Running the Server

The Radium server is automatically embedded in client applications (CLI, TUI, Desktop). You can also run it standalone:

```bash
# Run the standalone server
npm run server

# Or directly with Cargo
cargo run --bin radium-core
```

### Using the CLI

```bash
# Initialize a workspace
radium init

# Create a plan
radium plan --input "Build a web app"

# Execute the plan
radium craft <plan-id>

# Use structured output with JSON schema
rad step agent-id "Extract user data" --response-format json-schema --response-schema user-schema.json
```

### Using the Desktop App

```bash
npm run desktop
```

The desktop app automatically starts an embedded server on launch.

## Architecture

Radium uses a modular monorepo structure:

- **`crates/radium-core`**: Core backend with gRPC server and orchestration
- **`apps/cli`**: Command-line interface
- **`apps/tui`**: Terminal user interface
- **`apps/desktop`**: Tauri-based desktop application
- **`packages/`**: Shared TypeScript packages for web/desktop

## Embedded Server Lifecycle

Radium includes automatic server lifecycle management:

- **Desktop App**: Server automatically starts when the app launches
- **CLI/TUI**: Server starts on-demand when commands require it
- **Standalone**: Still available as a separate binary for advanced use cases

See [Embedded Server Documentation](docs/features/embedded-server-lifecycle.md) for details.

## Agent Configuration

Radium uses a declarative TOML-based configuration system for defining AI agents. Agents are automatically discovered from configured directories and can be managed through the CLI.

### Quick Example

Create an agent configuration file at `agents/core/my-agent.toml`:

```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "A custom agent for specific tasks"
prompt_path = "prompts/agents/core/my-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "medium"

[agent.persona]
[agent.persona.models]
primary = "gemini-2.0-flash-exp"
fallback = "gemini-2.0-flash-thinking"
premium = "gemini-1.5-pro"

[agent.persona.performance]
profile = "balanced"
estimated_tokens = 1500
```

Create the corresponding prompt file at `prompts/agents/core/my-agent.md`:

```markdown
# My Agent

## Role
Define the agent's role and responsibilities here.

## Instructions
Provide step-by-step instructions for the agent.
```

### Self-Hosted Models

Radium supports self-hosted AI models (Ollama, vLLM, LocalAI) for cost savings, data privacy, and air-gapped environments. See the [Self-Hosted Models Documentation](docs/self-hosted-models/README.md) for setup guides, configuration examples, and troubleshooting.

**Quick Start:**
```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh
ollama pull llama3.2

# Configure agent to use local model
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
```

### CLI Commands

```bash
# List all discovered agents
rad agents list

# Search for agents
rad agents search "architecture"

# Get agent information
rad agents info my-agent

# Validate agent configurations
rad agents validate

# Create a new agent template
rad agents create my-agent "My Agent" --category core
```

### Documentation

- [User Guide: Agent Configuration](docs/user-guide/agent-configuration.md) - Complete guide for configuring agents
- [Self-Hosted Models](docs/self-hosted-models/README.md) - Setup and configuration for Ollama, vLLM, and LocalAI
- [User Guide: Persona System](docs/user-guide/persona-system.md) - Intelligent model selection and cost optimization
- [Developer Guide: Agent System Architecture](docs/developer-guide/agent-system-architecture.md) - Technical architecture details
- [Examples](examples/agents/) - Example agent configurations

## MCP Proxy Server

The MCP Proxy Server provides centralized access to multiple MCP (Model Context Protocol) servers through a single endpoint, with load balancing, failover, security, and tool aggregation.

### Quick Start

```bash
# Initialize proxy configuration
rad mcp proxy init

# Start the proxy server
rad mcp proxy start

# Check status
rad mcp proxy status

# Stop the proxy
rad mcp proxy stop
```

### Features

- **Centralized Management**: Single configuration point for all upstream MCP servers
- **High Availability**: Automatic failover when upstream servers become unavailable
- **Load Balancing**: Distribute requests across multiple upstream servers
- **Security**: Centralized rate limiting, logging, and sensitive data redaction
- **Tool Aggregation**: Unified tool catalog with conflict resolution

See [MCP Proxy Documentation](docs/mcp-proxy.md) for detailed setup and configuration.

## Extension System

Radium's extension system allows you to package and share reusable agent configurations, MCP servers, custom commands, and hooks. Extensions enable the community to share workflows, tools, and configurations.

### Quick Start

```bash
# Install an extension
rad extension install ./my-extension

# List installed extensions
rad extension list

# Get extension information
rad extension info my-extension

# Create a new extension
rad extension create my-extension --author "Your Name" --description "My extension"
```

### Documentation

- [Extension System Guide](docs/extensions/README.md) - Complete user guide
- [Creating Extensions](docs/extensions/creating-extensions.md) - Guide for extension authors
- [Extension Architecture](docs/extensions/architecture.md) - Technical architecture details
- [Examples](examples/extensions/) - Example extension packages

## Context Files

Context files (GEMINI.md) provide persistent instructions to agents without repeating them in every prompt. They support hierarchical loading (global, project, subdirectory) and can import other files using `@file.md` syntax.

### Quick Example

Create a context file at your project root:

```markdown
# Project Context

This project uses Rust and follows these guidelines:
- Use `cargo fmt` for formatting
- Write comprehensive tests for all public APIs
- Document all public types and functions

@docs/coding-standards.md
```

### CLI Commands

```bash
# List all context files in workspace
rad context list

# Show which context files would be loaded for a path
rad context show src/

# Validate all context files
rad context validate
```

### Documentation

- [Context Files Feature Guide](docs/features/context-files.md) - Complete guide for context files
- [Examples](examples/context-files/) - Example context files and templates

## Orchestration

Radium's orchestration system provides intelligent, model-agnostic task routing that automatically selects and coordinates specialist agents without requiring manual agent selection.

### Quick Start

Orchestration is **enabled by default** in the TUI. Simply type naturally without command prefixes:

```
You: I need to refactor the authentication module

🤔 Analyzing...
📋 Invoking: senior-developer
✅ Complete (2.3s)

Assistant: I've refactored the authentication module...
```

### Key Features

- **Natural Conversation**: Type requests naturally without `/chat` or `/agents` commands
- **Intelligent Routing**: Automatically selects the best agent(s) for each task
- **Multi-Agent Workflows**: Coordinates multiple agents for complex tasks
- **Model-Agnostic**: Works with Gemini, Claude, OpenAI, and prompt-based fallback
- **Persistent Configuration**: Settings saved to `~/.radium/orchestration.toml`

### Configuration

Control orchestration via TUI commands:

```bash
# Show current status
/orchestrator

# Enable/disable
/orchestrator toggle

# Switch provider
/orchestrator switch gemini
/orchestrator switch claude
/orchestrator switch openai
```

### Documentation

- [Orchestration User Guide](docs/user-guide/orchestration.md) - Complete user guide
- [Orchestration Workflows](docs/examples/orchestration-workflows.md) - Example workflows
- [Orchestration Testing Guide](docs/user-guide/orchestration-testing.md) - Manual testing procedures

## Policy Engine

Radium's Policy Engine provides fine-grained control over tool execution to ensure security and prevent unwanted operations. Configure workspace-specific and enterprise-ready security policies with rule-based enforcement.

### Quick Start

```bash
# Initialize a default policy file
rad policy init

# List all policy rules
rad policy list

# Check if a tool would be allowed
rad policy check read_file config.toml

# Validate policy file syntax
rad policy validate
```

### Key Features

- **TOML-based configuration** - Simple, declarative policy rules
- **Priority-based matching** - Admin > User > Default priority tiers
- **Pattern matching** - Glob patterns for tool names and arguments
- **Approval modes** - Yolo, AutoEdit, and Ask modes for different security levels
- **Session constitutions** - Per-session rules for temporary constraints

### Example Configuration

Create `.radium/policy.toml` in your workspace:

```toml
approval_mode = "ask"

[[rules]]
name = "Allow safe file operations"
priority = "user"
action = "allow"
tool_pattern = "read_*"

[[rules]]
name = "Require approval for file writes"
priority = "user"
action = "ask_user"
tool_pattern = "write_*"

[[rules]]
name = "Deny dangerous shell commands"
priority = "admin"
action = "deny"
tool_pattern = "run_terminal_cmd"
arg_pattern = "rm -rf *"
```

### Documentation

- [Policy Engine Guide](docs/features/policy-engine.md) - Complete feature documentation
- [Policy Best Practices](docs/security/policy-best-practices.md) - Security guidelines
- [Example Configurations](examples/policy-examples.toml) - Example policy files

## Workflow Behaviors

Radium's workflow behavior system enables dynamic execution control, allowing agents to adapt to changing conditions, handle errors gracefully, and coordinate with other agents.

### Quick Start

Workflow behaviors are controlled via a `behavior.json` file placed at `.radium/memory/behavior.json`:

```json
{
  "action": "loop",
  "reason": "Tests failing, need to retry"
}
```

### Available Behaviors

- **Loop**: Repeat previous steps with max iterations and skip lists
- **Trigger**: Dynamically trigger other agents during execution
- **Checkpoint**: Pause workflow for manual review or approval
- **VibeCheck**: Request metacognitive oversight to prevent reasoning lock-in

### Key Features

- **Dynamic Control**: Agents can write behavior.json files during execution
- **Configurable Limits**: Max iterations, step back counts, skip lists
- **Error Recovery**: Graceful handling of invalid behavior files
- **Integration**: Automatically integrated with WorkflowExecutor

### Documentation

- [Workflow Behaviors Guide](docs/features/workflow-behaviors.md) - Complete feature documentation
- [Constitution System Guide](docs/features/constitution-system.md) - Session-based rules
- [Behavior Examples](examples/behaviors/) - Example behavior.json files
- [Policy Templates](examples/policies/) - Example policy configurations

## Engine Management

Radium supports multiple AI providers through a unified engine abstraction layer. You can list, configure, and switch between different engines seamlessly.

### Quick Start

```bash
# List all available engines
rad engines list

# Show detailed information about an engine
rad engines show gemini

# Check authentication status
rad engines status

# Set default engine
rad engines set-default gemini

# Check engine health
rad engines health
```

### Available Engines

- **Gemini** (`gemini`): Google's Gemini models (gemini-pro, gemini-2.0-flash-exp)
- **Claude** (`claude`): Anthropic's Claude models (claude-3-opus, claude-3-sonnet, claude-3-haiku)
- **OpenAI** (`openai`): OpenAI GPT models (gpt-4, gpt-4-turbo, gpt-3.5-turbo)
- **Mock** (`mock`): Testing engine for development

### Authentication

Set up API keys for each provider:

```bash
# Authenticate with a provider
rad auth login gemini
rad auth login claude
rad auth login openai
```

Credentials are securely stored in `~/.radium/credentials.json`.

### Configuration

Engine settings are stored in `.radium/config.toml`:

```toml
[engines]
default = "gemini"
```

### Documentation

- [Engine Abstraction Architecture](docs/architecture/engine-abstraction.md) - Technical architecture details
- [Adding New Engine Providers](docs/guides/adding-new-engine-provider.md) - Developer guide for adding providers

## Session Analytics

Radium automatically tracks every agent session, providing detailed analytics on costs, performance, and optimization opportunities.

### Quick Start

```bash
# View current session statistics
rad stats session

# View model usage breakdown
rad stats model

# View session history
rad stats history

# Compare two sessions
rad stats compare <session-id-1> <session-id-2>

# Export analytics data
rad stats export --output analytics.json
```

### Key Features

- **Cost Tracking**: Monitor token usage and estimated costs per model
- **Performance Metrics**: Analyze wall time, agent active time, and tool execution time
- **Session Comparison**: Compare sessions to identify improvements or regressions
- **Cache Optimization**: Track cache effectiveness and savings
- **Code Change Tracking**: Automatically track code changes via git diff

### Documentation

- [Session Analytics Guide](docs/features/session-analytics.md) - Complete feature documentation
- [Optimizing Costs](docs/guides/optimizing-costs.md) - Strategies for reducing session costs

## Metacognitive Oversight (Vibe Check)

Radium's Vibe Check system provides Chain-Pattern Interrupt (CPI) functionality to prevent reasoning lock-in and improve agent alignment. Research shows CPI systems improve agent success rates by +27% and reduce harmful actions by -41%.

### Quick Start

```bash
# Manual vibe check
rad vibecheck --goal "Build feature" --plan "Use React and Node.js"

# With phase specification
rad vibecheck --phase planning --goal "Design API" --plan "REST API"

# JSON output
rad vibecheck --goal "Test" --plan "Test plan" --json
```

### Key Features

- **Phase-Aware Feedback**: Oversight adapts to planning, implementation, and review phases
- **Learning Integration**: Mistakes and successes are automatically captured
- **Risk Assessment**: Risk scores (0.0-1.0) indicate potential issues
- **Pattern Detection**: Identifies traits like Complex Solution Bias, Feature Creep
- **Constitution Rules**: Session-scoped rules for workflow constraints

### Learning System

The learning system tracks mistakes, preferences, and successes:

```bash
# List learning entries
rad learning list

# Add a mistake
rad learning add-mistake --category "Feature Creep" --description "Added unnecessary feature"

# View skillbook
rad learning show-skillbook

# Tag a skill
rad learning tag-skill --skill-id "skill-00001" --tag "helpful"
```

### Documentation

- [Vibe Check User Guide](docs/user-guide/vibe-check.md) - Complete usage guide
- [Learning System Guide](docs/user-guide/learning-system.md) - Learning system documentation
- [Constitution Rules Guide](docs/user-guide/constitution-rules.md) - Session rules documentation
- [Vibe Check Workflow Example](docs/examples/vibe-check-workflow.md) - Complete workflow example

## Documentation

- [Project Overview](docs/project/00-project-overview.md)
- [Architecture](docs/architecture/)
- [Agent Enhancement Guide](docs/AGENT_ENHANCEMENT_GUIDE.md)
- [Agent Creation Guide](docs/guides/agent-creation-guide.md)

## Development

```bash
# Run tests
cargo test

# Run CLI
npm run cli

# Run TUI
npm run tui

# Run desktop app
npm run desktop
```

### Testing & Coverage

Radium uses `cargo-llvm-cov` for code coverage reporting.

```bash
# Install coverage tools (one-time setup)
cargo install cargo-llvm-cov

# Generate HTML coverage report
cargo llvm-cov --workspace --html

# Generate LCOV coverage report (for CI)
cargo llvm-cov --workspace --lcov --output-path lcov.info

# View HTML report (opens in browser)
open target/llvm-cov/html/index.html
```

See [Testing Documentation](docs/TESTING.md) for comprehensive testing guidelines (coming soon).

## Contributing

We welcome contributions! Please see our contributing guidelines and development rules:

- [Agent Rules](docs/rules/AGENT_RULES.md)
- [Development Guidelines](docs/rules/CLAUDE.md)

## License

MIT License - see LICENSE file for details

## Links

- [Documentation](docs/)
- [Architecture Overview](docs/architecture/architecture-backend.md)
- [Project Roadmap](docs/project/02-now-next-later.md)
