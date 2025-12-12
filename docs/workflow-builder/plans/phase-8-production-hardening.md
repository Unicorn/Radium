# Phase 8: Production Hardening

**Status**: Not Started
**Duration**: 2-3 weeks
**Prerequisites**: Phase 7 (Advanced Features)
**Blocks**: Phase 9

## Overview

Prepare the Rust compiler and generated TypeScript for production deployment. This phase focuses on performance optimization, error handling, monitoring, security hardening, and operational readiness.

## Goals

1. Optimize compiler performance
2. Implement comprehensive error handling
3. Add monitoring and observability
4. Harden security
5. Create deployment pipeline
6. Document operations procedures

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Production Architecture                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                         Load Balancer                            │    │
│  └───────────────────────────────┬─────────────────────────────────┘    │
│                                  │                                       │
│          ┌───────────────────────┼───────────────────────┐              │
│          │                       │                       │              │
│          ▼                       ▼                       ▼              │
│  ┌───────────────┐     ┌───────────────┐     ┌───────────────┐         │
│  │ Rust Compiler │     │ Rust Compiler │     │ Rust Compiler │         │
│  │  Instance 1   │     │  Instance 2   │     │  Instance N   │         │
│  └───────┬───────┘     └───────┬───────┘     └───────┬───────┘         │
│          │                     │                     │                  │
│          └─────────────────────┼─────────────────────┘                  │
│                                │                                        │
│                                ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      Shared Services                             │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │   │
│  │  │ Metrics  │  │ Tracing  │  │ Logging  │  │ Alerting │        │   │
│  │  │Prometheus│  │ Jaeger   │  │   Loki   │  │PagerDuty │        │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Production Readiness Checklist

| Category | Item | Priority | Status |
|----------|------|----------|--------|
| Performance | Compilation < 100ms | P0 | 7.1 |
| Performance | Memory < 256MB | P0 | 7.1 |
| Errors | Structured error responses | P0 | 7.2 |
| Errors | Error recovery | P1 | 7.2 |
| Monitoring | Metrics endpoint | P0 | 7.3 |
| Monitoring | Distributed tracing | P1 | 7.3 |
| Security | Input sanitization | P0 | 7.4 |
| Security | Rate limiting | P0 | 7.4 |
| Deployment | CI/CD pipeline | P0 | 7.5 |
| Deployment | Rollback procedure | P0 | 7.5 |
| Operations | Runbook | P0 | 7.6 |
| Operations | Alerting | P1 | 7.6 |

---

## Tasks

### 7.1 Performance Optimization

**Description**: Optimize compiler performance for production load

**Subtasks**:
- [ ] 7.1.1 Profile compilation pipeline
- [ ] 7.1.2 Implement template caching
- [ ] 7.1.3 Optimize validation passes
- [ ] 7.1.4 Add parallel schema validation
- [ ] 7.1.5 Optimize TypeScript generation
- [ ] 7.1.6 Implement compilation caching
- [ ] 7.1.7 Memory optimization
- [ ] 7.1.8 Create performance benchmarks

**Files to Create**:
```rust
// src/performance/mod.rs
mod cache;
mod profiler;
mod parallel;
mod benchmarks;

pub use cache::*;
pub use profiler::*;
pub use parallel::*;
```

```rust
// src/performance/cache.rs
use lru::LruCache;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::RwLock;

/// Cache for compiled workflow templates
pub struct CompilationCache {
    cache: RwLock<LruCache<WorkflowHash, CachedCompilation>>,
    stats: CacheStats,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowHash {
    /// Hash of the workflow definition
    definition_hash: u64,
    /// Compiler version
    compiler_version: String,
}

#[derive(Debug, Clone)]
pub struct CachedCompilation {
    pub typescript_code: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub compilation_time_ms: u64,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: std::sync::atomic::AtomicU64,
    pub misses: std::sync::atomic::AtomicU64,
    pub evictions: std::sync::atomic::AtomicU64,
}

impl CompilationCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity).expect("capacity must be non-zero")
            )),
            stats: CacheStats::default(),
        }
    }

    pub fn get(&self, hash: &WorkflowHash) -> Option<CachedCompilation> {
        let mut cache = self.cache.write().unwrap();
        if let Some(cached) = cache.get(hash) {
            self.stats.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(cached.clone())
        } else {
            self.stats.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    pub fn put(&self, hash: WorkflowHash, compilation: CachedCompilation) {
        let mut cache = self.cache.write().unwrap();
        if cache.len() == cache.cap().get() {
            self.stats.evictions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        cache.put(hash, compilation);
    }

    pub fn stats(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            hits: self.stats.hits.load(std::sync::atomic::Ordering::Relaxed),
            misses: self.stats.misses.load(std::sync::atomic::Ordering::Relaxed),
            evictions: self.stats.evictions.load(std::sync::atomic::Ordering::Relaxed),
            size: self.cache.read().unwrap().len(),
            capacity: self.cache.read().unwrap().cap().get(),
        }
    }

    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
    pub capacity: usize,
}

impl CacheStatsSnapshot {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}
```

```rust
// src/performance/profiler.rs
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Compilation profiler for performance analysis
pub struct CompilationProfiler {
    stages: Vec<StageProfile>,
    start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct StageProfile {
    pub name: String,
    pub duration: Duration,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfileReport {
    pub total_duration_ms: u64,
    pub stages: Vec<StageReport>,
    pub bottleneck: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StageReport {
    pub name: String,
    pub duration_ms: u64,
    pub percentage: f64,
    pub metadata: HashMap<String, String>,
}

impl CompilationProfiler {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            start_time: Instant::now(),
        }
    }

    pub fn stage<F, T>(&mut self, name: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        self.stages.push(StageProfile {
            name: name.to_string(),
            duration,
            metadata: HashMap::new(),
        });

        result
    }

    pub fn stage_with_metadata<F, T>(
        &mut self,
        name: &str,
        metadata: HashMap<String, String>,
        f: F,
    ) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        self.stages.push(StageProfile {
            name: name.to_string(),
            duration,
            metadata,
        });

        result
    }

    pub fn report(&self) -> ProfileReport {
        let total_duration = self.start_time.elapsed();
        let total_ms = total_duration.as_millis() as u64;

        let stages: Vec<StageReport> = self.stages.iter().map(|s| {
            let duration_ms = s.duration.as_millis() as u64;
            StageReport {
                name: s.name.clone(),
                duration_ms,
                percentage: if total_ms > 0 {
                    (duration_ms as f64 / total_ms as f64) * 100.0
                } else {
                    0.0
                },
                metadata: s.metadata.clone(),
            }
        }).collect();

        let bottleneck = stages.iter()
            .max_by_key(|s| s.duration_ms)
            .map(|s| s.name.clone());

        ProfileReport {
            total_duration_ms: total_ms,
            stages,
            bottleneck,
        }
    }
}

impl Default for CompilationProfiler {
    fn default() -> Self {
        Self::new()
    }
}
```

**Benchmarks**:
```rust
// benches/compilation_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use workflow_compiler::*;

fn benchmark_simple_workflow(c: &mut Criterion) {
    let workflow = create_simple_workflow();

    c.bench_function("compile_simple_workflow", |b| {
        b.iter(|| {
            let compiler = WorkflowCompiler::new();
            compiler.compile(black_box(&workflow))
        })
    });
}

fn benchmark_complex_workflow(c: &mut Criterion) {
    let workflow = create_complex_workflow_50_nodes();

    c.bench_function("compile_complex_workflow_50_nodes", |b| {
        b.iter(|| {
            let compiler = WorkflowCompiler::new();
            compiler.compile(black_box(&workflow))
        })
    });
}

fn benchmark_with_cache(c: &mut Criterion) {
    let cache = CompilationCache::new(100);
    let workflow = create_simple_workflow();

    // Warm up cache
    let compiler = WorkflowCompiler::new();
    let compiled = compiler.compile(&workflow).unwrap();
    cache.put(workflow.hash(), compiled);

    c.bench_function("compile_with_cache_hit", |b| {
        b.iter(|| {
            cache.get(black_box(&workflow.hash()))
        })
    });
}

criterion_group!(
    benches,
    benchmark_simple_workflow,
    benchmark_complex_workflow,
    benchmark_with_cache
);
criterion_main!(benches);
```

**Performance Targets**:
| Metric | Target | Measurement |
|--------|--------|-------------|
| Simple workflow compilation | < 50ms | p95 |
| Complex workflow (50 nodes) | < 100ms | p95 |
| Memory per compilation | < 50MB | peak |
| Cache hit rate | > 80% | average |

**Acceptance Criteria**:
- [ ] Compilation times meet targets
- [ ] Memory usage within limits
- [ ] Cache functioning correctly
- [ ] Benchmarks automated in CI

---

### 7.2 Error Handling

**Description**: Implement comprehensive, user-friendly error handling

**Subtasks**:
- [ ] 7.2.1 Create structured error types
- [ ] 7.2.2 Implement error codes
- [ ] 7.2.3 Add error context and suggestions
- [ ] 7.2.4 Create error recovery strategies
- [ ] 7.2.5 Implement graceful degradation
- [ ] 7.2.6 Add error serialization
- [ ] 7.2.7 Create error documentation
- [ ] 7.2.8 Test all error paths

**Files to Create**:
```rust
// src/errors/mod.rs
mod types;
mod codes;
mod recovery;
mod serialization;

pub use types::*;
pub use codes::*;
pub use recovery::*;
```

```rust
// src/errors/types.rs
use serde::{Deserialize, Serialize};
use std::fmt;

/// Structured error for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerError {
    /// Error code (e.g., "WF_VALIDATION_001")
    pub code: String,

    /// Human-readable message
    pub message: String,

    /// Error category
    pub category: ErrorCategory,

    /// Severity level
    pub severity: ErrorSeverity,

    /// Location in workflow (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ErrorLocation>,

    /// Suggested fixes
    #[serde(default)]
    pub suggestions: Vec<String>,

    /// Related documentation links
    #[serde(default)]
    pub docs_links: Vec<String>,

    /// Underlying cause (for chained errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Box<CompilerError>>,

    /// Additional context
    #[serde(default)]
    pub context: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCategory {
    Validation,
    Schema,
    Compilation,
    TypeSafety,
    Runtime,
    Configuration,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// Node ID where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    /// Field name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,

    /// Line number in generated code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,

    /// Column number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,

    /// Code snippet with error highlighted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

impl CompilerError {
    pub fn validation(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: format!("WF_VALIDATION_{}", code),
            message: message.into(),
            category: ErrorCategory::Validation,
            severity: ErrorSeverity::Error,
            location: None,
            suggestions: Vec::new(),
            docs_links: Vec::new(),
            cause: None,
            context: std::collections::HashMap::new(),
        }
    }

    pub fn schema(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: format!("WF_SCHEMA_{}", code),
            message: message.into(),
            category: ErrorCategory::Schema,
            severity: ErrorSeverity::Error,
            location: None,
            suggestions: Vec::new(),
            docs_links: Vec::new(),
            cause: None,
            context: std::collections::HashMap::new(),
        }
    }

    pub fn type_safety(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: format!("WF_TYPE_{}", code),
            message: message.into(),
            category: ErrorCategory::TypeSafety,
            severity: ErrorSeverity::Error,
            location: None,
            suggestions: Vec::new(),
            docs_links: Vec::new(),
            cause: None,
            context: std::collections::HashMap::new(),
        }
    }

    pub fn with_location(mut self, location: ErrorLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn with_docs(mut self, url: impl Into<String>) -> Self {
        self.docs_links.push(url.into());
        self
    }

    pub fn with_cause(mut self, cause: CompilerError) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(self.severity, ErrorSeverity::Warning | ErrorSeverity::Info)
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;

        if let Some(loc) = &self.location {
            if let Some(node_id) = &loc.node_id {
                write!(f, " (at node: {})", node_id)?;
            }
            if let Some(field) = &loc.field {
                write!(f, " (field: {})", field)?;
            }
        }

        if !self.suggestions.is_empty() {
            write!(f, "\nSuggestions:")?;
            for suggestion in &self.suggestions {
                write!(f, "\n  - {}", suggestion)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for CompilerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// Result type for compiler operations
pub type CompilerResult<T> = Result<T, CompilerError>;

/// Collection of errors (for batch validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCollection {
    pub errors: Vec<CompilerError>,
    pub warnings: Vec<CompilerError>,
}

impl ErrorCollection {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: CompilerError) {
        if error.severity == ErrorSeverity::Error {
            self.errors.push(error);
        } else {
            self.warnings.push(error);
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    pub fn into_result<T>(self, value: T) -> CompilerResult<T> {
        if self.has_errors() {
            Err(CompilerError {
                code: "WF_MULTIPLE_ERRORS".to_string(),
                message: format!("{} error(s), {} warning(s)", self.errors.len(), self.warnings.len()),
                category: ErrorCategory::Validation,
                severity: ErrorSeverity::Error,
                location: None,
                suggestions: Vec::new(),
                docs_links: Vec::new(),
                cause: self.errors.into_iter().next().map(Box::new),
                context: std::collections::HashMap::new(),
            })
        } else {
            Ok(value)
        }
    }
}

impl Default for ErrorCollection {
    fn default() -> Self {
        Self::new()
    }
}
```

```rust
// src/errors/codes.rs

/// Error codes and their documentation
pub struct ErrorCode {
    pub code: &'static str,
    pub description: &'static str,
    pub suggestions: &'static [&'static str],
    pub docs_url: &'static str,
}

pub const ERROR_CODES: &[ErrorCode] = &[
    // Validation Errors
    ErrorCode {
        code: "WF_VALIDATION_001",
        description: "Required field is missing",
        suggestions: &["Check that all required fields are provided"],
        docs_url: "/docs/errors/validation-001",
    },
    ErrorCode {
        code: "WF_VALIDATION_002",
        description: "Invalid field value",
        suggestions: &["Check the allowed values for this field", "Ensure the value type matches the schema"],
        docs_url: "/docs/errors/validation-002",
    },
    ErrorCode {
        code: "WF_VALIDATION_003",
        description: "Circular dependency detected",
        suggestions: &["Review the workflow graph for loops", "Use proper control flow components for iterations"],
        docs_url: "/docs/errors/validation-003",
    },

    // Schema Errors
    ErrorCode {
        code: "WF_SCHEMA_001",
        description: "Unknown component type",
        suggestions: &["Check available component types", "Ensure the component is registered"],
        docs_url: "/docs/errors/schema-001",
    },
    ErrorCode {
        code: "WF_SCHEMA_002",
        description: "Invalid schema definition",
        suggestions: &["Review the schema structure", "Ensure all required schema fields are present"],
        docs_url: "/docs/errors/schema-002",
    },

    // Type Safety Errors
    ErrorCode {
        code: "WF_TYPE_001",
        description: "Type mismatch",
        suggestions: &["Check input and output types match", "Add type conversion if needed"],
        docs_url: "/docs/errors/type-001",
    },
    ErrorCode {
        code: "WF_TYPE_002",
        description: "Incompatible connection types",
        suggestions: &["Ensure source output matches target input type", "Add type adapter component"],
        docs_url: "/docs/errors/type-002",
    },

    // Compilation Errors
    ErrorCode {
        code: "WF_COMPILE_001",
        description: "Template rendering failed",
        suggestions: &["Check template syntax", "Ensure all variables are defined"],
        docs_url: "/docs/errors/compile-001",
    },
    ErrorCode {
        code: "WF_COMPILE_002",
        description: "TypeScript compilation failed",
        suggestions: &["Review generated TypeScript for errors", "Check type definitions"],
        docs_url: "/docs/errors/compile-002",
    },
];

pub fn get_error_code(code: &str) -> Option<&'static ErrorCode> {
    ERROR_CODES.iter().find(|e| e.code == code)
}
```

**Acceptance Criteria**:
- [ ] All errors have codes
- [ ] Errors include suggestions
- [ ] Error documentation complete
- [ ] Recovery strategies implemented

---

### 7.3 Monitoring & Observability

**Description**: Add comprehensive monitoring and observability

**Subtasks**:
- [ ] 7.3.1 Implement Prometheus metrics
- [ ] 7.3.2 Add distributed tracing (OpenTelemetry)
- [ ] 7.3.3 Implement structured logging
- [ ] 7.3.4 Create health check endpoints
- [ ] 7.3.5 Add performance metrics
- [ ] 7.3.6 Create dashboard templates
- [ ] 7.3.7 Set up alerting rules
- [ ] 7.3.8 Document observability stack

**Files to Create**:
```rust
// src/observability/mod.rs
mod metrics;
mod tracing;
mod logging;
mod health;

pub use metrics::*;
pub use tracing::*;
pub use logging::*;
pub use health::*;
```

```rust
// src/observability/metrics.rs
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, Registry,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Compilation metrics
    pub static ref COMPILATIONS_TOTAL: CounterVec = CounterVec::new(
        prometheus::Opts::new("workflow_compilations_total", "Total number of workflow compilations"),
        &["status", "workflow_type"]
    ).unwrap();

    pub static ref COMPILATION_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new("workflow_compilation_duration_seconds", "Compilation duration in seconds")
            .buckets(vec![0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
        &["workflow_type"]
    ).unwrap();

    // Validation metrics
    pub static ref VALIDATIONS_TOTAL: CounterVec = CounterVec::new(
        prometheus::Opts::new("workflow_validations_total", "Total number of workflow validations"),
        &["status"]
    ).unwrap();

    pub static ref VALIDATION_ERRORS: CounterVec = CounterVec::new(
        prometheus::Opts::new("workflow_validation_errors_total", "Total validation errors by type"),
        &["error_code"]
    ).unwrap();

    // Cache metrics
    pub static ref CACHE_HITS: Counter = Counter::new(
        "workflow_cache_hits_total", "Total cache hits"
    ).unwrap();

    pub static ref CACHE_MISSES: Counter = Counter::new(
        "workflow_cache_misses_total", "Total cache misses"
    ).unwrap();

    pub static ref CACHE_SIZE: Gauge = Gauge::new(
        "workflow_cache_size", "Current cache size"
    ).unwrap();

    // Resource metrics
    pub static ref ACTIVE_COMPILATIONS: Gauge = Gauge::new(
        "workflow_active_compilations", "Currently active compilations"
    ).unwrap();

    pub static ref MEMORY_USAGE: Gauge = Gauge::new(
        "workflow_compiler_memory_bytes", "Compiler memory usage in bytes"
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(COMPILATIONS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(COMPILATION_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(VALIDATIONS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(VALIDATION_ERRORS.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_HITS.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_MISSES.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_SIZE.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_COMPILATIONS.clone())).unwrap();
    REGISTRY.register(Box::new(MEMORY_USAGE.clone())).unwrap();
}

/// Record a successful compilation
pub fn record_compilation_success(workflow_type: &str, duration_seconds: f64) {
    COMPILATIONS_TOTAL.with_label_values(&["success", workflow_type]).inc();
    COMPILATION_DURATION.with_label_values(&[workflow_type]).observe(duration_seconds);
}

/// Record a failed compilation
pub fn record_compilation_failure(workflow_type: &str, duration_seconds: f64) {
    COMPILATIONS_TOTAL.with_label_values(&["failure", workflow_type]).inc();
    COMPILATION_DURATION.with_label_values(&[workflow_type]).observe(duration_seconds);
}

/// Record a validation error
pub fn record_validation_error(error_code: &str) {
    VALIDATION_ERRORS.with_label_values(&[error_code]).inc();
}
```

```rust
// src/observability/health.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: Vec<ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl HealthCheck {
    pub fn check() -> Self {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();

        // Check template engine
        checks.push(check_template_engine());

        // Check TypeScript compiler availability
        checks.push(check_typescript_compiler());

        // Check cache
        checks.push(check_cache());

        let overall_status = if checks.iter().all(|c| c.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if checks.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };

        Self {
            status: overall_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: get_uptime_seconds(),
            checks,
        }
    }
}

fn check_template_engine() -> ComponentHealth {
    // Try to render a simple template
    match render_test_template() {
        Ok(latency) => ComponentHealth {
            name: "template_engine".to_string(),
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: Some(latency),
        },
        Err(e) => ComponentHealth {
            name: "template_engine".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(e.to_string()),
            latency_ms: None,
        },
    }
}

fn check_typescript_compiler() -> ComponentHealth {
    let start = std::time::Instant::now();
    let result = std::process::Command::new("npx")
        .args(["tsc", "--version"])
        .output();

    match result {
        Ok(output) if output.status.success() => ComponentHealth {
            name: "typescript_compiler".to_string(),
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: Some(start.elapsed().as_millis() as u64),
        },
        Ok(_) => ComponentHealth {
            name: "typescript_compiler".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some("tsc returned non-zero exit code".to_string()),
            latency_ms: None,
        },
        Err(e) => ComponentHealth {
            name: "typescript_compiler".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(e.to_string()),
            latency_ms: None,
        },
    }
}

fn check_cache() -> ComponentHealth {
    // Check cache is responsive
    ComponentHealth {
        name: "compilation_cache".to_string(),
        status: HealthStatus::Healthy,
        message: None,
        latency_ms: Some(1),
    }
}

fn render_test_template() -> Result<u64, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    // Render simple template
    Ok(start.elapsed().as_millis() as u64)
}

fn get_uptime_seconds() -> u64 {
    // Return server uptime
    0 // Placeholder
}
```

**Metrics Endpoints**:
```
GET /metrics         - Prometheus metrics
GET /health          - Health check
GET /health/live     - Liveness probe (Kubernetes)
GET /health/ready    - Readiness probe (Kubernetes)
```

**Acceptance Criteria**:
- [ ] Prometheus metrics exposed
- [ ] Health checks implemented
- [ ] Tracing spans created
- [ ] Dashboard templates ready

---

### 7.4 Security Hardening

**Description**: Implement security best practices

**Subtasks**:
- [ ] 7.4.1 Implement input sanitization
- [ ] 7.4.2 Add rate limiting
- [ ] 7.4.3 Implement request validation
- [ ] 7.4.4 Add audit logging
- [ ] 7.4.5 Secure generated code
- [ ] 7.4.6 Implement secrets handling
- [ ] 7.4.7 Security review
- [ ] 7.4.8 Document security practices

**Files to Create**:
```rust
// src/security/mod.rs
mod sanitization;
mod rate_limiting;
mod audit;
mod validation;

pub use sanitization::*;
pub use rate_limiting::*;
pub use audit::*;
pub use validation::*;
```

```rust
// src/security/sanitization.rs
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Patterns for dangerous content
    static ref SCRIPT_TAG: Regex = Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap();
    static ref EVENT_HANDLER: Regex = Regex::new(r"(?i)\bon\w+\s*=").unwrap();
    static ref JAVASCRIPT_PROTOCOL: Regex = Regex::new(r"(?i)javascript:").unwrap();
    static ref SQL_INJECTION: Regex = Regex::new(r"(?i)(union\s+select|drop\s+table|delete\s+from|insert\s+into)").unwrap();
    static ref PATH_TRAVERSAL: Regex = Regex::new(r"\.\./|\.\.\\").unwrap();
    static ref SHELL_INJECTION: Regex = Regex::new(r"[;&|`$]").unwrap();
}

/// Sanitization result
#[derive(Debug)]
pub struct SanitizationResult {
    pub sanitized: String,
    pub modifications: Vec<SanitizationModification>,
}

#[derive(Debug)]
pub struct SanitizationModification {
    pub position: usize,
    pub original: String,
    pub reason: &'static str,
}

/// Sanitize user input for workflow definitions
pub fn sanitize_workflow_input(input: &str) -> SanitizationResult {
    let mut result = input.to_string();
    let mut modifications = Vec::new();

    // Remove script tags
    if let Some(m) = SCRIPT_TAG.find(&result) {
        modifications.push(SanitizationModification {
            position: m.start(),
            original: m.as_str().to_string(),
            reason: "Removed script tag",
        });
        result = SCRIPT_TAG.replace_all(&result, "").to_string();
    }

    // Remove event handlers
    if let Some(m) = EVENT_HANDLER.find(&result) {
        modifications.push(SanitizationModification {
            position: m.start(),
            original: m.as_str().to_string(),
            reason: "Removed event handler",
        });
        result = EVENT_HANDLER.replace_all(&result, "").to_string();
    }

    SanitizationResult {
        sanitized: result,
        modifications,
    }
}

/// Validate identifier names
pub fn validate_identifier(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Identifier cannot be empty".to_string());
    }

    if name.len() > 128 {
        return Err("Identifier too long (max 128 characters)".to_string());
    }

    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return Err("Identifier must start with letter or underscore".to_string());
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Identifier can only contain letters, numbers, and underscores".to_string());
    }

    // Check for reserved words
    const RESERVED: &[&str] = &[
        "break", "case", "catch", "class", "const", "continue", "debugger",
        "default", "delete", "do", "else", "enum", "export", "extends",
        "false", "finally", "for", "function", "if", "import", "in",
        "instanceof", "new", "null", "return", "super", "switch", "this",
        "throw", "true", "try", "typeof", "var", "void", "while", "with",
        "yield", "let", "static", "implements", "interface", "package",
        "private", "protected", "public", "await", "async",
    ];

    if RESERVED.contains(&name.to_lowercase().as_str()) {
        return Err(format!("'{}' is a reserved word", name));
    }

    Ok(())
}

/// Check for potential code injection
pub fn check_code_injection(code: &str) -> Result<(), Vec<String>> {
    let mut issues = Vec::new();

    if SQL_INJECTION.is_match(code) {
        issues.push("Potential SQL injection pattern detected".to_string());
    }

    if PATH_TRAVERSAL.is_match(code) {
        issues.push("Path traversal pattern detected".to_string());
    }

    if SHELL_INJECTION.is_match(code) {
        issues.push("Shell metacharacters detected".to_string());
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}
```

```rust
// src/security/rate_limiting.rs
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Token bucket rate limiter
pub struct RateLimiter {
    buckets: RwLock<HashMap<String, TokenBucket>>,
    config: RateLimitConfig,
}

pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Window duration
    pub window: Duration,
    /// Burst allowance
    pub burst: u32,
}

struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    max_tokens: f64,
    refill_rate: f64,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            buckets: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Check if request is allowed
    pub fn check(&self, key: &str) -> RateLimitResult {
        let mut buckets = self.buckets.write().unwrap();

        let bucket = buckets.entry(key.to_string()).or_insert_with(|| {
            TokenBucket {
                tokens: self.config.max_requests as f64 + self.config.burst as f64,
                last_update: Instant::now(),
                max_tokens: self.config.max_requests as f64 + self.config.burst as f64,
                refill_rate: self.config.max_requests as f64 / self.config.window.as_secs_f64(),
            }
        });

        // Refill tokens
        let now = Instant::now();
        let elapsed = now.duration_since(bucket.last_update).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * bucket.refill_rate).min(bucket.max_tokens);
        bucket.last_update = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            RateLimitResult::Allowed {
                remaining: bucket.tokens as u32,
                reset_in: Duration::from_secs_f64((bucket.max_tokens - bucket.tokens) / bucket.refill_rate),
            }
        } else {
            RateLimitResult::Limited {
                retry_after: Duration::from_secs_f64((1.0 - bucket.tokens) / bucket.refill_rate),
            }
        }
    }
}

pub enum RateLimitResult {
    Allowed {
        remaining: u32,
        reset_in: Duration,
    },
    Limited {
        retry_after: Duration,
    },
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            burst: 20,
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Input sanitization working
- [ ] Rate limiting implemented
- [ ] Audit logging enabled
- [ ] Security review completed

---

### 7.5 Deployment Pipeline

**Description**: Create production deployment infrastructure

**Subtasks**:
- [ ] 7.5.1 Create Docker image
- [ ] 7.5.2 Create Kubernetes manifests
- [ ] 7.5.3 Set up CI/CD pipeline
- [ ] 7.5.4 Create blue-green deployment
- [ ] 7.5.5 Implement rollback procedure
- [ ] 7.5.6 Set up canary releases
- [ ] 7.5.7 Create deployment documentation
- [ ] 7.5.8 Test deployment process

**Files to Create**:
```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install TypeScript
RUN npm install -g typescript

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/workflow-compiler .
COPY templates ./templates

# Create non-root user
RUN useradd -r -s /bin/false compiler
USER compiler

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["./workflow-compiler"]
```

```yaml
# kubernetes/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: workflow-compiler
  labels:
    app: workflow-compiler
spec:
  replicas: 3
  selector:
    matchLabels:
      app: workflow-compiler
  template:
    metadata:
      labels:
        app: workflow-compiler
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      containers:
        - name: workflow-compiler
          image: workflow-compiler:latest
          ports:
            - containerPort: 8080
          resources:
            requests:
              memory: "128Mi"
              cpu: "100m"
            limits:
              memory: "512Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /health/live
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5
          env:
            - name: RUST_LOG
              value: "info"
            - name: CACHE_SIZE
              value: "1000"
---
apiVersion: v1
kind: Service
metadata:
  name: workflow-compiler
spec:
  selector:
    app: workflow-compiler
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: workflow-compiler
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: workflow-compiler
  minReplicas: 2
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
```

**Acceptance Criteria**:
- [ ] Docker image builds
- [ ] Kubernetes deployment works
- [ ] CI/CD pipeline functional
- [ ] Rollback procedure tested

---

### 7.6 Operations Procedures

**Description**: Document operational procedures

**Subtasks**:
- [ ] 7.6.1 Create runbook
- [ ] 7.6.2 Document troubleshooting guide
- [ ] 7.6.3 Create incident response procedures
- [ ] 7.6.4 Document backup/recovery
- [ ] 7.6.5 Create performance tuning guide
- [ ] 7.6.6 Document scaling procedures
- [ ] 7.6.7 Create on-call guide
- [ ] 7.6.8 Train operations team

**Runbook Structure**:
```markdown
# Workflow Compiler Operations Runbook

## Quick Reference

### Key URLs
- Metrics: https://grafana.example.com/d/workflow-compiler
- Logs: https://kibana.example.com/workflow-compiler
- Alerts: https://pagerduty.example.com/services/compiler

### Key Commands
```bash
# Check health
curl http://compiler.internal/health

# View logs
kubectl logs -l app=workflow-compiler -f

# Scale up
kubectl scale deployment workflow-compiler --replicas=5

# Rollback
kubectl rollout undo deployment/workflow-compiler
```

## Common Issues

### High Latency
1. Check CPU usage in Grafana
2. Check cache hit rate
3. Scale up if needed

### Compilation Failures
1. Check error logs
2. Verify TypeScript compiler available
3. Check input validation

### Out of Memory
1. Check memory metrics
2. Clear compilation cache
3. Restart pods if needed

## Escalation
- L1: On-call engineer
- L2: Platform team
- L3: Architecture team
```

**Acceptance Criteria**:
- [ ] Runbook complete
- [ ] Troubleshooting documented
- [ ] Team trained
- [ ] Procedures tested

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| P95 compilation latency | < 100ms | Prometheus |
| Availability | 99.9% | Uptime monitoring |
| Error rate | < 0.1% | Prometheus |
| Cache hit rate | > 80% | Prometheus |
| Memory per instance | < 512MB | Kubernetes metrics |

---

## Files to Create

```
packages/workflow-builder/
  rust-compiler/
    src/
      performance/
        mod.rs
        cache.rs
        profiler.rs
        parallel.rs
      errors/
        mod.rs
        types.rs
        codes.rs
        recovery.rs
      observability/
        mod.rs
        metrics.rs
        tracing.rs
        logging.rs
        health.rs
      security/
        mod.rs
        sanitization.rs
        rate_limiting.rs
        audit.rs
    benches/
      compilation_benchmark.rs
    Dockerfile
    kubernetes/
      deployment.yaml
      service.yaml
      hpa.yaml
      configmap.yaml
    docs/
      runbook.md
      troubleshooting.md
      security.md
```

---

## Checklist

Before marking Phase 8 complete:

- [ ] Performance targets met
- [ ] Error handling comprehensive
- [ ] Monitoring in place
- [ ] Security hardened
- [ ] Deployment pipeline working
- [ ] Operations procedures documented
- [ ] Team trained
- [ ] Load testing passed
- [ ] Security review completed
- [ ] Production sign-off obtained
