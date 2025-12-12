---
id: "extending-sources"
title: "Extending Context Sources"
sidebar_label: "Extending Context Sources"
---

# Extending Context Sources

This guide explains how to create custom source readers for the Radium context system, enabling integration with additional data sources beyond the built-in readers (Local, HTTP, Jira, Braingrid).

## Overview

The context source system uses a pluggable architecture where custom source readers can be registered to handle new URI schemes. This allows you to integrate with:

- Custom APIs
- Internal documentation systems
- Version control systems
- Database systems
- Any other data source accessible via URI

## SourceReader Trait

All source readers implement the `SourceReader` trait, which provides two main methods:

```rust
#[async_trait]
pub trait SourceReader: Send + Sync {
    /// Returns the URI scheme this reader handles (e.g., "file", "http", "jira").
    fn scheme(&self) -> &str;

    /// Verifies that a source exists and is accessible without downloading full content.
    async fn verify(&self, uri: &str) -> Result<SourceMetadata, SourceError>;

    /// Fetches the full content of a source.
    async fn fetch(&self, uri: &str) -> Result<String, SourceError>;
}
```

### Key Requirements

- **Send + Sync**: Readers must be thread-safe
- **Async**: Both `verify()` and `fetch()` are async operations
- **Error Handling**: Use `SourceError` for consistent error reporting
- **Metadata**: Return `SourceMetadata` with accessibility, size, and modification time when available

## Implementation Example

Here's a complete example of a custom source reader for a hypothetical "docs://" scheme:

```rust
use async_trait::async_trait;
use radium_core::context::sources::{
    SourceReader, SourceError, SourceMetadata
};
use std::sync::Arc;

/// Custom reader for internal documentation system.
pub struct DocsReader {
    /// Base URL for the documentation API.
    base_url: String,
    /// API key for authentication.
    api_key: String,
}

impl DocsReader {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self { base_url, api_key }
    }

    /// Extracts document ID from URI.
    fn extract_doc_id(&self, uri: &str) -> Result<String, SourceError> {
        uri.strip_prefix("docs://")
            .ok_or_else(|| SourceError::invalid_uri("Missing docs:// scheme"))
            .map(|s| s.to_string())
    }
}

#[async_trait]
impl SourceReader for DocsReader {
    fn scheme(&self) -> &str {
        "docs"
    }

    async fn verify(&self, uri: &str) -> Result<SourceMetadata, SourceError> {
        let doc_id = self.extract_doc_id(uri)?;
        
        // Make lightweight HEAD request to check if document exists
        let client = reqwest::Client::new();
        let url = format!("{}/api/docs/{}/meta", self.base_url, doc_id);
        
        match client
            .head(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let size = response.headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                
                Ok(SourceMetadata::with_details(
                    true,
                    size,
                    None, // Last modified not available from HEAD
                    Some("text/markdown".to_string()),
                ))
            }
            Ok(response) if response.status() == 404 => {
                Err(SourceError::not_found(&format!("Document {} not found", doc_id)))
            }
            Ok(response) if response.status() == 401 => {
                Err(SourceError::authentication_error("Invalid API key"))
            }
            Ok(response) => {
                Err(SourceError::network_error(&format!(
                    "HTTP {}: {}",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("Unknown")
                )))
            }
            Err(e) => Err(SourceError::network_error(&e.to_string())),
        }
    }

    async fn fetch(&self, uri: &str) -> Result<String, SourceError> {
        let doc_id = self.extract_doc_id(uri)?;
        
        // Fetch full document content
        let client = reqwest::Client::new();
        let url = format!("{}/api/docs/{}", self.base_url, doc_id);
        
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| SourceError::network_error(&e.to_string()))?;

        if !response.status().is_success() {
            return Err(SourceError::network_error(&format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        // Enforce size limit (10MB default)
        let content_length = response.content_length().unwrap_or(0);
        if content_length > 10 * 1024 * 1024 {
            return Err(SourceError::network_error(
                "Content exceeds 10MB size limit"
            ));
        }

        response
            .text()
            .await
            .map_err(|e| SourceError::network_error(&e.to_string()))
    }
}
```

## Registration

Once you've implemented your custom reader, register it with the `SourceRegistry`:

```rust
use radium_core::context::sources::SourceRegistry;

// Create registry
let mut registry = SourceRegistry::new();

// Register built-in readers
registry.register(Box::new(LocalFileReader::new()));
registry.register(Box::new(HttpReader::new()));
// ... other built-ins

// Register custom reader
let docs_reader = DocsReader::new(
    "https://docs.example.com".to_string(),
    std::env::var("DOCS_API_KEY")?,
);
registry.register(Box::new(docs_reader));

// Use registry in ContextManager
let manager = ContextManager::new(&workspace);
// The registry is automatically used when building context
```

## Error Handling

Use the `SourceError` type for consistent error reporting:

```rust
use radium_core::context::sources::SourceError;

// Common error types:
SourceError::invalid_uri("Invalid URI format")
SourceError::not_found("Resource not found")
SourceError::network_error("Network request failed")
SourceError::authentication_error("Authentication failed")
SourceError::IoError(io_error) // For I/O errors
```

## SourceMetadata

Return appropriate metadata in `verify()`:

```rust
use radium_core::context::sources::SourceMetadata;

// Basic metadata (just accessibility)
SourceMetadata::new(true)

// Detailed metadata
SourceMetadata::with_details(
    true,                              // accessible
    Some(1024),                        // size_bytes
    Some("2024-01-01T00:00:00Z"),     // last_modified (RFC3339)
    Some("text/markdown".to_string()), // content_type
)
```

## Best Practices

### 1. Lightweight Verification

`verify()` should be fast and avoid downloading full content:

```rust
// Good: HEAD request or metadata check
async fn verify(&self, uri: &str) -> Result<SourceMetadata, SourceError> {
    let response = client.head(&url).send().await?;
    // Check status, extract metadata from headers
}

// Bad: Downloading full content in verify()
async fn verify(&self, uri: &str) -> Result<SourceMetadata, SourceError> {
    let content = self.fetch(uri).await?; // Too expensive!
    // ...
}
```

### 2. Size Limits

Enforce size limits to prevent memory issues:

```rust
const MAX_SIZE: u64 = 10 * 1024 * 1024; // 10MB

if content_length > MAX_SIZE {
    return Err(SourceError::network_error("Content too large"));
}
```

### 3. Caching

Consider caching verification results for frequently accessed sources:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct CachedDocsReader {
    inner: DocsReader,
    cache: Arc<RwLock<HashMap<String, (SourceMetadata, SystemTime)>>>,
}
```

### 4. Timeout Handling

Always set timeouts for network requests:

```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;
```

### 5. Error Messages

Provide clear, actionable error messages:

```rust
// Good: Specific error message
Err(SourceError::not_found(&format!(
    "Document {} not found. Check if the ID is correct.",
    doc_id
)))

// Bad: Generic error
Err(SourceError::network_error("Error"))
```

## Testing

Create comprehensive tests for your custom reader:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_verify_existing_document() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("HEAD", "/api/docs/123/meta")
            .with_status(200)
            .with_header("content-length", "1024")
            .create();

        let reader = DocsReader::new(server.url(), "test-key".to_string());
        let result = reader.verify("docs://123").await;

        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert!(metadata.accessible);
        assert_eq!(metadata.size_bytes, Some(1024));
        
        mock.assert();
    }

    #[tokio::test]
    async fn test_verify_nonexistent_document() {
        // Test 404 handling
    }

    #[tokio::test]
    async fn test_fetch_document_content() {
        // Test full content fetch
    }

    #[tokio::test]
    async fn test_invalid_uri() {
        // Test URI parsing errors
    }
}
```

## Integration with ContextManager

Custom readers are automatically used when building context:

```rust
// Register custom reader
let mut registry = SourceRegistry::new();
registry.register(Box::new(DocsReader::new(...)));

// Use in context building
let manager = ContextManager::new(&workspace);
let context = manager.build_context(
    "agent[input:docs://document-123]",
    Some(req_id)
)?;
// The custom reader is automatically used for docs:// URIs
```

## Advanced: Configuration-Based Registration

For more complex setups, you can load readers from configuration:

```rust
pub fn load_readers_from_config(config: &Config) -> Result<Vec<Box<dyn SourceReader>>> {
    let mut readers = Vec::new();
    
    for source_config in &config.sources {
        match source_config.scheme.as_str() {
            "docs" => {
                let reader = DocsReader::new(
                    source_config.base_url.clone(),
                    source_config.api_key.clone(),
                );
                readers.push(Box::new(reader));
            }
            // Add other custom schemes
            _ => {}
        }
    }
    
    Ok(readers)
}
```

## Troubleshooting

### Reader Not Being Used

- Verify the scheme matches exactly (case-sensitive)
- Check that the reader is registered before use
- Ensure URI format is correct

### Verification Always Fails

- Check network connectivity
- Verify authentication credentials
- Review error messages for specific issues
- Test with `curl` or similar tools to isolate the problem

### Performance Issues

- Implement caching for verification results
- Use connection pooling for HTTP clients
- Consider async batch verification for multiple sources

## References

- [SourceReader Trait](../../crates/radium-core/src/context/sources/traits.rs) - Trait definition
- [Built-in Readers](../../crates/radium-core/src/context/sources/) - Example implementations
- [Source Registry](../../crates/radium-core/src/context/sources/registry.rs) - Registration system
- [Context Sources User Guide](../user-guide/context-sources.md) - User-facing documentation

