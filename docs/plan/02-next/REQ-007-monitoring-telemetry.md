---
req_id: REQ-007
title: Monitoring & Telemetry
phase: NEXT
status: Completed
priority: High
estimated_effort: 18-22 hours
dependencies: [REQ-001, REQ-002]
related_docs:
  - docs/project/02-now-next-later.md#step-6-monitoring--telemetry
  - docs/project/03-implementation-plan.md#step-6-monitoring--telemetry
  - docs/legacy/legacy-system-feature-backlog.md#71-agent-monitoring
---

# Monitoring & Telemetry

## Problem Statement

Users need visibility into agent execution, token usage, and costs. Without monitoring and telemetry, users cannot:
- Track agent lifecycle and execution status
- Monitor token usage and API costs
- Debug agent execution issues
- Analyze performance and optimization opportunities
- Track parent-child agent relationships
- Manage agent logs and outputs

The legacy system provided comprehensive monitoring with agent tracking, telemetry parsing, and log management. Radium needs an equivalent system that tracks agent execution, token usage, and costs.

## Solution Overview

Implement a comprehensive monitoring and telemetry system that provides:
- SQLite-based agent monitoring database
- Agent lifecycle tracking (start, complete, fail)
- Telemetry tracking (tokens, costs, cache statistics)
- Log file management with ANSI color stripping
- CLI commands for monitoring status and telemetry
- Integration with workflow execution for automatic tracking

The monitoring system enables users to track agent execution, analyze costs, and debug issues effectively.

## Functional Requirements

### FR-1: Agent Monitoring Database

**Description**: SQLite database for tracking agent lifecycle and status.

**Acceptance Criteria**:
- [x] Database schema for agents table (id, parent_id, plan_id, agent_type, status, process_id, timestamps)
- [x] Agent lifecycle tracking (start, complete, fail)
- [x] Parent-child relationship tracking
- [x] Process ID tracking
- [x] Agent status queries
- [x] Graceful cleanup on termination
- [x] Plan ID tracking for workflow context

**Implementation**: 
- `crates/radium-core/src/monitoring/schema.rs`
- `crates/radium-core/src/monitoring/service.rs`

### FR-2: Telemetry Tracking

**Description**: Track token usage, costs, and cache statistics per agent.

**Acceptance Criteria**:
- [x] Telemetry table schema (agent_id, tokens, costs, cache stats, model, provider)
- [x] Token counting (input, output, cached)
- [x] Cost calculation based on model pricing
- [x] Cache statistics (creation, read tokens)
- [x] Engine-specific telemetry parsers
- [x] Telemetry storage in database
- [x] Telemetry retrieval by agent ID
- [x] Total token and cost aggregation

**Implementation**: `crates/radium-core/src/monitoring/telemetry.rs`

### FR-3: Log File Management

**Description**: Manage agent-specific log files with color stripping.

**Acceptance Criteria**:
- [x] Agent-specific log files
- [x] Log file path tracking in database
- [x] Color marker transformation for log files
- [x] Dual-stream logging (UI + file)
- [x] Log file organization by agent ID

**Implementation**: `crates/radium-core/src/monitoring/logs.rs`

### FR-4: CLI Monitoring Commands

**Description**: CLI commands for viewing monitoring data and telemetry.

**Acceptance Criteria**:
- [x] `rad monitor status` - Show agent execution status
- [x] `rad monitor list` - List all agents with status
- [x] `rad monitor telemetry <agent-id>` - Show telemetry for agent
- [x] Human-readable and JSON output formats
- [x] Filtering and sorting options

**Implementation**: `apps/cli/src/commands/monitor.rs`

### FR-5: Workflow Integration

**Description**: Automatic agent registration and telemetry recording during workflow execution.

**Acceptance Criteria**:
- [x] Agent registration during workflow execution
- [x] Status updates during agent lifecycle
- [x] Telemetry recording when available
- [x] Plan ID tracking for workflow context
- [x] Automatic checkpoint creation before workflow steps

**Implementation**: 
- `crates/radium-core/src/workflow/service.rs`
- `crates/radium-core/src/workflow/executor.rs`

## Technical Requirements

### TR-1: Database Schema

**Description**: SQLite schema for agent monitoring and telemetry.

**Schema**:
```sql
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    parent_id TEXT,
    plan_id TEXT,
    agent_type TEXT NOT NULL,
    status TEXT NOT NULL,
    process_id INTEGER,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    exit_code INTEGER,
    error_message TEXT,
    log_file TEXT,
    FOREIGN KEY (parent_id) REFERENCES agents(id)
);

CREATE TABLE telemetry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cached_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    estimated_cost REAL NOT NULL DEFAULT 0.0,
    model TEXT,
    provider TEXT,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);
```

**Location**: `.radium/_internals/monitoring.db`

### TR-2: Monitoring Service API

**Description**: APIs for agent monitoring and telemetry.

**APIs**:
```rust
pub struct MonitoringService {
    conn: Connection,
}

impl MonitoringService {
    pub fn register_agent(&self, agent: &AgentRecord) -> Result<()>;
    pub fn update_agent_status(&self, agent_id: &str, status: AgentStatus) -> Result<()>;
    pub fn get_agent_status(&self, agent_id: &str) -> Result<Option<AgentRecord>>;
    pub fn list_agents(&self, filter: Option<AgentFilter>) -> Result<Vec<AgentRecord>>;
}

pub trait TelemetryTracking {
    fn record_telemetry(&self, record: &TelemetryRecord) -> Result<()>;
    fn get_agent_telemetry(&self, agent_id: &str) -> Result<Vec<TelemetryRecord>>;
    fn get_total_tokens(&self, agent_id: &str) -> Result<(u64, u64, u64)>;
    fn get_total_cost(&self, agent_id: &str) -> Result<f64>;
}
```

**Data Models**:
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
    pub log_file: Option<PathBuf>,
}

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
}
```

### TR-3: Cost Calculation

**Description**: Model-specific cost calculation based on token usage.

**Pricing Models**:
- GPT-4: $30/1M input, $60/1M output
- Claude Opus: $15/1M input, $75/1M output
- Claude Haiku: $0.25/1M input, $1.25/1M output
- Gemini: Variable pricing

**Implementation**: `crates/radium-core/src/monitoring/telemetry.rs::calculate_cost()`

## User Experience

### UX-1: Agent Status Monitoring

**Description**: Users can check agent execution status.

**Example**:
```bash
$ rad monitor status
Active Agents:
  code-agent-001 (plan: REQ-001) - Running (PID: 12345)
  review-agent-002 (plan: REQ-001) - Completed

$ rad monitor list
All Agents:
  code-agent-001 - Completed (exit: 0)
  review-agent-002 - Completed (exit: 0)
  test-agent-003 - Failed (exit: 1)
```

### UX-2: Telemetry Viewing

**Description**: Users can view token usage and costs.

**Example**:
```bash
$ rad monitor telemetry code-agent-001
Telemetry for code-agent-001:
  Input tokens: 15,234
  Output tokens: 8,456
  Total tokens: 23,690
  Estimated cost: $0.45
  Model: gpt-4
```

## Data Requirements

### DR-1: Monitoring Database

**Description**: SQLite database for agent and telemetry records.

**Location**: `.radium/_internals/monitoring.db`

**Schema**: See TR-1 Database Schema

### DR-2: Log Files

**Description**: Agent-specific log files with color stripping.

**Location**: `.radium/_internals/logs/<agent-id>.log`

**Format**: Plain text with ANSI color codes stripped

## Dependencies

- **REQ-001**: Workspace System - Required for workspace structure and database location
- **REQ-002**: Agent Configuration - Required for agent identification

## Success Criteria

1. [x] Agent lifecycle can be tracked in database
2. [x] Telemetry can be recorded and retrieved
3. [x] Token usage and costs can be calculated
4. [x] Log files are managed correctly
5. [x] CLI commands provide monitoring visibility
6. [x] Workflow execution automatically tracks agents
7. [x] All monitoring operations have comprehensive test coverage (44+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: 44+ passing tests (29 monitoring + 15 checkpoint)
- **Implementation**: Full monitoring and telemetry system
- **Files**: 
  - `crates/radium-core/src/monitoring/` (schema, service, telemetry, logs)
  - `apps/cli/src/commands/monitor.rs`

## Out of Scope

- Real-time telemetry streaming (future enhancement)
- Advanced analytics and reporting (covered in REQ-020)
- Telemetry aggregation across plans (future enhancement)
- Cost budgeting and alerts (future enhancement)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-6-monitoring--telemetry)
- [Implementation Plan](../project/03-implementation-plan.md#step-6-monitoring--telemetry)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md#71-agent-monitoring)
- [Monitoring Module Implementation](../../crates/radium-core/src/monitoring/)
- [Monitor Command Implementation](../../apps/cli/src/commands/monitor.rs)

