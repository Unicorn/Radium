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
- **🔧 Auto-Managed Server**: Embedded server lifecycle management for seamless deployment

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
```

Create the corresponding prompt file at `prompts/agents/core/my-agent.md`:

```markdown
# My Agent

## Role
Define the agent's role and responsibilities here.

## Instructions
Provide step-by-step instructions for the agent.
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
- [Developer Guide: Agent System Architecture](docs/developer-guide/agent-system-architecture.md) - Technical architecture details
- [Examples](examples/agents/) - Example agent configurations

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
