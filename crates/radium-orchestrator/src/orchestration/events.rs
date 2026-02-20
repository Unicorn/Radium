//! Orchestration event model for streaming progress to clients.
//!
//! This is the canonical event stream contract for headless orchestration.
//! CLI/TUI/daemon clients should consume these events for progress, tool calls,
//! approvals, and final results.

use serde::{Deserialize, Serialize};

use super::tool::{ToolCall, ToolResult};

/// A unique identifier for correlating events within an orchestration run.
pub type CorrelationId = String;

/// High-level orchestration events emitted during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrchestrationEvent {
    /// The user sent a message to the orchestrator.
    UserInput {
        correlation_id: CorrelationId,
        content: String,
    },

    /// The model produced an assistant message (may be empty if it is only calling tools).
    AssistantMessage {
        correlation_id: CorrelationId,
        content: String,
    },

    /// The model requested a tool call.
    ToolCallRequested {
        correlation_id: CorrelationId,
        call: ToolCall,
    },

    /// A tool call is about to execute.
    ToolCallStarted {
        correlation_id: CorrelationId,
        tool_name: String,
    },

    /// A tool call finished execution.
    ToolCallFinished {
        correlation_id: CorrelationId,
        tool_name: String,
        result: ToolResult,
    },

    /// A human approval is required before continuing (e.g., dangerous command, destructive edit).
    ApprovalRequired {
        correlation_id: CorrelationId,
        tool_name: String,
        reason: String,
    },

    /// An error occurred (non-fatal or fatal).
    Error {
        correlation_id: CorrelationId,
        message: String,
    },

    /// Orchestration finished.
    Done {
        correlation_id: CorrelationId,
        finish_reason: String,
    },

    /// Agent started a thinking session (transparent reasoning).
    ThinkingSessionStarted {
        correlation_id: CorrelationId,
        context: String,
    },

    /// Agent added a thinking step.
    ThinkingStepAdded {
        correlation_id: CorrelationId,
        description: String,
    },

    /// Agent updated a thinking step with status and findings.
    ThinkingStepUpdated {
        correlation_id: CorrelationId,
        status: ThinkingStatus,
        details: Option<String>,
    },

    /// Agent finished thinking session.
    ThinkingSessionEnded {
        correlation_id: CorrelationId,
    },

    /// Agent started a recommendations session.
    RecommendationsSessionStarted {
        correlation_id: CorrelationId,
        context: String,
    },

    /// Agent added a recommendation.
    RecommendationAdded {
        correlation_id: CorrelationId,
        description: String,
        command: Option<String>,
        details: Option<String>,
    },

    /// Agent requesting user confirmation to execute recommendations.
    RecommendationsExecutionRequested {
        correlation_id: CorrelationId,
    },
}

/// Status of a thinking step for transparent reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingStatus {
    /// Step is in progress
    InProgress,
    /// Step completed successfully
    Completed,
    /// Step completed with findings
    CompletedWithFindings,
    /// Step failed
    Failed,
}

