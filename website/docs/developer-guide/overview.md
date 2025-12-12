---
id: "developer-guide-overview"
title: "Developer Guide"
sidebar_label: "Developer Guide Overview"
---

# Developer Guide

Welcome to the Radium Developer Guide. This guide helps you extend Radium's capabilities, understand its architecture, and contribute to the project.

## Getting Started

New to Radium development?

- **[Architecture Overview](./architecture/overview.md)** - Understand Radium's architecture
- **[Agent System Architecture](./agent-system-architecture.md)** - How agents work
- **[Extension Development](../extensions/creating-extensions.md)** - Create extensions

## Architecture Documentation

### System Architecture

- **[Architecture Overview](./architecture/overview.md)** - High-level system architecture
- **[Agent Configuration System](./architecture/agent-configuration-system.md)** - Agent configuration architecture
- **[Checkpoint System](./architecture/checkpoint-system.md)** - Checkpoint and resume architecture
- **[Engine Abstraction](./architecture/engine-abstraction.md)** - AI provider abstraction layer
- **[TUI Architecture](./architecture/tui-architecture.md)** - Terminal UI architecture

### Design Documents

- **[Persona System Architecture](./design/persona-system-architecture.md)** - Persona system design

### Architecture Decision Records (ADRs)

- **[ADR 001: YOLO Mode Architecture](./adr/001-yolo-mode-architecture.md)** - YOLO mode design decisions

## Extending Radium

### Extension System

- **[Extension System](../extensions/README.md)** - Overview of extension system
- **[Creating Extensions](../extensions/creating-extensions.md)** - Build extensions
- **[Extension Architecture](../extensions/architecture.md)** - Technical architecture
- **[Extension API Reference](../extensions/api-reference.md)** - API documentation
- **[Extension Integration Guide](../extensions/integration-guide.md)** - Integration patterns

### MCP Integration

- **[Extension MCP Integration](./extension-mcp-integration.md)** - Integrate MCP servers
- **[MCP Architecture](../mcp/architecture.md)** - MCP system architecture

### Context Sources

- **[Extending Sources](./extending-sources.md)** - Add custom context sources

## Development Guides

### Development Process

- **[Agent Instructions](./development/agent-instructions.md)** - Instructions for AI agents working on Radium
- **[Colors](./development/colors.md)** - Color scheme and theming
- **[Deep Analysis Improvements](./development/deep-analysis-improvements.md)** - Analysis system improvements

### Testing

- **[Testing Coverage Analysis](./testing/coverage-analysis-REQ-172.md)** - Coverage analysis
- **[Testing Backlog](./testing/coverage-backlog.md)** - Testing improvements needed

### Guides

- **[JSON Schema Guide](./guides/json-schema-guide.md)** - Using JSON schemas with agents

## Roadmap & Future Architecture

### Technical Roadmap

- **[Technical Architecture Roadmap](../roadmap/technical-architecture.md)** - Technical implementation roadmap
  - Core Architecture Specification
  - Component Foundry Implementation
  - Global Component Graph Design
  - Agentic Component Integration
  - Performance & Scalability

### Vision & Innovation

- **[Vision & Innovation](../roadmap/vision.md)** - Long-term vision
  - Component Foundry Pattern (CFP)
  - Durable Autonomous Continuous Remediation (DACR)
  - Durable Recursive Component Generation (DRCG)
  - Autonomous Component-Centric Assembly (ACCA)

### Protocol Specifications

- **[Protocol Specifications](../roadmap/protocol-specifications.md)** - KOR protocol roadmap
- **[Governance & Operations](../roadmap/governance-operations.md)** - Governance roadmap

## API Reference

- **[API Overview](../api/overview.md)** - Complete API documentation
- **[Context Cache API](../api/context-cache-api.md)** - Context caching API
- **[Extension API](../extensions/api-reference.md)** - Extension system API
- **[Hooks API](../hooks/api-reference.md)** - Hooks API

## Codebase Structure

### Core Crates

- **`radium-core`**: Core backend with gRPC server and orchestration
- **`radium-orchestrator`**: Agent orchestration engine
- **`radium-models`**: Data models and types
- **`radium-abstraction`**: Abstraction layers

### Applications

- **`apps/cli`**: Command-line interface
- **`apps/tui`**: Terminal user interface
- **`apps/desktop`**: Tauri-based desktop application

### Packages

- **`packages/api-client`**: TypeScript API client
- **`packages/shared-types`**: Shared TypeScript types
- **`packages/state`**: State management
- **`packages/ui`**: UI components

## Development Workflow

### Building

```bash
# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p radium-core

# Build with optimizations
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p radium-core

# Run with coverage
cargo llvm-cov --workspace --html
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace

# Check for issues
cargo check --workspace
```

## Contributing

### Getting Started

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Code Standards

- Follow Rust style guidelines
- Write comprehensive tests
- Document public APIs
- Update relevant documentation

### Pull Request Process

1. Ensure all tests pass
2. Update documentation
3. Add changelog entry if needed
4. Request review

## Architecture Patterns

### Component Foundry Pattern

Systematic approach to creating, validating, and composing reusable components.

**Status**: 📋 Planned  
**Learn more**: [Roadmap: Component Foundry](../roadmap/vision.md#1-component-foundry-pattern-cfp)

### Durable Autonomous Continuous Remediation (DACR)

Self-healing systems that maintain component quality over time.

**Status**: 🔮 Future  
**Learn more**: [Roadmap: DACR](../roadmap/vision.md#2-durable-autonomous-continuous-remediation-dacr)

### Durable Recursive Component Generation (DRCG)

Components that generate other components recursively.

**Status**: 🔮 Future  
**Learn more**: [Roadmap: DRCG](../roadmap/vision.md#3-durable-recursive-component-generation-drcg)

### Autonomous Component-Centric Assembly (ACCA)

Systems that automatically assemble themselves from available components.

**Status**: 🔮 Future  
**Learn more**: [Roadmap: ACCA](../roadmap/vision.md#4-autonomous-component-centric-assembly-acca)

## Resources

### Documentation

- **[User Guide](../user-guide/overview.md)** - User-facing documentation
- **[CLI Reference](../cli/README.md)** - CLI documentation
- **[Features](../features/)** - Feature documentation

### External Resources

- **[GitHub Repository](https://github.com/clay-curry/RAD)** - Source code
- **[Issues](https://github.com/clay-curry/RAD/issues)** - Bug reports and feature requests
- **[Discussions](https://github.com/clay-curry/RAD/discussions)** - Community discussions

## Next Steps

- **[Architecture Overview](./architecture/overview.md)** - Deep dive into architecture
- **[Creating Extensions](../extensions/creating-extensions.md)** - Build your first extension
- **[API Reference](../api/overview.md)** - Explore the API

---

**Ready to contribute?** Start with [Architecture Overview](./architecture/overview.md) or [Creating Extensions](../extensions/creating-extensions.md).

