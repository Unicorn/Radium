---
id: "engine-abstraction"
title: "Engine Abstraction Layer Architecture"
sidebar_label: "Engine Abstraction Layer Ar..."
---

# Engine Abstraction Layer Architecture

## Overview

The Engine Abstraction Layer provides a unified interface for interacting with multiple AI providers (Claude, OpenAI, Gemini, etc.) through a pluggable, trait-based architecture. This design enables users to switch between providers seamlessly while maintaining a consistent API.

## Core Components

### Engine Trait

The `Engine` trait (`crates/radium-core/src/engines/engine_trait.rs`) defines the interface that all AI providers must implement:

```rust
#[async_trait]
pub trait Engine: Send + Sync {
    fn metadata(&self) -> &EngineMetadata;
    async fn is_available(&self) -> bool;
    async fn is_authenticated(&self) -> Result<bool>;
    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse>;
    fn default_model(&self) -> String;
    fn available_models(&self) -> Vec<String>;
}
```

**Key Responsibilities:**
- **Metadata**: Provides engine identification and capabilities
- **Availability**: Checks if the engine is ready to use (binary exists, API accessible)
- **Authentication**: Verifies credentials are configured
- **Execution**: Processes requests and returns responses
- **Model Management**: Lists available models and provides defaults

### Engine Metadata

`EngineMetadata` contains static information about an engine:

- `id`: Unique identifier (e.g., "claude", "openai", "gemini")
- `name`: Human-readable name
- `description`: Brief description of the engine
- `cli_command`: Optional CLI binary name (for CLI-based engines)
- `models`: List of supported model identifiers
- `requires_auth`: Whether authentication is required
- `version`: Optional version string

### Execution Request/Response

**ExecutionRequest** contains:
- `model`: Model identifier to use
- `prompt`: User prompt or message
- `system`: Optional system message
- `temperature`: Optional temperature (0.0-1.0)
- `max_tokens`: Optional maximum tokens to generate
- `params`: Additional provider-specific parameters

**ExecutionResponse** contains:
- `content`: Generated text content
- `usage`: Optional token usage information
- `model`: Model identifier used
- `raw`: Optional raw response for debugging

### Engine Registry

The `EngineRegistry` (`crates/radium-core/src/engines/registry.rs`) manages engine instances and provides:

**Core Operations:**
- `register(engine)`: Register a new engine instance
- `get(id)`: Retrieve an engine by ID
- `list()`: List all registered engines
- `set_default(id)`: Set the default engine
- `get_default()`: Get the default engine
- `check_health(timeout)`: Check health of all engines

**Configuration Management:**
- `load_config()`: Load configuration from `.radium/config.toml`
- `save_config()`: Persist configuration to disk
- `with_config_path(path)`: Create registry with config path

**Thread Safety:**
- Uses `Arc<RwLock<>>` for thread-safe concurrent access
- Engines are wrapped in `Arc<dyn Engine>` for shared ownership

## Architecture Patterns

### Registry Pattern

The registry pattern centralizes engine management:

```
┌─────────────────┐
│ EngineRegistry  │
│                 │
│ ┌─────────────┐ │
│ │  Engines    │ │─── HashMap<String, `Arc<dyn Engine>`>
│ │  (HashMap)  │ │
│ └─────────────┘ │
│                 │
│ ┌─────────────┐ │
│ │   Config    │ │─── .radium/config.toml
│ │   (TOML)    │ │
│ └─────────────┘ │
└─────────────────┘
```

### Provider Pattern

Each provider implements the `Engine` trait:

```
┌──────────────┐
│ Engine Trait │
└──────┬───────┘
       │
       ├─── MockEngine
       ├─── ClaudeEngine
       ├─── OpenAIEngine
       └─── GeminiEngine
```

### Configuration Hierarchy

Configuration follows a precedence order:

1. **Workspace Config** (`.radium/config.toml`) - Highest priority
2. **Global Config** (`~/.radium/config.toml`) - Fallback
3. **Defaults** - Built-in defaults

## Lifecycle Management

### Engine Registration

1. Engine instances are created (e.g., `ClaudeEngine::new()`)
2. Engines are registered with the registry: `registry.register(Arc::new(engine))`
3. Configuration is loaded from disk
4. Default engine is set (if configured)

### Request Execution Flow

```
CLI/API Request
    │
    ├─> EngineRegistry::get_default()
    │       │
    │       └─> Engine::execute(request)
    │               │
    │               ├─> Check authentication
    │               ├─> Validate request
    │               ├─> Call provider API
    │               └─> Return ExecutionResponse
    │
    └─> Response returned to caller
```

### Health Monitoring

Health checks verify engine availability and authentication:

```rust
pub async fn check_health(&self, timeout_secs: u64) -> Vec<EngineHealth>
```

Each health check:
1. Verifies engine availability (`is_available()`)
2. Checks authentication status (`is_authenticated()`)
3. Returns `HealthStatus` (Healthy, Warning, Failed)
4. Respects timeout for slow checks

## Error Handling

The engine system uses a unified error type (`EngineError`):

- `NotFound`: Engine not found in registry
- `AuthenticationFailed`: Credential issues
- `ExecutionError`: Request/response errors
- `InvalidConfig`: Configuration parsing errors
- `RegistryError`: Registry operation failures
- `Io`: File system errors

## Configuration Format

Engine configuration is stored in TOML format:

```toml
[engines]
default = "gemini"

[engines.gemini]
default_model = "gemini-2.0-flash-exp"
temperature = 0.7
max_tokens = 4096

[engines.openai]
default_model = "gpt-4-turbo"
temperature = 0.8
```

## Authentication

Engines use the `CredentialStore` for API key management:

- Credentials stored in `~/.radium/credentials.json`
- Provider-specific keys (Claude, OpenAI, Gemini)
- Secure storage with encryption support
- Environment variable fallback (future)

## Current Providers

### Mock Engine
- **ID**: `mock`
- **Purpose**: Testing and development
- **Auth**: Not required
- **Models**: `mock-model-1`, `mock-model-2`

### Claude Engine
- **ID**: `claude`
- **Provider**: Anthropic
- **Auth**: Required (API key)
- **Models**: `claude-3-opus-20240229`, `claude-3-sonnet-20240229`, `claude-3-haiku-20240307`
- **API**: REST API (https://api.anthropic.com/v1/messages)

### OpenAI Engine
- **ID**: `openai`
- **Provider**: OpenAI
- **Auth**: Required (API key)
- **Models**: `gpt-4`, `gpt-4-turbo`, `gpt-3.5-turbo`
- **API**: Uses `radium-models::OpenAIModel`

### Gemini Engine
- **ID**: `gemini`
- **Provider**: Google
- **Auth**: Required (API key)
- **Models**: `gemini-pro`, `gemini-pro-vision`, `gemini-2.0-flash-exp`
- **API**: Uses `radium-models::GeminiModel`

## Extension Points

### Adding New Providers

To add a new provider:

1. Implement the `Engine` trait
2. Create provider struct with metadata
3. Implement `execute()` method
4. Register in CLI initialization
5. Add authentication support (if needed)

See `docs/guides/adding-new-engine-provider.md` for detailed instructions.

### Custom Configuration

Engines can extend configuration by:
- Adding custom fields to `ExecutionRequest.params`
- Implementing provider-specific config sections
- Using environment variables for sensitive data

## Performance Considerations

- **Concurrent Access**: Registry uses `RwLock` for read-heavy workloads
- **Health Checks**: Timeout-based to prevent blocking
- **Caching**: Engine instances are reused (Arc-based)
- **Async Execution**: All I/O operations are async

## Future Enhancements

- Streaming response support
- Vision/multimodal capabilities
- Function calling/tool use
- Advanced metrics and monitoring
- Cost tracking per engine
- Provider-specific optimizations

