---
id: "agent-configuration-system"
title: "Agent Configuration System Architecture"
sidebar_label: "Agent Configuration System Architecture"
---

# Agent Configuration System Architecture

This document describes the technical architecture of Radium's agent configuration system for developers.

## Overview

The agent configuration system provides a standardized way to define, discover, and configure AI agents in Radium. It supports TOML-based configuration files, hierarchical agent discovery, prompt template processing with caching, and a thread-safe agent registry with advanced search and filtering capabilities.

## System Components

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Agent Configuration System               │
└──────────────┬──────────────────────────────────────────────┘
               │
     ┌─────────┼─────────┬──────────────┬──────────────┐
     ▼         ▼         ▼              ▼              ▼
┌─────────┐ ┌──────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│  Config │ │Discovery│ │ Registry │ │ Templates │ │ Metadata │
│  Parser │ │ Service │ │  Service │ │  System  │ │  Parser  │
└─────────┘ └──────┘ └──────────┘ └──────────┘ └──────────┘
```

## Architecture Details

### Agent Configuration Format

**Location**: `crates/radium-core/src/agents/config.rs`

The agent configuration system uses TOML files to define agent properties, capabilities, and behaviors.

#### Configuration Structure

```toml
[agent]
id = "arch-agent"
name = "Architecture Agent"
description = "Defines system architecture and technical design decisions"
prompt_path = "prompts/agents/core/arch-agent.md"
engine = "gemini"  # optional
model = "gemini-2.0-flash-exp"  # optional
reasoning_effort = "medium"  # optional: low, medium, high

[agent.loop_behavior]  # optional
steps = 2
max_iterations = 5
skip = ["step-1", "step-3"]

[agent.trigger_behavior]  # optional
trigger_agent_id = "code-agent"

[agent.capabilities]  # optional
model_class = "reasoning"  # fast, balanced, reasoning
cost_tier = "high"  # low, medium, high
max_concurrent_tasks = 3

[agent.persona]  # optional
[agent.persona.models]
primary = "gemini-2.0-flash-thinking"
fallback = "gemini-2.0-flash-exp"
premium = "gemini-1.5-pro"

[agent.persona.performance]
profile = "thinking"  # speed, balanced, thinking, expert
estimated_tokens = 2000

[agent.sandbox]  # optional
type = "docker"
image = "rust:latest"
profile = "restricted"
network_mode = "isolated"
```

#### Data Models

```rust
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt_path: PathBuf,
    pub mirror_path: Option<PathBuf>,
    pub engine: Option<String>,
    pub model: Option<String>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub loop_behavior: Option<AgentLoopBehavior>,
    pub trigger_behavior: Option<AgentTriggerBehavior>,
    pub category: Option<String>,  // derived from path
    pub file_path: Option<PathBuf>,  // set during loading
    pub capabilities: AgentCapabilities,
    pub sandbox: Option<SandboxConfig>,
    pub persona_config: Option<PersonaConfig>,
}

pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

pub enum ModelClass {
    Fast,      // Speed-optimized models
    Balanced,  // Balanced speed and quality
    Reasoning, // Deep reasoning models
}

pub enum CostTier {
    Low,     // $0.00 - $0.10 per 1M tokens
    Medium,  // $0.10 - $1.00 per 1M tokens
    High,    // $1.00 - $10.00 per 1M tokens
}
```

### Agent Discovery System

**Location**: `crates/radium-core/src/agents/discovery.rs`

The discovery system scans multiple directories for agent configuration files and loads them with hierarchical precedence.

#### Discovery Flow

```
1. Scan Search Paths (in order of precedence):
   ├─ Project-local: ./agents/
   ├─ User agents: ~/.radium/agents/
   ├─ Workspace agents: $RADIUM_WORKSPACE/agents/
   └─ Extension agents: ./.radium/extensions/*/agents/
      and ~/.radium/extensions/*/agents/

2. For each path:
   ├─ Recursively scan for *.toml files
   ├─ Load and parse TOML configuration
   ├─ Extract YAML frontmatter from prompt files
   ├─ Parse metadata and generate persona config
   ├─ Derive category from directory structure
   └─ Build agent registry (later entries override earlier)

3. Apply filters:
   └─ Sub-agent filter (if specified)
```

#### Search Path Precedence

Agents are discovered from multiple directories with the following precedence (highest to lowest):

1. **Project-local agents**: `./agents/` - Project-specific agents
2. **User agents**: `~/.radium/agents/` - User's personal agents
3. **Workspace agents**: `$RADIUM_WORKSPACE/agents/` - Workspace-level agents
4. **Extension agents**: 
   - `./.radium/extensions/*/agents/` - Project-level extensions
   - `~/.radium/extensions/*/agents/` - User-level extensions

**Precedence Rules:**
- Agents with the same ID from higher-precedence directories override those from lower-precedence directories
- This allows project-specific agents to override user-level or extension agents

#### Category Derivation

The agent's category is automatically derived from the directory structure:

- `agents/core/arch-agent.toml` → category: `"core"`
- `agents/custom/my-agent.toml` → category: `"custom"`
- `agents/rad-agents/design/design-agent.toml` → category: `"rad-agents/design"`

The category is determined by the parent directory path relative to the agents root.

#### Key Methods

```rust
pub struct AgentDiscovery {
    options: DiscoveryOptions,
}

impl AgentDiscovery {
    pub fn new() -> Self;
    pub fn with_options(options: DiscoveryOptions) -> Self;
    pub fn discover_all(&self) -> Result<HashMap<String, AgentConfig>>;
    fn discover_in_directory(&self, dir: &Path, agents: &mut HashMap<String, AgentConfig>) -> Result<()>;
    fn scan_directory(&self, root: &Path, current: &Path, agents: &mut HashMap<String, AgentConfig>) -> Result<()>;
    fn load_agent_config(&self, path: &Path, root: &Path, agents: &mut HashMap<String, AgentConfig>) -> Result<()>;
}
```

#### Metadata Extraction During Discovery

During discovery, the system:

1. Loads the agent configuration from TOML
2. Reads the prompt file specified in `prompt_path`
3. Extracts YAML frontmatter from the prompt file (if present)
4. Parses metadata using `AgentMetadata::from_markdown()`
5. Generates `PersonaConfig` from metadata if model recommendations are present
6. Attaches metadata and persona config to the agent configuration

### Prompt Template System

**Location**: 
- `crates/radium-core/src/prompts/templates.rs` - Core template loading and rendering
- `crates/radium-core/src/prompts/processing.rs` - Advanced processing (caching, file injection)

The prompt template system loads markdown templates and processes them with placeholder replacement, file injection, and caching.

#### Template Processing Flow

```
1. Load Template:
   ├─ Check cache (if enabled)
   ├─ Load from file (if cache miss)
   └─ Store in cache

2. Process Template:
   ├─ File injection ({{file:path}}, {{file:path:code}}, {{file:path:markdown}})
   ├─ Placeholder replacement ({{VAR}})
   ├─ Default values (if non-strict mode)
   └─ Validation (if strict mode)

3. Render Output:
   └─ Return processed template string
```

#### Placeholder Syntax

**Basic Placeholders:**
```markdown
Hello {{name}}! Your task is {{task}}.
```

**File Injection:**
```markdown
{{file:path/to/file.txt}}              # Plain text injection
{{file:path/to/file.rs:code}}          # Code block with syntax highlighting
{{file:path/to/file.md:markdown}}       # Markdown section
```

#### Template Rendering Modes

**Strict Mode:**
- Errors if any placeholder is missing
- No default values used
- Ensures all required context is provided

**Non-Strict Mode:**
- Uses default values for missing placeholders
- Continues processing even with missing values
- More forgiving for optional context

#### Prompt Caching

The system includes a thread-safe prompt cache to avoid repeated file I/O:

```rust
pub struct PromptCache {
    cache: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    ttl: Option<Duration>,
}

impl PromptCache {
    pub fn new() -> Self;  // No TTL (indefinite caching)
    pub fn with_ttl(ttl: Duration) -> Self;  // TTL-based caching
    pub fn load(&self, path: impl AsRef<Path>) -> Result<PromptTemplate>;
    pub fn clear(&self) -> Result<()>;
}
```

**Cache Behavior:**
- Templates are cached by file path
- TTL-based expiration (if configured)
- Thread-safe concurrent access using `Arc<RwLock<>>`
- Automatic cache invalidation on TTL expiration

#### Key Methods

```rust
pub struct PromptTemplate {
    content: String,
    file_path: Option<PathBuf>,
}

impl PromptTemplate {
    pub fn load(path: impl AsRef<Path>) -> Result<Self>;
    pub fn from_string(content: impl Into<String>) -> Self;
    pub fn render(&self, context: &PromptContext) -> Result<String>;
    pub fn render_strict(&self, context: &PromptContext) -> Result<String>;
    pub fn list_placeholders(&self) -> Vec<String>;
    pub fn content(&self) -> &str;
    pub fn file_path(&self) -> Option<&Path>;
}

pub fn process_with_file_injection(
    template: &PromptTemplate,
    context: &PromptContext,
    options: &FileInjectionOptions,
) -> Result<String>;
```

### Agent Registry

**Location**: `crates/radium-core/src/agents/registry.rs`

The agent registry provides thread-safe runtime management of discovered agents with advanced search, filtering, and sorting capabilities.

#### Registry Architecture

```rust
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentConfig>>>,
}
```

**Thread Safety:**
- Uses `Arc<RwLock<HashMap>>` for concurrent access
- Multiple readers can access simultaneously
- Writers acquire exclusive lock
- Lock poisoning is handled gracefully

#### Search Modes

**Exact Match:**
- Case-insensitive exact string matching
- Fast lookup by agent ID or exact name

**Contains Match:**
- Substring matching (case-insensitive)
- Default search mode
- Searches in name, description, and ID

**Fuzzy Match:**
- Levenshtein distance-based matching
- Configurable similarity threshold (0.0 to 1.0, default 0.7)
- Useful for typo tolerance and approximate matching

#### Filtering System

**Filter Criteria:**
```rust
pub struct FilterCriteria {
    pub category: Option<String>,      // Partial match
    pub engine: Option<String>,        // Exact match
    pub model: Option<String>,          // Partial match
    pub tags: Option<Vec<String>>,      // Any tag matches
    pub search_mode: SearchMode,        // Exact, Contains, Fuzzy
    pub logic_mode: LogicMode,          // AND or OR
    pub fuzzy_threshold: f64,           // 0.0-1.0
}
```

**Logic Modes:**
- **AND**: All criteria must match (default)
- **OR**: Any criterion can match

**Example:**
```rust
let criteria = FilterCriteria {
    category: Some("core".to_string()),
    engine: Some("gemini".to_string()),
    logic_mode: LogicMode::And,
    ..Default::default()
};
let agents = registry.filter(&criteria)?;
```

#### Sorting

**Sort Orders:**
- By name (alphabetical)
- By category (alphabetical)
- By engine (alphabetical)
- Multi-field sorting (chained)

**Example:**
```rust
let sorted = registry.sort(agents, SortOrder::Multiple(vec![
    SortField::Category,
    SortField::Name,
]));
```

#### Key Methods

```rust
impl AgentRegistry {
    pub fn new() -> Self;
    pub fn with_discovery() -> Result<Self>;
    pub fn register(&self, agent: AgentConfig) -> Result<()>;
    pub fn register_or_replace(&self, agent: AgentConfig) -> Result<()>;
    pub fn get(&self, id: &str) -> Result<AgentConfig>;
    pub fn list_all(&self) -> Result<Vec<AgentConfig>>;
    pub fn search(&self, query: &str, mode: SearchMode) -> Result<Vec<AgentConfig>>;
    pub fn filter(&self, criteria: &FilterCriteria) -> Result<Vec<AgentConfig>>;
    pub fn sort(&self, agents: Vec<AgentConfig>, order: SortOrder) -> Vec<AgentConfig>;
    pub fn count(&self) -> Result<usize>;
    pub fn discover_and_register(&self) -> Result<()>;
}
```

### Agent Metadata System

**Location**: `crates/radium-core/src/agents/metadata.rs`

The metadata system parses YAML frontmatter from agent prompt files to extract rich metadata including model recommendations, performance profiles, and capabilities.

#### YAML Frontmatter Format

```yaml
---
name: arch-agent
display_name: Architecture Agent
category: engineering
color: blue
description: Defines system architecture and technical design decisions

recommended_models:
  primary:
    engine: gemini
    model: gemini-2.0-flash-thinking
    priority: thinking
    cost_tier: high
  fallback:
    engine: gemini
    model: gemini-2.0-flash-exp
    priority: balanced
    cost_tier: medium
  premium:
    engine: gemini
    model: gemini-1.5-pro
    priority: expert
    cost_tier: premium

performance_profile:
  thinking_depth: high
  iteration_speed: medium
  context_requirements: extensive
  output_volume: high

capabilities:
  - system_design
  - architecture_decisions
  - technology_selection

quality_gates:
  - code_review
  - architecture_review

works_well_with:
  - code-agent
  - review-agent
---
```

#### Metadata Structure

```rust
pub struct AgentMetadata {
    pub name: String,
    pub display_name: Option<String>,
    pub category: Option<String>,
    pub color: String,
    pub description: String,
    pub recommended_models: Option<RecommendedModels>,
    pub capabilities: Option<Vec<String>>,
    pub performance_profile: Option<PerformanceProfile>,
    pub quality_gates: Option<Vec<String>>,
    pub works_well_with: Option<Vec<String>>,
    pub typical_workflows: Option<Vec<String>>,
    pub tools: Option<Vec<String>>,
    pub constraints: Option<HashMap<String, serde_yaml::Value>>,
}
```

#### Model Recommendations

**Priority Levels:**
- `speed`: Fast models, lower cost
- `balanced`: Balanced speed and quality
- `thinking`: Deep reasoning models
- `expert`: Expert-level reasoning, highest cost

**Cost Tiers:**
- `low`: $0.00 - $0.10 per 1M tokens
- `medium`: $0.10 - $1.00 per 1M tokens
- `high`: $1.00 - $10.00 per 1M tokens
- `premium`: $10.00+ per 1M tokens

#### Persona Configuration Generation

The metadata system automatically generates `PersonaConfig` from metadata:

```rust
impl AgentMetadata {
    pub fn to_persona_config(&self) -> Option<PersonaConfig>;
}
```

This conversion:
- Extracts model recommendations (primary, fallback, premium)
- Maps performance profile to persona performance config
- Sets up model selection priorities
- Configures cost optimization settings

#### Key Methods

```rust
impl AgentMetadata {
    pub fn from_markdown(content: &str) -> Result<(Self, String)>;
    pub fn from_file(path: impl AsRef<Path>) -> Result<(Self, String)>;
    pub fn validate(&self) -> Result<()>;
    pub fn to_persona_config(&self) -> Option<PersonaConfig>;
    pub fn get_display_name(&self) -> &str;
}
```

## Data Flow

### Agent Discovery Flow

```
┌─────────────┐
│   Discovery │
│   Service   │
└──────┬──────┘
      │
      ├─► Scan directories (recursive)
      │
      ├─► Load TOML config files
      │
      ├─► Parse agent configuration
      │
      ├─► Load prompt file
      │
      ├─► Extract YAML frontmatter
      │
      ├─► Parse metadata
      │
      ├─► Generate persona config
      │
      └─► Build registry (HashMap)
```

### Template Processing Flow

```
┌─────────────┐
│   Template  │
│   Request   │
└──────┬──────┘
      │
      ├─► Check cache
      │   ├─► Cache hit → Return cached
      │   └─► Cache miss → Continue
      │
      ├─► Load from file
      │
      ├─► Store in cache
      │
      ├─► Process file injections
      │   ├─► Resolve file paths
      │   ├─► Read file content
      │   └─► Format (plain/code/markdown)
      │
      ├─► Replace placeholders
      │   ├─► Get values from context
      │   ├─► Use defaults (if non-strict)
      │   └─► Error if missing (if strict)
      │
      └─► Return rendered template
```

### Registry Search Flow

```
┌─────────────┐
│   Search    │
│   Query     │
└──────┬──────┘
      │
      ├─► Acquire read lock
      │
      ├─► Select search mode
      │   ├─► Exact: Direct lookup
      │   ├─► Contains: Substring match
      │   └─► Fuzzy: Levenshtein distance
      │
      ├─► Apply filters (if specified)
      │   ├─► Category filter
      │   ├─► Engine filter
      │   ├─► Model filter
      │   ├─► Tags filter
      │   └─► Logic mode (AND/OR)
      │
      ├─► Sort results (if specified)
      │
      └─► Return filtered & sorted agents
```

## Thread Safety

The agent configuration system is designed for concurrent access:

### Registry Thread Safety

- **Read Operations**: Multiple threads can read simultaneously using `RwLock::read()`
- **Write Operations**: Exclusive access using `RwLock::write()`
- **Lock Poisoning**: Handled gracefully with error propagation

### Cache Thread Safety

- **Concurrent Reads**: Multiple threads can access cached templates simultaneously
- **Cache Updates**: Exclusive lock for cache modifications
- **TTL Expiration**: Thread-safe expiration checking

### Discovery Thread Safety

- **Discovery Operations**: Not thread-safe by design (typically called during initialization)
- **Result Storage**: Thread-safe registry for storing discovered agents

## Integration Points

### With Workflow System

- Agents are loaded during workflow initialization
- Agent configurations are used to set up workflow steps
- Loop and trigger behaviors are integrated with workflow execution

### With Engine System

- Agent engine/model preferences are used for model selection
- Persona configurations guide intelligent model selection
- Cost tiers inform budget-aware model selection

### With CLI

- `rad agents list` - Lists discovered agents
- `rad agents search` - Searches agents using registry
- `rad agents info` - Shows agent details
- `rad agents validate` - Validates agent configurations

### With Extension System

- Extension agents are discovered from extension directories
- Extension agents can override built-in agents
- Extension metadata is integrated with discovery

## Error Handling

### Configuration Errors

- **Invalid TOML**: Returns `AgentConfigError::Toml`
- **Missing Fields**: Returns `AgentConfigError::Invalid`
- **I/O Errors**: Returns `AgentConfigError::Io`

### Discovery Errors

- **Directory Not Found**: Skips directory (non-fatal)
- **Invalid Config**: Logs warning and skips (non-fatal)
- **Missing Prompt File**: Logs warning but continues (validation happens later)

### Template Errors

- **File Not Found**: Returns `PromptError::NotFound`
- **Missing Placeholder**: Returns `PromptError::MissingPlaceholder` (strict mode)
- **Invalid Syntax**: Returns `PromptError::InvalidSyntax`

### Registry Errors

- **Agent Not Found**: Returns `RegistryError::NotFound`
- **Lock Poisoned**: Returns `RegistryError::LockPoisoned`
- **Already Registered**: Returns `RegistryError::AlreadyRegistered`

## Performance Considerations

### Discovery Performance

- **Recursive Scanning**: Efficient directory traversal
- **Lazy Loading**: Configs loaded only when needed
- **Caching**: Prompt templates cached to avoid repeated I/O

### Registry Performance

- **HashMap Lookup**: O(1) average case for ID lookups
- **Search Operations**: O(n) for contains/fuzzy search
- **Filtering**: O(n) with early termination where possible

### Cache Performance

- **Cache Hits**: O(1) lookup from memory
- **Cache Misses**: File I/O operation
- **TTL Management**: Periodic cleanup of expired entries

## Testing

### Unit Tests

- Configuration parsing tests in `config.rs`
- Template rendering tests in `templates.rs`
- Registry operation tests in `registry.rs`
- Metadata parsing tests in `metadata.rs`

### Integration Tests

- Full discovery workflow tests
- Registry with discovery tests
- Template processing with file injection tests
- Metadata extraction during discovery tests

See `crates/radium-core/tests/agent_config_integration_test.rs` for comprehensive integration tests.

## Future Enhancements

### Planned Features

- Agent versioning and migration
- Agent dependency management
- Dynamic agent reloading
- Agent performance metrics
- Agent collaboration workflows

### Potential Improvements

- Incremental discovery (watch for file changes)
- Distributed agent discovery
- Agent marketplace integration
- Advanced caching strategies
- Template compilation and optimization

## References

- [Agent Creation Guide](../guides/agent-creation-guide.md) - User guide for creating agents
- [Agent Configuration Implementation](../../crates/radium-core/src/agents/config.rs)
- [Agent Discovery Implementation](../../crates/radium-core/src/agents/discovery.rs)
- [Agent Registry Implementation](../../crates/radium-core/src/agents/registry.rs)
- [Prompt Templates Implementation](../../crates/radium-core/src/prompts/templates.rs)
- [Agent Metadata Implementation](../../crates/radium-core/src/agents/metadata.rs)

