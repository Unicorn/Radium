//! Error Context
//!
//! Additional context information for debugging and resolution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Key-value context for errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Context key
    pub key: String,
    /// Context value
    pub value: String,
}

impl ErrorContext {
    /// Create a new context entry
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Builder for creating rich error context
#[derive(Debug, Clone, Default)]
pub struct ContextBuilder {
    entries: Vec<ErrorContext>,
}

impl ContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a context entry
    pub fn add(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries.push(ErrorContext::new(key, value));
        self
    }

    /// Add node ID context
    pub fn node_id(self, id: impl Into<String>) -> Self {
        self.add("node_id", id)
    }

    /// Add field name context
    pub fn field(self, name: impl Into<String>) -> Self {
        self.add("field", name)
    }

    /// Add expected value context
    pub fn expected(self, value: impl Into<String>) -> Self {
        self.add("expected", value)
    }

    /// Add actual value context
    pub fn actual(self, value: impl Into<String>) -> Self {
        self.add("actual", value)
    }

    /// Add variable name context
    pub fn variable(self, name: impl Into<String>) -> Self {
        self.add("variable", name)
    }

    /// Add type information context
    pub fn type_info(self, type_name: impl Into<String>) -> Self {
        self.add("type", type_name)
    }

    /// Add source reference context
    pub fn source(self, source: impl Into<String>) -> Self {
        self.add("source", source)
    }

    /// Add target reference context
    pub fn target(self, target: impl Into<String>) -> Self {
        self.add("target", target)
    }

    /// Add constraint context
    pub fn constraint(self, constraint: impl Into<String>) -> Self {
        self.add("constraint", constraint)
    }

    /// Build the context entries
    pub fn build(self) -> Vec<ErrorContext> {
        self.entries
    }
}

/// Additional context that can be attached to errors for debugging
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticContext {
    /// The workflow ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    /// The workflow name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    /// Current compilation phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
    /// Stack trace (if available)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub stack_trace: Vec<String>,
}

impl DiagnosticContext {
    /// Create a new diagnostic context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set workflow ID
    pub fn with_workflow_id(mut self, id: impl Into<String>) -> Self {
        self.workflow_id = Some(id.into());
        self
    }

    /// Set workflow name
    pub fn with_workflow_name(mut self, name: impl Into<String>) -> Self {
        self.workflow_name = Some(name.into());
        self
    }

    /// Set current phase
    pub fn with_phase(mut self, phase: impl Into<String>) -> Self {
        self.phase = Some(phase.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add stack trace frame
    pub fn with_stack_frame(mut self, frame: impl Into<String>) -> Self {
        self.stack_trace.push(frame.into());
        self
    }

    /// Capture current phase for tracing
    pub fn capture_phase(&mut self, phase: impl Into<String>) {
        self.phase = Some(phase.into());
    }
}

/// Trait for adding context to errors
pub trait WithContext {
    /// Add context to this error
    fn with_context(self, key: impl Into<String>, value: impl Into<String>) -> Self;

    /// Add multiple context entries
    fn with_contexts(self, contexts: Vec<ErrorContext>) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let ctx = ErrorContext::new("field", "name");
        assert_eq!(ctx.key, "field");
        assert_eq!(ctx.value, "name");
    }

    #[test]
    fn test_context_builder() {
        let contexts = ContextBuilder::new()
            .node_id("activity_1")
            .field("timeout")
            .expected("positive integer")
            .actual("-5")
            .build();

        assert_eq!(contexts.len(), 4);
        assert_eq!(contexts[0].key, "node_id");
        assert_eq!(contexts[1].key, "field");
    }

    #[test]
    fn test_diagnostic_context() {
        let ctx = DiagnosticContext::new()
            .with_workflow_id("wf_123")
            .with_workflow_name("Test Workflow")
            .with_phase("validation")
            .with_metadata("compiler_version", "1.0.0");

        assert_eq!(ctx.workflow_id, Some("wf_123".to_string()));
        assert_eq!(ctx.workflow_name, Some("Test Workflow".to_string()));
        assert_eq!(ctx.phase, Some("validation".to_string()));
        assert_eq!(
            ctx.metadata.get("compiler_version"),
            Some(&"1.0.0".to_string())
        );
    }

    #[test]
    fn test_context_builder_chain() {
        let contexts = ContextBuilder::new()
            .source("node_a")
            .target("node_b")
            .type_info("string")
            .constraint("max_length: 100")
            .build();

        assert_eq!(contexts.len(), 4);
    }
}
