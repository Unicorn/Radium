---
id: "integration-map"
title: "YOLO Mode Integration Map"
sidebar_label: "YOLO Mode Integration Map"
---

# YOLO Mode Integration Map

**Last Updated:** 2025-12-07
**Related ADR:** [ADR-001: YOLO Mode Architecture](../adr/001-yolo-mode-architecture.md)

This document maps existing Radium systems to YOLO Mode requirements and identifies integration points.

---

## 🗺️ System Integration Overview

```
User Command: "rad complete <source>"
     ↓
[Source Verification] → REQ-165 → SourceReader Registry
     ↓
[Workflow Generation] → REQ-170 → PlanGenerator + WorkflowEngine
     ↓
[Agent Selection] → REQ-166 → AgentSelector + ModelSelector
     ↓
[Execution Loop] → REQ-167 → Orchestration Service + Policy Engine
     ↓
[Error Recovery] → REQ-168 → Checkpoint + Circuit Breaker + Retry Logic
     ↓
[Multi-Agent Coordination] → REQ-171 → Message Bus + Task Queue
```

---

## 1. Source Verification & Analysis (REQ-165)

### Existing System
**File:** `crates/radium-core/src/context/sources/`
**Status:** ⚠️ Partially Implemented

**What Exists:**
- Context management with file injection (`crates/radium-core/src/context/mod.rs`)
- Memory context retrieval (`crates/radium-core/src/memory/mod.rs`)
- Workspace structure parsing (`crates/radium-core/src/workspace/mod.rs`)

**What's Needed:**
- `SourceReader` trait for protocol abstraction
- Concrete implementations: LocalFileReader, HttpReader, JiraReader, **BraingridReader**
- `SourceRegistry` for scheme-based routing
- `ValidateSources` gRPC endpoint

**Integration Point:**
```rust
// In rad complete command
pub async fn execute_complete(&self, source_uri: &str) -> Result<()> {
    // Step 1: Verify source accessibility
    let validator = SourceValidator::new(source_registry);
    let results = validator.validate_sources(vec![source_uri.to_string()]).await?;

    if !results.all_valid {
        display_validation_errors(&results);
        if !prompt_user_proceed()? {
            return Err(anyhow!("Source verification failed"));
        }
    }

    // Step 2: Fetch source content
    let reader = source_registry.get_reader(source_uri)?;
    let content = reader.fetch(source_uri).await?;

    // Proceed to workflow generation...
}
```

---

## 2. Workflow Engine Integration (REQ-170)

### Existing System
**File:** `crates/radium-core/src/workflow/engine.rs`
**Status:** ✅ Implemented (Sequential Execution)

**What Exists:**
- Sequential workflow step execution
- Step result tracking and variable passing
- Execution context management
- Control flow evaluation (`crates/radium-core/src/workflow/control_flow.rs`)

**What's Needed:**
- Task dependency DAG construction
- Dynamic agent selection during execution (currently static)
- Adaptive workflow modification based on execution results
- True parallel execution (currently sequential due to trait limitations)

**Integration Point:**
```rust
// Extend WorkflowEngine for autonomous execution
impl WorkflowEngine {
    pub async fn execute_autonomous(&self, plan: &WorkflowPlan, policy: &PolicyEngine) -> Result<WorkflowResult> {
        let mut context = ExecutionContext::new();
        let mut checkpoint_manager = CheckpointManager::new();

        // Create initial checkpoint
        checkpoint_manager.create_checkpoint("workflow_start", &context)?;

        for task_group in plan.dependency_groups() {
            // Select agents dynamically
            let agents = self.agent_selector.select_for_tasks(&task_group, &context).await?;

            // Execute with error recovery
            match self.execute_task_group(task_group, agents, &mut context).await {
                Ok(results) => {
                    context.merge_results(results);
                    checkpoint_manager.create_checkpoint(&format!("group_{}", task_group.id), &context)?;
                }
                Err(e) => {
                    // Apply recovery strategy (REQ-168)
                    let recovery = self.recovery_strategy.handle_failure(&e, &context).await?;
                    self.apply_recovery(recovery, &mut context, &checkpoint_manager).await?;
                }
            }

            // Check for critical errors
            if let Some(critical) = self.resource_manager.check_critical_errors() {
                return Err(anyhow!("Critical error: {}", critical));
            }
        }

        Ok(WorkflowResult { context, steps_completed: plan.tasks.len() })
    }
}
```

---

## 3. PlanGenerator Integration (REQ-170)

### Existing System
**File:** `crates/radium-core/src/planning/mod.rs`
**Status:** ✅ Implemented (AI-powered planning)

**What Exists:**
- AI-powered plan generation from specifications
- Iteration structuring and task extraction
- Plan execution with task results
- Dependency analysis

**What's Needed:**
- Extended dependency tracking for workflow DAG
- Resource estimation (tokens, time, cost)
- Agent assignment based on task type analysis

**Integration Point:**
```rust
// Extend PlanGenerator for autonomous workflow creation
impl PlanGenerator {
    pub async fn generate_autonomous_workflow(&self, goal: &str, sources: Vec<String>) -> Result<WorkflowPlan> {
        // 1. Parse goal and source content
        let parsed_goals = self.parse_goals(goal, sources).await?;

        // 2. Generate task tree
        let tasks = self.generate_tasks(&parsed_goals).await?;

        // 3. Analyze dependencies
        let dependencies = self.analyze_dependencies(&tasks)?;

        // 4. Estimate resources
        let estimated_cost = self.estimate_cost(&tasks)?;

        // 5. Assign agents based on task types
        let task_assignments = self.assign_agents(&tasks, &dependencies)?;

        Ok(WorkflowPlan {
            tasks,
            dependencies,
            estimated_cost,
            agent_assignments: task_assignments,
        })
    }

    fn assign_agents(&self, tasks: &[Task], dependencies: &HashMap<TaskId, Vec<TaskId>>) -> Result<HashMap<TaskId, AgentId>> {
        // Analyze task types and match to agent capabilities
        // Use agent metadata system (crates/radium-core/src/agents/metadata.rs)
    }
}
```

---

## 4. Agent Selection & Load Balancing (REQ-166)

### Existing System
**Files:**
- `crates/radium-core/src/agents/metadata.rs` (Agent capabilities)
- `crates/radium-core/src/agents/registry.rs` (Agent discovery)
- `crates/radium-core/src/models/selector.rs` (Model selection with budget)

**Status:** ⚠️ Partially Implemented (Static selection)

**What Exists:**
- Agent metadata with capability declarations
- Model priority selection (Speed, Balanced, Quality, Premium)
- Cost estimation and budget tracking
- Agent registry with filtering/search

**What's Needed:**
- Dynamic task analysis → agent capability matching
- Performance-based agent selection (track success rates)
- Load balancing across concurrent agents
- Fallback chains for agent failures

**Integration Point:**
```rust
pub struct AgentSelector {
    registry: Arc<AgentRegistry>,
    model_selector: Arc<ModelSelector>,
    performance_tracker: Arc<PerformanceTracker>,
}

impl AgentSelector {
    pub async fn select_agent(&self, task: &Task, context: &SelectionContext) -> Result<AgentConfig> {
        // 1. Extract task requirements
        let requirements = self.analyze_task_requirements(task)?;

        // 2. Filter agents by capability match
        let capable_agents = self.registry
            .filter(|agent| self.matches_requirements(agent, &requirements))
            .collect::<Vec<_>>();

        // 3. Score candidates by: performance + cost + availability
        let mut scores = Vec::new();
        for agent in capable_agents {
            let performance_score = self.performance_tracker.get_score(&agent.id);
            let cost_score = self.model_selector.estimate_cost(&agent.model, &task.estimated_tokens)?;
            let availability_score = self.get_availability_score(&agent.id, context);

            scores.push((agent, performance_score * 0.4 + cost_score * 0.3 + availability_score * 0.3));
        }

        // 4. Select highest scoring agent
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 5. Apply fallback if primary unavailable
        for (agent, _score) in scores {
            if self.is_available(&agent).await? {
                return Ok(agent.clone());
            }
        }

        Err(anyhow!("No suitable agents available"))
    }
}
```

---

## 5. Policy Engine Integration (REQ-167)

### Existing System
**File:** `crates/radium-core/src/policy/types.rs`
**Status:** ✅ Implemented

**What Exists:**
- **YOLO Approval Mode** (Line 56): `ApprovalMode::Yolo` - auto-approves all actions
- Policy actions (Allow, Deny, AskUser)
- Priority-based rule system (Default, User, Admin)
- Policy decision with reason tracking

**What's Needed:**
- Integration with continuous execution loop
- Automatic decision logging for audit trail
- Override mechanism for user intervention

**Integration Point:**
```rust
// In continuous execution mode (REQ-167)
impl ContinuousExecutor {
    pub async fn execute_until_complete(&self, workflow: WorkflowPlan) -> Result<()> {
        // Apply YOLO mode policy
        let policy = PolicyEngine::with_mode(ApprovalMode::Yolo);

        loop {
            let next_step = workflow.next_pending_step()?;

            // Make autonomous decision
            let decision = policy.evaluate(&next_step)?;

            match decision.action {
                PolicyAction::Allow => {
                    // Log decision for audit
                    self.telemetry.log_decision(&decision);

                    // Execute step
                    let result = self.execute_step(&next_step).await?;
                    workflow.mark_complete(next_step.id, result)?;
                }
                PolicyAction::Deny => {
                    return Err(anyhow!("Policy denied step: {}", decision.reason));
                }
                PolicyAction::AskUser => {
                    // Should not happen in YOLO mode, but handle gracefully
                    tracing::warn!("YOLO mode unexpectedly asked user approval");
                    return Err(anyhow!("Execution halted for user input"));
                }
            }

            // Check completion
            if workflow.is_complete() {
                return Ok(());
            }

            // Check critical errors (REQ-168)
            if let Some(critical) = self.resource_manager.check_critical_errors() {
                return Err(anyhow!("Execution halted: {}", critical));
            }
        }
    }
}
```

---

## 6. Checkpoint & Recovery Integration (REQ-168)

### Existing System
**File:** `crates/radium-core/src/checkpoint/mod.rs`
**Status:** ✅ Implemented

**What Exists:**
- Workflow state snapshots
- Restoration capability
- Checkpoint behavior integration (`crates/radium-core/src/workflow/behaviors/checkpoint.rs`)

**What's Needed:**
- Automatic checkpoint creation (every N steps)
- Retry logic with exponential backoff
- Circuit breaker pattern for failing providers
- Fallback chain execution

**Integration Point:**
```rust
pub struct RecoveryManager {
    checkpoint_manager: Arc<CheckpointManager>,
    retry_config: RetryConfig,
    circuit_breakers: HashMap<ProviderId, CircuitBreaker>,
}

impl RecoveryManager {
    pub async fn handle_failure(&self, error: &ExecutionError, context: &mut WorkflowContext) -> RecoveryAction {
        match error {
            ExecutionError::TransientError(_) if context.retries < self.retry_config.max_retries => {
                // Retry with backoff
                let delay = Duration::from_millis(self.retry_config.base_delay_ms * 2_u64.pow(context.retries));
                RecoveryAction::Retry { attempt: context.retries + 1, delay }
            }

            ExecutionError::AgentFailure(agent_id) => {
                // Switch to fallback agent
                let fallback = self.select_fallback_agent(agent_id, context)?;
                RecoveryAction::SwitchAgent { fallback_agent: fallback }
            }

            ExecutionError::QuotaExhausted(provider_id) => {
                // Open circuit breaker and switch provider
                self.circuit_breakers.get_mut(provider_id).unwrap().open();
                let fallback = self.select_fallback_provider(provider_id, context)?;
                RecoveryAction::SwitchProvider { fallback_provider: fallback }
            }

            _ => {
                // Restore from last checkpoint
                let last_checkpoint = context.checkpoint_history.last().unwrap();
                RecoveryAction::RestoreCheckpoint { checkpoint_id: last_checkpoint.id.clone() }
            }
        }
    }
}
```

---

## 7. Telemetry & Learning Integration

### Existing Systems
**Files:**
- `crates/radium-core/src/monitoring/telemetry.rs` (Telemetry records)
- `crates/radium-core/src/learning/mod.rs` (Learning system)

**Status:** ⚠️ Learning system exists but not integrated into execution flow

**What Exists:**
- Telemetry tracking (token usage, execution times)
- Learning entry recording (mistakes, patterns)
- ACE skillbook management

**What's Needed:**
- Automatic pattern application during agent selection
- Performance-based agent scoring
- Learning-driven decision optimization

**Integration Point:**
```rust
// Integrate learning into agent selection
impl AgentSelector {
    pub async fn select_agent_with_learning(&self, task: &Task) -> Result<AgentConfig> {
        // 1. Check learning system for similar past tasks
        let past_executions = self.learning_system.find_similar_tasks(task).await?;

        // 2. Extract successful patterns
        let successful_agents = past_executions
            .iter()
            .filter(|e| e.success)
            .map(|e| e.agent_id.clone())
            .collect::<Vec<_>>();

        // 3. Bias selection towards historically successful agents
        let candidates = self.registry.filter(|a| successful_agents.contains(&a.id));

        // 4. Fall back to capability matching if no learning data
        if candidates.is_empty() {
            self.select_agent(task, &SelectionContext::default()).await
        } else {
            Ok(candidates.first().unwrap().clone())
        }
    }
}
```

---

## 8. Multi-Agent Coordination (REQ-171)

### Existing Systems
**Files:**
- `crates/radium-orchestrator/src/queue.rs` (Task queue)
- `crates/radium-core/src/workspace/mod.rs` (Shared workspace)

**Status:** ❌ Not Implemented

**What Exists:**
- Priority-based task queue
- Shared workspace structure (.radium directory)

**What's Needed:**
- **NEW:** Message bus for agent-to-agent communication
- **NEW:** Task delegation protocol
- **NEW:** Conflict resolution mechanism
- **NEW:** Progress synchronization

**Integration Point:**
```rust
// New system required
pub struct AgentCoordinator {
    message_bus: Arc<MessageBus>,
    shared_workspace: Arc<Workspace>,
    task_queue: Arc<Mutex<TaskQueue>>,
}

impl AgentCoordinator {
    pub async fn delegate_subtask(&self, from: AgentId, to: AgentId, task: SubTask) -> Result<TaskHandle> {
        // 1. Validate target agent capabilities
        let target_agent = self.agent_registry.get(&to)?;
        if !target_agent.has_capability(&task.required_capability) {
            return Err(anyhow!("Agent {} lacks capability: {}", to, task.required_capability));
        }

        // 2. Add to task queue with dependency
        let handle = self.task_queue.lock().await.enqueue(Task {
            id: Uuid::new_v4(),
            agent_id: to,
            parent_task: Some(from),
            priority: task.priority,
            payload: task.payload,
        })?;

        // 3. Publish message to notify target agent
        self.message_bus.publish(AgentMessage {
            from,
            to,
            message_type: MessageType::TaskDelegation,
            payload: serde_json::to_value(&task)?,
        }).await?;

        Ok(handle)
    }
}
```

---

## Summary: Integration Checklist

| Component | Exists? | Needs Extension? | REQ |
|-----------|---------|------------------|-----|
| Source Verification | ⚠️ Partial | ✅ Yes | REQ-165 |
| Workflow Engine | ✅ Yes | ✅ Yes (DAG, parallel) | REQ-170 |
| Plan Generator | ✅ Yes | ✅ Yes (dependencies) | REQ-170 |
| Agent Selection | ⚠️ Partial | ✅ Yes (dynamic) | REQ-166 |
| Policy Engine (YOLO) | ✅ Yes | ❌ No | REQ-167 |
| Checkpoint/Recovery | ✅ Yes | ✅ Yes (retry logic) | REQ-168 |
| Telemetry | ✅ Yes | ❌ No | N/A |
| Learning System | ⚠️ Exists | ✅ Yes (integration) | REQ-170 |
| Multi-Agent Coordination | ❌ No | ✅ Yes (build new) | REQ-171 |

---

## Next Steps

1. **Complete REQ-164** (Test Coverage) for stable foundation
2. **Implement REQ-165** (Source Verification) - already 17% complete
3. **Extend REQ-170** (Workflow Decomposition) to integrate plan generator with workflow engine
4. **Build REQ-166** (Agent Selection) with dynamic capability matching
5. **Implement REQ-168** (Error Recovery) with retry/circuit breaker patterns
6. **Integrate REQ-167** (YOLO Mode) continuous execution loop
7. **Build REQ-171** (Multi-Agent Coordination) message bus and delegation
8. **Finalize REQ-169** (CLI Command) to tie everything together
