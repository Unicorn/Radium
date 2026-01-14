# radium-core

The core backend library for Radium, providing the gRPC server, orchestration engine, and fundamental services for agent management.

## Overview

`radium-core` is the heart of the Radium platform, implementing:

- **gRPC Server**: Production-ready server with gRPC-Web support
- **Agent Management**: Discovery, registration, and lifecycle management
- **Workflow Execution**: Sequential and parallel workflow orchestration
- **MCP Integration**: Model Context Protocol server and proxy
- **Tool System**: Extensible tool framework with security policies
- **Context Management**: Hierarchical context files (GEMINI.md/CLAUDE.md)
- **Extension System**: Plugin architecture for custom functionality
- **Budget Management**: Cost tracking and budget enforcement
- **Session Analytics**: Comprehensive usage and performance metrics
- **Vibe Check**: Metacognitive oversight for agent alignment

## Features

### Server & API
- gRPC server with tonic/tonic-web
- RESTful HTTP API endpoints
- Embedded server lifecycle management
- Health checks and monitoring

### Agent System
- Declarative TOML-based configuration
- Automatic agent discovery from configured directories
- Persona system for intelligent model selection
- Agent state management and execution tracking

### Workflow Engine
- DAG-based workflow execution
- Parallel task execution with dependency management
- Error recovery and retry policies
- Checkpoint and resume support
- Git integration for tracking changes

### MCP (Model Context Protocol)
- MCP server implementation
- MCP proxy for aggregating multiple servers
- Load balancing and failover
- Tool catalog management

### Context Files
- Hierarchical context loading (global → project → subdirectory)
- File imports with `@file.md` syntax
- Secret filtering and redaction
- Braingrid integration for cloud-based context

### Policy Engine
- Fine-grained tool execution control
- Rule-based policies with priority tiers
- Multiple approval modes (yolo, auto-edit, ask)
- Session constitutions for temporary rules

### Extension System
- Package and distribute agent configs, MCP servers, commands
- Versioning and dependency management
- Digital signatures for authenticity
- Auto-discovery and installation

### Monitoring
- Budget tracking with cost thresholds
- Error classification and severity analysis
- Usage metrics and analytics
- Performance profiling

## Usage

### As a Library

```rust
use radium_core::{
    agents::registry::AgentRegistry,
    server::RadiumServer,
    config::Config,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load()?;

    // Initialize agent registry
    let registry = AgentRegistry::new()?;
    registry.discover_agents().await?;

    // Start the gRPC server
    let server = RadiumServer::new(config)?;
    server.serve().await?;

    Ok(())
}
```

### As a Binary

```bash
# Run the standalone server
cargo run --bin radium-core

# Or after building
./target/release/radium-core
```

### With Features

```toml
[dependencies]
radium-core = { version = "0.1", features = ["workflow", "monitoring"] }
```

Available features:
- `workflow` - Enable workflow orchestration with radium-orchestrator
- `monitoring` - Enable budget and analytics tracking
- `mcp-progress` - Progress indicators for CLI usage
- `server` - Enable gRPC server (default)
- `http` - Enable HTTP API endpoints
- `syntax` - Syntax highlighting support
- `tui-theme` - TUI-specific theme loading

## Architecture

```
radium-core/
├── agents/         - Agent discovery and management
├── auth/           - Authentication and credentials
├── checkpoint/     - Workflow checkpointing
├── commands/       - Custom command system
├── config/         - Configuration management
├── context/        - Context file loading
├── engines/        - AI provider abstractions
├── extensions/     - Extension system
├── hooks/          - Lifecycle hooks
├── mcp/            - MCP protocol implementation
├── monitoring/     - Budget and analytics
├── playbooks/      - ACE Skillbook system
├── policy/         - Policy engine
├── server/         - gRPC server implementation
├── tools/          - Tool framework
├── workflow/       - Workflow execution engine
└── workspace/      - Workspace management
```

## Configuration

### Agent Configuration

Agents are defined in TOML files:

```toml
[agent]
id = "senior-dev"
name = "Senior Developer"
description = "Expert software engineer"
prompt_path = "prompts/agents/senior-dev.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"

[agent.persona]
[agent.persona.models]
primary = "gemini-2.0-flash-exp"
fallback = "gemini-2.0-flash-thinking"
premium = "gemini-1.5-pro"
```

### Server Configuration

```rust
use radium_core::config::Config;

let config = Config {
    server_port: 50051,
    enable_reflection: true,
    max_connections: 1000,
    // ... more options
};
```

## Development

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# With workflow feature
cargo build --features workflow
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_agent_discovery
```

### Code Coverage

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate HTML report
cargo llvm-cov --html

# View report
open target/llvm-cov/html/index.html
```

## Dependencies

### Major Dependencies
- **tonic/tonic-web** - gRPC server framework
- **tokio** - Async runtime
- **serde/serde_json** - Serialization
- **tracing** - Structured logging
- **tower** - Service middleware
- **anyhow/thiserror** - Error handling
- **rusqlite** - Embedded database for analytics
- **reqwest** - HTTP client for MCP

### Internal Dependencies
- **radium-abstraction** - Core trait definitions
- **radium-models** - AI model implementations
- **radium-orchestrator** - Workflow orchestration (optional)
- **radium-training** - ML training pipeline

## Performance

- Concurrent agent execution
- Efficient context caching
- Streaming responses for real-time updates
- Connection pooling for database operations
- Zero-copy serialization where possible

## Security

- API key encryption at rest
- Secret filtering in context files
- Policy-based tool execution control
- Sandboxed tool execution (Docker/Seatbelt)
- Digital signatures for extensions

## Contributing

See the main [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

### Module Organization
- Keep modules focused and single-purpose
- Use traits for extensibility
- Document public APIs thoroughly
- Add tests for new features
- Follow Rust API guidelines

## License

MIT - see [LICENSE](../../LICENSE) for details

## Links

- [Main Documentation](../../website/docs/)
- [Architecture Overview](../../website/docs/developer-guide/architecture/)
- [API Reference](https://docs.rs/radium-core)
- [Examples](../../examples/)
