---
id: "architecture"
title: "Monitoring System Architecture"
sidebar_label: "Monitoring System Architecture"
---

# Monitoring System Architecture

## Overview

The Radium monitoring system provides comprehensive tracking of agent execution, telemetry collection, and usage analytics. It uses a SQLite-based database to store agent lifecycle events, token usage, costs, and performance metrics.

## System Design

### Database Schema

The monitoring system uses three core tables:

#### 1. Agents Table

Tracks agent lifecycle and execution status:

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
```

**Key Fields:**
- `id`: Unique agent identifier
- `parent_id`: Links child agents to parent agents (supports agent hierarchies)
- `plan_id`: Associates agents with workflow plans (e.g., REQ-49)
- `agent_type`: Type of agent (e.g., "developer", "architect", "reviewer")
- `status`: Current lifecycle status (starting, running, completed, failed, terminated)
- `process_id`: Operating system process ID for active agents
- `start_time`/`end_time`: Unix timestamps for execution duration
- `exit_code`: Process exit code (0 = success, non-zero = failure)
- `error_message`: Error details if agent failed
- `log_file`: Path to agent-specific log file

**Indexes:**
- `idx_agents_parent`: Fast parent-child queries
- `idx_agents_plan`: Fast plan-based queries
- `idx_agents_status`: Fast status filtering

#### 2. Telemetry Table

Tracks token usage, costs, and tool execution:

```sql
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
    tool_name TEXT,
    tool_args TEXT,
    tool_approved BOOLEAN,
    tool_approval_type TEXT,
    engine_id TEXT,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);
```

**Key Fields:**
- `agent_id`: Links telemetry to agent
- `input_tokens`/`output_tokens`: Token counts from model API
- `cached_tokens`: Tokens reused from cache
- `cache_creation_tokens`/`cache_read_tokens`: Cache operation metrics
- `total_tokens`: Sum of all token types
- `estimated_cost`: Calculated cost in USD based on model pricing
- `model`: Model name (e.g., "gpt-4", "claude-3-opus")
- `provider`: Provider name (e.g., "openai", "anthropic")
- `tool_name`: Name of tool executed (if applicable)
- `tool_args`: Tool arguments as JSON string
- `tool_approved`: Whether tool execution was approved
- `tool_approval_type`: Approval mechanism ("user", "auto", "policy")
- `engine_id`: Execution engine identifier

**Indexes:**
- `idx_telemetry_agent`: Fast agent-based queries
- `idx_telemetry_timestamp`: Fast time-based queries

#### 3. Agent Usage Table

Aggregates performance metrics per agent:

```sql
CREATE TABLE agent_usage (
    agent_id TEXT PRIMARY KEY,
    execution_count INTEGER NOT NULL DEFAULT 0,
    total_duration INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    last_used_at INTEGER,
    category TEXT
);
```

**Key Fields:**
- `agent_id`: Unique agent identifier
- `execution_count`: Total number of executions
- `total_duration`: Cumulative execution time (milliseconds)
- `total_tokens`: Cumulative token usage
- `success_count`/`failure_count`: Success/failure tracking
- `last_used_at`: Unix timestamp of last execution
- `category`: Agent category for grouping

**Indexes:**
- `idx_agent_usage_category`: Fast category-based queries
- `idx_agent_usage_last_used`: Fast recency queries

## Data Flow

### Agent Lifecycle Tracking

1. **Agent Registration**: When an agent starts, `MonitoringService::register_agent()` creates an entry in the `agents` table with status "starting".

2. **Status Updates**: As the agent progresses, status updates occur:
   - `Starting` → `Running` (via `update_status()`)
   - `Running` → `Completed` (via `complete_agent()`)
   - `Running` → `Failed` (via `fail_agent()`)

3. **Parent-Child Relationships**: Child agents link to parents via `parent_id`, enabling hierarchical tracking of agent orchestrations.

4. **Plan Context**: Agents are associated with plans via `plan_id`, allowing workflow-level monitoring.

### Telemetry Collection

1. **Telemetry Recording**: When model calls complete, `TelemetryTracking::record_telemetry()` stores token usage and cost data.

2. **Hook Integration**: Before database storage, telemetry hooks can augment or modify telemetry data:
   - `TelemetryCollection` hooks execute asynchronously
   - Hooks can modify cost calculations or add custom fields
   - Hook failures are non-blocking (logged as warnings)

3. **Cost Calculation**: `TelemetryRecord::calculate_cost()` computes estimated costs based on:
   - Engine-specific pricing (preferred)
   - Model-based pricing (fallback)
   - Token counts (input, output, cached)

4. **Tool Tracking**: When tools are executed, telemetry records include:
   - Tool name and arguments
   - Approval status and type
   - Engine identifier

### Usage Analytics

1. **Automatic Aggregation**: When agents complete, usage statistics are updated in `agent_usage` table:
   - Execution counts incremented
   - Durations accumulated
   - Success/failure counts updated
   - Last used timestamp set

2. **Category Grouping**: Agents can be grouped by category for analytics.

## Component Architecture

### MonitoringService

Core service for agent lifecycle management:

```rust
pub struct MonitoringService {
    conn: Connection,
    hook_registry: Option<Arc<HookRegistry>>,
}
```

**Key Methods:**
- `register_agent()`: Register new agent (synchronous)
- `register_agent_with_hooks()`: Register with telemetry hooks (async)
- `update_status()`: Update agent status
- `complete_agent()`: Mark agent as completed
- `complete_agent_with_hooks()`: Complete with telemetry hooks (async)
- `fail_agent()`: Mark agent as failed
- `get_agent()`: Retrieve agent record
- `get_children()`: Get child agents
- `get_plan_agents()`: Get agents for a plan
- `list_agents()`: List all agents

### TelemetryTracking Trait

Extension trait for telemetry operations:

```rust
#[async_trait(?Send)]
pub trait TelemetryTracking {
    async fn record_telemetry(&self, record: &TelemetryRecord) -> Result<()>;
    fn get_agent_telemetry(&self, agent_id: &str) -> Result<Vec<TelemetryRecord>>;
    fn get_total_tokens(&self, agent_id: &str) -> Result<(u64, u64, u64)>;
    fn get_total_cost(&self, agent_id: &str) -> Result<f64>;
}
```

**Implementation Notes:**
- `record_telemetry()` executes hooks before database write
- Hooks can modify telemetry data (e.g., cost adjustments)
- Database writes are synchronous (no await needed)

### TelemetryParser

Utility for parsing token usage from model API responses:

- `parse_openai()`: Parses OpenAI-style usage JSON
- `parse_anthropic()`: Parses Anthropic-style usage JSON
- `parse_gemini()`: Parses Google Gemini-style usage JSON

### LogManager

Manages agent-specific log files:

- Creates log files per agent
- Strips ANSI color codes for file storage
- Supports reading, tailing, and listing logs

## Hook Integration

### Telemetry Hooks

The monitoring system integrates with the hook registry for telemetry collection:

1. **Agent Start Hooks**: Executed when `register_agent_with_hooks()` is called
   - Event type: "agent_start"
   - Data includes: agent_id, agent_type, parent_id, plan_id, start_time

2. **Agent Complete Hooks**: Executed when `complete_agent_with_hooks()` is called
   - Event type: "agent_complete"
   - Data includes: agent_id, exit_code, end_time, success

3. **Agent Fail Hooks**: Executed when `fail_agent_with_hooks()` is called
   - Event type: "agent_fail"
   - Data includes: agent_id, error_message, end_time

4. **Telemetry Collection Hooks**: Executed before telemetry storage
   - Hook type: `TelemetryCollection`
   - Can modify telemetry data (e.g., cost adjustments)
   - Non-blocking (failures logged as warnings)

### Hook Execution Flow

```
Agent Lifecycle Event
    ↓
Hook Registry (if enabled)
    ↓
Execute TelemetryCollection Hooks
    ↓
Modify/Augment Telemetry Data
    ↓
Database Write (synchronous)
```

## Database Location

The monitoring database is stored at:

```
.radium/_internals/monitoring.db
```

This location is within the Radium workspace directory, ensuring:
- Workspace-scoped monitoring
- Automatic cleanup with workspace removal
- Isolation between different workspaces

## Performance Considerations

### Indexes

All tables have appropriate indexes for common query patterns:
- Parent-child relationships
- Plan-based queries
- Status filtering
- Time-based queries
- Category grouping

### Query Optimization

- Use `get_telemetry_summary()` for aggregated queries (GROUP BY)
- Filter by status, plan, or category when possible
- Leverage indexes for efficient lookups

### Scalability

- SQLite handles moderate workloads efficiently
- For high-volume scenarios, consider:
  - Periodic aggregation to summary tables
  - Archival of old telemetry records
  - Database vacuum operations

## Error Handling

The monitoring system uses graceful error handling:

- **Database Errors**: Propagated as `MonitoringError::Database`
- **Hook Failures**: Logged as warnings, execution continues
- **Missing Agents**: Return `None` or `AgentNotFound` error
- **Invalid Data**: Validation errors with descriptive messages

## Testing

The monitoring system has comprehensive test coverage:

- Unit tests for each component
- Integration tests for database operations
- Hook integration tests
- Telemetry parsing tests

See test files in `crates/radium-core/src/monitoring/` for examples.

