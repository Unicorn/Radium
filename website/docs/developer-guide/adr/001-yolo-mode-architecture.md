---
id: "001-yolo-mode-architecture"
title: "ADR-001: YOLO Mode Autonomous Orchestration Architecture"
sidebar_label: "ADR-001: YOLO Mode Autonomo..."
---

# ADR-001: YOLO Mode Autonomous Orchestration Architecture

**Status:** Accepted
**Date:** 2025-12-07
**Decision Makers:** Radium Core Team
**Related REQs:** REQ-165, REQ-166, REQ-167, REQ-168, REQ-169, REQ-170, REQ-171

---

## Context

Implement fully autonomous execution mode where Radium can complete entire implementations from high-level goals (e.g., "complete the implementation in REQ-123") without user intervention. The system must make intelligent decisions about:

- **Agent selection** based on task requirements and agent capabilities
- **Resource allocation** across multiple agents and AI providers
- **Error recovery** when tasks fail or providers are exhausted
- **Multi-agent coordination** for complex tasks requiring specialized skills

### User Story

> "As a user, I would like to open Radium desktop, CLI, or TUI and simply execute a single command: 'please complete the entire implementation found in `<source>` for me.' Radium orchestrator would then verify and analyze source which could be Braingrid, Jira, local MD files, etc. Once it verified it had access and ability to read all source material, it would operate in a mode similar to YOLO mode in Gemini/Claude and would oversee execution entirely to completion."

---

## Decision

### 1. Workflow Generation Strategy

**Leverage Existing:** `PlanGenerator` (`crates/radium-core/src/planning/mod.rs`)

**Enhancements Required:**
- Extend plan generation to create full workflow DAGs with dependencies
- Add task dependency analysis using existing planning infrastructure
- Integrate with `WorkflowEngine` (`crates/radium-core/src/workflow/engine.rs`)

**Implementation:**
```rust
// Extend PlanGenerator with dependency tracking
pub struct WorkflowPlan {
    pub tasks: Vec<TaskNode>,
    pub dependencies: HashMap<TaskId, Vec<TaskId>>,
    pub estimated_cost: TokenBudget,
}

impl PlanGenerator {
    pub async fn generate_autonomous_workflow(&self, goal: &str, sources: Vec<String>) -> Result<WorkflowPlan> {
        // 1. Verify all sources accessible (REQ-165)
        // 2. Parse goals into task tree
        // 3. Analyze dependencies
        // 4. Estimate resource requirements
        // 5. Generate workflow DAG
    }
}
```

---

### 2. Agent Selection Algorithm

**Leverage Existing:** `AgentMetadata` system (`crates/radium-core/src/agents/metadata.rs`)

**Strategy:**
1. **Capability Matching**: Parse task requirements → match against agent capabilities
2. **Cost Optimization**: Use `ModelSelector` logic for budget-aware selection
3. **Performance Tracking**: Track agent success rates and execution times
4. **Dynamic Reassignment**: Switch agents on repeated failures

**Implementation:**
```rust
pub struct AgentSelector {
    registry: Arc<AgentRegistry>,
    model_selector: Arc<ModelSelector>,
    performance_tracker: Arc<PerformanceTracker>,
}

impl AgentSelector {
    pub async fn select_agent(&self, task: &Task, context: &SelectionContext) -> Result<AgentConfig> {
        // 1. Extract task requirements (code, docs, testing, etc.)
        // 2. Filter agents by capability match
        // 3. Score candidates by: performance + cost + availability
        // 4. Apply fallback chain if primary unavailable
    }
}
```

---

### 3. Error Recovery Strategy

**Leverage Existing:**
- `Checkpoint` system (`crates/radium-core/src/checkpoint/mod.rs`)
- Hook system (`crates/radium-orchestrator/src/executor.rs`)
- Model fallback (`crates/radium-core/src/models/selector.rs`)

**Recovery Hierarchy:**
1. **Retry with exponential backoff** (max 3 attempts)
2. **Checkpoint restoration** for workflow state recovery
3. **Agent fallback**: Primary → Specialized fallback → General-purpose agent
4. **Provider switching** when quotas exhausted

**Implementation:**
```rust
pub struct RecoveryStrategy {
    max_retries: u32,
    backoff_ms: u64,
    checkpoint_interval: u32, // Create checkpoint every N steps
}

impl RecoveryStrategy {
    pub async fn handle_failure(&self, error: &ExecutionError, context: &WorkflowContext) -> RecoveryAction {
        match error {
            ExecutionError::TransientError(_) => RecoveryAction::Retry { attempt: context.retries + 1, delay_ms: self.backoff_ms * 2_u64.pow(context.retries) },
            ExecutionError::AgentFailure(_) => RecoveryAction::SwitchAgent { fallback_agent: self.select_fallback(context) },
            ExecutionError::QuotaExhausted(_) => RecoveryAction::SwitchProvider { fallback_provider: self.next_provider(context) },
            ExecutionError::PermanentError(_) => RecoveryAction::Checkpoint { restore_to: context.last_checkpoint },
        }
    }
}
```

---

### 4. Resource Management

**Leverage Existing:**
- `TelemetryRecord` (`crates/radium-core/src/monitoring/telemetry.rs`)
- `ModelSelector` budget tracking (`crates/radium-core/src/models/selector.rs`)

**Critical Error Detection:**
- Track total tokens consumed across all providers
- Implement circuit breaker when provider returns 429 (quota exceeded)
- Pause execution when all authenticated providers exhausted
- Alert user with clear actionable message

**Implementation:**
```rust
pub struct ResourceManager {
    budget: TokenBudget,
    provider_quotas: HashMap<ProviderId, QuotaStatus>,
    circuit_breakers: HashMap<ProviderId, CircuitBreaker>,
}

impl ResourceManager {
    pub fn check_critical_errors(&self) -> Option<CriticalError> {
        let exhausted_providers = self.provider_quotas.values().filter(|q| q.is_exhausted()).count();
        if exhausted_providers == self.provider_quotas.len() {
            return Some(CriticalError::AllProvidersExhausted {
                message: "All AI providers have exhausted their quotas. Please add credits or wait for quota reset.",
                providers: self.provider_quotas.keys().cloned().collect(),
            });
        }
        None
    }
}
```

---

### 5. Multi-Agent Coordination (REQ-171)

**New System Required:** Agent-to-agent communication bus

**Key Components:**
- **Message Bus**: Pub/sub system for agent communication
- **Shared Workspace**: Read/write access to plan artifacts
- **Task Delegation**: Agents can spawn sub-tasks for other agents
- **Conflict Resolution**: Detect duplicate work and coordinate

**Implementation:**
```rust
pub struct AgentCoordinator {
    message_bus: Arc<MessageBus>,
    shared_workspace: Arc<Workspace>,
    task_queue: Arc<TaskQueue>,
}

impl AgentCoordinator {
    pub async fn delegate_subtask(&self, from: AgentId, to: AgentId, task: SubTask) -> Result<TaskHandle> {
        // 1. Validate target agent has required capabilities
        // 2. Submit to task queue with dependency tracking
        // 3. Subscribe to task completion events
        // 4. Return handle for monitoring
    }

    pub async fn resolve_conflict(&self, agents: Vec<AgentId>, resource: ResourceId) -> Resolution {
        // 1. Check task priorities
        // 2. Apply conflict resolution policy (first-come, priority-based, etc.)
        // 3. Notify losing agents to skip duplicate work
    }
}
```

---

## Consequences

### Positive

✅ **Leverages Existing Infrastructure**: 90% of required systems already exist
✅ **Backward Compatible**: Manual agent execution still works; YOLO mode is opt-in via policy engine
✅ **Incremental Rollout**: Can deploy features progressively (REQ-165 → REQ-170)
✅ **Production-Ready Design**: Built on battle-tested patterns (checkpoints, hooks, policies)

### Negative

⚠️ **Increased Complexity**: Adds orchestration layer on top of existing systems
⚠️ **Resource Intensive**: Autonomous mode consumes more tokens than guided execution
⚠️ **Debugging Challenges**: Multi-agent coordination failures can be hard to trace

### Risks

🚨 **Provider Costs**: Autonomous mode could rack up significant API costs if not monitored
🚨 **Runaway Execution**: Need kill switch for infinite loops or repeated failures
🚨 **Quality Control**: Autonomous decisions might not match user intent without oversight

---

## Mitigation Strategies

1. **Cost Controls**: Enforce hard token budgets, alert at 80% threshold
2. **Kill Switch**: User can abort at any time; max iteration limits enforced
3. **Quality Gates**: Use `VibeCheck` behavior (`crates/radium-core/src/workflow/behaviors/vibe_check.rs`) at checkpoints
4. **Audit Trail**: Full telemetry of all decisions, agent switches, and retries

---

## Implementation Plan

### Phase 1: Foundation (Weeks 1-2)
- REQ-164: Complete test coverage ✅
- REQ-165: Source verification (6 tasks) 🚧 17% complete
- REQ-168: Error handling & circuit breaker

### Phase 2: Intelligence (Weeks 3-4)
- REQ-170: Workflow decomposition with DAG
- REQ-166: Dynamic agent selection
- REQ-169: Unified `rad complete` command

### Phase 3: Coordination (Weeks 5-6)
- REQ-171: Multi-agent collaboration
- REQ-167: Continuous execution mode (YOLO)
- Integration testing and refinement

---

## References

- **Existing Systems:**
  - Plan Generator: `crates/radium-core/src/planning/mod.rs`
  - Workflow Engine: `crates/radium-core/src/workflow/engine.rs`
  - Model Selector: `crates/radium-core/src/models/selector.rs`
  - Policy Engine: `crates/radium-core/src/policy/types.rs:L56`
  - Checkpoint System: `crates/radium-core/src/checkpoint/mod.rs`
  - Learning System: `crates/radium-core/src/learning/mod.rs`

- **Related Documentation:**
  - Integration Map: `docs/yolo-mode/integration-map.md`
  - User Story: Autonomous Orchestration Request (2025-12-07)
