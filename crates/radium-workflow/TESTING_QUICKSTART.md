# Testing Quick Start Guide

**Goal**: Get you writing tests in < 15 minutes
**Audience**: Developers implementing the testing plan
**Prerequisite**: Read TESTING_SUMMARY.md first

---

## Setup (5 minutes)

### 1. Ensure You Have Node.js Installed

```bash
node --version  # Should be v18+
npm --version
```

**Don't have Node.js?** Tests will skip TypeScript compilation locally, but CI will enforce it.

### 2. Create Test Fixtures Directory

```bash
cd crates/radium-workflow
mkdir -p tests/fixtures
```

### 3. Create TypeScript Compiler Helper

**File**: `tests/fixtures/typescript_compiler.rs`

Copy this file exactly:

```rust
use std::process::Command;
use std::fs;
use tempfile::TempDir;

pub fn check_node_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn verify_typescript_compiles(ts_code: &str) -> Result<(), String> {
    if !check_node_available() {
        eprintln!("⚠️  Skipping TypeScript compilation (npm not available)");
        return Ok(());
    }

    let temp_dir = TempDir::new().map_err(|e| e.to_string())?;
    let ts_file = temp_dir.path().join("test.ts");
    fs::write(&ts_file, ts_code).map_err(|e| e.to_string())?;

    let tsconfig = r#"{
      "compilerOptions": {
        "target": "ES2020",
        "module": "commonjs",
        "strict": true,
        "noImplicitAny": true,
        "strictNullChecks": true
      }
    }"#;
    fs::write(temp_dir.path().join("tsconfig.json"), tsconfig)
        .map_err(|e| e.to_string())?;

    let package_json = r#"{
      "name": "test",
      "version": "1.0.0",
      "dependencies": {
        "@temporalio/workflow": "^1.10.0"
      },
      "devDependencies": {
        "typescript": "^5.3.0",
        "@types/node": "^20.0.0"
      }
    }"#;
    fs::write(temp_dir.path().join("package.json"), package_json)
        .map_err(|e| e.to_string())?;

    let install = Command::new("npm")
        .arg("install")
        .arg("--silent")
        .current_dir(temp_dir.path())
        .output()
        .map_err(|e| e.to_string())?;

    if !install.status.success() {
        return Err(format!("npm install failed:\n{}",
            String::from_utf8_lossy(&install.stderr)));
    }

    let output = Command::new("npx")
        .arg("tsc")
        .arg("--noEmit")
        .arg("test.ts")
        .current_dir(temp_dir.path())
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!("TypeScript compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}

pub fn verify_typescript_with_temporal(ts_code: &str) -> Result<(), String> {
    let full_code = format!(
        r#"
import {{ executeChild, defineSignal, defineQuery, setHandler, CancellationScope }} from '@temporalio/workflow';

{}
"#,
        ts_code
    );

    verify_typescript_compiles(&full_code)
}
```

### 4. Create Fixtures Module Declaration

**File**: `tests/fixtures/mod.rs`

```rust
pub mod typescript_compiler;
```

---

## Your First Test (10 minutes)

### 1. Create Test File

**File**: `tests/advanced_typescript_gen.rs`

```rust
//! TypeScript Generation Tests for Phase 7 Advanced Features

mod fixtures;
use fixtures::typescript_compiler::verify_typescript_with_temporal;
use radium_workflow::schema::advanced::child_orchestration::*;
use serde_json::json;

#[test]
fn test_child_workflow_uuid_strategy_compiles() {
    // 1. Create a child workflow config
    let config = ChildWorkflowOrchestration::new("ProcessOrder")
        .with_task_queue("orders");

    // 2. Generate TypeScript
    let ts = config.to_typescript();

    // 3. Verify it contains expected elements
    assert!(
        ts.contains("uuid4()") || ts.contains("uuid.v4()"),
        "UUID strategy should generate unique ID"
    );
    assert!(ts.contains("executeChild"), "Should use executeChild");
    assert!(ts.contains("'ProcessOrder'"), "Should reference workflow type");

    // 4. Verify TypeScript compiles
    verify_typescript_with_temporal(&ts)
        .expect("Generated TypeScript should compile");
}
```

### 2. Run Your Test

```bash
cd crates/radium-workflow
cargo test test_child_workflow_uuid_strategy_compiles
```

**Expected Output**:
```
running 1 test
test test_child_workflow_uuid_strategy_compiles ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 444 filtered out
```

**If npm not available**:
```
⚠️  Skipping TypeScript compilation (npm not available)
test test_child_workflow_uuid_strategy_compiles ... ok
```

---

## Test Writing Pattern

Every TypeScript generation test follows this pattern:

```rust
#[test]
fn test_<feature>_<scenario>_<expected>() {
    // 1. Create config
    let config = SomeConfig::new("name")
        .with_option(value);

    // 2. Generate TypeScript
    let ts = config.to_typescript();

    // 3. Assert TypeScript contains expected elements
    assert!(ts.contains("expectedString"), "Why this matters");

    // 4. Verify TypeScript compiles
    verify_typescript_with_temporal(&ts)
        .expect("Explanation if this fails");
}
```

---

## Common Patterns

### Pattern 1: Testing Multiple Variants

```rust
#[test]
fn test_all_cancellation_types_compile() {
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
            "Type {:?} should generate {}",
            cancel_type, expected_const
        );

        verify_typescript_with_temporal(&ts)
            .unwrap_or_else(|e| panic!("Type {:?} failed: {}", cancel_type, e));
    }
}
```

### Pattern 2: Testing Complex Configurations

```rust
#[test]
fn test_child_workflow_kitchen_sink_compiles() {
    use radium_workflow::schema::components::RetryConfig;

    let config = ChildWorkflowOrchestration::new("CompleteExample")
        .with_workflow_id("example-123")
        .with_task_queue("example-queue")
        .with_input("key", json!("value"))
        .with_execution_timeout(300000)
        .with_search_attribute("attr", SearchAttributeValue::String("test".into()))
        .with_memo("note", "test")
        .with_retry_policy(RetryConfig::default());

    let ts = config.to_typescript();

    // Verify all options are present
    let required = [
        "executeChild",
        "workflowId: 'example-123'",
        "taskQueue: 'example-queue'",
        "workflowExecutionTimeout",
        "searchAttributes",
        "memo",
        "retry",
    ];

    for req in required {
        assert!(ts.contains(req), "Missing: {}", req);
    }

    verify_typescript_with_temporal(&ts)
        .expect("Complete config should compile");
}
```

### Pattern 3: Serialization Round-Trip

```rust
#[test]
fn test_config_serialization_roundtrip() {
    let original = ChildWorkflowOrchestration::new("Test")
        .with_workflow_id("test-123")
        .with_execution_timeout(60000);

    let json = serde_json::to_string(&original)
        .expect("Should serialize");
    let restored: ChildWorkflowOrchestration = serde_json::from_str(&json)
        .expect("Should deserialize");

    assert_eq!(original.workflow_type, restored.workflow_type);
    assert_eq!(original.workflow_id, restored.workflow_id);
    assert_eq!(original.execution_timeout_ms, restored.execution_timeout_ms);
}
```

---

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Only New Tests
```bash
cargo test --test advanced_typescript_gen
```

### Run Specific Test
```bash
cargo test test_child_workflow_uuid
```

### Run With Output
```bash
cargo test test_child_workflow -- --nocapture
```

### Run Integration Tests (when available)
```bash
cargo test -- --ignored
```

---

## Debugging Failed Tests

### TypeScript Compilation Failure

**Error**:
```
TypeScript compilation failed:
test.ts(5,20): error TS2304: Cannot find name 'uuid4'.
```

**Fix**: Check that the generated TS imports the required function:
```rust
// In to_typescript() method, add:
code.push_str("import { uuid4 } from '@temporalio/workflow';\n");
```

### Assertion Failure

**Error**:
```
assertion failed: ts.contains("executeChild")
```

**Debug**:
1. Print the generated TypeScript:
   ```rust
   eprintln!("Generated TS:\n{}", ts);
   assert!(ts.contains("executeChild"));
   ```

2. Run test with output:
   ```bash
   cargo test test_name -- --nocapture
   ```

3. Verify the actual string in generated code

### npm Install Failures

**Error**:
```
npm install failed: ECONNREFUSED
```

**Fix**: This is usually a network issue. Tests will skip gracefully, but CI needs working npm.

---

## Adding More Tests

### Week 1: Child Workflows

Copy-paste these test skeletons and fill in assertions:

```rust
#[test]
fn test_child_workflow_explicit_id_compiles() {
    let config = ChildWorkflowOrchestration::new("Test")
        .with_workflow_id("explicit-123");

    let ts = config.to_typescript();

    // TODO: Add assertions
    assert!(ts.contains("workflowId: 'explicit-123'"));

    verify_typescript_with_temporal(&ts).expect("Should compile");
}

#[test]
fn test_child_workflow_parent_suffix_compiles() {
    let config = ChildWorkflowOrchestration::new("Test")
        .with_parent_suffix();

    let ts = config.to_typescript();

    // TODO: Add assertions

    verify_typescript_with_temporal(&ts).expect("Should compile");
}

#[test]
fn test_child_workflow_pattern_compiles() {
    let config = ChildWorkflowOrchestration::new("Test")
        .with_id_pattern("child-{parent_id}-{index}");

    let ts = config.to_typescript();

    // TODO: Add assertions

    verify_typescript_with_temporal(&ts).expect("Should compile");
}
```

### Week 2: Signals

```rust
use radium_workflow::schema::advanced::signals::*;

#[test]
fn test_signal_with_payload_compiles() {
    let schema = SignalSchema::with_fields(vec![
        SignalSchemaField {
            name: "value".to_string(),
            typescript_type: "string".to_string(),
            required: true,
            description: None,
            default: None,
        },
    ]);

    let signal = SignalDefinition::new("updateValue")
        .with_input_schema(schema);

    let ts = signal.to_typescript_definition();

    // TODO: Add assertions for interface and defineSignal

    verify_typescript_with_temporal(&ts).expect("Should compile");
}
```

---

## Tips & Best Practices

### 1. Write Failing Test First
```rust
#[test]
#[should_panic(expected = "Should compile")]
fn test_new_feature_compiles() {
    // Write this BEFORE implementing the feature
    let config = NewFeature::new("test");
    let ts = config.to_typescript();
    verify_typescript_with_temporal(&ts).expect("Should compile");
}
```

### 2. Use Descriptive Assertion Messages
```rust
// ❌ Bad
assert!(ts.contains("retry"));

// ✅ Good
assert!(
    ts.contains("retry"),
    "Retry policy config should generate 'retry' object in executeChild options"
);
```

### 3. Test Edge Cases
```rust
#[test]
fn test_empty_search_attributes_omitted() {
    let config = ChildWorkflowOrchestration::new("Test");
    // No search attributes added

    let ts = config.to_typescript();

    // Empty attributes should not appear in output
    assert!(
        !ts.contains("searchAttributes"),
        "Empty search attributes should be omitted"
    );
}
```

### 4. Group Related Tests
```rust
mod child_workflow_tests {
    use super::*;

    mod id_strategies {
        use super::*;

        #[test]
        fn test_uuid_strategy() { /* ... */ }

        #[test]
        fn test_explicit_strategy() { /* ... */ }
    }

    mod options {
        use super::*;

        #[test]
        fn test_timeouts() { /* ... */ }

        #[test]
        fn test_retry_policy() { /* ... */ }
    }
}
```

---

## Checklist Before Committing

- [ ] Test passes locally: `cargo test <test_name>`
- [ ] Test name follows convention: `test_<feature>_<scenario>_<expected>`
- [ ] Assertions have clear messages explaining why they matter
- [ ] TypeScript compilation is verified
- [ ] Edge cases considered
- [ ] Related tests grouped in modules
- [ ] All tests pass: `cargo test`
- [ ] No warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt`

---

## Getting Help

### Test Failing?
1. Add `eprintln!("Generated TS:\n{}", ts);` before assertion
2. Run with `--nocapture` to see output
3. Compare expected vs actual TypeScript

### Don't Know What to Assert?
1. Look at existing tests in `src/schema/advanced/<module>.rs`
2. Check the module's `#[cfg(test)]` section
3. Reference TESTING_ACTION_PLAN.md for specific test cases

### TypeScript Compilation Unclear?
1. Copy generated TS to a real .ts file
2. Run `tsc --noEmit file.ts` manually
3. Fix errors in Rust code generation

### Integration Tests?
1. See TESTING_ACTION_PLAN.md Week 4
2. Not needed until Week 4
3. Start with TypeScript generation tests first

---

## Next Steps

1. **Implement Week 1 Tests** (see TESTING_ACTION_PLAN.md)
   - Child workflow ID strategies (4 tests)
   - Child workflow options (6 tests)
   - Edge cases + serialization (5 tests)

2. **Run Tests Frequently**
   - After each test: `cargo test <test_name>`
   - Before commit: `cargo test`

3. **Track Progress**
   - Use checklist in TESTING_ACTION_PLAN.md
   - Update weekly in team meetings

---

## Questions?

- **Strategic questions**: See TESTING_STRATEGY.md
- **What to test**: See TESTING_GAP_ANALYSIS.md
- **Implementation details**: See TESTING_ACTION_PLAN.md
- **Quick reference**: See TESTING_SUMMARY.md
- **How to test**: This document

**Ready to start?** Create `tests/fixtures/typescript_compiler.rs` and write your first test!
