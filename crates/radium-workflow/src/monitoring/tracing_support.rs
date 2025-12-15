//! Tracing Support
//!
//! Provides distributed tracing infrastructure for workflow compilation
//! and execution tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Instant;

/// Unique trace identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(pub String);

impl TraceId {
    /// Generate a new trace ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from an existing ID
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique span identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpanId(pub String);

impl SpanId {
    /// Generate a new span ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string()[..16].to_string())
    }

    /// Create from an existing ID
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Default for SpanId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SpanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// Operation completed successfully
    Ok,
    /// Operation had an error
    Error,
    /// Operation was cancelled
    Cancelled,
}

/// A single span in a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Trace ID this span belongs to
    pub trace_id: TraceId,
    /// Unique span ID
    pub span_id: SpanId,
    /// Parent span ID (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<SpanId>,
    /// Operation name
    pub operation: String,
    /// Service name
    pub service: String,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Duration in microseconds
    pub duration_us: u64,
    /// Span status
    pub status: SpanStatus,
    /// Tags/attributes
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub tags: HashMap<String, String>,
    /// Logs/events
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub logs: Vec<SpanLog>,
}

/// A log entry within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLog {
    /// When the log was recorded
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log message
    pub message: String,
    /// Log level
    pub level: LogLevel,
    /// Additional fields
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub fields: HashMap<String, String>,
}

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Builder for creating spans
pub struct SpanBuilder {
    trace_id: TraceId,
    span_id: SpanId,
    parent_span_id: Option<SpanId>,
    operation: String,
    service: String,
    start_time: chrono::DateTime<chrono::Utc>,
    start_instant: Instant,
    tags: HashMap<String, String>,
    logs: Vec<SpanLog>,
}

impl SpanBuilder {
    /// Create a new span builder
    pub fn new(operation: impl Into<String>, service: impl Into<String>) -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            parent_span_id: None,
            operation: operation.into(),
            service: service.into(),
            start_time: chrono::Utc::now(),
            start_instant: Instant::now(),
            tags: HashMap::new(),
            logs: Vec::new(),
        }
    }

    /// Set the trace ID
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = trace_id;
        self
    }

    /// Set the parent span ID
    pub fn with_parent(mut self, parent_id: SpanId) -> Self {
        self.parent_span_id = Some(parent_id);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Add a log entry
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.logs.push(SpanLog {
            timestamp: chrono::Utc::now(),
            message: message.into(),
            level,
            fields: HashMap::new(),
        });
    }

    /// Finish the span with success
    pub fn finish(self) -> Span {
        self.finish_with_status(SpanStatus::Ok)
    }

    /// Finish the span with error
    pub fn finish_error(self) -> Span {
        self.finish_with_status(SpanStatus::Error)
    }

    /// Finish the span with a specific status
    pub fn finish_with_status(self, status: SpanStatus) -> Span {
        Span {
            trace_id: self.trace_id,
            span_id: self.span_id,
            parent_span_id: self.parent_span_id,
            operation: self.operation,
            service: self.service,
            start_time: self.start_time,
            duration_us: self.start_instant.elapsed().as_micros() as u64,
            status,
            tags: self.tags,
            logs: self.logs,
        }
    }
}

/// Trace context for propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Trace ID
    pub trace_id: TraceId,
    /// Current span ID
    pub span_id: SpanId,
    /// Sampling decision
    pub sampled: bool,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new() -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            sampled: true,
        }
    }

    /// Create from existing IDs
    pub fn from_ids(trace_id: TraceId, span_id: SpanId) -> Self {
        Self {
            trace_id,
            span_id,
            sampled: true,
        }
    }

    /// Create a child context
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: SpanId::new(),
            sampled: self.sampled,
        }
    }

    /// Serialize to W3C Trace Context header format
    pub fn to_traceparent(&self) -> String {
        let sampled = if self.sampled { "01" } else { "00" };
        format!("00-{}-{}-{}", self.trace_id.0, self.span_id.0, sampled)
    }

    /// Parse from W3C Trace Context header
    pub fn from_traceparent(header: &str) -> Option<Self> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 || parts[0] != "00" {
            return None;
        }

        Some(Self {
            trace_id: TraceId::from_string(parts[1]),
            span_id: SpanId::from_string(parts[2]),
            sampled: parts[3] == "01",
        })
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple in-memory trace collector
#[derive(Debug, Default)]
pub struct TraceCollector {
    spans: RwLock<Vec<Span>>,
    max_spans: usize,
    dropped_spans: AtomicU64,
}

impl TraceCollector {
    /// Create a new trace collector
    pub fn new(max_spans: usize) -> Self {
        Self {
            spans: RwLock::new(Vec::with_capacity(max_spans)),
            max_spans,
            dropped_spans: AtomicU64::new(0),
        }
    }

    /// Record a span
    pub fn record(&self, span: Span) {
        let mut spans = self.spans.write().unwrap();
        if spans.len() >= self.max_spans {
            spans.remove(0);
            self.dropped_spans.fetch_add(1, Ordering::Relaxed);
        }
        spans.push(span);
    }

    /// Get all spans for a trace
    pub fn get_trace(&self, trace_id: &TraceId) -> Vec<Span> {
        self.spans
            .read()
            .unwrap()
            .iter()
            .filter(|s| &s.trace_id == trace_id)
            .cloned()
            .collect()
    }

    /// Get recent spans
    pub fn recent_spans(&self, limit: usize) -> Vec<Span> {
        let spans = self.spans.read().unwrap();
        spans.iter().rev().take(limit).cloned().collect()
    }

    /// Get dropped span count
    pub fn dropped_count(&self) -> u64 {
        self.dropped_spans.load(Ordering::Relaxed)
    }

    /// Clear all spans
    pub fn clear(&self) {
        self.spans.write().unwrap().clear();
    }

    /// Export spans as JSON
    pub fn export_json(&self) -> String {
        let spans = self.spans.read().unwrap();
        serde_json::to_string_pretty(&*spans).unwrap_or_default()
    }
}

/// Compilation trace for tracking a complete compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationTrace {
    /// Trace ID
    pub trace_id: TraceId,
    /// Workflow ID being compiled
    pub workflow_id: String,
    /// Workflow name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End time (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Total duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Compilation stages
    pub stages: Vec<CompilationStageTrace>,
    /// Final status
    pub status: CompilationTraceStatus,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Status of a compilation trace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompilationTraceStatus {
    /// Compilation in progress
    InProgress,
    /// Compilation succeeded
    Succeeded,
    /// Compilation failed
    Failed,
    /// Compilation was cached (skipped)
    Cached,
}

/// Trace for a single compilation stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationStageTrace {
    /// Stage name
    pub stage: String,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether this stage succeeded
    pub success: bool,
    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
}

impl CompilationTrace {
    /// Create a new compilation trace
    pub fn start(workflow_id: impl Into<String>) -> Self {
        Self {
            trace_id: TraceId::new(),
            workflow_id: workflow_id.into(),
            workflow_name: None,
            start_time: chrono::Utc::now(),
            end_time: None,
            duration_ms: None,
            stages: Vec::new(),
            status: CompilationTraceStatus::InProgress,
            error: None,
        }
    }

    /// Set workflow name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.workflow_name = Some(name.into());
        self
    }

    /// Add a completed stage
    pub fn add_stage(&mut self, stage: CompilationStageTrace) {
        self.stages.push(stage);
    }

    /// Mark as succeeded
    pub fn succeed(&mut self) {
        self.end_time = Some(chrono::Utc::now());
        self.duration_ms = Some(
            (self.end_time.unwrap() - self.start_time).num_milliseconds() as u64,
        );
        self.status = CompilationTraceStatus::Succeeded;
    }

    /// Mark as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.end_time = Some(chrono::Utc::now());
        self.duration_ms = Some(
            (self.end_time.unwrap() - self.start_time).num_milliseconds() as u64,
        );
        self.status = CompilationTraceStatus::Failed;
        self.error = Some(error.into());
    }

    /// Mark as cached
    pub fn cached(&mut self) {
        self.end_time = Some(chrono::Utc::now());
        self.duration_ms = Some(0);
        self.status = CompilationTraceStatus::Cached;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id() {
        let id1 = TraceId::new();
        let id2 = TraceId::new();
        assert_ne!(id1, id2);

        let id3 = TraceId::from_string("test-trace-id");
        assert_eq!(id3.0, "test-trace-id");
    }

    #[test]
    fn test_span_builder() {
        let span = SpanBuilder::new("compile", "workflow-compiler")
            .with_tag("workflow_id", "wf_123")
            .with_tag("version", "1.0.0")
            .finish();

        assert_eq!(span.operation, "compile");
        assert_eq!(span.service, "workflow-compiler");
        assert_eq!(span.status, SpanStatus::Ok);
        assert_eq!(span.tags.get("workflow_id"), Some(&"wf_123".to_string()));
    }

    #[test]
    fn test_span_with_parent() {
        let parent_id = SpanId::new();
        let trace_id = TraceId::new();

        let span = SpanBuilder::new("child_op", "service")
            .with_trace_id(trace_id.clone())
            .with_parent(parent_id.clone())
            .finish();

        assert_eq!(span.trace_id, trace_id);
        assert_eq!(span.parent_span_id, Some(parent_id));
    }

    #[test]
    fn test_trace_context() {
        let ctx = TraceContext::new();
        assert!(ctx.sampled);

        let child = ctx.child();
        assert_eq!(child.trace_id, ctx.trace_id);
        assert_ne!(child.span_id, ctx.span_id);
    }

    #[test]
    fn test_traceparent_format() {
        let ctx = TraceContext::from_ids(
            TraceId::from_string("abc123"),
            SpanId::from_string("def456"),
        );

        let header = ctx.to_traceparent();
        assert!(header.starts_with("00-abc123-def456-01"));

        let parsed = TraceContext::from_traceparent(&header).unwrap();
        assert_eq!(parsed.trace_id.0, "abc123");
        assert_eq!(parsed.span_id.0, "def456");
        assert!(parsed.sampled);
    }

    #[test]
    fn test_trace_collector() {
        let collector = TraceCollector::new(100);

        let trace_id = TraceId::new();
        let span = SpanBuilder::new("op1", "service")
            .with_trace_id(trace_id.clone())
            .finish();
        collector.record(span);

        let spans = collector.get_trace(&trace_id);
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_trace_collector_limit() {
        let collector = TraceCollector::new(2);

        for i in 0..5 {
            let span = SpanBuilder::new(format!("op{}", i), "service").finish();
            collector.record(span);
        }

        let recent = collector.recent_spans(10);
        assert_eq!(recent.len(), 2);
        assert_eq!(collector.dropped_count(), 3);
    }

    #[test]
    fn test_compilation_trace() {
        let mut trace = CompilationTrace::start("wf_123")
            .with_name("Test Workflow");

        trace.add_stage(CompilationStageTrace {
            stage: "parsing".to_string(),
            start_time: chrono::Utc::now(),
            duration_ms: 10,
            success: true,
            metadata: HashMap::new(),
        });

        trace.succeed();

        assert_eq!(trace.status, CompilationTraceStatus::Succeeded);
        assert!(trace.duration_ms.is_some());
        assert_eq!(trace.stages.len(), 1);
    }

    #[test]
    fn test_compilation_trace_failure() {
        let mut trace = CompilationTrace::start("wf_123");
        trace.fail("Validation error");

        assert_eq!(trace.status, CompilationTraceStatus::Failed);
        assert_eq!(trace.error, Some("Validation error".to_string()));
    }
}
