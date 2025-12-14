# Testing Action Plan: Phase 7 Advanced Features

**Owner**: Development Team
**Timeline**: 4 weeks
**Priority**: CRITICAL (blocks production release)
**Status**: Ready to implement

---

## Quick Reference

- **Total New Tests**: 80-100
- **Estimated Effort**: 60-80 hours
- **Current Tests**: 444 passing
- **Target Tests**: 524-544 passing
- **Risk Reduction**: MEDIUM-HIGH → LOW

---

## Week 1: Foundation & Child Workflows (Priority 1)

**Goal**: Set up test infrastructure and verify child workflow TypeScript generation
**Tests to Add**: 25-30
**Estimated Hours**: 16-20

### Day 1-2: Test Infrastructure Setup

#### Task 1.1: Create Test Fixtures Directory
```bash
mkdir -p crates/radium-workflow/tests/fixtures
```

#### Task 1.2: Create TypeScript Compiler Helper
**File**: `tests/fixtures/typescript_compiler.rs`

```rust
use std::process::Command;
use std::fs;
use tempfile::TempDir;

/// Check if Node.js and npm are available
pub fn check_node_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Verify that TypeScript code compiles with strict settings
pub fn verify_typescript_compiles(ts_code: &str) -> Result<(), String> {
    if !check_node_available() {
        eprintln!("⚠️  Skipping TypeScript compilation (npm not available)");
        return Ok(());
    }

    let temp_dir = TempDir::new().map_err(|e| e.to_string())?;

    // Write TypeScript file
    let ts_file = temp_dir.path().join("test.ts");
    fs::write(&ts_file, ts_code).map_err(|e| e.to_string())?;

    // Write strict tsconfig.json
    let tsconfig = r#"{
      "compilerOptions": {
        "target": "ES2020",
        "module": "commonjs",
        "strict": true,
        "noImplicitAny": true,
        "strictNullChecks": true,
        "strictFunctionTypes": true,
        "noUnusedLocals": true,
        "noUnusedParameters": true,
        "noImplicitReturns": true,
        "skipLibCheck": false,
        "types": ["node"]
      }
    }"#;
    fs::write(temp_dir.path().join("tsconfig.json"), tsconfig)
        .map_err(|e| e.to_string())?;

    // Write minimal package.json with Temporal types
    let package_json = r#"{
      "name": "test",
      "version": "1.0.0",
      "dependencies": {
        "@temporalio/workflow": "^1.10.0",
        "@temporalio/activity": "^1.10.0"
      },
      "devDependencies": {
        "typescript": "^5.3.0",
        "@types/node": "^20.0.0"
      }
    }"#;
    fs::write(temp_dir.path().join("package.json"), package_json)
        .map_err(|e| e.to_string())?;

    // Run npm install (quiet mode)
    let install = Command::new("npm")
        .arg("install")
        .arg("--silent")
        .current_dir(temp_dir.path())
        .output()
        .map_err(|e| e.to_string())?;

    if !install.status.success() {
        return Err(format!(
            "npm install failed:\n{}",
            String::from_utf8_lossy(&install.stderr)
        ));
    }

    // Run TypeScript compiler
    let output = Command::new("npx")
        .arg("tsc")
        .arg("--noEmit")
        .arg("test.ts")
        .current_dir(temp_dir.path())
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "TypeScript compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Verify TypeScript code with Temporal SDK imports
pub fn verify_typescript_with_temporal(ts_code: &str) -> Result<(), String> {
    let full_code = format!(
        r#"
import {{ proxyActivities, executeChild, defineSignal, defineQuery, setHandler, CancellationScope, isCancellation, sleep }} from '@temporalio/workflow';

{}
"#,
        ts_code
    );

    verify_typescript_compiles(&full_code)
}
```

**Acceptance**: Helper compiles TS code successfully, skips gracefully if npm unavailable

---

#### Task 1.3: Create Workflow Fixtures
**File**: `tests/fixtures/workflows.rs`

```rust
use radium_workflow::schema::{
    NodeData, NodeType, Position, WorkflowDefinition, WorkflowEdge, WorkflowNode,
};

/// Minimal workflow with trigger and end
pub fn minimal_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        id: "test-minimal".to_string(),
        name: "Minimal".to_string(),
        nodes: vec![
            WorkflowNode {
                id: "trigger".to_string(),
                node_type: NodeType::Trigger,
                data: NodeData {
                    label: "Start".to_string(),
                    ..Default::default()
                },
                position: Position::default(),
            },
            WorkflowNode {
                id: "end".to_string(),
                node_type: NodeType::End,
                data: NodeData {
                    label: "End".to_string(),
                    ..Default::default()
                },
                position: Position::default(),
            },
        ],
        edges: vec![WorkflowEdge::new("e1", "trigger", "end")],
        variables: vec![],
        settings: Default::default(),
    }
}
```

**Acceptance**: Fixture compiles and can be used in tests

---

### Day 3-5: Child Workflow TypeScript Generation Tests

#### Task 1.4: Create Advanced TypeScript Generation Test File
**File**: `tests/advanced_typescript_gen.rs`

**Test Suite 1: Child Workflow ID Strategies** (4 tests)

```rust
use radium_workflow::schema::advanced::child_orchestration::*;
use serde_json::json;

mod fixtures;
use fixtures::typescript_compiler::verify_typescript_with_temporal;

#[test]
fn test_child_workflow_uuid_strategy_typescript() {
    let config = ChildWorkflowOrchestration::new("ProcessOrder")
        .with_task_queue("orders");

    let ts = config.to_typescript();

    // Verify TS contains UUID generation
    assert!(
        ts.contains("uuid4()") || ts.contains("uuid.v4()"),
        "UUID strategy must generate unique ID: {}",
        ts
    );
    assert!(ts.contains("executeChild"));
    assert!(ts.contains("'ProcessOrder'"));

    // Verify compiles
    verify_typescript_with_temporal(&ts)
        .expect("Child workflow UUID strategy TS should compile");
}

#[test]
fn test_child_workflow_explicit_id_typescript() {
    let config = ChildWorkflowOrchestration::new("ProcessOrder")
        .with_workflow_id("order-12345");

    let ts = config.to_typescript();

    // Verify explicit ID is used
    assert!(
        ts.contains("workflowId: 'order-12345'"),
        "Explicit ID must be in generated TS: {}",
        ts
    );
    assert!(ts.contains("executeChild"));

    verify_typescript_with_temporal(&ts)
        .expect("Child workflow explicit ID TS should compile");
}

#[test]
fn test_child_workflow_parent_suffix_strategy_typescript() {
    let config = ChildWorkflowOrchestration::new("SubWorkflow")
        .with_parent_suffix();

    let ts = config.to_typescript();

    // Verify parent workflow ID is referenced
    assert!(
        ts.contains("workflowInfo().workflowId"),
        "Parent suffix must use workflowInfo(): {}",
        ts
    );
    assert!(
        ts.contains("-child-"),
        "Parent suffix must contain '-child-': {}",
        ts
    );

    verify_typescript_with_temporal(&ts)
        .expect("Child workflow parent suffix TS should compile");
}

#[test]
fn test_child_workflow_pattern_strategy_typescript() {
    let config = ChildWorkflowOrchestration::new("SubWorkflow")
        .with_id_pattern("child-{parent_id}-{index}-{timestamp}");

    let ts = config.to_typescript();

    // Verify pattern replacement
    assert!(
        ts.contains("${workflowInfo().workflowId}"),
        "Pattern must replace {{parent_id}}: {}",
        ts
    );
    assert!(
        ts.contains("${childIndex++}") || ts.contains("index"),
        "Pattern must replace {{index}}: {}",
        ts
    );
    assert!(
        ts.contains("${Date.now()}") || ts.contains("timestamp"),
        "Pattern must replace {{timestamp}}: {}",
        ts
    );

    verify_typescript_with_temporal(&ts)
        .expect("Child workflow pattern strategy TS should compile");
}
```

**Acceptance**: 4 tests pass, TS compiles with Temporal imports

---

**Test Suite 2: Child Workflow Options** (6 tests)

```rust
#[test]
fn test_child_workflow_search_attributes_all_types() {
    use chrono::Utc;

    let config = ChildWorkflowOrchestration::new("Test")
        .with_search_attribute("StringAttr", SearchAttributeValue::String("test".into()))
        .with_search_attribute("IntAttr", SearchAttributeValue::Int(42))
        .with_search_attribute("DoubleAttr", SearchAttributeValue::Double(3.14))
        .with_search_attribute("BoolAttr", SearchAttributeValue::Bool(true))
        .with_search_attribute("DatetimeAttr", SearchAttributeValue::Datetime(Utc::now()))
        .with_search_attribute("ArrayAttr", SearchAttributeValue::StringArray(vec!["a".into(), "b".into()]));

    let ts = config.to_typescript();

    // Verify all attribute types are present
    assert!(ts.contains("searchAttributes"));
    assert!(ts.contains("StringAttr"));
    assert!(ts.contains("IntAttr"));
    assert!(ts.contains("42"));
    assert!(ts.contains("3.14"));
    assert!(ts.contains("true"));

    verify_typescript_with_temporal(&ts)
        .expect("Search attributes TS should compile");
}

#[test]
fn test_child_workflow_timeouts_typescript() {
    let config = ChildWorkflowOrchestration::new("Test")
        .with_execution_timeout(300000)
        .with_run_timeout(60000)
        .with_task_timeout(5000);

    let ts = config.to_typescript();

    assert!(ts.contains("workflowExecutionTimeout"));
    assert!(ts.contains("300000ms"));
    assert!(ts.contains("workflowRunTimeout"));
    assert!(ts.contains("60000ms"));
    assert!(ts.contains("workflowTaskTimeout"));
    assert!(ts.contains("5000ms"));

    verify_typescript_with_temporal(&ts)
        .expect("Timeouts TS should compile");
}

#[test]
fn test_child_workflow_retry_policy_typescript() {
    use radium_workflow::schema::components::RetryConfig;

    let retry = RetryConfig {
        max_attempts: 5,
        initial_interval_ms: 1000,
        max_interval_ms: 10000,
        backoff_coefficient: 2.0,
        ..Default::default()
    };

    let config = ChildWorkflowOrchestration::new("Test")
        .with_retry_policy(retry);

    let ts = config.to_typescript();

    assert!(ts.contains("maximumAttempts: 5"));
    assert!(ts.contains("initialInterval: '1000ms'"));
    assert!(ts.contains("backoffCoefficient: 2"));

    verify_typescript_with_temporal(&ts)
        .expect("Retry policy TS should compile");
}

#[test]
fn test_child_workflow_fire_and_forget_typescript() {
    let config = ChildWorkflowOrchestration::new("BackgroundJob")
        .fire_and_forget();

    let ts = config.to_typescript();

    // Should NOT await result
    assert!(
        !ts.contains("await childHandle.result()"),
        "Fire-and-forget should not await result: {}",
        ts
    );
    assert!(ts.contains("Fire and forget") || ts.contains("not awaiting"));

    verify_typescript_with_temporal(&ts)
        .expect("Fire-and-forget TS should compile");
}

#[test]
fn test_child_workflow_cancellation_types_typescript() {
    let types = [
        (CancellationType::WaitCancellationCompleted, "WAIT_CANCELLATION_COMPLETED"),
        (CancellationType::TryCancel, "TRY_CANCEL"),
        (CancellationType::Abandon, "ABANDON"),
    ];

    for (cancel_type, expected_const) in types {
        let config = ChildWorkflowOrchestration::new("Test")
            .with_cancellation_type(cancel_type);

        let ts = config.to_typescript();

        assert!(
            ts.contains(expected_const),
            "Cancellation type {:?} must generate {}: {}",
            cancel_type, expected_const, ts
        );

        verify_typescript_with_temporal(&ts)
            .expect(&format!("Cancellation type {:?} TS should compile", cancel_type));
    }
}

#[test]
fn test_child_workflow_parent_close_policies_typescript() {
    use radium_workflow::schema::components::ParentClosePolicy;

    let policies = [
        (ParentClosePolicy::Terminate, "PARENT_CLOSE_POLICY_TERMINATE"),
        (ParentClosePolicy::Abandon, "PARENT_CLOSE_POLICY_ABANDON"),
        (ParentClosePolicy::RequestCancel, "PARENT_CLOSE_POLICY_REQUEST_CANCEL"),
    ];

    for (policy, expected_const) in policies {
        let config = ChildWorkflowOrchestration::new("Test")
            .with_parent_close_policy(policy);

        let ts = config.to_typescript();

        assert!(
            ts.contains(expected_const),
            "Parent close policy {:?} must generate {}: {}",
            policy, expected_const, ts
        );

        verify_typescript_with_temporal(&ts)
            .expect(&format!("Parent close policy {:?} TS should compile", policy));
    }
}
```

**Acceptance**: 6 more tests pass, all child workflow options verified

---

**Test Suite 3: Child Workflow Edge Cases** (3 tests)

```rust
#[test]
fn test_child_workflow_memo_and_input_typescript() {
    let config = ChildWorkflowOrchestration::new("Test")
        .with_input("orderId", json!("order-123"))
        .with_input("customerId", json!("cust-456"))
        .with_memo("reason", "customer request")
        .with_memo("priority", "high");

    let ts = config.to_typescript();

    // Verify input object
    assert!(ts.contains("orderId"));
    assert!(ts.contains("order-123"));

    // Verify memo object
    assert!(ts.contains("memo"));
    assert!(ts.contains("reason"));
    assert!(ts.contains("customer request"));

    verify_typescript_with_temporal(&ts)
        .expect("Input and memo TS should compile");
}

#[test]
fn test_child_workflow_cron_schedule_typescript() {
    let config = ChildWorkflowOrchestration::new("RecurringJob")
        .with_cron_schedule("0 */12 * * *");

    let ts = config.to_typescript();

    assert!(ts.contains("cronSchedule"));
    assert!(ts.contains("0 */12 * * *"));

    verify_typescript_with_temporal(&ts)
        .expect("Cron schedule TS should compile");
}

#[test]
fn test_child_workflow_complete_example_typescript() {
    // Kitchen sink test with all options
    let config = ChildWorkflowOrchestration::new("CompleteExample")
        .with_workflow_id("example-123")
        .with_task_queue("example-queue")
        .with_input("key", json!("value"))
        .with_execution_timeout(300000)
        .with_run_timeout(60000)
        .with_search_attribute("CustomAttr", SearchAttributeValue::String("test".into()))
        .with_memo("note", "test")
        .with_retry_policy(RetryConfig::default());

    let ts = config.to_typescript();

    // Should have all components
    assert!(ts.contains("executeChild"));
    assert!(ts.contains("workflowId: 'example-123'"));
    assert!(ts.contains("taskQueue: 'example-queue'"));
    assert!(ts.contains("workflowExecutionTimeout"));
    assert!(ts.contains("searchAttributes"));
    assert!(ts.contains("memo"));
    assert!(ts.contains("retry"));

    verify_typescript_with_temporal(&ts)
        .expect("Complete example TS should compile");
}
```

**Acceptance**: 3 edge case tests pass

---

**Test Suite 4: Serialization Round-Trips** (2 tests)

```rust
#[test]
fn test_child_workflow_orchestration_serialization_roundtrip() {
    let original = ChildWorkflowOrchestration::new("ProcessOrder")
        .with_workflow_id("order-123")
        .with_task_queue("orders")
        .with_input("orderId", json!("123"))
        .with_search_attribute("CustomerId", SearchAttributeValue::String("cust-456".into()))
        .with_execution_timeout(60000);

    let json = serde_json::to_string(&original)
        .expect("Should serialize");
    let restored: ChildWorkflowOrchestration = serde_json::from_str(&json)
        .expect("Should deserialize");

    assert_eq!(original.workflow_type, restored.workflow_type);
    assert_eq!(original.workflow_id, restored.workflow_id);
    assert_eq!(original.task_queue, restored.task_queue);
    assert_eq!(original.execution_timeout_ms, restored.execution_timeout_ms);
    assert_eq!(original.search_attributes.len(), restored.search_attributes.len());
}

#[test]
fn test_search_attribute_value_serialization_roundtrip() {
    use chrono::Utc;

    let values = vec![
        SearchAttributeValue::String("test".into()),
        SearchAttributeValue::Int(42),
        SearchAttributeValue::Double(3.14),
        SearchAttributeValue::Bool(true),
        SearchAttributeValue::Datetime(Utc::now()),
        SearchAttributeValue::StringArray(vec!["a".into(), "b".into()]),
    ];

    for value in values {
        let json = serde_json::to_string(&value).expect("Should serialize");
        let restored: SearchAttributeValue = serde_json::from_str(&json).expect("Should deserialize");

        // Compare string representations (Datetime precision might differ)
        assert_eq!(
            format!("{:?}", value),
            format!("{:?}", restored),
            "Round-trip failed for: {:?}",
            value
        );
    }
}
```

**Acceptance**: Serialization tests pass

---

### Week 1 Deliverables

- [ ] Test fixtures infrastructure (`tests/fixtures/`)
- [ ] TypeScript compilation helper working
- [ ] 15+ child workflow TypeScript generation tests
- [ ] 2+ serialization round-trip tests
- [ ] All tests green in CI
- [ ] Documentation updated

**Total Tests Added**: 17-20
**Running Total**: 461-464 tests

---

## Week 2: Signals & Queries (Priority 1)

**Goal**: Verify signal and query TypeScript generation
**Tests to Add**: 20-25
**Estimated Hours**: 16-20

### Signal Handler TypeScript Generation (12 tests)

```rust
// Add to tests/advanced_typescript_gen.rs

mod signal_tests {
    use super::*;
    use radium_workflow::schema::advanced::signals::*;

    #[test]
    fn test_signal_with_typed_payload_typescript() {
        let schema = SignalSchema::with_fields(vec![
            SignalSchemaField {
                name: "approved".to_string(),
                typescript_type: "boolean".to_string(),
                required: true,
                description: Some("Approval decision".to_string()),
                default: None,
            },
            SignalSchemaField {
                name: "approver".to_string(),
                typescript_type: "string".to_string(),
                required: true,
                description: None,
                default: None,
            },
        ]);

        let signal = SignalDefinition::new("approveOrder")
            .with_input_schema(schema);

        let ts = signal.to_typescript_definition();

        // Verify interface
        assert!(ts.contains("interface ApproveOrderPayload"));
        assert!(ts.contains("approved: boolean"));
        assert!(ts.contains("approver: string"));

        // Verify signal definition
        assert!(ts.contains("defineSignal<ApproveOrderPayload>('approveOrder')"));

        verify_typescript_with_temporal(&ts)
            .expect("Signal with typed payload should compile");
    }

    #[test]
    fn test_signal_handler_variable_updates_all_sources() {
        let signal = SignalDefinition::new("updateState");
        let handler = SignalHandler::new("updateState")
            .with_update(VariableUpdate::new(
                "status",
                VariableSource::from_payload("newStatus")
            ))
            .with_update(VariableUpdate::new(
                "timestamp",
                VariableSource::from_expression("Date.now()")
            ))
            .with_update(VariableUpdate::new(
                "isActive",
                VariableSource::from_constant(json!(true))
            ));

        let ts = handler.to_typescript(&signal);

        assert!(ts.contains("state.variables.status = payload.newStatus"));
        assert!(ts.contains("state.variables.timestamp = Date.now()"));
        assert!(ts.contains("state.variables.isActive = true"));

        verify_typescript_with_temporal(&ts)
            .expect("Variable updates should compile");
    }

    // Add 10 more signal tests following the pattern in TESTING_GAP_ANALYSIS.md
    // ...
}
```

**Acceptance**: 12 signal tests pass

### Query Handler TypeScript Generation (8 tests)

```rust
mod query_tests {
    use super::*;
    use radium_workflow::schema::advanced::queries::*;

    #[test]
    fn test_query_state_projection_all_variables() {
        let query = QueryDefinition::new(
            "getState",
            QuerySchema::any(),
            QueryHandlerLogic::project(vec!["*"])
        );

        let ts = query.to_typescript();

        assert!(ts.contains("...state.variables"));
        verify_typescript_with_temporal(&ts)
            .expect("State projection should compile");
    }

    #[test]
    fn test_query_computed_expression() {
        let query = QueryDefinition::new(
            "getProgress",
            QuerySchema::object(vec![("percent", "number")]),
            QueryHandlerLogic::computed("(state.completedSteps / state.totalSteps) * 100")
        );

        let ts = query.to_typescript();

        assert!(ts.contains("return (state.completedSteps / state.totalSteps) * 100"));
        verify_typescript_with_temporal(&ts)
            .expect("Computed query should compile");
    }

    #[test]
    fn test_standard_queries_typescript() {
        let queries = WorkflowQueries::with_standard_queries();
        let ts = queries.to_typescript();

        // All standard queries must be present
        assert!(ts.contains("getStateQuery"));
        assert!(ts.contains("getProgressQuery"));
        assert!(ts.contains("getStatusQuery"));

        verify_typescript_with_temporal(&ts)
            .expect("Standard queries should compile");
    }

    // Add 5 more query tests
    // ...
}
```

**Acceptance**: 8 query tests pass

### Week 2 Deliverables

- [ ] 12+ signal handler tests
- [ ] 8+ query handler tests
- [ ] Serialization round-trips for signals/queries
- [ ] All tests green

**Total Tests Added**: 20-25
**Running Total**: 481-489 tests

---

## Week 3: Cancellation & Patterns (Priority 1)

**Goal**: Verify cancellation and pattern TypeScript generation
**Tests to Add**: 25-30
**Estimated Hours**: 20-24

### Cancellation Scope Tests (8 tests)

```rust
mod cancellation_tests {
    use super::*;
    use radium_workflow::schema::advanced::cancellation::*;

    #[test]
    fn test_cancellable_scope_with_cleanup_typescript() {
        let cleanup = CleanupConfig::new()
            .with_activity(CleanupActivity::new("releaseResources"))
            .with_state_update(StateUpdate::new("status", json!("cancelled")));

        let scope = CancellationScope::new("orderProcessing")
            .with_cleanup(cleanup)
            .with_cleanup_timeout(30000);

        let ts = scope.to_typescript("await processOrder();");

        // Verify structure
        assert!(ts.contains("try {"));
        assert!(ts.contains("isCancellation(err)"));
        assert!(ts.contains("CancellationScope.nonCancellable"));
        assert!(ts.contains("activities.releaseResources"));

        verify_typescript_with_temporal(&ts)
            .expect("Cancellation scope should compile");
    }

    // Add 7 more cancellation tests
    // ...
}
```

### Saga Pattern Tests (10 tests)

```rust
mod saga_tests {
    use super::*;
    use radium_workflow::schema::patterns::saga::*;

    #[test]
    fn test_saga_with_compensation_reverse_order() {
        let saga = SagaDefinition::new("orderSaga")
            .with_step(
                SagaStep::new("reserveInventory", SagaAction::Activity {
                    activity_name: "reserveInventory".to_string(),
                    input: json!({}),
                })
                .with_compensation(SagaAction::Activity {
                    activity_name: "releaseInventory".to_string(),
                    input: json!({}),
                })
            )
            .with_step(
                SagaStep::new("chargePayment", SagaAction::Activity {
                    activity_name: "chargePayment".to_string(),
                    input: json!({}),
                })
                .with_compensation(SagaAction::Activity {
                    activity_name: "refundPayment".to_string(),
                    input: json!({}),
                })
            );

        let ts = saga.to_typescript();

        // Verify forward execution
        assert!(ts.contains("reserveInventory"));
        assert!(ts.contains("chargePayment"));

        // Verify reverse compensation
        assert!(ts.contains("[...context.completedSteps].reverse()"));
        assert!(ts.contains("releaseInventory"));
        assert!(ts.contains("refundPayment"));

        verify_typescript_with_temporal(&ts)
            .expect("Saga should compile");
    }

    // Add 9 more saga tests
    // ...
}
```

### Scatter-Gather Tests (7 tests)

```rust
mod scatter_gather_tests {
    use super::*;
    use radium_workflow::schema::patterns::scatter_gather::*;

    #[test]
    fn test_scatter_gather_with_timeout() {
        // Implementation as shown in TESTING_GAP_ANALYSIS.md
    }

    // Add 6 more scatter-gather tests
    // ...
}
```

### Week 3 Deliverables

- [ ] 8+ cancellation tests
- [ ] 10+ saga tests
- [ ] 7+ scatter-gather tests
- [ ] All tests green

**Total Tests Added**: 25-30
**Running Total**: 506-519 tests

---

## Week 4: Integration & Polish (Priority 2)

**Goal**: Add Temporal integration tests and polish
**Tests to Add**: 15-20
**Estimated Hours**: 16-24

### Temporal Integration Setup

**File**: `tests/fixtures/temporal_setup.rs`

```rust
use std::process::{Command, Child};

pub struct TemporalTestServer {
    process: Child,
}

impl TemporalTestServer {
    pub fn start() -> Option<Self> {
        // Check if temporal CLI is available
        let available = Command::new("temporal")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !available {
            eprintln!("⚠️  Temporal CLI not available, skipping integration test");
            return None;
        }

        // Start test server
        let process = Command::new("temporal")
            .arg("server")
            .arg("start-dev")
            .arg("--headless")
            .spawn()
            .ok()?;

        // Wait for server to be ready
        std::thread::sleep(std::time::Duration::from_secs(5));

        Some(Self { process })
    }
}

impl Drop for TemporalTestServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}
```

### Integration Tests (3-5 tests)

**File**: `tests/temporal_integration.rs`

```rust
#![cfg(feature = "integration-tests")]

mod fixtures;
use fixtures::temporal_setup::TemporalTestServer;

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_child_workflow_execution_integration() {
    let _server = match TemporalTestServer::start() {
        Some(s) => s,
        None => return, // Skip if Temporal not available
    };

    // TODO: Implement actual workflow execution test
    // This requires:
    // 1. Generate workflow with child
    // 2. Compile TypeScript
    // 3. Start worker
    // 4. Execute workflow
    // 5. Verify child was created
}

// Add 2-4 more integration tests
```

### Property-Based Tests (5-8 tests)

Add `proptest` to dev-dependencies:

```toml
[dev-dependencies]
proptest = "1.4"
```

**File**: `tests/property_tests.rs`

```rust
use proptest::prelude::*;
use radium_workflow::schema::advanced::child_orchestration::*;

proptest! {
    #[test]
    fn test_child_workflow_timeouts_always_valid(
        execution_timeout in 0u64..1000000u64,
        run_timeout in 0u64..1000000u64,
    ) {
        let config = ChildWorkflowOrchestration::new("Test")
            .with_execution_timeout(execution_timeout)
            .with_run_timeout(run_timeout);

        let result = config.validate_config();

        // If run > execution, should fail
        if run_timeout > execution_timeout {
            assert!(result.is_err());
        }
    }

    // Add 4-7 more property tests
}
```

### Week 4 Deliverables

- [ ] Temporal integration infrastructure
- [ ] 3-5 integration tests (ignored by default)
- [ ] 5-8 property-based tests
- [ ] Documentation for running integration tests
- [ ] CI configuration updated

**Total Tests Added**: 8-13
**Running Total**: 514-532 tests

---

## CI/CD Integration

### GitHub Actions Workflow

**File**: `.github/workflows/radium-workflow.yml`

```yaml
name: Radium Workflow Tests

on:
  pull_request:
    paths:
      - 'crates/radium-workflow/**'
  push:
    branches:
      - main

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run unit tests
        run: cargo test --manifest-path crates/radium-workflow/Cargo.toml --lib

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Run integration tests
        run: cargo test --manifest-path crates/radium-workflow/Cargo.toml --test '*'
        env:
          NODE_ENV: test

  typescript-compilation:
    runs-on: ubuntu-latest
    needs: unit-tests
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Run TypeScript generation tests
        run: cargo test --manifest-path crates/radium-workflow/Cargo.toml --test advanced_typescript_gen
        env:
          NODE_ENV: test
```

---

## Success Metrics & Validation

### After Week 1
- [ ] Can verify TypeScript compilation
- [ ] 17+ new tests passing
- [ ] CI runs successfully

### After Week 2
- [ ] Signal/query generation verified
- [ ] 37+ new tests passing
- [ ] No test flakiness

### After Week 3
- [ ] All Phase 7 features have TS compilation tests
- [ ] 62+ new tests passing
- [ ] Pattern generation verified

### After Week 4
- [ ] Integration tests documented
- [ ] Property-based tests finding edge cases
- [ ] 70-80+ new tests passing
- [ ] 514-524 total tests

### Final Acceptance Criteria
- [ ] No regressions in existing 444 tests
- [ ] TypeScript compilation verified for all features
- [ ] All Priority 1 gaps closed
- [ ] Documentation updated
- [ ] CI passing consistently

---

## Tracking Progress

Use this checklist to track implementation:

### Week 1: Foundation
- [ ] Day 1: Test fixtures setup
- [ ] Day 2: TypeScript compiler helper
- [ ] Day 3: Child workflow ID strategies (4 tests)
- [ ] Day 4: Child workflow options (6 tests)
- [ ] Day 5: Edge cases + serialization (5 tests)

### Week 2: Signals & Queries
- [ ] Day 1: Signal schema tests (4 tests)
- [ ] Day 2: Signal handler tests (6 tests)
- [ ] Day 3: Signal edge cases (2 tests)
- [ ] Day 4: Query tests (6 tests)
- [ ] Day 5: Query edge cases (2 tests)

### Week 3: Cancellation & Patterns
- [ ] Day 1: Cancellation scope tests (5 tests)
- [ ] Day 2: Cancellation edge cases (3 tests)
- [ ] Day 3: Saga tests (7 tests)
- [ ] Day 4: Scatter-gather tests (5 tests)
- [ ] Day 5: Pattern edge cases (5 tests)

### Week 4: Integration & Polish
- [ ] Day 1: Temporal setup infrastructure
- [ ] Day 2: Integration tests (3 tests)
- [ ] Day 3: Property-based tests (5 tests)
- [ ] Day 4: Documentation + CI
- [ ] Day 5: Final validation + release prep

---

## Conclusion

This action plan provides:
- **Clear priorities**: Week-by-week breakdown
- **Specific tests**: Copy-paste ready test implementations
- **Incremental progress**: Deliverables each week
- **Quality gates**: CI integration and validation

**Start Date**: [YYYY-MM-DD]
**Target Completion**: 4 weeks from start
**Owner**: Development Team
**Review Cadence**: End of each week

**Next Step**: Begin Week 1, Task 1.1 - Create test fixtures directory
