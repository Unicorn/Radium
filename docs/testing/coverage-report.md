# Test Coverage Report

This document describes the test coverage infrastructure for the hooks system.

## Coverage Requirements

The hooks system targets **>80% test coverage** for all hook-related code in `crates/radium-core/src/hooks/`.

## Coverage Tools

We use `cargo-llvm-cov` for coverage measurement:

- **Fast**: Uses LLVM's built-in coverage instrumentation
- **Accurate**: Precise line and branch coverage
- **Multiple formats**: Supports LCOV, HTML, and JSON output

## Generating Coverage Reports

### Local Development

Run the coverage script:

```bash
./scripts/coverage-report.sh
```

This generates:
- `coverage.lcov` - LCOV format for CI tools
- `coverage-html/` - HTML report for local viewing
- `hooks-coverage.lcov` - Hooks-specific coverage

### Manual Generation

```bash
cd crates/radium-core
cargo llvm-cov --locked --workspace --lcov --output-path ../../coverage.lcov --tests
cargo llvm-cov --locked --workspace --html --output-path ../../coverage-html --tests
```

## Coverage Metrics

### Current Coverage

Run coverage analysis:

```bash
cargo llvm-cov --locked --workspace --summary --tests --lib --hooks
```

### Target Coverage by Module

- **hooks/registry.rs**: >90% (critical path)
- **hooks/types.rs**: >80% (core types)
- **hooks/model.rs**: >80% (model hooks)
- **hooks/tool.rs**: >80% (tool hooks)
- **hooks/error_hooks.rs**: >80% (error handling)
- **hooks/config.rs**: >80% (configuration)
- **hooks/integration.rs**: >80% (integration helpers)
- **hooks/adapters.rs**: >80% (adapters)

## CI Integration

Coverage is automatically generated in CI:

1. **On every push**: Coverage report is generated
2. **On PRs**: Coverage diff is shown
3. **Coverage threshold**: Fails if coverage <80%

## Viewing Coverage Reports

### HTML Report

Open `coverage-html/index.html` in a browser to view:
- Line-by-line coverage
- Branch coverage
- Function coverage
- Uncovered code highlighting

### LCOV Report

Use tools like:
- **Codecov**: Automatic PR comments
- **Coveralls**: Coverage tracking
- **lcov-html**: Generate HTML from LCOV

## Coverage Gaps

To identify coverage gaps:

1. Generate coverage report
2. Review HTML report for uncovered lines
3. Add tests for uncovered code paths
4. Re-run coverage to verify improvement

## Best Practices

1. **Test critical paths first**: Focus on high-impact code
2. **Test edge cases**: Cover error conditions and boundaries
3. **Test integration**: Verify hooks work in real scenarios
4. **Maintain coverage**: Don't let coverage drop below threshold
5. **Review coverage regularly**: Check reports before merging

## Coverage Exclusions

Some code is excluded from coverage:

- **Test code**: Tests themselves aren't counted
- **Benchmark code**: Benchmarks are excluded
- **Example code**: Documentation examples excluded
- **Panic handlers**: Unreachable error paths

## Improving Coverage

To improve coverage:

1. **Identify gaps**: Review coverage report
2. **Add unit tests**: Test individual functions
3. **Add integration tests**: Test hook execution
4. **Test error paths**: Cover error conditions
5. **Test edge cases**: Boundary conditions

## Coverage History

Coverage is tracked over time:
- Baseline: Initial implementation
- Target: >80% for all modules
- Current: See CI reports

## Troubleshooting

### Coverage not generating

- Ensure `llvm-tools-preview` is installed
- Check that tests are running
- Verify `cargo-llvm-cov` is installed

### Coverage too low

- Review uncovered code
- Add missing tests
- Check for excluded code that should be tested

### CI failures

- Check coverage threshold
- Review coverage diff
- Add tests for new code

