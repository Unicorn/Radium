// Orchestration module - Model-agnostic orchestration system
//
// This module provides abstractions for intelligent task routing and agent coordination
// across different AI providers (Gemini, Claude, OpenAI, and prompt-based fallback).

pub mod agent_tools;
pub mod context;
pub mod providers;
pub mod tool;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

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
