# radium-orchestrator

Advanced multi-agent orchestration and workflow execution engine for Radium.

## Overview

`radium-orchestrator` provides intelligent task routing, multi-agent coordination, and parallel workflow execution capabilities. It serves as the brain of Radium's agent system, determining which agents to invoke and how to coordinate their work.

## Features

### Intelligent Orchestration
- **Automatic Agent Routing**: Analyzes tasks and selects appropriate specialist agents
- **Multi-Agent Workflows**: Coordinates multiple agents working together
- **Context-Aware Decision Making**: Uses task history and agent capabilities
- **Model-Agnostic**: Works with Gemini, Claude, OpenAI, and prompt-based fallback

### Workflow Execution
- **DAG-Based Execution**: Directed acyclic graph workflow engine
- **Parallel Execution**: Concurrent task execution with dependency management
- **Error Handling**: Automatic retry, fallback, and recovery strategies
- **Progress Tracking**: Real-time progress updates and status monitoring

### Skill-Based Routing
- **Skill Classification**: Categorizes tasks by required skills (coding, research, analysis, etc.)
- **Agent Capabilities**: Matches tasks to agents based on their skill profiles
- **Performance Tracking**: Learns from past successes to improve routing
- **Cost Optimization**: Routes to cost-effective agents when appropriate

### Event System
- **Real-Time Events**: Streaming events for UI updates
- **Comprehensive Event Types**: User input, assistant messages, tool calls, approvals, errors
- **Thinking Mode Support**: Transparent reasoning with thinking session events
- **Recommendation System**: Actionable recommendations with execution requests

### Tool Integration
- **File Operations**: Read, write, edit, search with workspace boundary validation
- **MCP Tools**: Integration with Model Context Protocol servers
- **Hook System**: Before/after tool execution hooks
- **Context Loading**: Automatic context file loading from workspace

## Architecture

```
radium-orchestrator/
├── src/
│   ├── lib.rs              - Public API and re-exports
│   ├── executor.rs         - Core workflow executor
│   ├── error_router.rs     - Error classification and routing
│   ├── skill_classifier.rs - Task skill classification
│   ├── routing/            - Agent routing and cost tracking
│   │   ├── agent_router.rs
│   │   └── cost_tracker.rs
│   └── orchestration/      - Orchestration primitives
│       ├── events.rs       - Event definitions
│       ├── context_loader.rs
│       ├── file_tools.rs
│       ├── hooks.rs
│       └── mcp_tools.rs
└── tests/                  - Integration tests
```

## Usage

### Basic Orchestration

```rust
use radium_orchestrator::{
    executor::WorkflowExecutor,
    orchestration::events::OrchestrationEvent,
};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create event channel
    let (tx, mut rx) = broadcast::channel(100);

    // Create executor
    let executor = WorkflowExecutor::new(
        agent_registry,
        tool_provider,
        Some(tx),
    );

    // Execute workflow
    let result = executor.execute(workflow_id, input).await?;

    // Handle events
    while let Ok(event) = rx.recv().await {
        match event {
            OrchestrationEvent::ToolCallStarted { tool_name, .. } => {
                println!("Executing tool: {}", tool_name);
            }
            OrchestrationEvent::Done { finish_reason, .. } => {
                println!("Finished: {}", finish_reason);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Skill-Based Routing

```rust
use radium_orchestrator::skill_classifier::SkillClassifier;

let classifier = SkillClassifier::new();

// Classify a task
let task = "Implement a REST API endpoint for user authentication";
let skills = classifier.classify(task);

// Skills might include: ["coding", "api_development", "security"]
```

### Error Handling

```rust
use radium_orchestrator::error_router::ErrorRouter;

let router = ErrorRouter::new(error_classifier);

// Handle and route errors
let error_result = router.handle_error(
    agent_id,
    error,
    context,
    max_attempts,
).await?;

match error_result {
    ErrorHandlingResult::Retry { strategy, .. } => {
        // Retry with backoff
    }
    ErrorHandlingResult::Fallback { agent_id, .. } => {
        // Route to fallback agent
    }
    ErrorHandlingResult::Abort { reason, .. } => {
        // Terminate workflow
    }
}
```

## Event Types

### Core Events
- `UserInput` - User provides input to the orchestrator
- `AssistantMessage` - Agent generates a response
- `ToolCallRequested` - Agent requests tool execution
- `ToolCallStarted` - Tool execution begins
- `ToolCallFinished` - Tool execution completes
- `ApprovalRequired` - Tool requires user approval
- `Error` - Error occurred during execution
- `Done` - Orchestration completed

### Thinking Mode Events
- `ThinkingSessionStarted` - Agent begins reasoning
- `ThinkingStepAdded` - New reasoning step added
- `ThinkingStepUpdated` - Reasoning step status updated
- `ThinkingSessionEnded` - Reasoning complete

### Recommendation Events
- `RecommendationsSessionStarted` - Recommendations being generated
- `RecommendationAdded` - New recommendation added
- `RecommendationsExecutionRequested` - Ready for execution

## Configuration

### Workflow Configuration

```rust
use radium_orchestrator::executor::ExecutorConfig;

let config = ExecutorConfig {
    max_iterations: 10,
    timeout_seconds: 300,
    enable_parallel: true,
    budget_limit: Some(10.0), // $10 limit
    retry_policy: RetryPolicy::ExponentialBackoff {
        initial_delay_ms: 1000,
        max_delay_ms: 30000,
        max_attempts: 3,
    },
};
```

### Agent Routing

```rust
use radium_orchestrator::routing::agent_router::AgentRouter;

let router = AgentRouter::new(agent_registry);

// Route task to best agent
let agent_id = router.route_task(
    task_description,
    required_skills,
    cost_constraint,
).await?;
```

## Trait Integration

The orchestrator uses traits to avoid circular dependencies with `radium-core`:

```rust
/// Budget management trait
pub trait BudgetManagerTrait: Send + Sync {
    fn check_budget_available(&self, estimated_cost: f64)
        -> Result<(), BudgetCheckResult>;
    fn record_cost(&self, actual_cost: f64) -> Result<(), BudgetError>;
    fn get_remaining_budget(&self) -> f64;
}

/// Hook execution trait
pub trait HookExecutor: Send + Sync {
    async fn execute_before_tool(&self, tool: &str, args: &serde_json::Value)
        -> Result<(), String>;
    async fn execute_after_tool(&self, tool: &str, result: &ToolResult)
        -> Result<(), String>;
}

/// Sandbox operations trait
pub trait SandboxOperations: Send + Sync {
    async fn execute_in_sandbox(&self, command: &str, args: &[String])
        -> Result<SandboxResult, String>;
}
```

## Performance

- **Concurrent Execution**: Truly parallel task execution with async/await
- **Efficient Event Streaming**: Broadcast channels for low-latency events
- **Smart Caching**: Caches agent capabilities and skill classifications
- **Optimized Routing**: Fast agent selection with O(1) lookups

## Integration with radium-core

The orchestrator integrates with `radium-core` through the workflow feature:

```toml
# In radium-core/Cargo.toml
[features]
workflow = ["radium-orchestrator", "orchestrator-integration", "mcp-progress"]

[dependencies]
radium-orchestrator = { workspace = true, optional = true }
```

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test '*'

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Benchmarks

```bash
# Skill routing benchmark
cargo bench --bench skill_routing_benchmark

# Orchestration benchmark
cargo bench --bench orchestration_benchmark
```

## Development

### Adding New Event Types

1. Add to `OrchestrationEvent` enum in `src/orchestration/events.rs`
2. Update event handlers in clients (CLI, TUI, Desktop)
3. Add tests for event emission
4. Update documentation

### Adding New Skills

1. Add skill category to `SkillCategory` enum
2. Update skill classifier rules in `skill_classifier.rs`
3. Add tests for skill detection
4. Update agent skill profiles

## Dependencies

### Major Dependencies
- **radium-abstraction** - Core trait definitions
- **radium-models** - AI model implementations
- **radium-ml** - Machine learning for skill routing
- **tokio** - Async runtime
- **serde/serde_json** - Serialization
- **tracing** - Logging
- **anyhow/thiserror** - Error handling

### Dev Dependencies
- **tempfile** - Test fixtures
- **criterion** - Benchmarking
- **tokio-test** - Async test utilities

## Roadmap

- [ ] True parallel execution (requires Arc<dyn Repository + Send + Sync>)
- [ ] Advanced cost optimization with ML models
- [ ] Multi-tenancy support with resource isolation
- [ ] Distributed orchestration across multiple nodes
- [ ] Enhanced skill learning from execution history
- [ ] Real-time performance profiling and optimization

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

MIT - see [LICENSE](../../LICENSE) for details

## Links

- [Orchestration User Guide](../../website/docs/user-guide/orchestration.md)
- [Workflow Examples](../../website/docs/examples/orchestration-workflows.md)
- [Architecture Overview](../../website/docs/developer-guide/architecture/)
