---
id: "agent-system-architecture"
title: "Agent System Architecture"
sidebar_label: "Agent System Architecture"
---

# Agent System Architecture

This document describes the technical architecture of Radium's agent configuration system for developers.

## System Overview

The agent configuration system provides a declarative way to define, discover, and manage AI agents. It consists of four main components:

1. **Agent Configuration** (`AgentConfig`) - Data structures and TOML parsing
2. **Agent Discovery** (`AgentDiscovery`) - Recursive directory scanning and loading
3. **Agent Registry** (`AgentRegistry`) - Thread-safe runtime management
4. **Prompt Templates** (`PromptTemplate`) - Template loading and placeholder replacement

```
┌─────────────────┐
│  Agent Config  │  TOML files in agents/ directories
│     Files       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Discovery     │  Scans directories, loads configs
│   System        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Registry     │  Thread-safe storage and lookup
│   (Runtime)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Prompt        │  Loads and renders templates
│   Templates     │
└─────────────────┘
```

## Component Details

### AgentConfig Data Structure

**Location**: `crates/radium-core/src/agents/config.rs`

The `AgentConfig` struct represents a single agent configuration:

```rust
pub struct AgentConfig {
    pub id: String,                    // Required: unique identifier
    pub name: String,                   // Required: human-readable name
    pub description: String,            // Required: agent description
    pub prompt_path: PathBuf,          // Required: path to prompt template
    pub mirror_path: Option<PathBuf>,  // Optional: mirror path for RAD-agents
    pub engine: Option<String>,        // Optional: default AI engine
    pub model: Option<String>,          // Optional: default model
    pub reasoning_effort: Option<ReasoningEffort>, // Optional: reasoning level
    pub loop_behavior: Option<AgentLoopBehavior>,  // Optional: loop config
    pub trigger_behavior: Option<AgentTriggerBehavior>, // Optional: trigger config
    pub capabilities: AgentCapabilities, // Optional: model class, cost tier, concurrency
    pub sandbox: Option<SandboxConfig>,  // Optional: sandbox configuration
    pub category: Option<String>,       // Derived from path (not in TOML)
    pub file_path: Option<PathBuf>,    // Set during loading (not in TOML)
}
```

**Key Features**:
- TOML serialization/deserialization via `serde`
- Builder pattern for programmatic construction
- Validation of required fields
- Support for optional behaviors (loop, trigger)

**Related Types**:
- `AgentConfigFile` - Wrapper for TOML file operations
- `ReasoningEffort` - Enum: `Low`, `Medium`, `High`
- `AgentLoopBehavior` - Loop configuration with steps, max_iterations, skip
- `AgentTriggerBehavior` - Trigger configuration with trigger_agent_id

### AgentDiscovery Mechanism

**Location**: `crates/radium-core/src/agents/discovery.rs`

The discovery system recursively scans directories for agent configuration files.

**Discovery Process**:

1. **Search Path Resolution**: Determines directories to scan (default or custom)
2. **Recursive Scanning**: Walks directory trees looking for `.toml` files
3. **Configuration Loading**: Parses each TOML file into `AgentConfig`
4. **Category Derivation**: Extracts category from directory structure
5. **Path Resolution**: Resolves relative prompt paths
6. **Filtering**: Applies sub-agent filters if configured
7. **Deduplication**: Handles duplicate IDs (later entries override)

**Default Search Paths** (in precedence order):

1. `./agents/` - Project-local agents (highest precedence)
2. `~/.radium/agents/` - User-level agents
3. Workspace agents (if applicable)
4. `./.radium/extensions/*/agents/` - Project-level extension agents
5. `~/.radium/extensions/*/agents/` - User-level extension agents (lowest precedence)

**API**:

```rust
pub struct AgentDiscovery {
    options: DiscoveryOptions,
}

impl AgentDiscovery {
    pub fn new() -> Self;
    pub fn with_options(options: DiscoveryOptions) -> Self;
    pub fn discover_all(&self) -> Result<HashMap<String, AgentConfig>>;
    pub fn find_by_id(&self, id: &str) -> Result<Option<AgentConfig>>;
    pub fn list_ids(&self) -> Result<Vec<String>>;
}
```

**Category Derivation Algorithm**:

The category is derived from the file path relative to the discovery root:

- `agents/core/arch-agent.toml` → category: `"core"`
- `agents/custom/my-agent.toml` → category: `"custom"`
- `agents/rad-agents/design/design-agent.toml` → category: `"rad-agents/design"`

### AgentRegistry Implementation

**Location**: `crates/radium-core/src/agents/registry.rs`

The registry provides thread-safe runtime management of discovered agents.

**Thread Safety**:

Uses `Arc<RwLock<HashMap<String, AgentConfig>>>` for concurrent access:
- Multiple readers can access simultaneously (read lock)
- Writers have exclusive access (write lock)
- Lock poisoning is handled gracefully

**API**:

```rust
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentConfig>>>,
}

impl AgentRegistry {
    pub fn new() -> Self;
    pub fn with_discovery() -> Result<Self>;
    pub fn with_discovery_options(options: DiscoveryOptions) -> Result<Self>;
    
    // Registration
    pub fn register(&self, agent: AgentConfig) -> Result<()>;
    pub fn register_or_replace(&self, agent: AgentConfig) -> Result<()>;
    
    // Lookup
    pub fn get(&self, id: &str) -> Result<AgentConfig>;
    pub fn contains(&self, id: &str) -> Result<bool>;
    
    // Listing
    pub fn list_all(&self) -> Result<Vec<AgentConfig>>;
    pub fn list_ids(&self) -> Result<Vec<String>>;
    
    // Search and Filter
    pub fn search(&self, query: &str) -> Result<Vec<AgentConfig>>;
    pub fn filter<F>(&self, predicate: F) -> Result<Vec<AgentConfig>>
        where F: Fn(&AgentConfig) -> bool;
    
    // Management
    pub fn count(&self) -> Result<usize>;
    pub fn clear(&self) -> Result<()>;
    pub fn remove(&self, id: &str) -> Result<AgentConfig>;
    
    // Discovery Integration
    pub fn discover_and_register(&self) -> Result<()>;
    pub fn discover_and_register_with_options(&self, options: DiscoveryOptions) -> Result<()>;
}
```

**Performance Characteristics**:
- Lookup by ID: O(1) - HashMap lookup
- Search: O(n) - Linear scan with predicate
- Filter: O(n) - Linear scan with predicate
- Thread-safe reads: Concurrent (RwLock read)
- Thread-safe writes: Exclusive (RwLock write)

### PromptTemplate System

**Location**: `crates/radium-core/src/prompts/templates.rs`

The prompt template system loads markdown templates and performs placeholder replacement.

**Placeholder Syntax**:

Placeholders use double braces: `{{KEY}}`

- Detected via character-by-character parsing
- Supports whitespace: `{{ KEY }}` → `KEY`
- Handles edge cases (nested braces, unclosed placeholders)

**Rendering Modes**:

1. **Strict Mode**: Errors if placeholder is missing from context
2. **Non-Strict Mode**: Uses default value or empty string for missing placeholders

**API**:

```rust
pub struct PromptTemplate {
    content: String,
    file_path: Option<PathBuf>,
}

impl PromptTemplate {
    pub fn load(path: impl AsRef<Path>) -> Result<Self>;
    pub fn from_string(content: impl Into<String>) -> Self;
    pub fn render(&self, context: &PromptContext) -> Result<String>;
    pub fn render_with_options(&self, context: &PromptContext, options: &RenderOptions) -> Result<String>;
    pub fn list_placeholders(&self) -> Vec<String>;
}

pub struct PromptContext {
    values: HashMap<String, String>,
}

impl PromptContext {
    pub fn new() -> Self;
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>);
    pub fn get(&self, key: &str) -> Option<&str>;
    pub fn contains(&self, key: &str) -> bool;
}
```

**Placeholder Detection Algorithm**:

1. Scan content character by character
2. Detect `{{` opening sequence
3. Collect characters until `}}` closing sequence
4. Trim whitespace from placeholder name
5. Add to unique list of placeholders
6. Handle edge cases (nested braces, unclosed, empty)

## Component Interactions

### Discovery → Registry Flow

```
1. AgentDiscovery::discover_all()
   └─> Scans directories
   └─> Returns HashMap<String, AgentConfig>

2. AgentRegistry::with_discovery()
   └─> Creates AgentDiscovery
   └─> Calls discover_all()
   └─> Populates registry HashMap
```

### Registry → Template Flow

```
1. AgentRegistry::get(id)
   └─> Returns AgentConfig

2. AgentConfig.prompt_path
   └─> Path to prompt template

3. PromptTemplate::load(path)
   └─> Loads markdown file

4. PromptTemplate::render(context)
   └─> Replaces {{KEY}} placeholders
   └─> Returns rendered string
```

## Thread Safety Considerations

### AgentRegistry

- **Read Operations**: Concurrent access via `RwLock::read()`
  - Multiple threads can call `get()`, `search()`, `filter()` simultaneously
  - No blocking between readers

- **Write Operations**: Exclusive access via `RwLock::write()`
  - `register()`, `remove()`, `clear()` block all other operations
  - Writers wait for all readers to finish

- **Lock Poisoning**: All methods handle `PoisonError` gracefully
  - Returns `RegistryError::LockPoisoned` instead of panicking
  - Allows recovery from panicked threads

### AgentDiscovery

- **Stateless**: No shared mutable state
- **Thread-Safe**: Can be used concurrently from multiple threads
- **No Locking Required**: Each discovery operation is independent

### PromptTemplate

- **Immutable**: Template content doesn't change after creation
- **Thread-Safe**: Can be shared across threads without synchronization
- **Context is Not Thread-Safe**: `PromptContext` should not be shared

## Extension Points

### Custom Discovery Options

```rust
let mut options = DiscoveryOptions::default();
options.search_paths = vec![PathBuf::from("/custom/path")];
options.sub_agent_filter = Some(vec!["agent-1".to_string()]);

let discovery = AgentDiscovery::with_options(options);
```

### Custom Registry Filtering

```rust
let registry = AgentRegistry::with_discovery()?;

// Filter by custom predicate
let filtered = registry.filter(|agent| {
    agent.engine.as_ref().map(|e| e == "gemini").unwrap_or(false)
})?;
```

### Custom Prompt Rendering

```rust
let template = PromptTemplate::load("prompt.md")?;
let mut context = PromptContext::new();
context.set("key", "value");

let options = RenderOptions {
    strict: false,
    default_value: Some("default".to_string()),
};

let rendered = template.render_with_options(&context, &options)?;
```

## Testing Guidelines

### Unit Tests

Each component has comprehensive unit tests:

- **config.rs**: Tests for TOML parsing, validation, builder pattern
- **discovery.rs**: Tests for directory scanning, category derivation
- **registry.rs**: Tests for thread-safety, search, filtering
- **templates.rs**: Tests for placeholder detection, rendering modes

### Integration Tests

**Location**: `crates/radium-core/tests/agent_config_integration_test.rs`

Integration tests cover:

1. **Full Discovery Workflow**: Multiple agents, categories, path resolution
2. **Registry with Discovery**: Lookup, search, filtering, thread-safety
3. **Prompt Templates**: Loading, rendering, placeholder replacement
4. **Behaviors**: Loop and trigger behavior parsing
5. **Error Scenarios**: Missing files, invalid TOML, duplicate IDs

**Test Helpers**:

- `create_test_workspace()` - Creates temporary directory structure
- `create_test_agent()` - Creates agent config and prompt files
- `create_test_agent_full()` - Creates agent with all optional fields
- `create_test_agent_with_loop_behavior()` - Creates agent with loop behavior
- `create_test_agent_with_trigger_behavior()` - Creates agent with trigger behavior

### Running Tests

```bash
# Run all agent configuration tests
cargo test --test agent_config_integration_test

# Run specific test
cargo test --test agent_config_integration_test test_full_agent_discovery_workflow

# Run with output
cargo test --test agent_config_integration_test -- --nocapture
```

## Error Handling

### Error Types

- **`AgentConfigError`**: Configuration parsing and validation errors
- **`DiscoveryError`**: I/O errors, configuration errors during discovery
- **`RegistryError`**: Lookup failures, lock poisoning, duplicate registration
- **`PromptError`**: Template loading, missing placeholders, I/O errors

### Error Propagation

All errors use `Result<T, E>` types and implement `std::error::Error`:

```rust
pub type Result<T> = std::result::Result<T, AgentConfigError>;
pub type Result<T> = std::result::Result<T, DiscoveryError>;
pub type Result<T> = std::result::Result<T, RegistryError>;
pub type Result<T> = std::result::Result<T, PromptError>;
```

### Error Messages

Error messages are descriptive and actionable:

- `"agent ID cannot be empty"` - Clear validation error
- `"agent not found: arch-agent"` - Specific lookup failure
- `"template not found: /path/to/template.md"` - File not found with path
- `"missing placeholder value: user_name"` - Missing context value

## Performance Considerations

### Discovery Performance

- **Directory Scanning**: Uses `fs::read_dir` for recursive traversal
- **File I/O**: Sequential file reading (could be parallelized if needed)
- **Caching**: No built-in caching - discovery happens on-demand
- **Optimization**: Consider caching discovered agents if discovery is frequent

### Registry Performance

- **Lookup**: O(1) HashMap lookup - very fast
- **Search**: O(n) linear scan - acceptable for typical agent counts (< 100)
- **Thread Contention**: Minimal with read-heavy workloads
- **Memory**: Stores full `AgentConfig` copies - acceptable for typical sizes

### Template Performance

- **Placeholder Detection**: O(n) single pass through content
- **Replacement**: O(n*m) where n is content length, m is placeholder count
- **Caching**: No built-in caching - templates are loaded on-demand
- **Optimization**: Consider caching loaded templates if same template is used frequently

## Future Enhancements

Potential improvements:

1. **Discovery Caching**: Cache discovered agents to avoid repeated scanning
2. **Template Caching**: Cache loaded templates for better performance
3. **Parallel Discovery**: Parallelize directory scanning for large agent sets
4. **Hot Reloading**: Watch for file changes and reload agents automatically
5. **Validation Rules**: Extensible validation rules for agent configurations
6. **Agent Metadata**: Additional metadata (version, author, tags)
7. **Agent Dependencies**: Support for agent dependencies and ordering

## Related Documentation

- [User Guide: Agent Configuration](../user-guide/agent-configuration.md) - User-facing documentation
- [API Documentation](../../crates/radium-core/src/agents/) - Rust API docs
- [CLI Reference](../../README.md#agent-configuration) - Command-line interface

