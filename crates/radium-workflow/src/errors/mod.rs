//! Structured Error Handling Module
//!
//! Provides a comprehensive error system with:
//! - Unique error codes for each error type
//! - Categorized errors (validation, compilation, runtime)
//! - Actionable suggestions for resolution
//! - Rich context for debugging

mod codes;
mod context;
mod suggestions;

pub use codes::*;
pub use context::*;
pub use suggestions::*;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Errors that prevent compilation/execution
    Error,
    /// Issues that should be addressed but don't block
    Warning,
    /// Informational messages for potential improvements
    Info,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Info => write!(f, "INFO"),
        }
    }
}

/// Error categories for grouping related errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Schema validation errors
    Validation,
    /// TypeScript compilation errors
    Compilation,
    /// Workflow graph structure errors
    Graph,
    /// Type checking errors
    Type,
    /// Runtime/execution errors
    Runtime,
    /// Configuration errors
    Configuration,
    /// I/O and file system errors
    IO,
    /// Internal/unexpected errors
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Validation => write!(f, "Validation"),
            ErrorCategory::Compilation => write!(f, "Compilation"),
            ErrorCategory::Graph => write!(f, "Graph"),
            ErrorCategory::Type => write!(f, "Type"),
            ErrorCategory::Runtime => write!(f, "Runtime"),
            ErrorCategory::Configuration => write!(f, "Configuration"),
            ErrorCategory::IO => write!(f, "I/O"),
            ErrorCategory::Internal => write!(f, "Internal"),
        }
    }
}

/// A structured workflow error with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    /// Unique error code
    pub code: ErrorCode,
    /// Error message
    pub message: String,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// Source location if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ErrorLocation>,
    /// Additional context
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub context: Vec<ErrorContext>,
    /// Suggestions for resolution
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub suggestions: Vec<String>,
    /// Related errors (for error chaining)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related: Vec<Box<WorkflowError>>,
}

impl WorkflowError {
    /// Create a new workflow error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            severity: code.severity(),
            category: code.category(),
            location: None,
            context: vec![],
            suggestions: code.suggestions(),
            related: vec![],
        }
    }

    /// Create a validation error
    pub fn validation(code: ErrorCode, message: impl Into<String>) -> Self {
        debug_assert!(code.category() == ErrorCategory::Validation);
        Self::new(code, message)
    }

    /// Create a compilation error
    pub fn compilation(code: ErrorCode, message: impl Into<String>) -> Self {
        debug_assert!(code.category() == ErrorCategory::Compilation);
        Self::new(code, message)
    }

    /// Create a graph error
    pub fn graph(code: ErrorCode, message: impl Into<String>) -> Self {
        debug_assert!(code.category() == ErrorCategory::Graph);
        Self::new(code, message)
    }

    /// Create a type error
    pub fn type_error(code: ErrorCode, message: impl Into<String>) -> Self {
        debug_assert!(code.category() == ErrorCategory::Type);
        Self::new(code, message)
    }

    /// Add source location
    pub fn with_location(mut self, location: ErrorLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push(ErrorContext::new(key, value));
        self
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// Add a related error
    pub fn with_related(mut self, error: WorkflowError) -> Self {
        self.related.push(Box::new(error));
        self
    }

    /// Get the error code as a string (e.g., "WF-V001")
    pub fn code_string(&self) -> String {
        self.code.to_string()
    }

    /// Check if this is a blocking error
    pub fn is_blocking(&self) -> bool {
        self.severity == ErrorSeverity::Error
    }

    /// Format the error for display
    pub fn format_display(&self) -> String {
        let mut output = format!(
            "[{}] {}: {}\n",
            self.code,
            self.severity,
            self.message
        );

        if let Some(loc) = &self.location {
            output.push_str(&format!("  at {}\n", loc));
        }

        if !self.context.is_empty() {
            output.push_str("  Context:\n");
            for ctx in &self.context {
                output.push_str(&format!("    {}: {}\n", ctx.key, ctx.value));
            }
        }

        if !self.suggestions.is_empty() {
            output.push_str("  Suggestions:\n");
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                output.push_str(&format!("    {}. {}\n", i + 1, suggestion));
            }
        }

        if !self.related.is_empty() {
            output.push_str("  Related errors:\n");
            for related in &self.related {
                output.push_str(&format!("    - [{}] {}\n", related.code, related.message));
            }
        }

        output
    }
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for WorkflowError {}

/// Error location in source files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// File path (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Node ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// Line number (1-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Column number (1-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

impl ErrorLocation {
    /// Create a location from a node ID
    pub fn node(node_id: impl Into<String>) -> Self {
        Self {
            file: None,
            node_id: Some(node_id.into()),
            line: None,
            column: None,
        }
    }

    /// Create a location from a file
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            file: Some(path.into()),
            node_id: None,
            line: None,
            column: None,
        }
    }

    /// Create a location from file and line
    pub fn file_line(path: impl Into<String>, line: usize) -> Self {
        Self {
            file: Some(path.into()),
            node_id: None,
            line: Some(line),
            column: None,
        }
    }

    /// Create a location from file, line, and column
    pub fn file_line_column(path: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            file: Some(path.into()),
            node_id: None,
            line: Some(line),
            column: Some(column),
        }
    }

    /// Add a node ID
    pub fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }
}

impl fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.file, &self.node_id, self.line, self.column) {
            (Some(file), Some(node), Some(line), Some(col)) => {
                write!(f, "{}:{}:{} (node: {})", file, line, col, node)
            }
            (Some(file), Some(node), Some(line), None) => {
                write!(f, "{}:{} (node: {})", file, line, node)
            }
            (Some(file), None, Some(line), Some(col)) => {
                write!(f, "{}:{}:{}", file, line, col)
            }
            (Some(file), None, Some(line), None) => {
                write!(f, "{}:{}", file, line)
            }
            (Some(file), Some(node), None, _) => {
                write!(f, "{} (node: {})", file, node)
            }
            (Some(file), None, None, Some(col)) => {
                write!(f, "{}:?:{}", file, col)
            }
            (Some(file), None, None, None) => write!(f, "{}", file),
            (None, Some(node), _, _) => write!(f, "node: {}", node),
            (None, None, _, _) => write!(f, "<unknown>"),
        }
    }
}

/// Collection of workflow errors
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowErrors {
    errors: Vec<WorkflowError>,
}

impl WorkflowErrors {
    /// Create an empty error collection
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    /// Add an error
    pub fn push(&mut self, error: WorkflowError) {
        self.errors.push(error);
    }

    /// Check if there are any blocking errors
    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.is_blocking())
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.errors
            .iter()
            .any(|e| e.severity == ErrorSeverity::Warning)
    }

    /// Get all blocking errors
    pub fn errors(&self) -> Vec<&WorkflowError> {
        self.errors.iter().filter(|e| e.is_blocking()).collect()
    }

    /// Get all warnings
    pub fn warnings(&self) -> Vec<&WorkflowError> {
        self.errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Warning)
            .collect()
    }

    /// Get all errors and warnings
    pub fn all(&self) -> &[WorkflowError] {
        &self.errors
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.iter().filter(|e| e.is_blocking()).count()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Warning)
            .count()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get total count
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Format all errors for display
    pub fn format_display(&self) -> String {
        let mut output = String::new();

        if self.is_empty() {
            return "No errors".to_string();
        }

        output.push_str(&format!(
            "Found {} error(s) and {} warning(s):\n\n",
            self.error_count(),
            self.warning_count()
        ));

        for error in &self.errors {
            output.push_str(&error.format_display());
            output.push('\n');
        }

        output
    }

    /// Merge another collection into this one
    pub fn merge(&mut self, other: WorkflowErrors) {
        self.errors.extend(other.errors);
    }

    /// Convert to a result - Ok if no blocking errors, Err otherwise
    pub fn into_result<T>(self, value: T) -> Result<T, Self> {
        if self.has_errors() {
            Err(self)
        } else {
            Ok(value)
        }
    }
}

impl IntoIterator for WorkflowErrors {
    type Item = WorkflowError;
    type IntoIter = std::vec::IntoIter<WorkflowError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a> IntoIterator for &'a WorkflowErrors {
    type Item = &'a WorkflowError;
    type IntoIter = std::slice::Iter<'a, WorkflowError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}

impl From<WorkflowError> for WorkflowErrors {
    fn from(error: WorkflowError) -> Self {
        Self {
            errors: vec![error],
        }
    }
}

impl fmt::Display for WorkflowErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_display())
    }
}

impl std::error::Error for WorkflowErrors {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_error_creation() {
        let error = WorkflowError::new(
            ErrorCode::ValidationMissingRequiredField,
            "Field 'name' is required",
        );

        assert_eq!(error.code, ErrorCode::ValidationMissingRequiredField);
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert_eq!(error.category, ErrorCategory::Validation);
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_workflow_error_with_context() {
        let error = WorkflowError::new(
            ErrorCode::ValidationMissingRequiredField,
            "Field 'name' is required",
        )
        .with_context("field", "name")
        .with_context("node", "activity_1")
        .with_location(ErrorLocation::node("activity_1"));

        assert_eq!(error.context.len(), 2);
        assert!(error.location.is_some());
    }

    #[test]
    fn test_error_location_display() {
        let loc = ErrorLocation::file_line_column("workflow.json", 10, 5);
        assert_eq!(loc.to_string(), "workflow.json:10:5");

        let loc = ErrorLocation::node("activity_1");
        assert_eq!(loc.to_string(), "node: activity_1");
    }

    #[test]
    fn test_workflow_errors_collection() {
        let mut errors = WorkflowErrors::new();

        errors.push(WorkflowError::new(
            ErrorCode::ValidationMissingRequiredField,
            "Missing field",
        ));
        errors.push(WorkflowError::new(
            ErrorCode::ValidationUnusedVariable,
            "Unused variable",
        ));

        assert_eq!(errors.len(), 2);
        assert_eq!(errors.error_count(), 1);
        assert_eq!(errors.warning_count(), 1);
        assert!(errors.has_errors());
        assert!(errors.has_warnings());
    }

    #[test]
    fn test_error_display_format() {
        let error = WorkflowError::new(
            ErrorCode::ValidationMissingRequiredField,
            "Field 'name' is required",
        )
        .with_location(ErrorLocation::node("activity_1"))
        .with_context("field", "name");

        let display = error.format_display();
        assert!(display.contains("WF-V001"));
        assert!(display.contains("Field 'name' is required"));
        assert!(display.contains("activity_1"));
    }

    #[test]
    fn test_errors_into_result() {
        let errors = WorkflowErrors::new();
        assert!(errors.into_result(42).is_ok());

        let mut errors = WorkflowErrors::new();
        errors.push(WorkflowError::new(
            ErrorCode::ValidationMissingRequiredField,
            "Error",
        ));
        assert!(errors.into_result(42).is_err());
    }
}
