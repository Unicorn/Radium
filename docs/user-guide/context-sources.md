# Context Sources

Context sources allow you to fetch and inject content from external systems into agent prompts. Radium supports multiple source protocols for flexible context gathering.

## Overview

Context sources enable agents to access information from:
- **Local Files**: File system files via `file://` URIs
- **HTTP/HTTPS**: Web resources via `http://` and `https://` URIs
- **Jira**: Jira issues and projects via `jira://` URIs
- **Braingrid**: Braingrid requirements and tasks via `braingrid://` URIs

## Local File Sources

Local file sources use the `file://` protocol to read files from the file system.

### Basic Usage

```rust
let reader = LocalFileReader::with_base_dir(&workspace_root);
let content = reader.fetch("file:///path/to/file.txt").await?;
```

### In ContextManager

Local file sources are automatically registered and used when building context:

```rust
let manager = ContextManager::new(&workspace);
let context = manager.build_context("agent[input:file:///path/to/file.md]", None)?;
```

File URIs in injection syntax are automatically resolved:
```bash
rad step agent[input:file:///absolute/path/to/file.md]
```

## HTTP/HTTPS Sources

HTTP sources allow fetching content from web resources.

### Basic Usage

```rust
let reader = HttpReader::new();
let metadata = reader.verify("https://example.com/document.md").await?;
let content = reader.fetch("https://example.com/document.md").await?;
```

### In ContextManager

HTTP sources are automatically available:

```rust
// Fetch content from HTTP source
let context = manager.build_context(
    "agent[input:https://api.example.com/spec.json]",
    None
)?;
```

### Error Handling

HTTP sources handle common errors:
- Network timeouts
- HTTP errors (404, 500, etc.)
- SSL certificate errors
- Size limits (default: 10MB)

## Jira Sources

Jira sources fetch content from Jira issues using the `jira://` protocol.

### URI Format

```
jira://<project-key>-<issue-number>
```

Examples:
- `jira://PROJ-123` - Issue PROJ-123
- `jira://REQ-456` - Issue REQ-456

### Authentication

Jira sources require authentication credentials configured in your workspace:

```toml
# .radium/config.toml
[auth.jira]
username = "user@example.com"
api_token = "your-api-token"
# or
password = "your-password"
```

### Basic Usage

```rust
let reader = JiraReader::new();
let content = reader.fetch("jira://PROJ-123").await?;
```

### In ContextManager

```rust
let context = manager.build_context(
    "agent[input:jira://PROJ-123]",
    None
)?;
```

## Braingrid Sources

Braingrid sources fetch requirements and tasks from Braingrid using the `braingrid://` protocol.

### URI Format

```
braingrid://<node-id>
```

Examples:
- `braingrid://REQ-69` - Requirement REQ-69
- `braingrid://TASK-1` - Task TASK-1

### Authentication

Braingrid sources use the `braingrid` CLI tool for authentication. Ensure you're logged in:

```bash
braingrid auth login
```

### Basic Usage

```rust
let reader = BraingridReader::new();
let content = reader.fetch("braingrid://REQ-69").await?;
```

### In ContextManager

```rust
let context = manager.build_context(
    "agent[input:braingrid://REQ-69]",
    None
)?;
```

## Source Registry

The Source Registry automatically routes URIs to the correct reader based on the URI scheme.

### Automatic Registration

All source readers are automatically registered in ContextManager:

```rust
let manager = ContextManager::new(&workspace);
// SourceRegistry is initialized with all readers:
// - LocalFileReader
// - HttpReader
// - JiraReader
// - BraingridReader
```

### Manual Registration

You can manually register custom readers:

```rust
use radium_core::context::sources::{SourceRegistry, SourceReader};

let mut registry = SourceRegistry::new();
registry.register(Box::new(MyCustomReader::new()));
```

### URI Routing

The registry automatically routes URIs:
- `file://...` → LocalFileReader
- `http://...` or `https://...` → HttpReader
- `jira://...` → JiraReader
- `braingrid://...` → BraingridReader

## Source Verification

Before fetching, you can verify a source is accessible:

```rust
let metadata = reader.verify("https://example.com/doc.md").await?;
if metadata.accessible {
    let content = reader.fetch("https://example.com/doc.md").await?;
}
```

### Verification Results

SourceMetadata includes:
- **accessible**: Whether the source is reachable
- **size**: Content size in bytes (if available)
- **last_modified**: Last modification time (if available)

## Error Handling

All source readers return consistent errors:

- **InvalidUri**: URI format is invalid
- **NotFound**: Source doesn't exist or is inaccessible
- **NetworkError**: Network request failed
- **AuthenticationError**: Authentication failed (Jira, Braingrid)

### Example Error Handling

```rust
match reader.fetch(&uri).await {
    Ok(content) => {
        // Use content
    }
    Err(SourceError::NotFound(_)) => {
        // Source doesn't exist
    }
    Err(SourceError::AuthenticationError(_)) => {
        // Need to authenticate
    }
    Err(e) => {
        // Other error
    }
}
```

## Best Practices

### Source Reliability

- Use local files for critical context (most reliable)
- HTTP sources may be unavailable (handle errors gracefully)
- Jira/Braingrid require authentication (configure credentials)

### Performance

- Sources are fetched asynchronously
- HTTP sources have a 10MB size limit by default
- Cache content locally if fetching repeatedly

### Security

- Don't expose sensitive credentials in URIs
- Use environment variables or config files for authentication
- Verify SSL certificates for HTTPS sources

## Troubleshooting

### HTTP Source Fails

- Check network connectivity
- Verify URL is correct and accessible
- Check for SSL certificate issues
- Verify size limits aren't exceeded

### Jira Source Fails

- Verify credentials in `.radium/config.toml`
- Check Jira URL is accessible
- Verify API token has read permissions
- Check issue key format is correct

### Braingrid Source Fails

- Ensure `braingrid` CLI is installed and in PATH
- Verify you're logged in: `braingrid auth status`
- Check project ID is correct
- Verify node ID exists in Braingrid

