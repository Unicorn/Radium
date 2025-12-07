// Error types for orchestration

use thiserror::Error;

/// Result type for orchestration operations
pub type Result<T> = std::result::Result<T, OrchestrationError>;

/// Orchestration errors
#[derive(Debug, Error)]
pub enum OrchestrationError {
    /// Invalid tool arguments
    #[error("Invalid tool arguments for '{tool}': {reason}")]
    InvalidToolArguments {
        /// Tool name
        tool: String,
        /// Reason why arguments are invalid
        reason: String,
    },

    /// Tool execution failed
    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    /// Model error
    #[error("Model error: {0}")]
    Model(#[from] radium_abstraction::ModelError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Max iterations reached
    #[error("Maximum tool iterations ({0}) reached")]
    MaxIterations(u32),

    /// Orchestration cancelled
    #[error("Orchestration cancelled by user")]
    Cancelled,

    /// Other error
    #[error("Orchestration error: {0}")]
    Other(String),
}
