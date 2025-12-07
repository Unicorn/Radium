# CLI Performance Guide

This document describes performance benchmarks, optimization strategies, and performance guidelines for the Radium CLI.

## Benchmark Suite

The CLI includes a benchmark suite using Criterion located in `apps/cli/benches/`.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench -p radium-cli

# Run specific benchmark
cargo bench -p radium-cli --bench workspace_bench
```

### Benchmark Categories

1. **Workspace Operations**
   - Workspace discovery performance
   - Workspace initialization time
   - Directory structure creation

2. **Command Execution**
   - Command parsing overhead
   - JSON serialization performance
   - Output formatting

## Performance Targets

### Command Startup

- **Cold start**: <100ms (excluding dependency loading)
- **Warm start**: <50ms
- **Command parsing**: <10ms

### Common Operations

- **Workspace discovery**: <50ms (even in deep directory trees)
- **Agent list**: <200ms (for 100+ agents)
- **Status command**: <300ms (including all checks)
- **Plan generation**: <2s (excluding AI model calls)
- **JSON output**: <10ms overhead per command

### File Operations

- **Workspace init**: <500ms
- **File reading**: <100ms per file
- **Directory traversal**: <200ms for typical workspace

## Optimization Strategies

### Workspace Discovery

**Current Implementation:**
- Searches upward from current directory
- Stops at first `.radium` directory found
- Caches result where possible

**Optimization Opportunities:**
- Cache workspace path in environment variable
- Use faster path operations (avoid canonicalization where not needed)
- Limit search depth (e.g., max 10 levels up)

### Command Parsing

**Current Implementation:**
- Uses Clap derive macros
- Parses all arguments upfront

**Optimization Opportunities:**
- Lazy argument parsing for rarely-used flags
- Cache parsed command structure
- Minimize string allocations

### JSON Output

**Current Implementation:**
- Uses `serde_json::to_string_pretty()`
- Creates full JSON structure before output

**Optimization Opportunities:**
- Stream JSON output for large datasets
- Use `to_string()` instead of `to_string_pretty()` for production
- Reuse JSON serializers where possible

### Async Operations

**Current Implementation:**
- All commands use async/await
- Tokio runtime for I/O operations

**Optimization Opportunities:**
- Use `tokio::fs` for async file operations
- Batch file system operations
- Parallelize independent operations

## Performance Monitoring

### Baseline Measurements

Run benchmarks regularly to establish baselines:

```bash
# Generate baseline report
cargo bench -p radium-cli -- --save-baseline baseline

# Compare against baseline
cargo bench -p radium-cli -- --baseline baseline
```

### Regression Testing

Add performance regression tests to CI:

```rust
#[test]
fn test_workspace_discovery_performance() {
    let start = std::time::Instant::now();
    // ... perform operation
    let duration = start.elapsed();
    assert!(duration.as_millis() < 50, "Workspace discovery too slow");
}
```

## Profiling

### Using Criterion

Criterion automatically generates HTML reports with detailed profiling information:

```bash
cargo bench -p radium-cli
# Reports available in target/criterion/
```

### Using Flamegraph

For detailed profiling:

```bash
# Install flamegraph
cargo install flamegraph

# Profile command execution
cargo flamegraph --bin radium-cli -- status
```

### Using Perf (Linux)

```bash
perf record --call-graph=dwarf cargo run --release -p radium-cli -- status
perf report
```

## Common Bottlenecks

### File System Operations

**Problem**: Slow file I/O operations

**Solutions**:
- Use async file operations (`tokio::fs`)
- Batch file reads/writes
- Cache file metadata
- Avoid unnecessary `canonicalize()` calls

### String Allocations

**Problem**: Excessive string allocations

**Solutions**:
- Use string slices where possible
- Reuse string buffers
- Use `Cow<str>` for conditional ownership
- Minimize format!() calls in hot paths

### JSON Serialization

**Problem**: Slow JSON output generation

**Solutions**:
- Use streaming serialization for large datasets
- Avoid pretty printing in production
- Cache serialized structures where possible
- Use `serde_json::to_writer()` for direct output

### Workspace Discovery

**Problem**: Slow workspace discovery in deep trees

**Solutions**:
- Cache workspace path
- Limit search depth
- Use faster path operations
- Store workspace marker in parent directories

## Performance Guidelines for New Commands

When implementing new commands:

1. **Measure First**: Establish baseline performance
2. **Profile Hot Paths**: Identify bottlenecks
3. **Optimize Incrementally**: Make small, measurable improvements
4. **Test Regressions**: Ensure optimizations don't break functionality
5. **Document Targets**: Set and document performance targets

### Example Performance Test

```rust
#[test]
fn test_command_performance() {
    let start = std::time::Instant::now();
    // ... execute command
    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 1000,
        "Command took {}ms, target is <1000ms",
        duration.as_millis()
    );
}
```

## Memory Usage

### Targets

- **Command startup**: <10MB
- **Typical command execution**: <50MB
- **Large operations (plan generation)**: <200MB

### Monitoring

Use tools like `valgrind` or `heaptrack` to monitor memory usage:

```bash
# Using valgrind
valgrind --tool=massif cargo run --release -p radium-cli -- status

# Using heaptrack (Linux)
heaptrack cargo run --release -p radium-cli -- status
```

## Best Practices

1. **Lazy Loading**: Load resources only when needed
2. **Caching**: Cache expensive computations and file reads
3. **Batching**: Batch file operations where possible
4. **Async I/O**: Use async file operations for better concurrency
5. **Minimize Allocations**: Reuse buffers and avoid unnecessary allocations
6. **Profile Regularly**: Run benchmarks and profiles regularly
7. **Set Targets**: Define and track performance targets
8. **Test Regressions**: Add performance tests to prevent regressions

## Troubleshooting Slow Commands

1. **Profile the command**: Use `cargo flamegraph` or `perf`
2. **Check file I/O**: Look for excessive file operations
3. **Check network calls**: Verify no unexpected network requests
4. **Check dependencies**: Ensure no slow dependencies
5. **Review algorithms**: Look for inefficient algorithms
6. **Check memory**: Verify no memory leaks or excessive allocations

## Performance Checklist

When reviewing code for performance:

- [ ] File operations use async I/O where appropriate
- [ ] Workspace discovery is cached
- [ ] JSON output is optimized (no unnecessary pretty printing)
- [ ] String allocations are minimized
- [ ] Expensive operations are lazy-loaded
- [ ] Performance tests exist for critical paths
- [ ] Benchmarks are up to date
- [ ] No obvious bottlenecks in hot paths

