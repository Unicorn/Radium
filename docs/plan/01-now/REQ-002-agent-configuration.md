---
req_id: REQ-002
title: Agent Configuration System
phase: NOW
status: Completed
priority: Critical
estimated_effort: 15-18 hours
dependencies: [REQ-001]
related_docs:
  - docs/project/02-now-next-later.md#step-1-agent-configuration-system
  - docs/project/03-implementation-plan.md#step-1-agent-configuration-system
  - docs/legacy/legacy-system-feature-backlog.md#34-agent-configuration
---

# Agent Configuration System

## Problem Statement

Radium needs a standardized way to define, discover, and configure AI agents. Without a consistent agent configuration system, it's impossible to:
- Define agent capabilities and behaviors
- Load and execute agents programmatically
- Organize agent prompts and templates
- Discover agents from multiple directories
- Configure agent-specific settings (engine, model, reasoning effort)
- Support agent behaviors (loop, trigger) and module configurations

The legacy system used TOML-based agent configuration files with prompt templates. Radium needs an equivalent system that supports agent discovery, prompt loading, and configuration management.

## Solution Overview

Implement a comprehensive agent configuration system that provides:
- TOML-based agent configuration format
- Agent discovery from multiple directories (project-local, user, built-in)
- Prompt template loading and processing with placeholder replacement
- Agent registry for managing discovered agents
- Support for agent behaviors (loop, trigger) and module configuration
- Prompt caching and validation

The agent configuration system enables all agent execution features in Radium, serving as the foundation for workflow execution and plan generation.

## Functional Requirements

### FR-1: Agent Configuration Format

**Description**: Define TOML-based agent configuration format with all required fields.

**Acceptance Criteria**:
- [x] AgentConfig struct with TOML serialization
- [x] Required fields: `id`, `name`, `description`, `prompt_path`
- [x] Optional fields: `engine`, `model`, `reasoning_effort`, `mirror_path`
- [x] Behavior configuration: `loop_behavior`, `trigger_behavior`
- [x] TOML parsing and validation
- [x] Error handling for invalid configurations

**Implementation**: `crates/radium-core/src/agents/config.rs`

### FR-2: Agent Discovery

**Description**: Discover agents from multiple directories with recursive scanning.

**Acceptance Criteria**:
- [x] Scan agent directories for `.toml` files recursively
- [x] Default search paths: `./agents/`, `~/.radium/agents/`
- [x] Custom search paths support
- [x] Load and parse agent configs
- [x] Build agent registry (HashMap by agent ID)
- [x] Resolve prompt file paths (absolute or relative)
- [x] Filter by sub-agent IDs (for templates)
- [x] Category derivation from file path
- [x] Handle duplicate agent IDs (later entries override)

**Implementation**: `crates/radium-core/src/agents/discovery.rs`

### FR-3: Prompt Template System

**Description**: Load and process prompt templates with placeholder replacement.

**Acceptance Criteria**:
- [x] Load prompt template files (.md format)
- [x] Basic placeholder replacement (`{{VAR}}` syntax)
- [x] Prompt validation (check file exists, valid format)
- [x] Prompt caching (avoid reloading same prompts)
- [x] File content injection support
- [x] Template processing with context variables

**Implementation**: `crates/radium-core/src/prompts/templates.rs`, `crates/radium-core/src/prompts/processing.rs`

### FR-4: Agent Registry

**Description**: Manage discovered agents in a registry for lookup and execution.

**Acceptance Criteria**:
- [x] Agent registry with agent lookup by ID
- [x] List all discovered agents
- [x] Search agents by name or description
- [x] Validate agent configurations
- [x] Get agent metadata and configuration

**Implementation**: `crates/radium-core/src/agents/registry.rs`

## Technical Requirements

### TR-1: Agent Configuration Format

**Description**: TOML-based agent configuration structure.

**TOML Format**:
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
max_iterations = 3
step_back = 2

[agent.trigger_behavior]  # optional
trigger_agent_id = "code-agent"
```

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}
```

### TR-2: Agent Discovery API

**Description**: APIs for discovering and loading agents.

**APIs**:
```rust
pub struct AgentDiscovery {
    options: DiscoveryOptions,
}

impl AgentDiscovery {
    pub fn new() -> Self;
    pub fn with_options(options: DiscoveryOptions) -> Self;
    pub fn discover_all(&self) -> Result<HashMap<String, AgentConfig>>;
    pub fn discover_in_directory(&self, dir: &Path) -> Result<HashMap<String, AgentConfig>>;
}

pub struct DiscoveryOptions {
    pub search_paths: Vec<PathBuf>,
    pub sub_agent_filter: Option<Vec<String>>,
}
```

### TR-3: Prompt Template API

**Description**: APIs for loading and processing prompt templates.

**APIs**:
```rust
pub struct PromptTemplate {
    content: String,
    path: PathBuf,
}

impl PromptTemplate {
    pub fn load(path: &Path) -> Result<Self>;
    pub fn process(&self, context: &HashMap<String, String>) -> Result<String>;
    pub fn replace_placeholders(&self, vars: &HashMap<String, String>) -> String;
}
```

### TR-4: Agent Registry API

**Description**: APIs for managing agent registry.

**APIs**:
```rust
pub struct AgentRegistry {
    agents: HashMap<String, AgentConfig>,
}

impl AgentRegistry {
    pub fn new() -> Self;
    pub fn from_discovery(discovery: &AgentDiscovery) -> Result<Self>;
    pub fn get(&self, id: &str) -> Option<&AgentConfig>;
    pub fn list(&self) -> Vec<&AgentConfig>;
    pub fn search(&self, query: &str) -> Vec<&AgentConfig>;
    pub fn validate(&self, id: &str) -> Result<()>;
}
```

## User Experience

### UX-1: Agent Configuration

**Description**: Users create agent configuration files in TOML format.

**Example**:
```toml
# agents/core/arch-agent.toml
[agent]
id = "arch-agent"
name = "Architecture Agent"
description = "Defines system architecture"
prompt_path = "prompts/agents/core/arch-agent.md"
```

### UX-2: Agent Discovery

**Description**: Agents are automatically discovered from configured directories.

**Example**:
```bash
$ rad agents list
Found 5 agents:
  arch-agent (core)
  plan-agent (core)
  code-agent (core)
  review-agent (core)
  doc-agent (core)
```

### UX-3: Agent Validation

**Description**: Users can validate agent configurations.

**Example**:
```bash
$ rad agents validate arch-agent
✓ Agent configuration valid
✓ Prompt file found: prompts/agents/core/arch-agent.md
✓ All required fields present
```

## Data Requirements

### DR-1: Agent Configuration Files

**Description**: TOML files containing agent configuration.

**Location**: `agents/<category>/<agent-id>.toml`

**Schema**: See TR-1 TOML Format

### DR-2: Prompt Template Files

**Description**: Markdown files containing agent prompts.

**Location**: `prompts/agents/<category>/<agent-id>.md`

**Format**: Markdown with optional placeholder syntax (`{{VAR}}`)

## Dependencies

- **REQ-001**: Workspace System - Required for workspace structure and directory management

## Success Criteria

1. [x] Agent TOML configs can be parsed and validated
2. [x] Agents can be discovered from multiple directories
3. [x] Prompt templates can be loaded and processed
4. [x] Placeholder replacement works correctly
5. [x] Agent registry manages discovered agents
6. [x] Agent behaviors (loop, trigger) are configurable
7. [x] All agent operations have comprehensive test coverage

**Completion Metrics**:
- **Status**: ✅ Complete
- **Lines of Code**: ~1,070 lines across:
  - `crates/radium-core/src/agents/config.rs` (337 lines)
  - `crates/radium-core/src/agents/discovery.rs` (377 lines)
  - `crates/radium-core/src/prompts/templates.rs` (356 lines)
- **Implementation**: Core agent configuration system fully implemented
- **Files**: 
  - `crates/radium-core/src/agents/mod.rs`
  - `crates/radium-core/src/agents/config.rs`
  - `crates/radium-core/src/agents/discovery.rs`
  - `crates/radium-core/src/agents/registry.rs`
  - `crates/radium-core/src/prompts/templates.rs`
  - `crates/radium-core/src/prompts/processing.rs`

## Out of Scope

- MCP (Model Context Protocol) integration (deferred to REQ-009)
- Context Files (GEMINI.md) hierarchical loading (deferred to REQ-011)
- Agent execution logic (covered in workflow system)
- Agent library porting (covered in REQ-017)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-1-agent-configuration-system)
- [Implementation Plan](../project/03-implementation-plan.md#step-1-agent-configuration-system)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md#34-agent-configuration)
- [Agent Config Implementation](../../crates/radium-core/src/agents/config.rs)
- [Agent Discovery Implementation](../../crates/radium-core/src/agents/discovery.rs)
- [Prompt Templates Implementation](../../crates/radium-core/src/prompts/templates.rs)

