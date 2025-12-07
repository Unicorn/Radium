# Testing Guide for Radium

**Last Updated:** 2025-12-07
**Related REQ:** REQ-164 - Comprehensive Test Coverage Strategy

This document provides comprehensive guidelines for testing Radium components across all crates.

---

## 📋 Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Test Infrastructure](#test-infrastructure)
3. [Test Coverage Requirements](#test-coverage-requirements)
4. [Running Tests](#running-tests)
5. [Writing Tests](#writing-tests)
6. [Code Coverage](#code-coverage)
7. [Continuous Integration](#continuous-integration)
8. [Troubleshooting](#troubleshooting)

---

## 🎯 Testing Philosophy

Radium follows a comprehensive testing strategy with three layers:

### Test Pyramid

```
       /\
      /  \    E2E Tests (Golden Path Workflows)
     /____\
    /      \  Integration Tests (CLI, gRPC, Multi-component)
   /________\
  /          \ Unit Tests (Individual functions, modules, structs)
 /____________\
```

**Guidelines:**
- **Unit Tests**: Test individual functions/methods in isolation. Should be fast (<1ms) and comprehensive.
- **Integration Tests**: Test component interactions (CLI commands, gRPC endpoints, workflows). Can be slower (<100ms).
- **E2E Tests**: Test complete user workflows from command input to final output. Allowed to be slow (seconds).

**Coverage Targets:**
- Core functionality: **90%+ coverage**
- Error handling paths: **80%+ coverage**
- Happy path workflows: **100% coverage**

---

## 🛠️ Test Infrastructure

### Coverage Tool

Radium uses `cargo-llvm-cov` for code coverage reporting.

**Installation:**
```bash
cargo install cargo-llvm-cov
```

**Features:**
- LCOV report generation for CI integration
- HTML reports with line-by-line coverage visualization
- Workspace-wide coverage aggregation
- Exclusion of test code from coverage metrics

### CI/CD Integration

**GitHub Actions Workflow:** `.github/workflows/test-coverage.yml`

**Triggers:**
- Push to `main` branch
- Pull requests targeting `main`
- Manual workflow dispatch

**Workflow Steps:**
1. Install Rust toolchain with `llvm-tools-preview`
2. Cache cargo dependencies for faster runs
3. Install `cargo-llvm-cov`
4. Run full test suite: `cargo test --workspace`
5. Generate coverage reports (LCOV + HTML)
6. Display coverage summary in CI logs

---

## 📊 Test Coverage Requirements

### Current Coverage (as of 2025-12-07)

| Crate | Tests | Status | Coverage |
|-------|-------|--------|----------|
| **radium-core** | 301 | ✅ Passing | High |
| - agents | 42 | ✅ Passing | 95%+ |
| - workflow | 169 | ✅ Passing | 90%+ |
| - storage | 58 | ✅ Passing | 85%+ |
| - policy | 32 | ✅ Passing | 90%+ |
| **radium-orchestrator** | 122 | ✅ Passing | 85%+ |
| **radium-models** | 10 | ✅ Passing | 80%+ |
| **radium-cli** | 34 | 🔨 In Progress | TBD |
| **Total** | **467+** | | **~88%** |

---

## 🚀 Running Tests

### Quick Commands

```bash
# Run all tests in workspace
cargo test --workspace

# Run tests for specific crate
cargo test --package radium-core
cargo test --package radium-orchestrator
cargo test --package radium-models
cargo test --package radium-cli

# Run tests for specific module
cargo test --package radium-core --lib agents
cargo test --package radium-core --lib workflow
cargo test --package radium-core --lib storage
cargo test --package radium-core --lib policy

# Run specific test by name
cargo test --package radium-core test_agent_registry_new

# Run tests with output (helpful for debugging)
cargo test -- --nocapture

# Run tests in parallel with specific thread count
cargo test -- --test-threads=4
```

### Coverage Reports

```bash
# Generate HTML coverage report (opens in browser)
cargo llvm-cov --workspace --html
open target/llvm-cov/html/index.html

# Generate LCOV report for CI integration
cargo llvm-cov --workspace --lcov --output-path lcov.info

# Display coverage summary in terminal
cargo llvm-cov --workspace --summary-only

# Generate coverage for specific crate
cargo llvm-cov --package radium-core --html
```

### Integration Tests

```bash
# Run CLI integration tests
cargo test --package radium-cli --test '*'

# Run specific CLI test file
cargo test --package radium-cli --test cli_e2e_test

# Run E2E golden path workflow test
cargo test --package radium-cli --test golden_path_workflow
```

---

## ✍️ Writing Tests

### Unit Test Structure

Radium uses the **Arrange-Act-Assert (AAA)** pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_registry_new() {
        // Arrange: Set up test data and dependencies
        let registry = AgentRegistry::new();

        // Act: Perform the operation being tested
        let result = registry.list_ids();

        // Assert: Verify the expected outcome
        assert!(result.is_empty());
    }
}
```

### Async Tests

Use `#[tokio::test]` for async tests that require Tokio runtime:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_operation() {
        // Arrange
        let manager = ConstitutionManager::new();

        // Act
        manager.update_constitution("session-1", "rule".to_string());
        let rules = manager.get_constitution("session-1");

        // Assert
        assert_eq!(rules.len(), 1);
    }
}
```

### Test Helpers

Create reusable test helpers to reduce duplication:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create test agent configurations
    fn create_test_agent(id: &str, name: &str) -> AgentConfig {
        AgentConfig {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("Test agent: {}", name),
            prompt_path: PathBuf::from("test.md"),
            mirror_path: None,
            engine: None,
            model: None,
            reasoning_effort: None,
            loop_behavior: None,
            trigger_behavior: None,
            category: None,
            file_path: None,
        }
    }

    #[test]
    fn test_agent_creation() {
        let agent = create_test_agent("test-agent", "Test Agent");
        assert_eq!(agent.id, "test-agent");
        assert_eq!(agent.name, "Test Agent");
    }
}
```

### Mocking External Dependencies

For integration tests with external providers (Gemini, OpenAI), use mocks:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_with_mock_provider() {
        let mock_provider = MockProvider::new()
            .with_response("Test response")
            .with_token_count(100);

        let result = mock_provider.generate("Test prompt").await;

        assert_eq!(result.content, "Test response");
        assert_eq!(result.tokens, 100);
    }
}
```

### Testing Error Paths

Always test both success and failure cases:

```rust
#[test]
fn test_register_duplicate_fails() {
    let registry = AgentRegistry::new();
    let agent = create_test_agent("agent-1", "Agent 1");

    // First registration should succeed
    registry.register(agent.clone()).unwrap();

    // Duplicate registration should fail
    let result = registry.register(agent.clone());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RegistryError::DuplicateAgent(_)));
}
```

---

## 📈 Code Coverage

### Coverage Reports

Coverage artifacts are generated in `target/llvm-cov/`:

```
target/llvm-cov/
├── html/              # HTML report (viewable in browser)
│   ├── index.html     # Main coverage summary
│   └── src/           # Per-file coverage details
├── lcov.info          # LCOV format (for CI tools)
└── cobertura.xml      # Cobertura format (alternative CI format)
```

### Excluded Files

The following are excluded from coverage metrics:

- `**/tests/**` - Test code itself
- `**/examples/**` - Example code
- `**/benches/**` - Benchmark code
- Generated code (e.g., `proto/radium.proto` compiled output)

### Coverage Interpretation

**Line Coverage Levels:**
- 🟢 **90-100%**: Excellent - comprehensive testing
- 🟡 **75-89%**: Good - adequate testing, some gaps
- 🟠 **60-74%**: Fair - significant gaps, needs improvement
- 🔴 **<60%**: Poor - insufficient testing

**What to focus on:**
- **Critical paths**: Authentication, data persistence, workflow execution
- **Error handling**: All error variants should be tested
- **Edge cases**: Boundary conditions, empty inputs, max limits

---

## 🔄 Continuous Integration

### GitHub Actions Workflow

**Location:** `.github/workflows/test-coverage.yml`

**Triggers:**
- All pushes to `main`
- All pull requests targeting `main`
- Manual dispatch via GitHub UI

**Workflow Highlights:**
```yaml
- name: Run tests
  run: cargo test --workspace

- name: Generate coverage report
  run: |
    cargo llvm-cov --workspace --lcov --output-path lcov.info
    cargo llvm-cov --workspace --html

- name: Display coverage summary
  run: cargo llvm-cov --workspace --summary-only
```

**Caching Strategy:**
- Cargo registry cache (speeds up dependency downloads)
- Cargo build cache (speeds up compilation)
- Cache key: OS + `Cargo.lock` hash

---

## 🐛 Troubleshooting

### Common Issues

#### 1. Tokio Runtime Error

**Error:**
```
there is no reactor running, must be called from the context of a Tokio 1.x runtime
```

**Solution:** Use `#[tokio::test]` instead of `#[test]`:
```rust
#[tokio::test]
async fn test_async_function() {
    // Your async test code
}
```

#### 2. Tests Hanging

**Symptom:** Tests run but never complete

**Common Causes:**
- Deadlock in async code
- Waiting on a channel/future that never resolves
- Infinite loop

**Debug Commands:**
```bash
# Run with timeout
cargo test -- --test-threads=1 --nocapture

# Add RUST_BACKTRACE for deadlock investigation
RUST_BACKTRACE=1 cargo test
```

#### 3. Flaky Tests

**Symptom:** Tests pass sometimes, fail other times

**Common Causes:**
- Race conditions in parallel tests
- Shared mutable state
- Time-dependent assertions

**Solutions:**
```bash
# Run tests serially to isolate race conditions
cargo test -- --test-threads=1

# Run specific test multiple times
cargo test test_name -- --test-threads=1 --nocapture
```

#### 4. Coverage Report Not Generating

**Error:**
```
error: could not compile radium-core
```

**Solution:** Ensure code compiles before generating coverage:
```bash
# First verify compilation
cargo check --workspace --all-targets

# Then run coverage
cargo llvm-cov --workspace --html
```

---

## 📚 Best Practices

### DO:

✅ Write tests before or alongside implementation (TDD)
✅ Test both success and error paths
✅ Use descriptive test names (e.g., `test_agent_selection_with_insufficient_budget`)
✅ Keep tests focused on a single behavior
✅ Use test helpers to reduce duplication
✅ Test edge cases (empty inputs, max limits, boundary conditions)
✅ Mock external dependencies (AI providers, network calls)
✅ Update tests when changing implementation

### DON'T:

❌ Write tests that depend on execution order
❌ Use `unwrap()` in test assertions (use `assert!`, `assert_eq!`, `Result`)
❌ Test implementation details (test behavior, not internals)
❌ Share mutable state between tests
❌ Use real API keys or network calls in unit tests
❌ Ignore test failures ("works on my machine")
❌ Write tests without assertions

---

## 🎓 Examples

### Example 1: Unit Test

**File:** `crates/radium-core/src/agents/registry.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_or_replace() {
        // Arrange
        let registry = AgentRegistry::new();
        let agent1 = create_test_agent("agent-1", "First Version");
        let agent2 = create_test_agent("agent-1", "Second Version");

        // Act
        registry.register_or_replace(agent1.clone());
        registry.register_or_replace(agent2.clone());

        // Assert
        let retrieved = registry.get("agent-1").unwrap();
        assert_eq!(retrieved.name, "Second Version");
    }
}
```

### Example 2: Async Integration Test

**File:** `crates/radium-orchestrator/src/orchestration/engine.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multi_turn_orchestration() {
        // Arrange
        let provider = Arc::new(MockProvider::new_with_tool_support());
        let engine = MultiTurnEngine::new(provider, config);

        // Act
        let result = engine.orchestrate("Execute workflow").await;

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.iterations > 0);
        assert!(!output.final_content.is_empty());
    }
}
```

### Example 3: CLI E2E Test

**File:** `apps/cli/tests/cli_e2e_test.rs`

```rust
#[test]
fn test_agent_list_command() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    create_test_workspace(&temp_dir);

    // Act
    let output = Command::new("radium-cli")
        .args(&["agents", "list"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    // Assert
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("test-agent"));
}
```

---

## 📞 Getting Help

- **Documentation:** `docs/` directory
- **Examples:** `examples/` directory
- **GitHub Issues:** Report test failures or coverage gaps
- **Code Reviews:** Ask for testing feedback in PRs

---

## 🔗 Related Documentation

- [REQ-164: Comprehensive Test Coverage Strategy](../roadmap/PROGRESS.md)
- [GitHub Actions CI Workflow](../.github/workflows/test-coverage.yml)
- [ADR-001: YOLO Mode Architecture](adr/001-yolo-mode-architecture.md)
- [Integration Map](yolo-mode/integration-map.md)

---

**Happy Testing! 🧪**
