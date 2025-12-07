# Monitoring System Usage Guide

## Overview

This guide provides practical examples for using the Radium monitoring system to track agent execution, analyze costs, and monitor performance.

## Basic Agent Tracking

### Registering an Agent

```rust
use radium_core::monitoring::{MonitoringService, AgentRecord, AgentStatus};

// Open monitoring service (uses workspace database)
let workspace = Workspace::discover()?;
let monitoring_path = workspace.radium_dir().join("monitoring.db");
let monitoring = MonitoringService::open(monitoring_path)?;

// Create and register an agent
let agent = AgentRecord::new("agent-123".to_string(), "developer".to_string())
    .with_plan("REQ-49".to_string())
    .with_process_id(12345)
    .with_log_file("/path/to/logs/agent-123.log".to_string());

monitoring.register_agent(&agent)?;
```

### Updating Agent Status

```rust
// Mark agent as running
monitoring.update_status("agent-123", AgentStatus::Running)?;

// Complete agent successfully
monitoring.complete_agent("agent-123", 0)?;

// Mark agent as failed
monitoring.fail_agent("agent-123", "Connection timeout")?;
```

### Using Hooks for Async Tracking

```rust
use std::sync::Arc;
use radium_core::hooks::registry::HookRegistry;

// Create monitoring service with hooks
let hook_registry = Arc::new(HookRegistry::new());
let monitoring = MonitoringService::open_with_hooks(monitoring_path, hook_registry)?;

// Register agent with hooks (async)
monitoring.register_agent_with_hooks(&agent).await?;

// Complete agent with hooks (async)
monitoring.complete_agent_with_hooks("agent-123", 0).await?;
```

## Telemetry Collection

### Recording Telemetry

```rust
use radium_core::monitoring::{TelemetryRecord, TelemetryTracking};

// Create telemetry record
let mut telemetry = TelemetryRecord::new("agent-123".to_string())
    .with_tokens(1500, 800)
    .with_cache_stats(200, 50, 150)
    .with_model("gpt-4".to_string(), "openai".to_string())
    .with_engine_id("openai".to_string());

// Calculate cost
telemetry.calculate_cost();

// Record telemetry (async, executes hooks)
monitoring.record_telemetry(&telemetry).await?;
```

### Parsing Telemetry from API Responses

```rust
use radium_core::monitoring::TelemetryParser;

// Parse OpenAI response
let openai_response = r#"
{
  "usage": {
    "prompt_tokens": 1500,
    "completion_tokens": 800,
    "total_tokens": 2300
  }
}
"#;

let (input_tokens, output_tokens) = TelemetryParser::parse_openai(openai_response)?;

// Parse Anthropic response
let anthropic_response = r#"
{
  "usage": {
    "input_tokens": 1500,
    "output_tokens": 800
  }
}
"#;

let (input_tokens, output_tokens) = TelemetryParser::parse_anthropic(anthropic_response)?;
```

### Tracking Tool Execution

```rust
// Record telemetry with tool information
let mut telemetry = TelemetryRecord::new("agent-123".to_string())
    .with_tokens(100, 50)
    .with_tool_approval(
        "read_file".to_string(),
        Some(vec!["path/to/file.rs".to_string()]),
        true,
        "auto".to_string(),
    )
    .with_engine_id("claude".to_string());

telemetry.calculate_cost();
monitoring.record_telemetry(&telemetry).await?;
```

## Querying Agent Data

### Get Agent Status

```rust
// Get single agent
let agent = monitoring.get_agent("agent-123")?;
println!("Agent: {} - Status: {:?}", agent.id, agent.status);

// Get all agents for a plan
let plan_agents = monitoring.get_plan_agents("REQ-49")?;
for agent in plan_agents {
    println!("Agent: {} - Type: {}", agent.id, agent.agent_type);
}

// Get child agents
let children = monitoring.get_children("parent-agent-123")?;
println!("Found {} child agents", children.len());

// List all agents
let all_agents = monitoring.list_agents()?;
for agent in all_agents {
    println!("{}: {:?}", agent.id, agent.status);
}
```

### Get Telemetry Data

```rust
// Get all telemetry for an agent
let telemetry = monitoring.get_agent_telemetry("agent-123")?;
for record in telemetry {
    println!(
        "Tokens: {} in, {} out, Cost: ${:.4}",
        record.input_tokens,
        record.output_tokens,
        record.estimated_cost
    );
}

// Get total tokens
let (input, output, cached) = monitoring.get_total_tokens("agent-123")?;
println!("Total: {} in, {} out, {} cached", input, output, cached);

// Get total cost
let total_cost = monitoring.get_total_cost("agent-123")?;
println!("Total cost: ${:.4}", total_cost);
```

## CLI Usage

### View Agent Status

```bash
# Show status for specific agent
rad monitor status agent-123

# Show status for all agents
rad monitor status

# JSON output
rad monitor status agent-123 --json
```

**Example Output:**
```
Agent: agent-123
Type: developer
Status: Running
Plan: REQ-49
Process ID: 12345
Duration: 45s
```

### List Agents

```bash
# List all agents
rad monitor list

# Filter by status
rad monitor list --status running

# JSON output
rad monitor list --json
```

**Example Output:**
```
ID                            Type           Status             Plan        Duration
agent-123                     developer      Running            REQ-49      45s
agent-124                     architect      Completed          REQ-49      120s
```

### View Telemetry

```bash
# Telemetry for specific agent
rad monitor telemetry agent-123

# Summary for all agents
rad monitor telemetry

# JSON output
rad monitor telemetry agent-123 --json
```

**Example Output:**
```
Telemetry for agent: agent-123
Total Cost: $0.0450

Timestamp            Input Tokens    Output Tokens   Total Tokens    Cost            Model
2025-12-07 10:30:00  1500            800             2300            $0.0450         gpt-4
2025-12-07 10:31:00  2000            1200            3200            $0.0720         gpt-4
```

## Filtering and Querying Strategies

### Filter by Status

```rust
let all_agents = monitoring.list_agents()?;
let running_agents: Vec<_> = all_agents
    .iter()
    .filter(|a| a.status == AgentStatus::Running)
    .collect();
```

### Filter by Plan

```rust
let plan_agents = monitoring.get_plan_agents("REQ-49")?;
```

### Filter by Time Range

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();
let one_hour_ago = now - 3600;

let recent_agents: Vec<_> = monitoring
    .list_agents()?
    .into_iter()
    .filter(|a| a.start_time >= one_hour_ago)
    .collect();
```

### Aggregate Telemetry

```rust
// Get telemetry for multiple agents
let agents = monitoring.get_plan_agents("REQ-49")?;
let mut total_cost = 0.0;
let mut total_tokens = 0u64;

for agent in &agents {
    let cost = monitoring.get_total_cost(&agent.id)?;
    total_cost += cost;
    
    let telemetry = monitoring.get_agent_telemetry(&agent.id)?;
    for t in telemetry {
        total_tokens += t.total_tokens;
    }
}

println!("Plan REQ-49: ${:.4} total cost, {} total tokens", total_cost, total_tokens);
```

## Log Management

### Using LogManager

```rust
use radium_core::monitoring::LogManager;

// Create log manager
let logs_dir = workspace.radium_dir().join("logs");
let log_manager = LogManager::new(logs_dir)?;

// Create log file for agent
let mut log_file = log_manager.create_log("agent-123")?;
writeln!(log_file, "Agent started")?;

// Append to log
log_manager.append_log("agent-123", "Processing task 1")?;
log_manager.append_log("agent-123", "Processing task 2")?;

// Read log
let log_content = log_manager.read_log("agent-123")?;

// Tail log (last 10 lines)
let tail = log_manager.tail_log("agent-123", 10)?;

// List all logs
let all_logs = log_manager.list_logs()?;
```

## Integration with Workflows

### Automatic Agent Tracking

When using the workflow executor, agents are automatically tracked:

```rust
use radium_core::workflow::WorkflowExecutor;

// Workflow executor automatically:
// 1. Registers agents when steps start
// 2. Updates status as steps progress
// 3. Records telemetry when available
// 4. Completes agents when steps finish
```

### Manual Integration

```rust
// In your agent execution code
let agent = AgentRecord::new(agent_id, agent_type)
    .with_plan(plan_id)
    .with_process_id(std::process::id());

monitoring.register_agent(&agent)?;
monitoring.update_status(&agent_id, AgentStatus::Running)?;

// ... execute agent work ...

// Record telemetry
let telemetry = TelemetryRecord::new(agent_id.clone())
    .with_tokens(input_tokens, output_tokens)
    .with_model(model, provider);
telemetry.calculate_cost();
monitoring.record_telemetry(&telemetry).await?;

// Complete agent
monitoring.complete_agent(&agent_id, exit_code)?;
```

## Error Handling

### Handling Monitoring Errors

```rust
use radium_core::monitoring::{MonitoringError, Result};

match monitoring.get_agent("agent-123") {
    Ok(agent) => println!("Found agent: {}", agent.id),
    Err(MonitoringError::AgentNotFound(id)) => {
        println!("Agent {} not found", id);
    }
    Err(MonitoringError::Database(e)) => {
        eprintln!("Database error: {}", e);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### Graceful Degradation

```rust
// Monitoring failures shouldn't break agent execution
if let Err(e) = monitoring.register_agent(&agent) {
    tracing::warn!("Failed to register agent: {}", e);
    // Continue execution without monitoring
}
```

## Best Practices

### 1. Always Register Agents

Register agents at the start of execution to enable tracking:

```rust
let agent = AgentRecord::new(agent_id, agent_type);
monitoring.register_agent(&agent)?;
```

### 2. Update Status Regularly

Keep status up-to-date for accurate monitoring:

```rust
monitoring.update_status(&agent_id, AgentStatus::Running)?;
// ... do work ...
monitoring.update_status(&agent_id, AgentStatus::Completed)?;
```

### 3. Record Telemetry After Model Calls

Record telemetry immediately after model API calls:

```rust
let (input_tokens, output_tokens) = parse_model_response(&response)?;
let mut telemetry = TelemetryRecord::new(agent_id)
    .with_tokens(input_tokens, output_tokens)
    .with_model(model, provider);
telemetry.calculate_cost();
monitoring.record_telemetry(&telemetry).await?;
```

### 4. Use Hooks for Custom Logic

Register hooks for custom telemetry processing:

```rust
// Hooks can modify telemetry, add custom fields, etc.
// See hooks documentation for details
```

### 5. Filter Queries Efficiently

Use specific queries instead of filtering in memory:

```rust
// Good: Use database query
let plan_agents = monitoring.get_plan_agents("REQ-49")?;

// Less efficient: Filter in memory
let all_agents = monitoring.list_agents()?;
let plan_agents: Vec<_> = all_agents
    .iter()
    .filter(|a| a.plan_id.as_deref() == Some("REQ-49"))
    .collect();
```

## Common Patterns

### Pattern 1: Track Agent with Parent

```rust
// Parent agent
let parent = AgentRecord::new("parent-123".to_string(), "orchestrator".to_string());
monitoring.register_agent(&parent)?;

// Child agent
let child = AgentRecord::new("child-456".to_string(), "developer".to_string())
    .with_parent("parent-123".to_string());
monitoring.register_agent(&child)?;
```

### Pattern 2: Track Plan Execution

```rust
// All agents for a plan
let agents = monitoring.get_plan_agents("REQ-49")?;

// Calculate plan metrics
let mut total_cost = 0.0;
let mut total_duration = 0u64;

for agent in &agents {
    total_cost += monitoring.get_total_cost(&agent.id).unwrap_or(0.0);
    if let Some(end_time) = agent.end_time {
        total_duration += end_time - agent.start_time;
    }
}
```

### Pattern 3: Monitor Tool Usage

```rust
// Get telemetry with tool information
let telemetry = monitoring.get_agent_telemetry("agent-123")?;

let tool_usage: std::collections::HashMap<String, u64> = telemetry
    .iter()
    .filter_map(|t| t.tool_name.as_ref().map(|name| (name.clone(), 1)))
    .fold(std::collections::HashMap::new(), |mut acc, (name, count)| {
        *acc.entry(name).or_insert(0) += count;
        acc
    });

for (tool, count) in tool_usage {
    println!("{}: {} executions", tool, count);
}
```

## Troubleshooting

### Database Not Found

If you get "Failed to open monitoring database", ensure you're in a Radium workspace:

```bash
rad init  # Create workspace if needed
```

### Missing Telemetry

If telemetry is missing, check:
1. Are you calling `record_telemetry()` after model calls?
2. Are hooks executing successfully? (check logs)
3. Is the agent_id correct?

### High Memory Usage

For large telemetry datasets:
1. Use summary queries instead of loading all records
2. Filter by time range
3. Consider archiving old telemetry

