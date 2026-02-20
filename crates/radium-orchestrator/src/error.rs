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

    /// Invalid tool mode configuration
    ///
    /// This error occurs when tool configuration is invalid, such as:
    /// - Mode is ANY but no tools are provided
    /// - Invalid mode value
    /// - Conflicting configuration options
    #[error("Invalid tool mode: {0}")]
    InvalidToolMode(String),

    /// Tool mode constraint violated
    ///
    /// This error occurs when the model's behavior violates the configured mode:
    /// - Mode is ANY but model returned no tool calls
    /// - Mode is NONE but model attempted to call tools
    #[error("Tool mode violation: {0}")]
    ModeViolation(String),

    /// Other error
    #[error("Orchestration error: {0}")]
    Other(String),
}

/// Critical errors that require dispatcher shutdown.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CriticalError {
    /// Authentication failure - cannot continue execution.
    #[error("Authentication failure: {0}")]
    AuthenticationFailure(String),

    /// Credit/quota exhausted - cannot continue execution.
    #[error("Credit exhausted: {0}")]
    CreditExhausted(String),

    /// Other critical error.
    #[error("Critical error: {0}")]
    Other(String),
}

impl CriticalError {
    /// Checks if a ModelError represents a critical error.
    pub fn from_model_error(error: &radium_abstraction::ModelError) -> Option<Self> {
        match error {
            radium_abstraction::ModelError::QuotaExceeded { provider, message } => {
                Some(CriticalError::CreditExhausted(
                    message.clone().unwrap_or_else(|| format!("Provider {} quota exceeded", provider)),
                ))
            }
            radium_abstraction::ModelError::UnsupportedModelProvider(msg) if msg.contains("not found") || msg.contains("credential") => {
                Some(CriticalError::AuthenticationFailure(msg.clone()))
            }
            _ => None,
        }
    }

    /// Checks if an error string indicates a critical error.
    pub fn is_critical_error(error_msg: &str) -> bool {
        let lower = error_msg.to_lowercase();
        lower.contains("credential not found")
            || lower.contains("authentication failed")
            || lower.contains("permission denied")
            || lower.contains("quota exceeded")
            || lower.contains("credit exhausted")
            || lower.contains("insufficient_quota")
    }
}
