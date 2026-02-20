---
id: "api-reference"
title: "Monitoring System API Reference"
sidebar_label: "Monitoring System API Reference"
---

# Monitoring System API Reference

## Overview

Complete API reference for the Radium monitoring system. All types and methods are documented with examples.

## Core Types

### AgentStatus

Agent lifecycle status enumeration.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Starting,    // Agent is starting up
    Running,     // Agent is running
    Completed,   // Agent completed successfully
    Failed,      // Agent failed with an error
    Terminated,  // Agent was terminated
}
```

**Methods:**

- `as_str() -> &str`: Convert status to string representation
- `from_str(s: &str) -> Result<Self>`: Parse status from string

**Example:**
```rust
let status = AgentStatus::Running;
assert_eq!(status.as_str(), "running");

let parsed = AgentStatus::from_str("completed")?;
assert_eq!(parsed, AgentStatus::Completed);
```

### AgentRecord

Agent record structure for lifecycle tracking.

```rust
#[derive(Debug, Clone)]
pub struct AgentRecord {
    pub id: String,
    pub parent_id: Option<String>,
    pub plan_id: Option<String>,
    pub agent_type: String,
    pub status: AgentStatus,
    pub process_id: Option<u32>,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub exit_code: Option<i32>,
    pub error_message: Option<String>,
    pub log_file: Option<String>,
}
```

**Methods:**

- `new(id: String, agent_type: String) -> Self`: Create new agent record
- `with_parent(mut self, parent_id: String) -> Self`: Set parent agent ID
- `with_plan(mut self, plan_id: String) -> Self`: Set plan ID
- `with_process_id(mut self, process_id: u32) -> Self`: Set process ID
- `with_log_file(mut self, log_file: String) -> Self`: Set log file path

**Example:**
```rust
let agent = AgentRecord::new("agent-123".to_string(), "developer".to_string())
    .with_parent("parent-456".to_string())
    .with_plan("REQ-49".to_string())
    .with_process_id(12345)
    .with_log_file("/path/to/log".to_string());
```

### TelemetryRecord

Telemetry record for token usage and cost tracking.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRecord {
    pub agent_id: String,
    pub timestamp: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost: f64,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub tool_name: Option<String>,
    pub tool_args: Option<String>,
    pub tool_approved: Option<bool>,
    pub tool_approval_type: Option<String>,
    pub engine_id: Option<String>,
}
```

**Methods:**

- `new(agent_id: String) -> Self`: Create new telemetry record
- `with_tokens(mut self, input: u64, output: u64) -> Self`: Set token counts
- `with_cache_stats(mut self, cached: u64, creation: u64, read: u64) -> Self`: Set cache statistics
- `with_model(mut self, model: String, provider: String) -> Self`: Set model information
- `with_tool_approval(mut self, tool_name: String, tool_args: Option<Vec<String>>, approved: bool, approval_type: String) -> Self`: Set tool approval information
- `with_engine_id(mut self, engine_id: String) -> Self`: Set engine ID
- `calculate_cost(&mut self) -> &mut Self`: Calculate estimated cost based on model pricing

**Example:**
```rust
let mut telemetry = TelemetryRecord::new("agent-123".to_string())
    .with_tokens(1500, 800)
    .with_cache_stats(200, 50, 150)
    .with_model("gpt-4".to_string(), "openai".to_string())
    .with_engine_id("openai".to_string());

telemetry.calculate_cost();
```

## MonitoringService

Core service for agent monitoring and lifecycle management.

### Construction

```rust
impl MonitoringService {
    /// Create new monitoring service with in-memory database
    pub fn new() -> Result<Self>;
    
    /// Create new monitoring service with hook registry
    pub fn with_hooks(hook_registry: Arc<HookRegistry>) -> Result<Self>;
    
    /// Open monitoring service with database file
    pub fn open(path: impl AsRef<Path>) -> Result<Self>;
    
    /// Open monitoring service with database file and hook registry
    pub fn open_with_hooks(
        path: impl AsRef<Path>,
        hook_registry: Arc<HookRegistry>
    ) -> Result<Self>;
}
```

**Example:**
```rust
// In-memory database (for testing)
let monitoring = MonitoringService::new()?;

// Persistent database
let workspace = Workspace::discover()?;
let db_path = workspace.radium_dir().join("monitoring.db");
let monitoring = MonitoringService::open(db_path)?;

// With hooks
let hook_registry = Arc::new(HookRegistry::new());
let monitoring = MonitoringService::open_with_hooks(db_path, hook_registry)?;
```

### Agent Management

```rust
impl MonitoringService {
    /// Register a new agent (synchronous)
    pub fn register_agent(&self, record: &AgentRecord) -> Result<()>;
    
    /// Register a new agent with telemetry hooks (async)
    pub async fn register_agent_with_hooks(&self, record: &AgentRecord) -> Result<()>;
    
    /// Update agent status
    pub fn update_status(&self, agent_id: &str, status: AgentStatus) -> Result<()>;
    
    /// Mark agent as completed
    pub fn complete_agent(&self, agent_id: &str, exit_code: i32) -> Result<()>;
    
    /// Mark agent as completed with hooks (async)
    pub async fn complete_agent_with_hooks(&self, agent_id: &str, exit_code: i32) -> Result<()>;
    
    /// Mark agent as failed
    pub fn fail_agent(&self, agent_id: &str, error_message: &str) -> Result<()>;
    
    /// Mark agent as failed with hooks (async)
    pub async fn fail_agent_with_hooks(&self, agent_id: &str, error_message: &str) -> Result<()>;
}
```

**Example:**
```rust
// Register agent
let agent = AgentRecord::new("agent-123".to_string(), "developer".to_string());
monitoring.register_agent(&agent)?;

// Update status
monitoring.update_status("agent-123", AgentStatus::Running)?;

// Complete agent
monitoring.complete_agent("agent-123", 0)?;

// With hooks (async)
monitoring.register_agent_with_hooks(&agent).await?;
monitoring.complete_agent_with_hooks("agent-123", 0).await?;
```

### Agent Queries

```rust
impl MonitoringService {
    /// Get agent record by ID
    pub fn get_agent(&self, agent_id: &str) -> Result<AgentRecord>;
    
    /// Get all child agents of a parent
    pub fn get_children(&self, parent_id: &str) -> Result<Vec<AgentRecord>>;
    
    /// Get all agents for a plan
    pub fn get_plan_agents(&self, plan_id: &str) -> Result<Vec<AgentRecord>>;
    
    /// List all agents (ordered by start_time descending)
    pub fn list_agents(&self) -> Result<Vec<AgentRecord>>;
}
```

**Example:**
```rust
// Get single agent
let agent = monitoring.get_agent("agent-123")?;

// Get child agents
let children = monitoring.get_children("parent-456")?;

// Get plan agents
let plan_agents = monitoring.get_plan_agents("REQ-49")?;

// List all agents
let all_agents = monitoring.list_agents()?;
```

### Hook Registry Management

```rust
impl MonitoringService {
    /// Set hook registry
    pub fn set_hook_registry(&mut self, hook_registry: Arc<HookRegistry>);
    
    /// Get hook registry (if available)
    pub fn get_hook_registry(&self) -> Option<Arc<HookRegistry>>;
}
```

## TelemetryTracking Trait

Extension trait for telemetry operations.

```rust
#[async_trait(?Send)]
pub trait TelemetryTracking {
    /// Record telemetry for an agent (async, executes hooks)
    async fn record_telemetry(&self, record: &TelemetryRecord) -> Result<()>;
    
    /// Get telemetry records for an agent
    fn get_agent_telemetry(&self, agent_id: &str) -> Result<Vec<TelemetryRecord>>;
    
    /// Get total token usage for an agent
    fn get_total_tokens(&self, agent_id: &str) -> Result<(u64, u64, u64)>;
    
    /// Get total estimated cost for an agent
    fn get_total_cost(&self, agent_id: &str) -> Result<f64>;
}
```

**Implementation:** `MonitoringService` implements `TelemetryTracking`.

**Example:**
```rust
// Record telemetry
let telemetry = TelemetryRecord::new("agent-123".to_string())
    .with_tokens(1500, 800);
monitoring.record_telemetry(&telemetry).await?;

// Get telemetry
let records = monitoring.get_agent_telemetry("agent-123")?;

// Get totals
let (input, output, cached) = monitoring.get_total_tokens("agent-123")?;
let total_cost = monitoring.get_total_cost("agent-123")?;
```

## TelemetryParser

Utility for parsing token usage from model API responses.

```rust
pub struct TelemetryParser;

impl TelemetryParser {
    /// Parse OpenAI-style usage output
    pub fn parse_openai(json: &str) -> Result<(u64, u64)>;
    
    /// Parse Anthropic-style usage output
    pub fn parse_anthropic(json: &str) -> Result<(u64, u64)>;
    
    /// Parse Google Gemini-style usage output
    pub fn parse_gemini(json: &str) -> Result<(u64, u64)>;
}
```

**Example:**
```rust
let openai_response = r#"
{
  "usage": {
    "prompt_tokens": 1500,
    "completion_tokens": 800
  }
}
"#;

let (input, output) = TelemetryParser::parse_openai(openai_response)?;
```

## LogManager

Manages agent-specific log files with ANSI color code stripping.

```rust
pub struct LogManager {
    logs_dir: PathBuf,
}

impl LogManager {
    /// Create new log manager
    pub fn new(logs_dir: impl AsRef<Path>) -> Result<Self>;
    
    /// Get log file path for an agent
    pub fn log_path(&self, agent_id: &str) -> PathBuf;
    
    /// Create new log file for an agent
    pub fn create_log(&self, agent_id: &str) -> Result<File>;
    
    /// Append a line to an agent's log
    pub fn append_log(&self, agent_id: &str, line: &str) -> Result<()>;
    
    /// Read an agent's log file
    pub fn read_log(&self, agent_id: &str) -> Result<String>;
    
    /// Read the last N lines of an agent's log
    pub fn tail_log(&self, agent_id: &str, lines: usize) -> Result<String>;
    
    /// List all agent log files
    pub fn list_logs(&self) -> Result<Vec<String>>;
    
    /// Delete an agent's log file
    pub fn delete_log(&self, agent_id: &str) -> Result<()>;
}
```

**Example:**
```rust
let log_manager = LogManager::new("/path/to/logs")?;

// Create log
let mut file = log_manager.create_log("agent-123")?;
writeln!(file, "Agent started")?;

// Append to log
log_manager.append_log("agent-123", "Processing task")?;

// Read log
let content = log_manager.read_log("agent-123")?;

// Tail log
let tail = log_manager.tail_log("agent-123", 10)?;
```

## Error Types

### MonitoringError

Error type for monitoring operations.

```rust
#[derive(Debug, Error)]
pub enum MonitoringError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("invalid status: {0}")]
    InvalidStatus(String),
    
    #[error("telemetry parse error: {0}")]
    TelemetryParse(String),
    
    #[error("other error: {0}")]
    Other(String),
}
```

**Example:**
```rust
match monitoring.get_agent("agent-123") {
    Ok(agent) => println!("Found: {}", agent.id),
    Err(MonitoringError::AgentNotFound(id)) => {
        println!("Agent {} not found", id);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Result Type

```rust
pub type Result<T> = std::result::Result<T, MonitoringError>;
```

## Module Exports

```rust
pub use error::{MonitoringError, Result};
pub use logs::LogManager;
pub use schema::initialize_schema;
pub use service::{AgentRecord, AgentStatus, MonitoringService};
pub use telemetry::{TelemetryParser, TelemetryRecord, TelemetryTracking};
```

## Usage Example

Complete example showing full monitoring workflow:

```rust
use radium_core::monitoring::{
    MonitoringService, AgentRecord, AgentStatus,
    TelemetryRecord, TelemetryTracking,
};
use radium_core::workspace::Workspace;

async fn example() -> Result<()> {
    // Open monitoring service
    let workspace = Workspace::discover()?;
    let db_path = workspace.radium_dir().join("monitoring.db");
    let monitoring = MonitoringService::open(db_path)?;
    
    // Register agent
    let agent = AgentRecord::new("agent-123".to_string(), "developer".to_string())
        .with_plan("REQ-49".to_string());
    monitoring.register_agent(&agent)?;
    
    // Update status
    monitoring.update_status("agent-123", AgentStatus::Running)?;
    
    // Record telemetry
    let mut telemetry = TelemetryRecord::new("agent-123".to_string())
        .with_tokens(1500, 800)
        .with_model("gpt-4".to_string(), "openai".to_string());
    telemetry.calculate_cost();
    monitoring.record_telemetry(&telemetry).await?;
    
    // Complete agent
    monitoring.complete_agent("agent-123", 0)?;
    
    // Query data
    let agent = monitoring.get_agent("agent-123")?;
    let telemetry = monitoring.get_agent_telemetry("agent-123")?;
    let total_cost = monitoring.get_total_cost("agent-123")?;
    
    println!("Agent: {} - Cost: ${:.4}", agent.id, total_cost);
    
    Ok(())
}
```

