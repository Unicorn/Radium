---
id: "rust-api"
title: "Rust API Documentation"
sidebar_label: "Rust API"
---

# Rust API Documentation

Complete Rust API documentation for all Radium crates. All documentation is automatically generated from source code using `cargo doc`.

## Available Crates

### Core API

**[radium-core](/api/radium_core)** - Main Radium API

The core crate provides the primary Radium functionality:

- **Agents**: Agent configuration, discovery, registry, and execution
- **Workspace**: File operations, patches, transactions, and tool integration
- **Planning**: Autonomous planning, plan execution, and DAG dependencies
- **Extensions**: Extension management, discovery, marketplace, and signing
- **Hooks**: Hook system for customizing agent behavior
- **MCP**: Model Context Protocol integration
- **Context**: Context management and caching
- **Engines**: LLM engine abstraction and providers
- **Monitoring**: Agent monitoring and analytics
- **Policy**: Policy engine and constitution system
- **Security**: Security features and secret management

### Model Abstraction

**[radium-abstraction](/api/radium_abstraction)** - Model abstraction layer

Provides the core `Model` trait and abstractions for AI model providers:

- `Model` trait for text generation and chat completions
- `ModelFactory` for creating model instances
- Provider-agnostic interfaces

### Model Implementations

**[radium-models](/api/radium_models)** - Model provider implementations

Concrete implementations of the model abstraction:

- **OpenAI**: GPT-3.5, GPT-4, and other OpenAI models
- **Claude**: Anthropic Claude models
- **Gemini**: Google Gemini models
- **Ollama**: Local Ollama model integration
- **Universal**: Universal provider for custom endpoints

Includes context caching, extended parameters, and streaming support.

### Orchestration

**[radium-orchestrator](/api/radium_orchestrator)** - Orchestration system

Intelligent task routing and agent coordination:

- Agent routing and selection
- Load balancing and failover
- Batch execution
- Progress tracking
- Queue management

## Quick Navigation

- [radium-core API](/api/radium_core) - Start here for most use cases
- [radium-abstraction API](/api/radium_abstraction) - For implementing custom model providers
- [radium-models API](/api/radium_models) - For model-specific functionality
- [radium-orchestrator API](/api/radium_orchestrator) - For orchestration features

## Documentation Generation

All Rust API documentation is automatically generated from source code comments using `cargo doc`. The documentation includes:

- Type definitions and trait implementations
- Function and method signatures
- Code examples from doc comments
- Module organization
- Cross-references between types

## Using the API

### Installation

Add Radium crates to your `Cargo.toml`:

```toml
[dependencies]
radium-core = { git = "https://github.com/Unicorn/Radium" }
radium-abstraction = { git = "https://github.com/Unicorn/Radium" }
radium-models = { git = "https://github.com/Unicorn/Radium" }
radium-orchestrator = { git = "https://github.com/Unicorn/Radium" }
```

### Example Usage

```rust
use radium_core::Workspace;
use radium_core::agents::AgentRegistry;
use radium_abstraction::Model;

// Initialize workspace
let workspace = Workspace::new("/path/to/workspace")?;

// Get agent registry
let registry = AgentRegistry::new();
let agent = registry.get_agent("agent-id")?;

// Use model abstraction
let model: Box<dyn Model> = /* create model */;
let response = model.generate_text("Hello, world!", None).await?;
```

## Related Documentation

- [API Overview](./overview.md) - Complete API reference guide
- [Extension API](../extensions/api-reference.md) - Extension system
- [Hooks API](../hooks/api-reference.md) - Hooks system
- [Developer Guide](../developer-guide/overview.md) - Development documentation

---

**Note**: Rust API documentation is generated automatically. If you find any issues or have suggestions, please [open an issue](https://github.com/Unicorn/Radium/issues).

