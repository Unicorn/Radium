// Orchestration module - Model-agnostic orchestration system
//
// This module provides abstractions for intelligent task routing and agent coordination
// across different AI providers (Gemini, Claude, OpenAI, and prompt-based fallback).

pub mod agent_tools;
pub mod config;
pub mod context;
pub mod context_loader;
pub mod engine;
pub mod file_tools;
pub mod hooks;
pub mod mcp_tools;
pub mod providers;
pub mod service;
pub mod terminal_tool;
pub mod tool;
pub mod tool_registry;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

use self::context::OrchestrationContext;
use self::tool::{Tool, ToolCall};
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
