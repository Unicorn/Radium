# Radium Workflow Testing Strategy

**Status**: Phase 7 Advanced Features - Comprehensive Testing Required
**Last Updated**: 2025-12-14
**Criticality**: MISSION CRITICAL - This is code generation infrastructure

---

## Executive Summary

The radium-workflow crate is **the foundation** of the entire workflow builder system. It:
- Generates TypeScript code for Temporal workflows
- Validates workflow definitions and data flow
- Implements advanced Temporal patterns (sagas, scatter-gather, signals, queries)
- Manages state, variables, and type safety

**Current State**: 444 tests passing, good unit test coverage in existing modules
**Phase 7 Gap**: Advanced features (child orchestration, signals, queries, cancellation, patterns) have unit tests but lack integration, TypeScript generation, and Temporal SDK integration testing

**Testing Philosophy**:
- Customer experience comes first - broken generated code == broken workflows in production
- Developer sanity matters - tests must be fast, deterministic, and provide clear signal
- Truth over fantasy - if generated code doesn't compile or behaves incorrectly, tests MUST catch it

---

## Testing Pyramid for radium-workflow

### Base Layer: Fast, Deterministic Checks (< 10s total)

**What**: Validate the Rust code itself and basic serialization
**Coverage**:
- ✅ **Linting**: `cargo clippy -- -D warnings` (already enforced via workspace lints)
- ✅ **Formatting**: `cargo fmt --check` (already enforced)
- ✅ **Type Checking**: `cargo check` (implicit in build)
- ✅ **Unit Tests**: 444 existing tests for Rust structs, builders, validation logic
  - Serialization round-trips (JSON ↔ Rust)
  - Builder patterns and fluent APIs
  - Validation logic (constraints, required fields, timeouts)
  - Enum TypeScript conversions

**Gaps Identified**:
- ❌ **Property-based testing** for schema validation edge cases
- ❌ **Constraint validation** stress testing (extreme values, boundary conditions)
- ⚠️ **Error message quality** - errors should be actionable, not cryptic

**Action Items**:
1. Add `proptest` for fuzz testing schema validation
2. Test error messages contain: field name, expected value, actual value, how to fix
3. Add performance benchmarks for validation (workflows with 100+ nodes)

---

### Middle Layer: Integration & TypeScript Generation (< 60s total)

**What**: Verify generated TypeScript compiles and matches Temporal SDK expectations
**Coverage**:
- ✅ **TypeScript Generation Tests**: Basic workflow, activities, worker code generation
- ❌ **Advanced Feature TypeScript**: Child workflows, signals, queries, cancellation
- ❌ **TypeScript Compilation**: tsc verification with strict mode
- ❌ **Type Safety Verification**: No `any` types, proper inference, strict null checks

**Critical Tests Needed**:

#### 1. Child Workflow Orchestration TypeScript Generation
```rust
#[test]
fn test_child_workflow_typescript_all_strategies() {
    // Test each WorkflowIdStrategy generates correct TS
    let strategies = [
        WorkflowIdStrategy::Uuid,
        WorkflowIdStrategy::Explicit,
        WorkflowIdStrategy::ParentSuffix,
        WorkflowIdStrategy::Pattern,
    ];

    for strategy in strategies {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_id_strategy(strategy);
        let ts = config.to_typescript();

        // Verify TS compiles
        verify_typescript_compiles(&ts);

        // Verify correct Temporal SDK imports
        assert!(ts.contains("executeChild"));
        assert!(ts.contains("from '@temporalio/workflow'"));
    }
}

#[test]
fn test_child_workflow_search_attributes_typescript() {
    // Verify all SearchAttributeValue types convert correctly
    let config = ChildWorkflowOrchestration::new("Test")
        .with_search_attribute("CustomerId", SearchAttributeValue::String("123"))
        .with_search_attribute("OrderTotal", SearchAttributeValue::Double(99.99))
        .with_search_attribute("Tags", SearchAttributeValue::StringArray(vec!["urgent"]));

    let ts = config.to_typescript();
    verify_typescript_compiles(&ts);
    assert!(ts.contains("searchAttributes"));
}
```

#### 2. Signal Handler TypeScript Generation
```rust
#[test]
fn test_signal_with_schema_typescript() {
    let schema = SignalSchema::with_fields(vec![
        SignalSchemaField {
            name: "approved".to_string(),
            typescript_type: "boolean".to_string(),
            required: true,
            description: Some("Approval status".to_string()),
            default: None,
        }
    ]);

    let signal = SignalDefinition::new("approveOrder")
        .with_input_schema(schema)
        .with_buffering(SignalBuffering::Latest);

    let handler = SignalHandler::new("approveOrder")
        .with_update(VariableUpdate::new(
            "isApproved",
            VariableSource::from_payload("approved")
        ));

    let combined = SignalWithHandler::new(signal, handler);
    let ts = combined.to_typescript();

    // Verify interface generation
    assert!(ts.contains("interface ApproveOrderPayload"));
    assert!(ts.contains("approved: boolean"));

    // Verify signal definition
    assert!(ts.contains("defineSignal<ApproveOrderPayload>('approveOrder')"));

    // Verify handler
    assert!(ts.contains("setHandler"));
    assert!(ts.contains("state.variables.isApproved = payload.approved"));

    verify_typescript_compiles(&ts);
}

#[test]
fn test_signal_buffering_strategies_typescript() {
    // Each buffering strategy should generate correct comment/code
    for buffering in [SignalBuffering::Ordered, SignalBuffering::Latest, SignalBuffering::Immediate] {
        let signal = SignalDefinition::new("test").with_buffering(buffering);
        let handler = SignalHandler::new("test");
        let ts = SignalWithHandler::new(signal, handler).to_typescript();

        // Verify comment indicates buffering behavior
        assert!(ts.contains("Signals are") || ts.contains("Signal"));
    }
}
```

#### 3. Query Handler TypeScript Generation
```rust
#[test]
fn test_query_state_projection_typescript() {
    // Test all variables projection
    let query = QueryDefinition::new(
        "getState",
        QuerySchema::any(),
        QueryHandlerLogic::project(vec!["*"])
    );

    let ts = query.to_typescript();
    assert!(ts.contains("...state.variables"));
    verify_typescript_compiles(&ts);
}

#[test]
fn test_query_computed_typescript() {
    let query = QueryDefinition::new(
        "getProgress",
        QuerySchema::object(vec![("percent", "number")]),
        QueryHandlerLogic::computed("state.completedSteps / state.totalSteps * 100")
    );

    let ts = query.to_typescript();
    assert!(ts.contains("return state.completedSteps / state.totalSteps * 100"));
    verify_typescript_compiles(&ts);
}

#[test]
fn test_standard_queries_typescript() {
    let queries = WorkflowQueries::with_standard_queries();
    let ts = queries.to_typescript();

    // All standard queries must be present
    assert!(ts.contains("getStateQuery"));
    assert!(ts.contains("getProgressQuery"));
    assert!(ts.contains("getStatusQuery"));

    verify_typescript_compiles(&ts);
}
```

#### 4. Cancellation Scope TypeScript Generation
```rust
#[test]
fn test_cancellation_scope_with_cleanup_typescript() {
    let cleanup = CleanupConfig::new()
        .with_activity(CleanupActivity::new("releaseResources"))
        .with_state_update(StateUpdate::new("status", json!("cancelled")));

    let scope = CancellationScope::new("orderProcessing")
        .with_cleanup(cleanup)
        .with_cleanup_timeout(30000);

    let ts = scope.to_typescript("await processOrder();");

    // Verify try/catch structure
    assert!(ts.contains("try {"));
    assert!(ts.contains("isCancellation(err)"));

    // Verify cleanup in non-cancellable scope
    assert!(ts.contains("CancellationScope.nonCancellable"));
    assert!(ts.contains("activities.releaseResources"));
    assert!(ts.contains("state.variables.status = \"cancelled\""));

    verify_typescript_compiles(&ts);
}

#[test]
fn test_shielded_scope_typescript() {
    let scope = CancellationScope::shielded("saveState");
    let ts = scope.to_typescript("await saveState();");

    assert!(ts.contains("CancellationScope.nonCancellable"));
    verify_typescript_compiles(&ts);
}
```

#### 5. Saga Pattern TypeScript Generation
```rust
#[test]
fn test_saga_with_compensation_typescript() {
    let saga = SagaDefinition::new("orderSaga")
        .with_step(
            SagaStep::new("reserveInventory", SagaAction::Activity {
                activity_name: "reserveInventory".to_string(),
                input: json!({"orderId": "123"}),
            })
            .with_compensation(SagaAction::Activity {
                activity_name: "releaseInventory".to_string(),
                input: json!({"orderId": "123"}),
            })
        )
        .with_step(
            SagaStep::new("chargePayment", SagaAction::Activity {
                activity_name: "chargePayment".to_string(),
                input: json!({"orderId": "123"}),
            })
            .with_compensation(SagaAction::Activity {
                activity_name: "refundPayment".to_string(),
                input: json!({"orderId": "123"}),
            })
        );

    let ts = saga.to_typescript();

    // Verify forward steps
    assert!(ts.contains("reserveInventory"));
    assert!(ts.contains("chargePayment"));

    // Verify compensation logic
    assert!(ts.contains("compensateStep"));
    assert!(ts.contains("releaseInventory"));
    assert!(ts.contains("refundPayment"));

    // Verify reverse order compensation
    assert!(ts.contains("[...context.completedSteps].reverse()"));

    verify_typescript_compiles(&ts);
}

#[test]
fn test_saga_parallel_compensation_typescript() {
    let saga = SagaDefinition::new("parallelSaga")
        .parallel_compensation()
        .with_step(SagaStep::new("step1", SagaAction::Activity {
            activity_name: "step1".to_string(),
            input: json!({}),
        }));

    let ts = saga.to_typescript();
    assert!(ts.contains("Promise.allSettled"));
    verify_typescript_compiles(&ts);
}
```

#### 6. Scatter-Gather Pattern TypeScript Generation
```rust
#[test]
fn test_scatter_gather_typescript() {
    let scatter = ScatterGatherDefinition::new("fanout")
        .with_scatter(ScatterConfig {
            workers: vec![
                WorkerConfig::Activity { activity_name: "worker1".to_string(), input: json!({}) },
                WorkerConfig::Activity { activity_name: "worker2".to_string(), input: json!({}) },
            ],
        })
        .with_gather(GatherConfig {
            strategy: GatherStrategy::WaitAll,
            threshold: None,
        })
        .with_timeout(60000);

    let ts = scatter.to_typescript();

    // Verify parallel execution
    assert!(ts.contains("workers.map"));

    // Verify timeout handling
    assert!(ts.contains("Promise.race"));
    assert!(ts.contains("60000"));

    verify_typescript_compiles(&ts);
}
```

**Implementation Approach**:
- Create `tests/typescript_generation_advanced.rs`
- Use helper function `verify_typescript_compiles()` that:
  1. Writes TS to temp file
  2. Creates minimal `tsconfig.json` with strict settings
  3. Runs `tsc --noEmit` if Node.js available
  4. Skips gracefully if Node.js not available (CI can enforce)

---

### Top Layer: End-to-End & Temporal SDK Integration (< 5 min)

**What**: Verify generated workflows actually work with Temporal
**Coverage**:
- ❌ **Temporal Test Server Integration**: Run generated workflows against test server
- ❌ **Child Workflow Execution**: Verify parent-child relationships work
- ❌ **Signal/Query Handling**: Send signals, query state, verify responses
- ❌ **Cancellation Behavior**: Cancel workflows, verify cleanup runs
- ❌ **Pattern Execution**: Execute sagas, scatter-gather, verify results

**Critical Tests Needed**:

#### Temporal Test Server Setup
```rust
// tests/temporal_integration.rs

/// Only run if Temporal test server is available
fn check_temporal_available() -> bool {
    // Check if temporal server is running on localhost:7233
    std::process::Command::new("temporal")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
#[ignore] // Run with --ignored flag
fn test_child_workflow_execution() {
    if !check_temporal_available() {
        eprintln!("Skipping: Temporal not available");
        return;
    }

    // 1. Generate parent workflow with child
    let parent = WorkflowDefinition::with_child_workflow(/* ... */);
    let code = CodeGenerator::new().generate(&parent).unwrap();

    // 2. Compile TypeScript
    let temp_dir = setup_temporal_project(&code);
    compile_typescript(&temp_dir);

    // 3. Start worker
    let worker_handle = start_temporal_worker(&temp_dir);

    // 4. Execute workflow
    let client = TemporalClient::connect("localhost:7233").await.unwrap();
    let handle = client.start_workflow("ParentWorkflow", json!({})).await.unwrap();

    // 5. Verify child workflow was created
    let result = handle.result().await.unwrap();
    assert!(result["childWorkflowExecuted"].as_bool().unwrap());

    worker_handle.shutdown();
}

#[test]
#[ignore]
fn test_signal_handler_execution() {
    // Similar pattern: generate workflow with signal, send signal, verify state updated
}

#[test]
#[ignore]
fn test_query_handler_execution() {
    // Generate workflow with query, query it, verify correct response
}

#[test]
#[ignore]
fn test_saga_compensation_execution() {
    // Generate saga that fails on step 2, verify compensation runs in reverse
}
```

**Implementation Notes**:
- These tests are **expensive** - only run in CI or with `--ignored` flag
- Use `#[ignore]` attribute + `cargo test -- --ignored` to run selectively
- Require Temporal test server running (document setup in README)
- Use test fixtures for common workflow patterns
- **Critical**: These tests are the ULTIMATE truth - if these fail, the system is broken

---

## Cross-Cutting Concerns

### 1. Serialization Round-Trip Testing

**Goal**: Ensure JSON ↔ Rust ↔ JSON is lossless

```rust
#[test]
fn test_child_orchestration_roundtrip() {
    let original = ChildWorkflowOrchestration::new("ProcessOrder")
        .with_workflow_id("order-123")
        .with_search_attribute("CustomerId", SearchAttributeValue::String("cust-456"))
        .with_retry_policy(RetryConfig::default());

    let json = serde_json::to_string(&original).unwrap();
    let restored: ChildWorkflowOrchestration = serde_json::from_str(&json).unwrap();

    // Deep equality check
    assert_eq!(original.workflow_type, restored.workflow_type);
    assert_eq!(original.workflow_id, restored.workflow_id);
    assert_eq!(original.search_attributes, restored.search_attributes);
}
```

**Coverage**: Every struct with `Serialize`/`Deserialize` needs roundtrip test

### 2. Error Message Quality

**Goal**: Errors must be actionable, not cryptic

```rust
#[test]
fn test_validation_error_messages_are_actionable() {
    let config = ChildWorkflowOrchestration {
        id_strategy: WorkflowIdStrategy::Explicit,
        workflow_id: None,  // Missing required field
        ..Default::default()
    };

    let result = config.validate_config();
    assert!(result.is_err());

    let error = result.unwrap_err().join(", ");

    // Error must contain:
    // 1. What's wrong
    assert!(error.contains("Explicit ID strategy"));
    // 2. What field
    assert!(error.contains("workflow_id"));
    // 3. How to fix (implicit: "requires workflow_id")
}
```

### 3. TypeScript Type Safety

**Goal**: No `any` types, strict null checks, proper inference

```rust
#[test]
fn test_generated_typescript_has_no_any_type() {
    let workflow = create_complex_workflow();
    let code = CodeGenerator::new().generate(&workflow).unwrap();

    // Scan all generated TS files
    for ts_code in [code.workflow, code.activities, code.worker] {
        // Should not contain ': any' or 'as any'
        assert!(
            !ts_code.contains(": any") && !ts_code.contains("as any"),
            "Generated TypeScript contains 'any' type: {}",
            ts_code
        );
    }
}
```

### 4. Performance Benchmarks

**Goal**: Validation and generation must be fast

```rust
// benches/validation_bench.rs (requires criterion)

#[bench]
fn bench_validate_large_workflow(b: &mut Bencher) {
    let workflow = create_workflow_with_nodes(100);

    b.iter(|| {
        validate_data_flow(&workflow)
    });
}

#[bench]
fn bench_generate_typescript_large_workflow(b: &mut Bencher) {
    let workflow = create_workflow_with_nodes(100);

    b.iter(|| {
        CodeGenerator::new().generate(&workflow).unwrap()
    });
}
```

**Acceptance Criteria**:
- Workflow with 100 nodes: validate < 100ms
- Workflow with 100 nodes: generate TS < 500ms

---

## Test Organization

### Directory Structure
```
crates/radium-workflow/
├── tests/
│   ├── integration_tests.rs          # Existing: variables, expressions, state
│   ├── typescript_verification.rs    # Existing: basic TS generation
│   ├── component_verification.rs     # Existing: component schemas
│   ├── migration_*.rs                # Existing: migration system
│   │
│   ├── advanced_typescript_gen.rs    # NEW: Phase 7 TS generation
│   ├── temporal_integration.rs       # NEW: Temporal SDK integration (ignored by default)
│   ├── property_tests.rs             # NEW: Property-based testing
│   └── fixtures/                     # NEW: Shared test fixtures
│       ├── workflows.rs
│       ├── temporal_setup.rs
│       └── typescript_compiler.rs
│
├── benches/
│   └── validation_bench.rs           # NEW: Performance benchmarks
│
└── src/
    └── (existing unit tests in each module)
```

### Test Naming Convention
- `test_<feature>_<scenario>_<expected_outcome>`
- Examples:
  - `test_child_workflow_uuid_strategy_generates_unique_id`
  - `test_signal_buffering_latest_drops_old_signals`
  - `test_saga_compensation_runs_in_reverse_order`

### Test Fixtures
Create reusable fixtures in `tests/fixtures/`:

```rust
// tests/fixtures/workflows.rs

pub fn minimal_workflow() -> WorkflowDefinition { /* ... */ }
pub fn workflow_with_child() -> WorkflowDefinition { /* ... */ }
pub fn workflow_with_signals() -> WorkflowDefinition { /* ... */ }
pub fn workflow_with_saga() -> WorkflowDefinition { /* ... */ }

// tests/fixtures/typescript_compiler.rs

pub fn verify_typescript_compiles(ts_code: &str) -> Result<(), String> {
    if !check_node_available() {
        return Ok(()); // Skip if Node.js not available
    }

    let temp_dir = TempDir::new()?;
    write_typescript_project(&temp_dir, ts_code)?;

    let output = Command::new("npm")
        .arg("run")
        .arg("typecheck")
        .current_dir(temp_dir.path())
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "TypeScript compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

// tests/fixtures/temporal_setup.rs

pub fn start_temporal_test_server() -> TemporalTestServer {
    // Start local Temporal test server
}

pub fn create_temporal_client() -> TemporalClient {
    // Connect to test server
}
```

---

## CI/CD Integration

### Pre-commit Checks (< 30s)
```bash
# .git/hooks/pre-commit
cargo fmt --check
cargo clippy -- -D warnings
cargo test --lib  # Unit tests only
```

### Pull Request Checks (< 5 min)
```bash
# .github/workflows/radium-workflow.yml

- name: Unit Tests
  run: cargo test --lib

- name: Integration Tests
  run: cargo test --test '*'

- name: TypeScript Generation Tests
  run: cargo test --test advanced_typescript_gen
  env:
    NODE_ENV: test
```

### Nightly/Release Checks (< 30 min)
```bash
# Run expensive tests
cargo test -- --ignored  # Temporal integration tests
cargo bench              # Performance benchmarks

# Verify against real Temporal server
./scripts/run_temporal_tests.sh
```

---

## Mock Validation Strategy

**Rule**: Every mock MUST be validated against the real system

### Contract Tests for Temporal SDK

```rust
#[test]
fn test_child_workflow_options_match_temporal_sdk() {
    // This is a CONTRACT TEST
    // If Temporal SDK changes, this test MUST fail

    let config = ChildWorkflowOrchestration::new("Test")
        .with_execution_timeout(60000);

    let ts = config.to_typescript();

    // Verify we're using the exact Temporal SDK API
    assert!(ts.contains("workflowExecutionTimeout: '60000ms'"));
    assert!(ts.contains("executeChild"));

    // If this fails, we need to update our code generation
}

#[test]
fn test_signal_definition_matches_temporal_sdk() {
    let signal = SignalDefinition::new("test");
    let ts = signal.to_typescript_definition();

    // Verify we're using defineSignal correctly
    assert!(ts.contains("defineSignal"));
    assert!(ts.contains("from '@temporalio/workflow'"));
}
```

**Validation Schedule**:
- Before each release: Run generated code against latest Temporal SDK
- Monthly: Review Temporal SDK changelog for breaking changes
- Update contract tests when SDK changes

---

## Prioritization by Risk/Impact

### Priority 1: CRITICAL - Must Have Before Release

1. **TypeScript Compilation Tests** - If TS doesn't compile, workflows are broken
   - Child workflow TypeScript generation
   - Signal handler TypeScript generation
   - Query handler TypeScript generation
   - Cancellation scope TypeScript generation

2. **Serialization Round-Trip** - Data loss == data corruption
   - All advanced feature structs

3. **Validation Logic** - Invalid workflows == runtime failures
   - Child workflow validation (timeouts, ID strategies)
   - Saga step validation
   - Scatter-gather threshold validation

### Priority 2: HIGH - Should Have Soon

4. **Temporal SDK Integration** - Verify behavior matches expectations
   - Child workflow execution
   - Signal/query handling
   - Saga compensation execution

5. **Error Message Quality** - Developer experience matters
   - All validation errors have actionable messages

6. **Property-Based Testing** - Find edge cases before users do
   - Schema validation fuzz testing
   - TypeScript generation with random valid inputs

### Priority 3: MEDIUM - Nice to Have

7. **Performance Benchmarks** - Prevent regressions
   - Validation benchmarks
   - Code generation benchmarks

8. **Snapshot Testing** - Detect unintended changes
   - TypeScript generation snapshot tests

---

## Success Metrics

### Test Coverage Targets
- **Line Coverage**: > 85% (existing: ~90% based on test count)
- **Branch Coverage**: > 80%
- **Integration Coverage**: 100% of Phase 7 features have at least one integration test

### Test Quality Metrics
- **Flakiness**: 0 flaky tests allowed
- **Runtime**: Fast tests < 10s, Integration < 60s, E2E < 5min
- **Signal-to-Noise**: Failed test == real bug (no false positives)

### Detection Metrics (retroactive)
- **TypeScript Compilation Errors**: 100% caught before commit
- **Temporal SDK Incompatibilities**: 100% caught in integration tests
- **Invalid Workflow Definitions**: 100% caught in validation tests

---

## Implementation Plan

### Phase 1: Foundation (Week 1)
- [ ] Set up test fixtures (`tests/fixtures/`)
- [ ] Create `verify_typescript_compiles()` helper
- [ ] Add TypeScript generation tests for child workflows
- [ ] Add TypeScript generation tests for signals
- [ ] Add TypeScript generation tests for queries

### Phase 2: Patterns (Week 2)
- [ ] Add TypeScript generation tests for cancellation
- [ ] Add TypeScript generation tests for sagas
- [ ] Add TypeScript generation tests for scatter-gather
- [ ] Add serialization round-trip tests for all advanced features

### Phase 3: Integration (Week 3)
- [ ] Set up Temporal test server infrastructure
- [ ] Add child workflow execution test
- [ ] Add signal handling test
- [ ] Add saga execution test
- [ ] Document Temporal test setup in README

### Phase 4: Quality & Performance (Week 4)
- [ ] Add property-based tests with `proptest`
- [ ] Add performance benchmarks
- [ ] Review and improve error messages
- [ ] Add snapshot tests for generated TypeScript
- [ ] CI/CD integration

---

## Open Questions / Risks

1. **Temporal Test Server Availability**: How do we ensure CI has Temporal running?
   - **Mitigation**: Use `#[ignore]` + Docker compose for local dev

2. **TypeScript Compilation in CI**: Node.js dependency in Rust CI?
   - **Mitigation**: Optional test, skip if npm not available, enforce in separate CI job

3. **Test Execution Time**: Will Temporal integration tests be too slow?
   - **Mitigation**: Run on `--ignored` flag, nightly only, use test fixtures

4. **Mock Drift**: How do we keep mocks in sync with Temporal SDK?
   - **Mitigation**: Contract tests that fail when SDK changes, monthly review

---

## Conclusion

This testing strategy is designed to make radium-workflow **bulletproof**:

- **Fast feedback** via unit tests (< 10s)
- **Compilation safety** via TypeScript generation tests (< 60s)
- **Runtime correctness** via Temporal integration tests (< 5min, on-demand)
- **Zero tolerance** for flakiness, cryptic errors, or unvalidated mocks

The testing pyramid ensures:
- Developers get immediate feedback on breakage
- Code generation is always valid TypeScript
- Temporal SDK integration actually works

**Next Steps**: Begin Phase 1 implementation, starting with test fixtures and TypeScript compilation verification.
