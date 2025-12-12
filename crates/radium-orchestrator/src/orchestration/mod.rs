// Orchestration module - Model-agnostic orchestration system
//
// This module provides abstractions for intelligent task routing and agent coordination
// across different AI providers (Gemini, Claude, OpenAI, and prompt-based fallback).

pub mod agent_tools;
pub mod code_analysis_tool;
pub mod config;
pub mod context;
pub mod context_loader;
// TODO: Fix type mismatch between radium_abstraction::Tool and orchestration::tool::Tool
// pub mod continuation;
pub mod engine;
pub mod events;
pub mod execution;
pub mod file_tools;
pub mod git_extended_tools;
pub mod hooks;
pub mod mcp_tools;
pub mod planner;
pub mod project_scan_tool;
pub mod providers;
pub mod service;
pub mod terminal_tool;
pub mod tool;
pub mod tool_builder;
pub mod tool_registry;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

use self::context::OrchestrationContext;
use self::tool::{Tool, ToolCall};
// TODO: Fix type mismatch
// pub use self::continuation::execute_with_continuation;
pub use self::execution::execute_tool_calls;
use crate::error::Result;

/// Reasons why orchestration finished
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FinishReason {
    /// Model completed successfully
    Stop,
    /// Reached maximum tool iterations
    MaxIterations,
    /// Tool execution failed
    ToolError,
    /// User cancelled
    Cancelled,
    /// Model error
    Error,
}

impl fmt::Display for FinishReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stop => write!(f, "stop"),
            Self::MaxIterations => write!(f, "max_iterations"),
            Self::ToolError => write!(f, "tool_error"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Result of orchestration execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    /// Text response from the orchestrator
    pub response: String,
    /// Tool calls requested by the orchestrator
    pub tool_calls: Vec<ToolCall>,
    /// Reason orchestration finished
    pub finish_reason: FinishReason,
}

impl OrchestrationResult {
    /// Create a new orchestration result
    pub fn new(response: String, tool_calls: Vec<ToolCall>, finish_reason: FinishReason) -> Self {
        Self { response, tool_calls, finish_reason }
    }

    /// Check if orchestration completed successfully
    pub fn is_success(&self) -> bool {
        matches!(self.finish_reason, FinishReason::Stop)
    }

    /// Check if there are tool calls to execute
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

/// Strategy for executing multiple tool calls.
///
/// Controls how tool calls are executed when multiple tools are requested in parallel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FunctionExecutionStrategy {
    /// Execute all tool calls concurrently (in parallel).
    ///
    /// All tools start executing at the same time, which is fastest
    /// but may consume more resources.
    Concurrent,
    /// Execute tool calls sequentially (one after another).
    ///
    /// Tools execute in order, waiting for each to complete before
    /// starting the next. Slower but more predictable.
    Sequential,
    /// Execute tool calls in batches of specified size.
    ///
    /// Tools are grouped into batches of `batch_size` and each batch
    /// executes concurrently, but batches execute sequentially.
    /// This balances speed and resource usage.
    ConcurrentBatched {
        /// Number of tools to execute concurrently per batch
        batch_size: usize,
    },
}

/// Behavior for continuing tool execution loops.
///
/// Controls whether and how to automatically continue sending tool results
/// back to the model for multi-turn agent orchestration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContinuationBehavior {
    /// Return tool results to caller without auto-continuation.
    ///
    /// The caller is responsible for manually sending results back to
    /// the model if continuation is desired.
    Manual,
    /// Automatically continue for a fixed number of rounds.
    ///
    /// The system will automatically send tool results back to the model
    /// and continue the conversation for up to `max_rounds` iterations.
    AutoContinue {
        /// Maximum number of continuation rounds
        max_rounds: usize,
    },
    /// Automatically continue until a specific condition is met.
    ///
    /// The system will continue until the specified condition is satisfied
    /// (e.g., no more tool calls, token limit reached, timeout).
    AutoContinueUntil {
        /// Condition that stops continuation
        condition: ContinuationCondition,
    },
}

/// Condition that stops automatic continuation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContinuationCondition {
    /// Stop when model stops calling tools (returns no tool_calls).
    NoToolCalls,
    /// Stop when total tokens exceed the specified limit.
    MaxTokens(usize),
    /// Stop when total execution time exceeds the specified duration.
    Timeout(Duration),
}

/// Strategy for handling tool execution errors.
///
/// Controls how the system responds when a tool execution fails.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolErrorHandling {
    /// Return error as tool result to the model.
    ///
    /// The model receives the error and can retry, use alternative tools,
    /// or report to the user. This is the default and most robust approach.
    ReturnToModel,
    /// Retry failed tool calls with exponential backoff.
    ///
    /// The system will retry the failed tool call up to `max_retries` times,
    /// with delays increasing exponentially (initial_delay * 2^attempt).
    RetryWithBackoff {
        /// Maximum number of retry attempts
        max_retries: usize,
        /// Initial delay before first retry
        initial_delay: Duration,
    },
    /// Fail immediately and propagate error.
    ///
    /// Any tool execution failure immediately stops execution and returns
    /// an error. No retries or error recovery.
    FailFast,
    /// Skip failed tools and continue with successful ones.
    ///
    /// Failed tool calls are skipped (marked as failed) but execution
    /// continues with other tools. Partial results are returned.
    SkipAndContinue,
}

/// Unified configuration for tool execution.
///
/// Combines all tool execution settings including execution strategy,
/// continuation behavior, error handling, and resource limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionConfig {
    /// Strategy for executing multiple tool calls
    pub execution_strategy: FunctionExecutionStrategy,
    /// Behavior for continuing tool execution loops
    pub continuation: ContinuationBehavior,
    /// Strategy for handling tool execution errors
    pub error_handling: ToolErrorHandling,
    /// Per-call timeout for individual tool executions
    pub timeout_per_call: Duration,
    /// Maximum number of parallel tool executions (None = unlimited)
    ///
    /// When using Concurrent or ConcurrentBatched strategies, this limits
    /// the maximum number of tools that execute simultaneously.
    pub max_parallel: Option<usize>,
    /// Maximum number of continuation rounds
    ///
    /// Used as a safety limit regardless of continuation behavior to prevent
    /// infinite loops.
    pub max_rounds: usize,
}

impl Default for ToolExecutionConfig {
    fn default() -> Self {
        Self {
            execution_strategy: FunctionExecutionStrategy::Concurrent,
            continuation: ContinuationBehavior::AutoContinue { max_rounds: 5 },
            error_handling: ToolErrorHandling::ReturnToModel,
            timeout_per_call: Duration::from_secs(30),
            max_parallel: Some(10),
            max_rounds: 5,
        }
    }
}

/// Validate tool mode constraints.
///
/// This function validates that the model's behavior matches the configured tool mode.
/// It performs both configuration validation (checking if mode is valid given available tools)
/// and response validation (checking if model's response matches mode constraints).
///
/// # Arguments
/// * `mode` - The configured tool use mode (Auto, Any, or None)
/// * `available_tools` - Tools that were available to the model
/// * `response` - The model's response to validate
///
/// # Returns
/// `Ok(())` if validation passes, `Err(OrchestrationError)` with actionable error message if validation fails.
///
/// # Errors
/// - `InvalidToolMode` - Configuration error (e.g., ANY mode but no tools available)
/// - `ModeViolation` - Runtime violation (e.g., ANY mode but model didn't call tools)
///
/// # Example
///
/// ```rust
/// use radium_orchestrator::orchestration::validate_tool_mode;
/// use radium_orchestrator::orchestration::tool::Tool;
/// use radium_abstraction::{ToolUseMode, ModelResponse};
///
/// // Mode ANY requires at least one available tool.
/// let mode = ToolUseMode::Any;
/// let tools: Vec<Tool> = vec![];
/// let response = ModelResponse {
///     content: "test".to_string(),
///     model_id: None,
///     usage: None,
///     metadata: None,
///     tool_calls: None,
/// };
///
/// // This returns InvalidToolMode since ANY mode requires tools.
/// assert!(validate_tool_mode(&mode, &tools, &response).is_err());
/// ```
pub fn validate_tool_mode(
    mode: &radium_abstraction::ToolUseMode,
    available_tools: &[Tool],
    response: &radium_abstraction::ModelResponse,
) -> crate::error::Result<()> {
    match mode {
        radium_abstraction::ToolUseMode::Any => {
            // Mode ANY requires tools to be available
            if available_tools.is_empty() {
                return Err(crate::error::OrchestrationError::InvalidToolMode(
                    "Mode is ANY but no tools are available. Either provide tools or use AUTO/NONE mode.".to_string()
                ));
            }
            
            // Mode ANY requires model to call at least one tool
            let has_tool_calls = response.tool_calls.as_ref()
                .map(|calls| !calls.is_empty())
                .unwrap_or(false);
            
            if !has_tool_calls {
                return Err(crate::error::OrchestrationError::ModeViolation(
                    "Mode is ANY but model did not call any tools. Try AUTO mode or add more specific instructions.".to_string()
                ));
            }
        }
        radium_abstraction::ToolUseMode::None => {
            // Mode NONE requires model to not call any tools
            let has_tool_calls = response.tool_calls.as_ref()
                .map(|calls| !calls.is_empty())
                .unwrap_or(false);
            
            if has_tool_calls {
                return Err(crate::error::OrchestrationError::ModeViolation(
                    "Mode is NONE but model attempted to call tools. This should not happen.".to_string()
                ));
            }
        }
        radium_abstraction::ToolUseMode::Auto => {
            // AUTO mode has no constraints - model decides whether to use tools
            // No validation needed
        }
    }
    
    Ok(())
}

/// Model-agnostic orchestration provider trait
///
/// Implementations of this trait provide orchestration capabilities using
/// different AI models and techniques (function calling, tool use, prompt-based).
#[async_trait]
pub trait OrchestrationProvider: Send + Sync {
    /// Execute user input with available tools
    ///
    /// The orchestrator analyzes the input, decides which tools to call,
    /// and returns the result along with any tool calls to execute.
    ///
    /// # Arguments
    /// * `input` - User input to process
    /// * `tools` - Available tools the orchestrator can invoke
    /// * `context` - Conversation history and session state
    ///
    /// # Returns
    /// Orchestration result with response and tool calls
    async fn execute_with_tools(
        &self,
        input: &str,
        tools: &[Tool],
        context: &OrchestrationContext,
    ) -> Result<OrchestrationResult>;

    /// Check if provider supports native function calling
    ///
    /// Returns true if the provider has native function/tool calling support
    /// (like Gemini function_declarations, Claude tool use, OpenAI functions).
    /// Returns false for prompt-based orchestration.
    fn supports_function_calling(&self) -> bool;

    /// Get provider name for logging/debugging
    fn provider_name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finish_reason_display() {
        assert_eq!(FinishReason::Stop.to_string(), "stop");
        assert_eq!(FinishReason::MaxIterations.to_string(), "max_iterations");
        assert_eq!(FinishReason::ToolError.to_string(), "tool_error");
        assert_eq!(FinishReason::Cancelled.to_string(), "cancelled");
        assert_eq!(FinishReason::Error.to_string(), "error");
    }

    #[test]
    fn test_validate_tool_mode_auto() {
        use radium_abstraction::{ModelResponse, ToolUseMode};
        use std::sync::Arc;

        // AUTO mode should always pass validation
        let mode = ToolUseMode::Auto;
        let tools = vec![];
        let response_with_tools = ModelResponse {
            content: "test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
            tool_calls: Some(vec![
                radium_abstraction::ToolCall {
                    id: "call_1".to_string(),
                    name: "test_tool".to_string(),
                    arguments: serde_json::json!({}),
                }
            ]),
        };
        let response_without_tools = ModelResponse {
            content: "test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
            tool_calls: None,
        };

        // AUTO mode should pass regardless of tool calls
        assert!(validate_tool_mode(&mode, &tools, &response_with_tools).is_ok());
        assert!(validate_tool_mode(&mode, &tools, &response_without_tools).is_ok());
    }

    #[test]
    fn test_validate_tool_mode_any_no_tools() {
        use radium_abstraction::{ModelResponse, ToolUseMode};
        use crate::orchestration::tool::Tool;
        use std::sync::Arc;

        // ANY mode without tools should fail
        let mode = ToolUseMode::Any;
        let tools = vec![]; // No tools available
        let response = ModelResponse {
            content: "test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
            tool_calls: None,
        };

        let result = validate_tool_mode(&mode, &tools, &response);
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::OrchestrationError::InvalidToolMode(msg) => {
                assert!(msg.contains("no tools are available"));
            }
            _ => panic!("Expected InvalidToolMode error"),
        }
    }

    #[test]
    fn test_validate_tool_mode_any_no_calls() {
        use radium_abstraction::{ModelResponse, ToolUseMode};
        use crate::orchestration::tool::{Tool, ToolParameters, ToolArguments, ToolResult};
        use crate::orchestration::tool::ToolHandler;
        use std::sync::Arc;
        use async_trait::async_trait;

        // Create a mock tool
        struct MockToolHandler;
        #[async_trait]
        impl ToolHandler for MockToolHandler {
            async fn execute(&self, _args: &ToolArguments) -> crate::error::Result<ToolResult> {
                Ok(ToolResult::success("ok"))
            }
        }

        let mode = ToolUseMode::Any;
        let tools = vec![Tool::new(
            "test_tool",
            "test_tool",
            "A test tool",
            ToolParameters::new(),
            Arc::new(MockToolHandler),
        )];
        let response = ModelResponse {
            content: "test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
            tool_calls: None, // Model didn't call any tools
        };

        let result = validate_tool_mode(&mode, &tools, &response);
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::OrchestrationError::ModeViolation(msg) => {
                assert!(msg.contains("did not call any tools"));
            }
            _ => panic!("Expected ModeViolation error"),
        }
    }

    #[test]
    fn test_validate_tool_mode_any_with_calls() {
        use radium_abstraction::{ModelResponse, ToolUseMode};
        use crate::orchestration::tool::{Tool, ToolParameters, ToolArguments, ToolResult};
        use crate::orchestration::tool::ToolHandler;
        use std::sync::Arc;
        use async_trait::async_trait;

        // Create a mock tool
        struct MockToolHandler;
        #[async_trait]
        impl ToolHandler for MockToolHandler {
            async fn execute(&self, _args: &ToolArguments) -> crate::error::Result<ToolResult> {
                Ok(ToolResult::success("ok"))
            }
        }

        let mode = ToolUseMode::Any;
        let tools = vec![Tool::new(
            "test_tool",
            "test_tool",
            "A test tool",
            ToolParameters::new(),
            Arc::new(MockToolHandler),
        )];
        let response = ModelResponse {
            content: "test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
            tool_calls: Some(vec![
                radium_abstraction::ToolCall {
                    id: "call_1".to_string(),
                    name: "test_tool".to_string(),
                    arguments: serde_json::json!({}),
                }
            ]),
        };

        // ANY mode with tools and tool calls should pass
        assert!(validate_tool_mode(&mode, &tools, &response).is_ok());
    }

    #[test]
    fn test_validate_tool_mode_none_with_calls() {
        use radium_abstraction::{ModelResponse, ToolUseMode};

        // NONE mode with tool calls should fail
        let mode = ToolUseMode::None;
        let tools = vec![];
        let response = ModelResponse {
            content: "test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
            tool_calls: Some(vec![
                radium_abstraction::ToolCall {
                    id: "call_1".to_string(),
                    name: "test_tool".to_string(),
                    arguments: serde_json::json!({}),
                }
            ]),
        };

        let result = validate_tool_mode(&mode, &tools, &response);
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::OrchestrationError::ModeViolation(msg) => {
                assert!(msg.contains("attempted to call tools"));
            }
            _ => panic!("Expected ModeViolation error"),
        }
    }

    #[test]
    fn test_validate_tool_mode_none_without_calls() {
        use radium_abstraction::{ModelResponse, ToolUseMode};

        // NONE mode without tool calls should pass
        let mode = ToolUseMode::None;
        let tools = vec![];
        let response = ModelResponse {
            content: "test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
            tool_calls: None,
        };

        assert!(validate_tool_mode(&mode, &tools, &response).is_ok());
    }

    #[test]
    fn test_orchestration_result_is_success() {
        let success = OrchestrationResult::new("Done".to_string(), vec![], FinishReason::Stop);
        assert!(success.is_success());

        let error = OrchestrationResult::new("Error".to_string(), vec![], FinishReason::Error);
        assert!(!error.is_success());
    }

    #[test]
    fn test_orchestration_result_has_tool_calls() {
        let no_tools = OrchestrationResult::new("Done".to_string(), vec![], FinishReason::Stop);
        assert!(!no_tools.has_tool_calls());

        let with_tools = OrchestrationResult::new(
            "Calling tool".to_string(),
            vec![ToolCall {
                id: "call_1".to_string(),
                name: "test_tool".to_string(),
                arguments: serde_json::json!({}),
            }],
            FinishReason::Stop,
        );
        assert!(with_tools.has_tool_calls());
    }
}
