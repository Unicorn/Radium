# Radium Integration Plan

**Status**: Planning
**Prerequisites**: Phase 4 (Radium Migration) complete
**Integration Phases**: R1-R7, parallel to Phases 5-9

## Overview

This document outlines how to deeply integrate the Workflow Builder with Radium's infrastructure after the physical migration (Phase 4). These integration phases (R1-R7) enable the Workflow Builder to leverage Radium's orchestration, MCP, policy, and analytics systems.

**Note**: The physical move of code from `production-agent-coordinators` to `Radium` is handled in [Phase 4: Radium Migration](./phase-4-radium-migration.md). This document covers post-migration integration work.

## Project Analysis

### Radium Capabilities

Radium (`./Radium`) is a Rust-based agentic orchestration platform with:

| Component | Description | Integration Opportunity |
|-----------|-------------|------------------------|
| **radium-core** | Core library with workflow engine, agents, MCP, policy, hooks | Primary integration target |
| **radium-orchestrator** | Agent orchestration, task dispatch, execution queue | Run workflow components as agents |
| **radium-abstraction** | Model abstraction layer (Claude, Gemini, OpenAI) | Unified LLM access for activities |
| **radium-models** | Shared model types and traits | Type sharing across both projects |
| **Workflow Engine** | Built-in workflow behaviors, step tracking, recovery | Complement Temporal-based workflows |
| **MCP Module** | MCP client with tool/prompt support | Expose activities as MCP tools |
| **Policy Engine** | Fine-grained tool execution control | Activity-level access control |
| **Analytics** | Session metrics, cost tracking | Workflow execution monitoring |
| **Hook System** | Pre/post execution hooks | Workflow event integration |

### Workflow Builder Rust Migration

The Rust migration plan creates:
- Rust-based workflow schema validation and compilation
- TypeScript code generation for Temporal workflows
- Axum-based API service (Kong-proxied)
- Component migration to typed Rust schemas

## Integration Strategy

### Phase R1: Internal Type Sharing

**Goal**: Establish shared types between radium-workflow and radium-core.

**Note**: After Phase 4, the workflow code is already inside Radium. R1 focuses on creating proper interfaces between the new `radium-workflow` crate and existing Radium crates.

```
Radium/
  crates/
    radium-core/
      src/
        workflow/            # Existing Radium workflow behaviors
    radium-workflow/         # NEW: From workflow-builder (after R0)
      src/
        types/               # Shared type definitions
          activity.rs
          workflow.rs
          schema.rs
    radium-models/           # Existing shared models
      src/
        workflow_types.rs    # NEW: Types used by both workflow systems
```

**Tasks**:
- [ ] R1.1 Define workflow types in radium-models (shared across crates)
- [ ] R1.2 Create trait bounds for activity execution
- [ ] R1.3 Implement traits in radium-workflow for Temporal activities
- [ ] R1.4 Implement traits in radium-core for native Radium behaviors
- [ ] R1.5 Add radium-models dependency to radium-workflow

**Type Sharing Example**:
```rust
// Radium/crates/radium-models/src/workflow_types.rs
use serde::{Deserialize, Serialize};

/// Activity definition shared between radium-workflow and radium-core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDefinition {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub timeout_ms: Option<u64>,
    pub retry_policy: Option<RetryPolicy>,
}

/// Trait for activity execution (implemented differently in each system)
#[async_trait::async_trait]
pub trait ActivityExecutor: Send + Sync {
    async fn execute(
        &self,
        activity: &ActivityDefinition,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, ActivityError>;
}
```

---

### Phase R2: Radium Agent Bridge

**Goal**: Enable Workflow Builder components to run as Radium agents.

**Architecture**:
```
                     Radium Orchestrator
                            │
            ┌───────────────┼───────────────┐
            ▼               ▼               ▼
     Native Radium    Workflow Builder   External
        Agents           Adapters         Agents
            │               │               │
            └───────────────┼───────────────┘
                            ▼
                    Execution Queue
```

**Tasks**:
- [ ] R2.1 Create WorkflowComponentAgent adapter implementing radium_orchestrator::Agent
- [ ] R2.2 Implement agent lifecycle (start/stop/pause/resume) for workflow components
- [ ] R2.3 Bridge workflow activities to Radium's task dispatch system
- [ ] R2.4 Support Radium's selection criteria for workflow component routing
- [ ] R2.5 Integrate with Radium's progress reporting

**Agent Adapter**:
```rust
// Radium/crates/radium-workflow/src/bridge/agent.rs
use radium_orchestrator::{Agent, AgentContext, AgentOutput};
use radium_models::workflow_types::ActivityDefinition;
use async_trait::async_trait;

/// Adapter that wraps a Workflow Builder activity as a Radium agent
pub struct WorkflowActivityAgent {
    activity: ActivityDefinition,
    executor: Arc<dyn ActivityExecutor>,
}

#[async_trait]
impl Agent for WorkflowActivityAgent {
    fn id(&self) -> &str {
        &self.activity.name
    }

    fn description(&self) -> &str {
        self.activity.description.as_deref().unwrap_or("")
    }

    async fn execute(
        &self,
        input: &str,
        context: AgentContext<'_>,
    ) -> Result<AgentOutput, ModelError> {
        let input_value: serde_json::Value = serde_json::from_str(input)?;
        let result = self.executor.execute(&self.activity, input_value).await?;
        Ok(AgentOutput::StructuredData(result))
    }
}
```

---

### Phase R3: MCP Tool Exposure

**Goal**: Expose Workflow Builder activities as MCP tools accessible to Radium and external clients.

**Architecture**:
```
                    MCP Clients
                        │
                        ▼
                ┌───────────────┐
                │  MCP Proxy    │ (Radium)
                └───────┬───────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
  Native MCP      Workflow Builder   External
    Servers          Activities      MCP Servers
```

**Tasks**:
- [ ] R3.1 Implement MCP tool generation from activity schemas
- [ ] R3.2 Create McpActivityBridge for tool execution
- [ ] R3.3 Register workflow activities with Radium's McpToolRegistry
- [ ] R3.4 Support dynamic tool discovery when workflows are deployed
- [ ] R3.5 Handle activity timeout/retry via MCP protocol

**MCP Bridge**:
```rust
// Radium/crates/radium-workflow/src/bridge/mcp.rs
use radium_core::mcp::{McpTool, McpToolResult, McpContent};
use radium_models::workflow_types::ActivityDefinition;

/// Convert workflow activity to MCP tool definition
pub fn activity_to_mcp_tool(activity: &ActivityDefinition) -> McpTool {
    McpTool {
        name: format!("workflow.{}", activity.name),
        description: activity.description.clone(),
        input_schema: Some(activity.input_schema.clone()),
    }
}

/// Bridge MCP tool calls to activity execution
pub struct McpActivityBridge {
    activities: HashMap<String, Arc<dyn ActivityExecutor>>,
}

impl McpActivityBridge {
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<McpToolResult, McpError> {
        let activity_name = tool_name.strip_prefix("workflow.").unwrap_or(tool_name);
        let executor = self.activities.get(activity_name)
            .ok_or_else(|| McpError::ToolNotFound(tool_name.to_string()))?;

        // Execute and convert result
        let result = executor.execute_raw(args).await?;
        Ok(McpToolResult {
            content: vec![McpContent::Text { text: result.to_string() }],
            is_error: false,
        })
    }
}
```

---

### Phase R4: Workflow Engine Unification

**Goal**: Determine the relationship between Radium's workflow engine and Temporal-based workflows.

**Options Analysis**:

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **A: Parallel Systems** | No migration risk, both available | Duplication, user confusion | Short-term |
| **B: Radium as Temporal Alternative** | Simpler for light workflows | Loses Temporal durability | Specific use cases |
| **C: Temporal via Radium** | Unified interface | Complex adapter layer | Long-term goal |
| **D: Feature Split** | Clear responsibilities | Requires clear boundaries | Recommended |

**Recommended Approach: Feature Split (D)**

- **Radium Workflow Engine**: Real-time, low-latency agent coordination
  - Vibe checks, loop behaviors, checkpoints
  - Sub-second response workflows
  - Interactive agent workflows

- **Temporal (via Workflow Builder)**: Durable, long-running business processes
  - Multi-step approval workflows
  - Scheduled/batch processes
  - Saga patterns with compensation

**Tasks**:
- [ ] R4.1 Document use-case boundaries for each engine
- [ ] R4.2 Create workflow type detection (real-time vs durable)
- [ ] R4.3 Implement routing logic based on workflow characteristics
- [ ] R4.4 Bridge Radium behaviors as Temporal activities where needed
- [ ] R4.5 Support hybrid workflows (Radium steps within Temporal workflows)

**Hybrid Workflow Support**:
```rust
// Bridge Radium workflow step as Temporal activity
pub async fn execute_radium_behavior(
    behavior_config: RadiumBehaviorConfig,
    context: ExecutionContext,
) -> Result<BehaviorResult, WorkflowError> {
    // Execute Radium behavior evaluator
    let evaluator = match &behavior_config {
        RadiumBehaviorConfig::VibeCheck(config) => {
            VibeCheckEvaluator::new(config.clone())
        }
        RadiumBehaviorConfig::Loop(config) => {
            LoopEvaluator::new(config.clone())
        }
        RadiumBehaviorConfig::Trigger(config) => {
            TriggerEvaluator::new(config.clone())
        }
    };

    evaluator.evaluate(&context).await
}
```

---

### Phase R5: Policy Integration

**Goal**: Apply Radium's policy engine to workflow activity execution.

**Architecture**:
```
              Workflow Execution Request
                        │
                        ▼
              ┌───────────────────┐
              │  Policy Engine    │
              │  (Constitution)   │
              └─────────┬─────────┘
                        │
         ┌──────────────┼──────────────┐
         ▼              ▼              ▼
      ALLOW          REQUIRE        DENY
         │           APPROVAL          │
         │              │              │
         ▼              ▼              ▼
    Execute       Queue for       Return
    Activity       Review         Error
```

**Tasks**:
- [ ] R5.1 Map workflow activities to Radium policy actions
- [ ] R5.2 Implement policy check hook before activity execution
- [ ] R5.3 Support approval workflows for sensitive activities
- [ ] R5.4 Integrate with Radium's constitution manager
- [ ] R5.5 Add audit logging for policy decisions

**Policy Hook**:
```rust
// Integrate with Radium policy engine
pub async fn check_activity_policy(
    policy_engine: &PolicyEngine,
    activity: &ActivityDefinition,
    input: &serde_json::Value,
    context: &ExecutionContext,
) -> Result<PolicyDecision, PolicyError> {
    let action = PolicyAction::ToolExecution {
        tool_name: activity.name.clone(),
        arguments: input.clone(),
    };

    policy_engine.evaluate(action, context).await
}
```

---

### Phase R6: Analytics and Monitoring

**Goal**: Unified analytics across Workflow Builder and Radium.

**Metrics to Track**:
- Workflow execution counts and durations
- Activity success/failure rates
- Token usage per workflow
- Cost attribution by workflow type
- Policy decision statistics

**Tasks**:
- [ ] R6.1 Emit Radium session metrics from workflow activities
- [ ] R6.2 Bridge workflow compiler metrics to Radium analytics
- [ ] R6.3 Create unified dashboard for workflow+agent metrics
- [ ] R6.4 Integrate cost tracking with Radium budget analytics
- [ ] R6.5 Support workflow-level anomaly detection

**Analytics Integration**:
```rust
// Emit workflow metrics to Radium analytics
pub fn record_workflow_execution(
    analytics: &SessionAnalytics,
    workflow_id: &str,
    workflow_type: &str,
    duration_ms: u64,
    token_usage: TokenUsage,
    status: WorkflowStatus,
) {
    analytics.record_event(SessionEvent::WorkflowExecution {
        workflow_id: workflow_id.to_string(),
        workflow_type: workflow_type.to_string(),
        duration_ms,
        input_tokens: token_usage.input_tokens,
        output_tokens: token_usage.output_tokens,
        status: status.to_string(),
    });
}
```

---

### Phase R7: UI Integration

**Goal**: Integrate Workflow Builder UI into Radium's desktop application.

**Architecture Options**:

| Option | Description | Complexity | Performance |
|--------|-------------|------------|-------------|
| **Embedded WebView** | Load workflow-builder UI in Tauri WebView | Low | Medium |
| **Native Port** | Rewrite UI in native (egui/iced) | High | High |
| **Hybrid** | React UI with Rust rendering backend | Medium | High |

**Recommended: Embedded WebView with Deep Integration**

**Tasks**:
- [ ] R7.1 Create Tauri command bridge for workflow operations
- [ ] R7.2 Implement IPC protocol for workflow events
- [ ] R7.3 Add workflow builder view to Radium desktop
- [ ] R7.4 Share authentication state between Radium and workflow UI
- [ ] R7.5 Support drag-and-drop between Radium TUI and workflow canvas

**Tauri Command Bridge**:
```rust
// Radium desktop workflow commands
#[tauri::command]
async fn compile_workflow(
    state: State<'_, AppState>,
    workflow: WorkflowDefinition,
) -> Result<CompilationResult, String> {
    let compiler = state.workflow_compiler.lock().await;
    compiler.compile(workflow)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn deploy_workflow(
    state: State<'_, AppState>,
    workflow_id: String,
    environment: String,
) -> Result<DeploymentResult, String> {
    // Use Radium orchestrator to deploy
    let orchestrator = state.orchestrator.lock().await;
    orchestrator.deploy_workflow(&workflow_id, &environment)
        .await
        .map_err(|e| e.to_string())
}
```

---

## Directory Structure After Integration

```
Radium/
  Cargo.toml                           # Workspace with all crates
  crates/
    radium-core/
      src/
        workflow/                      # Existing Radium workflow behaviors
        mcp/                           # MCP integration
    radium-orchestrator/               # Agent orchestration
    radium-abstraction/                # Model abstraction
    radium-models/
      src/
        workflow_types.rs              # NEW: Shared workflow types
    radium-workflow/                   # NEW: Moved from workflow-builder
      Cargo.toml
      src/
        lib.rs
        schemas/                       # Workflow schemas (Phase 5-7)
          mod.rs
          activity.rs
          workflow.rs
          variables.rs
          components/
        compiler/                      # TypeScript compiler
          mod.rs
          generator.rs
          templates.rs
        validation/                    # Schema validation
        bridge/                        # Radium integration (R1-R3)
          mod.rs
          agent.rs                     # WorkflowActivityAgent
          mcp.rs                       # McpActivityBridge
          policy.rs                    # PolicyIntegration
          analytics.rs                 # Analytics bridge
      templates/                       # Handlebars templates
      tests/
  apps/
    cli/                               # Existing Radium CLI
    tui/                               # Existing Radium TUI
    desktop/                           # Tauri desktop app
      src/
        workflow_commands.rs           # NEW: Workflow Tauri commands (R7)
    workflow-builder/                  # NEW: React web UI
      package.json
      src/
        components/                    # Canvas, nodes, toolbar
        hooks/                         # useWorkflow, useCompiler
        api/                           # API client (talks to radium-workflow)
      public/

production-agent-coordinators/
  packages/
    workflow-builder/                  # DEPRECATED after Phase 4
      README.md                        # "Moved to Radium/apps/workflow-builder"
```

---

## Migration Timeline

```
                          PHYSICAL MOVE (Phase 4)
                               │
Phase 1-3 (in prod-agent-coord)│  Phase 5-9 + R1-R7 (in Radium)
                               │
        ├── Phase 1: Kong ─────┤
        ├── Phase 2: Compiler ─┤
        ├── Phase 3: Workflows ┤
                               │
═══════════════════════════════╪═══════════════════════════════════════
        Phase 4: MIGRATION ────┤  Move workflow-builder → Radium
═══════════════════════════════╪═══════════════════════════════════════
                               │
                               ├── R1: Type Sharing ───────────────────┐
                               ├── R2: Agent Bridge ───────────────────┤
                               ├── R3: MCP Tools ──────────────────────┤
        ├── Phase 5: Variables ┼── R4: Engine Unification ─────────────┤
        ├── Phase 6: Components┼── R5: Policy Integration ─────────────┤
        ├── Phase 7: Advanced ─┼── R6: Analytics ──────────────────────┤
        ├── Phase 8: Production┼── R7: UI Integration ─────────────────┤
        └── Phase 9: Agent ────┴───────────────────────────────────────┘
```

**Key Milestone: Phase 4 (Physical Move)**
- Happens immediately after Phase 3 verification
- All subsequent work (Phases 5-9 and R1-R7) occurs in Radium directory
- Single codebase, unified workspace, no cross-repo dependencies

---

## Success Criteria

| Milestone | Criteria | Measurement |
|-----------|----------|-------------|
| R1 Complete | Shared types used by radium-workflow and radium-core | CI green |
| R2 Complete | Activity runs as Radium agent | Integration test |
| R3 Complete | Activity callable via MCP | MCP client test |
| R4 Complete | Workflow type routing works | E2E test |
| R5 Complete | Policy blocks sensitive activity | Policy test |
| R6 Complete | Unified metrics visible | Dashboard demo |
| R7 Complete | UI accessible in Radium desktop | User test |

**Note**: Phase 4 (Physical Migration) success criteria are in [phase-4-radium-migration.md](./phase-4-radium-migration.md).

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Circular dependencies between crates | Build failures | Strict layering, shared-crates at bottom |
| Feature parity gaps | User confusion | Clear documentation of which engine for what |
| Performance overhead from bridges | Latency | Benchmark early, optimize hot paths |
| Version drift between projects | Integration breaks | Automated integration tests in CI |
| Temporal vs Radium workflow conflict | Complexity | Clear use-case separation |

---

## Open Questions

1. **Temporal Worker Location**: Should Temporal workers run inside Radium process or separate?
   - Recommendation: Separate process initially, consider embedding later
2. **Real-time Preview**: Can Radium TUI preview workflow execution in real-time?
   - Investigate: Could use radium-core's step tracking
3. **Multi-tenancy**: How to isolate workflows in a shared Radium deployment?
   - Depends on Radium's existing multi-tenancy model
4. **Kong Migration**: Does R0 require updating Kong routes immediately?
   - Could proxy to new location or update routes as part of R0
5. **Web UI Serving**: Should workflow-builder UI be served by Radium or separately?
   - Options: Tauri WebView, standalone, or served from Radium server

---

## Next Steps

1. Complete Phase 4 (Radium Migration) - see [phase-4-radium-migration.md](./phase-4-radium-migration.md)
2. Begin R1 (Internal Type Sharing) once code is in Radium
3. Continue Phases 5-9 in parallel with R1-R7 integration work
4. Review integration progress at each phase milestone

---

## References

### Migration Phases
- [Phase 1: Kong Abstraction](./phase-1-kong-abstraction.md)
- [Phase 2: Rust Compiler Foundation](./phase-2-rust-compiler-foundation.md)
- [Phase 3: Basic Workflows](./phase-3-basic-workflows.md)
- [Phase 4: Radium Migration](./phase-4-radium-migration.md) - Physical move to Radium
- [Phase 5: Variables State](./phase-5-variables-state.md)
- [Phase 6: Component Migration](./phase-6-component-migration.md)
- [Phase 7: Advanced Features](./phase-7-advanced-features.md)
- [Phase 8: Production Hardening](./phase-8-production-hardening.md)
- [Phase 9: Component Builder](./phase-9-component-builder.md)

### Radium
- [Radium README](../../../../../../Radium/README.md)
- [Radium Workflow Engine](../../../../../../Radium/crates/radium-core/src/workflow/mod.rs)
- [Radium Orchestrator](../../../../../../Radium/crates/radium-orchestrator/src/lib.rs)
