---
id: "adding-new-engine-provider"
title: "Adding a New Engine Provider"
sidebar_label: "Adding a New Engine Provider"
---

# Adding a New Engine Provider

This guide walks through implementing a new AI provider for the Radium engine abstraction layer.

## Overview

Adding a new engine provider involves:
1. Creating a provider struct that implements the `Engine` trait
2. Implementing required methods
3. Registering the provider in the CLI
4. Adding tests
5. Documenting the provider

## Step-by-Step Implementation

### Step 1: Create Provider Module

Create a new file in `crates/radium-core/src/engines/providers/`:

```rust
//! YourProvider engine implementation.

use crate::auth::{CredentialStore, ProviderType};
use crate::engines::engine_trait::{
    Engine, EngineMetadata, ExecutionRequest, ExecutionResponse, TokenUsage,
};
use crate::engines::error::{EngineError, Result};
use async_trait::async_trait;
use std::sync::Arc;

/// YourProvider engine implementation.
pub struct YourProviderEngine {
    /// Engine metadata.
    metadata: EngineMetadata,
    /// Credential store for API key retrieval.
    credential_store: Arc<CredentialStore>,
}

impl YourProviderEngine {
    /// Creates a new YourProvider engine.
    pub fn new() -> Self {
        let metadata = EngineMetadata::new(
            "your-provider".to_string(),
            "Your Provider".to_string(),
            "Description of your provider".to_string(),
        )
        .with_models(vec![
            "model-1".to_string(),
            "model-2".to_string(),
        ])
        .with_auth_required(true);

        let credential_store = CredentialStore::new().unwrap_or_else(|_| {
            let temp_path = std::env::temp_dir().join("radium_credentials.json");
            CredentialStore::with_path(temp_path)
        });

        Self {
            metadata,
            credential_store: Arc::new(credential_store),
        }
    }

    /// Gets the API key from credential store.
    fn get_api_key(&self) -> Result<String> {
        self.credential_store
            .get(ProviderType::YourProvider) // Add to ProviderType enum
            .map_err(|e| EngineError::AuthenticationFailed(format!("Failed to get API key: {}", e)))
    }
}

impl Default for YourProviderEngine {
    fn default() -> Self {
        Self::new()
    }
}
```

### Step 2: Implement Engine Trait

Implement the required trait methods:

```rust
#[async_trait]
impl Engine for YourProviderEngine {
    fn metadata(&self) -> &EngineMetadata {
        &self.metadata
    }

    async fn is_available(&self) -> bool {
        // For API-based providers, always return true
        // For CLI-based providers, check if binary exists
        true
    }

    async fn is_authenticated(&self) -> Result<bool> {
        match self.get_api_key() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse> {
        let api_key = self.get_api_key()?;

        // Build API request
        // Make HTTP request to provider API
        // Parse response
        // Convert to ExecutionResponse

        Ok(ExecutionResponse {
            content: "Response content".to_string(),
            usage: Some(TokenUsage {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            }),
            model: request.model,
            raw: Some("Raw response".to_string()),
        })
    }

    fn default_model(&self) -> String {
        "model-1".to_string()
    }
}
```

### Step 3: Add to Provider Module

Update `crates/radium-core/src/engines/providers/mod.rs`:

```rust
pub mod your_provider; // Add this line

pub use your_provider::YourProviderEngine; // Add this line
```

### Step 4: Add Authentication Support

If your provider requires authentication, add it to the credential store:

1. Add provider type to `crates/radium-core/src/auth/mod.rs`:
```rust
pub enum ProviderType {
    // ... existing providers
    YourProvider,
}
```

2. Update credential store to handle your provider type

### Step 5: Register in CLI

Update `apps/cli/src/commands/engines.rs`:

```rust
use radium_core::engines::providers::{YourProviderEngine, /* ... */};

fn init_registry() -> EngineRegistry {
    // ... existing code
    
    // Register your provider
    let _ = registry.register(Arc::new(YourProviderEngine::new()));
    
    // ... rest of initialization
}
```

### Step 6: Write Tests

Create comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_your_provider_metadata() {
        let engine = YourProviderEngine::new();
        let metadata = engine.metadata();

        assert_eq!(metadata.id, "your-provider");
        assert_eq!(metadata.name, "Your Provider");
        assert!(metadata.requires_auth);
    }

    #[tokio::test]
    async fn test_your_provider_is_available() {
        let engine = YourProviderEngine::new();
        assert!(engine.is_available().await);
    }

    #[tokio::test]
    async fn test_your_provider_default_model() {
        let engine = YourProviderEngine::new();
        assert_eq!(engine.default_model(), "model-1");
    }

    // Add more tests for execute(), authentication, etc.
}
```

## Implementation Examples

### API-Based Provider (Claude-style)

For REST API providers:

```rust
use reqwest::Client;

pub struct ApiProviderEngine {
    metadata: EngineMetadata,
    client: Arc<Client>,
    credential_store: Arc<CredentialStore>,
}

impl ApiProviderEngine {
    pub fn new() -> Self {
        // ... metadata setup
        Self {
            metadata,
            client: Arc::new(Client::new()),
            credential_store: Arc::new(credential_store),
        }
    }
}

#[async_trait]
impl Engine for ApiProviderEngine {
    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse> {
        let api_key = self.get_api_key()?;
        
        // Build request
        let api_request = ApiRequest {
            model: request.model,
            prompt: request.prompt,
            // ... other fields
        };

        // Make HTTP request
        let response = self
            .client
            .post("https://api.provider.com/v1/chat")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&api_request)
            .send()
            .await?;

        // Parse and return
        // ...
    }
}
```

### Wrapper Provider (Gemini-style)

For providers that use existing model abstractions:

```rust
use radium_models::YourModel;
use radium_abstraction::{Model, ModelParameters};

#[async_trait]
impl Engine for WrapperProviderEngine {
    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse> {
        let api_key = self.get_api_key()?;
        let model = YourModel::with_api_key(request.model.clone(), api_key);
        
        let parameters = ModelParameters {
            temperature: request.temperature,
            max_tokens: request.max_tokens.map(|t| t as u32),
            // ...
        };

        let response = model
            .generate_text(&request.prompt, Some(parameters))
            .await?;

        Ok(ExecutionResponse {
            content: response.content,
            usage: convert_usage(response.usage),
            model: response.model_id.unwrap_or(request.model),
            raw: None,
        })
    }
}
```

## Testing Requirements

### Unit Tests

- Metadata correctness
- Default model selection
- Availability checks
- Authentication status
- Error handling

### Integration Tests

- End-to-end execution (with mock API)
- Configuration loading
- Registry registration
- Health checks

### Example Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() { /* ... */ }
    
    #[tokio::test]
    async fn test_is_available() { /* ... */ }
    
    #[tokio::test]
    async fn test_is_authenticated() { /* ... */ }
    
    #[tokio::test]
    async fn test_execute_success() { /* ... */ }
    
    #[tokio::test]
    async fn test_execute_auth_failure() { /* ... */ }
    
    #[tokio::test]
    async fn test_execute_api_error() { /* ... */ }
}
```

## Best Practices

### Error Handling

- Use `EngineError` types consistently
- Provide clear error messages
- Handle network errors gracefully
- Validate inputs before API calls

### Performance

- Reuse HTTP clients (`Arc&lt;Client&gt;`)
- Use async/await for I/O
- Cache authentication status when possible
- Implement timeouts for API calls

### Security

- Never log API keys
- Use secure credential storage
- Validate API responses
- Handle rate limits appropriately

### Code Organization

- Keep provider logic in separate modules
- Use helper functions for common operations
- Document public APIs
- Follow existing code patterns

## Documentation

After implementing your provider:

1. Update `docs/architecture/engine-abstraction.md` with provider details
2. Add usage examples to README
3. Document authentication setup
4. Include model capabilities and limitations

## Checklist

- [ ] Provider struct created
- [ ] Engine trait implemented
- [ ] Added to providers module
- [ ] Registered in CLI
- [ ] Authentication support added
- [ ] Unit tests written
- [ ] Integration tests written
- [ ] Documentation updated
- [ ] Error handling implemented
- [ ] Code follows project conventions

## Troubleshooting

### Common Issues

**Authentication fails:**
- Verify ProviderType is added to auth module
- Check credential store path
- Ensure API key format is correct

**Engine not found:**
- Verify registration in CLI init_registry()
- Check engine ID matches metadata
- Ensure module is exported

**Tests fail:**
- Check async test attributes (`#[tokio::test]`)
- Verify mock setup for API calls
- Ensure error types match

## Next Steps

After implementing your provider:

1. Run full test suite: `cargo test`
2. Test CLI commands: `rad engines list`
3. Verify health checks: `rad engines health`
4. Test execution with real API (if available)
5. Submit for code review

## Reference Implementations

- **Claude**: `crates/radium-core/src/engines/providers/claude.rs` - Direct API implementation
- **Gemini**: `crates/radium-core/src/engines/providers/gemini.rs` - Wrapper around radium-models
- **Mock**: `crates/radium-core/src/engines/providers/mock.rs` - Testing provider

