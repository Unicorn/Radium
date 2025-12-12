---
id: "api-overview"
title: "API Reference"
sidebar_label: "API Reference Overview"
---

# API Reference

Complete API documentation for Radium. This reference covers all public APIs for extending and integrating with Radium.

## Rust API Documentation

The primary Radium API is written in Rust. Full Rust API documentation is available:

- **[Rust API Docs](/RAD/api/radium_core)** - Complete Rust API documentation (generated from code)

## Core APIs

### Context Cache API

- **[Context Cache API](./context-cache-api.md)** - Context caching and retrieval API
  - Cache management
  - Context retrieval
  - Cache invalidation

## Extension System API

The extension system provides APIs for creating, managing, and integrating extensions.

- **[Extension API Reference](../extensions/api-reference.md)** - Complete extension system API
  - Extension manifest
  - Extension manager
  - Discovery API
  - Marketplace API
  - Signing API

### Extension Types

- **ExtensionManifest**: Manifest parsing and validation
- **ExtensionManager**: Installation and management
- **ExtensionDiscovery**: Discovery and search
- **ExtensionMarketplace**: Marketplace integration

## Hooks API

Native and WASM hooks for customizing agent behavior.

- **[Hooks API Reference](../hooks/api-reference.md)** - Complete hooks API
  - Hook lifecycle
  - Hook types
  - Hook development

## MCP Integration API

Model Context Protocol integration for external tools and services.

- **[MCP Architecture](../mcp/architecture.md)** - MCP system architecture
- **[MCP Tools](../mcp/tools.md)** - Available MCP tools
- **[MCP Configuration](../mcp/configuration.md)** - Configuration API

## Agent System API

### Agent Configuration

- **[Agent Configuration](../user-guide/agent-configuration.md)** - Agent configuration format
- **[Agent System Architecture](../developer-guide/agent-system-architecture.md)** - System architecture

### Agent Execution

- Agent execution API (via CLI/TUI)
- Agent orchestration API
- Agent memory API

## Orchestration API

Intelligent task routing and agent coordination.

- **[Orchestration Guide](../user-guide/orchestration.md)** - User guide
- **[Orchestration Configuration](../user-guide/orchestration-configuration.md)** - Configuration

## Planning API

Autonomous planning and workflow generation.

- **[Autonomous Planning](../features/planning/autonomous-planning.md)** - Planning system
- **[Plan Execution](../features/plan-execution.md)** - Execution API

## Policy Engine API

Fine-grained tool execution control.

- **[Policy Engine](../features/policy-engine.md)** - Policy system
- **[Policy Configuration](../features/security/configuration.md)** - Configuration API

## Memory & Context API

Plan-scoped memory and context management.

- **[Memory & Context](../user-guide/memory-and-context.md)** - Memory system
- **[Context Sources](../user-guide/context-sources.md)** - Context source API

## Self-Hosted Models API

Integration with self-hosted AI models.

- **[Self-Hosted API Reference](../self-hosted/api-reference.md)** - Self-hosted models API
- **[Ollama Integration](../self-hosted/setup/ollama.md)** - Ollama setup
- **[vLLM Integration](../self-hosted/setup/vllm.md)** - vLLM setup
- **[LocalAI Integration](../self-hosted/setup/localai.md)** - LocalAI setup

## Monitoring API

Agent monitoring and analytics.

- **[Monitoring Architecture](../features/monitoring/architecture.md)** - Monitoring system
- **[Monitoring Usage Guide](../features/monitoring/usage-guide.md)** - Usage guide
- **[Monitoring API Reference](../features/monitoring/api-reference.md)** - API reference

## Session Analytics API

Track costs, performance, and optimize agent sessions.

- **[Session Analytics](../features/session-analytics.md)** - Analytics system

## CLI API

Command-line interface for all operations.

- **[CLI Documentation](../cli/README.md)** - CLI overview
- **[CLI Commands](../cli/commands/)** - All commands
- **[CLI Architecture](../cli/architecture.md)** - CLI architecture

## gRPC API

Radium uses gRPC for internal communication between components.

### Core Services

- **AgentService**: Agent execution and management
- **OrchestrationService**: Task routing and coordination
- **PlanningService**: Plan generation and execution
- **MemoryService**: Memory storage and retrieval
- **ContextService**: Context gathering and management

### Protocol Buffers

Protocol buffer definitions are in `crates/radium-core/proto/`.

## TypeScript/JavaScript API

TypeScript packages for web and desktop applications.

- **`packages/api-client`**: TypeScript API client
- **`packages/shared-types`**: Shared TypeScript types
- **`packages/state`**: State management
- **`packages/ui`**: UI components

## Future APIs

### Component Foundry API

APIs for the Component Foundry Pattern (planned).

- Component creation
- Component validation
- Component composition
- Component versioning

**Status**: 📋 Planned  
**Learn more**: [Roadmap: Component Foundry](../roadmap/vision.md#1-component-foundry-pattern-cfp)

### Global Component Graph API

APIs for component discovery and composition (planned).

- Component search
- Component discovery
- Composition engine
- Relationship tracking

**Status**: 📋 Planned  
**Learn more**: [Roadmap: Global Component Graph](../roadmap/technical-architecture.md#global-component-graph-design-t3)

### KOR Protocol API

KOR protocol APIs for component exchange (planned).

- Component publishing
- Component retrieval
- Marketplace integration
- Economic models

**Status**: 📋 Planned  
**Learn more**: [Roadmap: Protocol Specifications](../roadmap/protocol-specifications.md)

## API Versioning

Radium APIs follow semantic versioning:

- **Major versions**: Breaking changes
- **Minor versions**: New features, backward compatible
- **Patch versions**: Bug fixes, backward compatible

## API Stability

### Stable APIs

- Agent configuration format
- Extension manifest format
- CLI command interface
- gRPC service definitions

### Experimental APIs

- Component Foundry APIs
- Global Component Graph APIs
- KOR Protocol APIs

## Getting Help

- **[Developer Guide](../developer-guide/overview.md)** - Development documentation
- **[Architecture Overview](../developer-guide/architecture/overview.md)** - System architecture
- **[Examples](../examples/)** - Code examples
- **[GitHub Issues](https://github.com/clay-curry/RAD/issues)** - Report issues

## Next Steps

- **[Rust API Docs](/RAD/api/radium_core)** - Complete Rust API documentation
- **[Extension API](../extensions/api-reference.md)** - Extension system API
- **[Hooks API](../hooks/api-reference.md)** - Hooks API

---

**Need help?** Check the [Developer Guide](../developer-guide/overview.md) or [open an issue](https://github.com/clay-curry/RAD/issues).

