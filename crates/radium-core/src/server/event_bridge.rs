//! Event bridge for converting orchestration events to gRPC session events.
//!
//! This module bridges the orchestration event stream from the orchestrator
//! to the gRPC session event stream for clients.
//!
//! This module is only available when the `workflow` feature is enabled.

#![cfg(feature = "workflow")]

use crate::proto::{
    ApprovalRequestEvent, MessageEvent, SessionEvent, ToolCallEvent, ToolResultEvent,
};
use radium_orchestrator::orchestration::events::OrchestrationEvent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, warn};

/// Event bridge that converts orchestration events to session events and routes them to sessions.
pub struct EventBridge {
    /// Map of session ID to sender channels for that session
    session_senders: Arc<RwLock<HashMap<String, mpsc::Sender<Result<SessionEvent, tonic::Status>>>>>,
}

impl EventBridge {
    /// Create a new event bridge.
    pub fn new() -> Self {
        Self {
            session_senders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a session stream to receive events.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier
    /// * `sender` - Channel sender for session events
    pub async fn register_session(&self, session_id: String, sender: mpsc::Sender<Result<SessionEvent, tonic::Status>>) {
        let mut senders = self.session_senders.write().await;
        senders.insert(session_id.clone(), sender);
        debug!(session_id = %session_id, "Registered session for event streaming");
    }

    /// Unregister a session stream.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier to remove
    pub async fn unregister_session(&self, session_id: &str) {
        let mut senders = self.session_senders.write().await;
        senders.remove(session_id);
        debug!(session_id = %session_id, "Unregistered session from event streaming");
    }

    /// Start listening to orchestration events and forwarding them to sessions.
    ///
    /// # Arguments
    ///
    /// * `event_rx` - Receiver for orchestration events from the orchestrator
    pub fn start_forwarding(&self, mut event_rx: broadcast::Receiver<OrchestrationEvent>) {
        let session_senders = Arc::clone(&self.session_senders);

        tokio::spawn(async move {
            loop {
                match event_rx.recv().await {
                    Ok(orch_event) => {
                        // Extract session/correlation ID from event
                        let session_id = extract_session_id(&orch_event);

                        // Convert orchestration event to session event
                        if let Some(session_event) = convert_to_session_event(&orch_event) {
                            // Send to appropriate session
                            let senders = session_senders.read().await;
                            if let Some(sender) = senders.get(&session_id) {
                                let result: Result<SessionEvent, tonic::Status> = Ok(session_event);
                                if let Err(e) = sender.send(result).await {
                                    warn!(
                                        session_id = %session_id,
                                        error = %e,
                                        "Failed to send event to session"
                                    );
                                }
                            } else {
                                debug!(
                                    session_id = %session_id,
                                    "No active stream for session, event dropped"
                                );
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Event bridge lagged, {} events skipped", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Orchestration event channel closed, stopping event bridge");
                        break;
                    }
                }
            }
        });
    }
}

/// Extract session/correlation ID from an orchestration event.
fn extract_session_id(event: &OrchestrationEvent) -> String {
    match event {
        OrchestrationEvent::UserInput { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::AssistantMessage { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::ToolCallRequested { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::ToolCallStarted { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::ToolCallFinished { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::ApprovalRequired { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::Error { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::Done { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::ThinkingSessionStarted { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::ThinkingStepAdded { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::ThinkingStepUpdated { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::ThinkingSessionEnded { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::RecommendationsSessionStarted { correlation_id, .. } => {
            correlation_id.clone()
        }
        OrchestrationEvent::RecommendationAdded { correlation_id, .. } => correlation_id.clone(),
        OrchestrationEvent::RecommendationsExecutionRequested { correlation_id, .. } => {
            correlation_id.clone()
        }
    }
}

/// Convert an orchestration event to a session event (proto format).
fn convert_to_session_event(event: &OrchestrationEvent) -> Option<SessionEvent> {
    match event {
        OrchestrationEvent::ToolCallRequested { correlation_id, call } => {
            Some(SessionEvent {
                event: Some(crate::proto::session_event::Event::ToolCall(
                    ToolCallEvent {
                        session_id: correlation_id.clone(),
                        tool_name: call.name.clone(),
                        arguments_json: serde_json::to_string(&call.arguments).unwrap_or_default(),
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        call_id: Some(call.id.clone()),
                    },
                )),
            })
        }
        OrchestrationEvent::ToolCallFinished {
            correlation_id,
            tool_name,
            result,
        } => Some(SessionEvent {
            event: Some(crate::proto::session_event::Event::ToolResult(
                ToolResultEvent {
                    session_id: correlation_id.clone(),
                    tool_name: tool_name.clone(),
                    result_json: result.output.clone(),
                    duration_ms: 0, // Duration not tracked in current event
                    success: result.success,
                    error: if result.is_error {
                        Some(result.output.clone())
                    } else {
                        None
                    },
                    call_id: None, // Would need to track call ID through execution
                    timestamp: chrono::Utc::now().timestamp_millis(),
                },
            )),
        }),
        OrchestrationEvent::ApprovalRequired {
            correlation_id,
            tool_name,
            reason,
        } => Some(SessionEvent {
            event: Some(crate::proto::session_event::Event::ApprovalRequest(
                ApprovalRequestEvent {
                    session_id: correlation_id.clone(),
                    tool_name: tool_name.clone(),
                    arguments_json: "{}".to_string(), // Arguments not included in current event
                    policy_rule: reason.clone(),
                    request_id: None, // Would need to generate unique request ID
                    timestamp: chrono::Utc::now().timestamp_millis(),
                },
            )),
        }),
        OrchestrationEvent::AssistantMessage {
            correlation_id,
            content,
        } => Some(SessionEvent {
            event: Some(crate::proto::session_event::Event::Message(MessageEvent {
                session_id: correlation_id.clone(),
                message: content.clone(),
                role: "assistant".to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            })),
        }),
        OrchestrationEvent::Error {
            correlation_id,
            message,
        } => Some(SessionEvent {
            event: Some(crate::proto::session_event::Event::Message(MessageEvent {
                session_id: correlation_id.clone(),
                message: format!("Error: {}", message),
                role: "system".to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            })),
        }),
        // Other events don't have direct session event mappings
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_orchestrator::orchestration::tool::{ToolCall, ToolResult};

    #[test]
    fn test_extract_session_id() {
        let event = OrchestrationEvent::ToolCallRequested {
            correlation_id: "session-123".to_string(),
            call: ToolCall {
                id: "call-1".to_string(),
                name: "test_tool".to_string(),
                arguments: serde_json::json!({}),
            },
        };

        assert_eq!(extract_session_id(&event), "session-123");
    }

    #[test]
    fn test_convert_tool_call_requested() {
        let event = OrchestrationEvent::ToolCallRequested {
            correlation_id: "session-123".to_string(),
            call: ToolCall {
                id: "call-1".to_string(),
                name: "test_tool".to_string(),
                arguments: serde_json::json!({"arg": "value"}),
            },
        };

        let session_event = convert_to_session_event(&event);
        assert!(session_event.is_some());

        if let Some(SessionEvent {
            event: Some(crate::proto::session_event::Event::ToolCall(tool_call)),
        }) = session_event
        {
            assert_eq!(tool_call.session_id, "session-123");
            assert_eq!(tool_call.tool_name, "test_tool");
            assert!(tool_call.arguments_json.contains("arg"));
        } else {
            panic!("Expected ToolCall event");
        }
    }

    #[test]
    fn test_convert_tool_call_finished() {
        let event = OrchestrationEvent::ToolCallFinished {
            correlation_id: "session-123".to_string(),
            tool_name: "test_tool".to_string(),
            result: ToolResult {
                output: "Success".to_string(),
                success: true,
                is_error: false,
            },
        };

        let session_event = convert_to_session_event(&event);
        assert!(session_event.is_some());
    }
}
