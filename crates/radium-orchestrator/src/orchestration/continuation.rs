// Continuation behavior for tool execution loops
//
// This module implements automatic continuation of tool execution loops,
// sending tool results back to the model for multi-turn agent orchestration.

use radium_abstraction::{ChatMessage, MessageContent, Model, ModelError, ModelResponse};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{
    tool::{Tool, ToolCall, ToolResult},
    ContinuationBehavior, ContinuationCondition, ToolExecutionConfig,
};
use crate::error::OrchestrationError;
use crate::orchestration::execution::execute_tool_calls;

/// Execute a tool-enabled conversation with automatic continuation.
///
/// This function implements the continuation loop pattern:
/// 1. Send prompt with available tools to model
/// 2. Receive response with optional tool calls
/// 3. If tool calls present and continuation condition met:
///    - Execute tool calls according to execution strategy
///    - Add tool results to conversation history
///    - Send updated conversation back to model
///    - Repeat from step 2
/// 4. Return final response when continuation stops
///
/// # Arguments
/// * `model` - The model to use for generation
/// * `prompt` - Initial user prompt
/// * `tools` - Available tools the model can call
/// * `config` - Execution configuration including continuation behavior
///
/// # Returns
/// Final model response after continuation completes
pub async fn execute_with_continuation(
    model: &dyn Model,
    prompt: &str,
    tools: &[Tool],
    config: &ToolExecutionConfig,
) -> Result<ModelResponse, OrchestrationError> {
    let mut messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text(prompt.to_string()),
    }];

    let mut round = 0;
    let start_time = Instant::now();
    let mut call_tracker = CallHistoryTracker::new();

    loop {
        // Generate response with tools
        let response = model
            .generate_with_tools(&messages, tools, None)
            .await
            .map_err(|e| OrchestrationError::Model(e))?;

        // Check continuation condition
        let should_continue = check_continuation(
            &config.continuation,
            &response,
            round,
            config.max_rounds,
            start_time,
        )?;

        if !should_continue {
            return Ok(response);
        }

        // Execute tool calls if present
        let tool_calls = match response.tool_calls.as_ref() {
            Some(calls) if !calls.is_empty() => calls,
            _ => {
                // No tool calls - stop continuation
                return Ok(response);
            }
        };

        // Check for circular calls before executing
        for call in tool_calls {
            if call_tracker.check_circular(call) {
                return Err(OrchestrationError::Other(
                    format!("Circular tool call detected: tool '{}' called 3 times with identical arguments", call.name)
                ));
            }
        }

        // Record calls before execution
        for call in tool_calls {
            call_tracker.record_call(call);
        }

        let execution_results = execute_tool_calls(tool_calls, tools, config).await;

        // Convert execution results to ToolResults (handle errors according to strategy)
        let tool_results: Vec<ToolResult> = execution_results
            .into_iter()
            .map(|r| match r {
                Ok(tr) => tr,
                Err(e) => ToolResult::error_for_model(format!("Error: {}", e)),
            })
            .collect();

        // Add assistant response to history
        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: MessageContent::Text(response.content.clone()),
        });

        // Format tool results as user message
        let results_content = format_tool_results(&tool_results);
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text(results_content),
        });

        round += 1;
    }
}

/// Check if continuation should continue based on the configured behavior.
fn check_continuation(
    behavior: &ContinuationBehavior,
    response: &ModelResponse,
    round: usize,
    max_rounds: usize,
    start_time: Instant,
) -> Result<bool, OrchestrationError> {
    match behavior {
        ContinuationBehavior::Manual => {
            // Manual mode never auto-continues
            Ok(false)
        }
        ContinuationBehavior::AutoContinue { max_rounds: behavior_max_rounds } => {
            // Check if we have tool calls and haven't exceeded max rounds
            let has_tool_calls = response.tool_calls.as_ref()
                .map(|calls| !calls.is_empty())
                .unwrap_or(false);
            
            let max_rounds_effective = (*behavior_max_rounds).min(max_rounds);
            Ok(has_tool_calls && round < max_rounds_effective)
        }
        ContinuationBehavior::AutoContinueUntil { condition } => {
            match condition {
                ContinuationCondition::NoToolCalls => {
                    // Stop if no tool calls
                    let has_tool_calls = response.tool_calls.as_ref()
                        .map(|calls| !calls.is_empty())
                        .unwrap_or(false);
                    Ok(has_tool_calls && round < max_rounds)
                }
                ContinuationCondition::MaxTokens(limit) => {
                    // Stop if token limit exceeded
                    let total_tokens = response.usage
                        .as_ref()
                        .map(|u| u.total_tokens as usize)
                        .unwrap_or(0);
                    
                    if total_tokens >= *limit {
                        return Ok(false);
                    }
                    
                    // Continue if we have tool calls and haven't exceeded max rounds
                    let has_tool_calls = response.tool_calls.as_ref()
                        .map(|calls| !calls.is_empty())
                        .unwrap_or(false);
                    Ok(has_tool_calls && round < max_rounds)
                }
                ContinuationCondition::Timeout(duration) => {
                    // Stop if timeout exceeded
                    if start_time.elapsed() >= *duration {
                        return Ok(false);
                    }
                    
                    // Continue if we have tool calls and haven't exceeded max rounds
                    let has_tool_calls = response.tool_calls.as_ref()
                        .map(|calls| !calls.is_empty())
                        .unwrap_or(false);
                    Ok(has_tool_calls && round < max_rounds)
                }
            }
        }
    }
}

/// Tracks tool call history to detect circular patterns.
struct CallHistoryTracker {
    /// Map from (tool_name, args_hash) to call count
    calls: HashMap<(String, String), usize>,
}

impl CallHistoryTracker {
    fn new() -> Self {
        Self {
            calls: HashMap::new(),
        }
    }

    /// Record a tool call in history.
    fn record_call(&mut self, call: &ToolCall) {
        let key = (call.name.clone(), hash_args(&call.arguments));
        *self.calls.entry(key).or_insert(0) += 1;
    }

    /// Check if a call would create a circular pattern (3+ identical calls).
    fn check_circular(&self, call: &ToolCall) -> bool {
        let key = (call.name.clone(), hash_args(&call.arguments));
        self.calls.get(&key).map_or(false, |&count| count >= 3)
    }
}

/// Create a stable hash string representation of tool call arguments.
fn hash_args(args: &serde_json::Value) -> String {
    serde_json::to_string(args).unwrap_or_default()
}

/// Format tool results as a message for the model.
fn format_tool_results(results: &[ToolResult]) -> String {
    let mut parts = Vec::new();
    
    for (i, result) in results.iter().enumerate() {
        if result.is_error {
            parts.push(format!("Tool call {} failed: {}", i + 1, result.output));
        } else if result.success {
            parts.push(format!("Tool call {} succeeded: {}", i + 1, result.output));
        } else {
            parts.push(format!("Tool call {} returned: {}", i + 1, result.output));
        }
    }
    
    if parts.is_empty() {
        "No tool results.".to_string()
    } else {
        parts.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_continuation_manual() {
        let behavior = ContinuationBehavior::Manual;
        let response = ModelResponse {
            content: "test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
            tool_calls: Some(vec![]),
        };
        
        assert!(!check_continuation(&behavior, &response, 0, 5, Instant::now()).unwrap());
    }

    #[test]
    fn test_check_continuation_auto_continue() {
        let behavior = ContinuationBehavior::AutoContinue { max_rounds: 3 };
        let response_with_calls = ModelResponse {
            content: "test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
            tool_calls: Some(vec![
                radium_abstraction::ToolCall {
                    id: "call1".to_string(),
                    name: "tool1".to_string(),
                    arguments: serde_json::json!({}),
                }
            ]),
        };
        
        // Should continue when we have tool calls and haven't exceeded max rounds
        assert!(check_continuation(&behavior, &response_with_calls, 0, 5, Instant::now()).unwrap());
        assert!(check_continuation(&behavior, &response_with_calls, 2, 5, Instant::now()).unwrap());
        
        // Should stop when max rounds exceeded
        assert!(!check_continuation(&behavior, &response_with_calls, 3, 5, Instant::now()).unwrap());
    }

    #[test]
    fn test_format_tool_results() {
        let results = vec![
            ToolResult::success("Success message"),
            ToolResult::error_for_model("Error message"),
        ];
        
        let formatted = format_tool_results(&results);
        assert!(formatted.contains("Tool call 1 succeeded"));
        assert!(formatted.contains("Tool call 2 failed"));
    }
}

