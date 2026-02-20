//! Event stream renderer for CLI orchestration events.
//!
//! Provides real-time rendering of OrchestrationEvent stream to terminal output.
//!
//! # Usage
//!
//! When using OrchestrationService, subscribe to events and render them:
//!
//! ```rust,no_run
//! use radium_orchestrator::orchestration::OrchestrationService;
//! use crate::commands::event_renderer;
//!
//! let service = OrchestrationService::initialize(...).await?;
//! let event_rx = service.subscribe_events();
//! let correlation_id = "session-123".to_string();
//!
//! // Spawn event renderer in background
//! let renderer_handle = event_renderer::spawn_event_renderer(event_rx, correlation_id.clone());
//!
//! // Execute orchestration
//! let result = service.handle_input(&correlation_id, input, Some(&current_dir)).await?;
//!
//! // Wait for event renderer to finish
//! let _ = renderer_handle.await;
//! ```

use colored::*;
use radium_orchestrator::orchestration::events::OrchestrationEvent;
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};

/// Render orchestration events to terminal in real-time.
///
/// This function subscribes to the event stream and renders events as they arrive,
/// providing real-time feedback to the user about orchestration progress.
#[allow(dead_code)]
pub async fn render_event_stream(
    mut rx: broadcast::Receiver<OrchestrationEvent>,
    correlation_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Track tool execution state for better UX
    // Use a map to track start times for concurrent execution display
    use std::collections::HashMap;
    let mut active_tools: HashMap<String, std::time::Instant> = HashMap::new();

    loop {
        // Use a timeout to prevent indefinite blocking
        match timeout(Duration::from_secs(1), rx.recv()).await {
            Ok(Ok(event)) => {
                // Only process events for this correlation ID
                let event_correlation_id = match &event {
                    OrchestrationEvent::UserInput { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::AssistantMessage { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::ToolCallRequested { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::ToolCallStarted { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::ToolCallFinished { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::ApprovalRequired { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::Error { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::Done { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::ThinkingSessionStarted { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::ThinkingStepAdded { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::ThinkingStepUpdated { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::ThinkingSessionEnded { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::RecommendationsSessionStarted { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::RecommendationAdded { correlation_id, .. } => correlation_id,
                    OrchestrationEvent::RecommendationsExecutionRequested { correlation_id, .. } => correlation_id,
                };

                if event_correlation_id != correlation_id {
                    continue; // Skip events for other sessions
                }

                match event {
                    OrchestrationEvent::UserInput { content, .. } => {
                        // User input is already displayed, but we can show a subtle indicator
                        println!("{}", format!("→ {}", content.trim()).dimmed());
                    }
                    OrchestrationEvent::AssistantMessage { content, .. } => {
                        if !content.trim().is_empty() {
                            println!("\n{}", content);
                        }
                    }
                    OrchestrationEvent::ToolCallRequested { call, .. } => {
                        println!("\n  {} Requesting tool: {}", "🔧".cyan(), call.name.cyan().bold());
                    }
                    OrchestrationEvent::ToolCallStarted { tool_name, .. } => {
                        active_tools.insert(tool_name.clone(), std::time::Instant::now());
                        
                        // Show concurrent execution status
                        if active_tools.len() > 1 {
                            let active_list: Vec<&str> = active_tools.keys().map(|s| s.as_str()).collect();
                            println!(
                                "  {} Executing {} ({} tools running in parallel: {})",
                                "⏳".yellow(),
                                tool_name.cyan(),
                                active_tools.len(),
                                active_list.join(", ").cyan()
                            );
                        } else {
                            println!("  {} Executing {}...", "⏳".yellow(), tool_name.cyan());
                        }
                    }
                    OrchestrationEvent::ToolCallFinished { tool_name, result, .. } => {
                        let duration = active_tools.remove(&tool_name)
                            .map(|start| start.elapsed())
                            .map(|d| format!(" ({:.2}s)", d.as_secs_f64()))
                            .unwrap_or_default();
                        
                        if result.success {
                            let output_preview = if result.output.len() > 100 {
                                format!("{}...", &result.output[..100])
                            } else {
                                result.output.clone()
                            };
                            println!(
                                "  {} {} completed{} {}",
                                "✓".green(),
                                tool_name.cyan(),
                                duration.dimmed(),
                                output_preview.dimmed()
                            );
                            
                            // Show remaining active tools if any
                            if !active_tools.is_empty() {
                                let remaining: Vec<&str> = active_tools.keys().map(|s| s.as_str()).collect();
                                println!(
                                    "    {} Still running: {}",
                                    "⏳".yellow().dimmed(),
                                    remaining.join(", ").cyan().dimmed()
                                );
                            }
                        } else {
                            println!(
                                "  {} {} failed{}: {}",
                                "✗".red(),
                                tool_name.red(),
                                duration.dimmed(),
                                result.output.dimmed()
                            );
                        }
                    }
                    OrchestrationEvent::ApprovalRequired { tool_name, reason, .. } => {
                        println!(
                            "\n  {} {} requires approval: {}",
                            "⚠️".yellow().bold(),
                            tool_name.yellow().bold(),
                            reason
                        );
                        println!("  {} Press Enter to approve, or Ctrl+C to cancel", "→".dimmed());
                        
                        // Wait for user input (simplified - in real implementation, this would
                        // be handled by the orchestrator service with a callback/channel)
                        use std::io::{self, BufRead};
                        let stdin = io::stdin();
                        let mut line = String::new();
                        if stdin.lock().read_line(&mut line).is_ok() {
                            let trimmed = line.trim();
                            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("y") || trimmed.eq_ignore_ascii_case("yes") {
                                println!("  {} Approved", "✓".green());
                            } else {
                                println!("  {} Denied", "✗".red());
                                // Note: In full implementation, this would signal denial to orchestrator
                            }
                        }
                    }
                    OrchestrationEvent::Error { message, .. } => {
                        println!("\n  {} Error: {}", "✗".red().bold(), message.red());
                    }
                    OrchestrationEvent::Done { finish_reason, .. } => {
                        match finish_reason.as_str() {
                            "stop" => {
                                println!("\n{}", "✓ Completed".green().bold());
                            }
                            "max_iterations" => {
                                println!(
                                    "\n{}",
                                    "⚠ Reached maximum iterations".yellow().bold()
                                );
                            }
                            "tool_error" => {
                                println!("\n{}", "✗ Tool execution error".red().bold());
                            }
                            "error" => {
                                println!("\n{}", "✗ Execution error".red().bold());
                            }
                            _ => {
                                println!("\n{} {}", "→".dimmed(), finish_reason.dimmed());
                            }
                        }
                        break; // Exit event loop on Done
                    }
                    OrchestrationEvent::ThinkingSessionStarted { context, .. } => {
                        println!("  {} Starting thinking session: {}", "💭".cyan(), context.dimmed());
                    }
                    OrchestrationEvent::ThinkingStepAdded { description, .. } => {
                        println!("  {} {}", "💭".cyan(), description.dimmed());
                    }
                    OrchestrationEvent::ThinkingStepUpdated { status, details, .. } => {
                        let status_str = format!("{:?}", status);
                        let details_str = details.as_deref().unwrap_or("");
                        println!("  {} Updated [{}]: {}", "💭".cyan(), status_str, details_str.dimmed());
                    }
                    OrchestrationEvent::ThinkingSessionEnded { .. } => {
                        println!("  {} Thinking session complete", "💭".green());
                    }
                    OrchestrationEvent::RecommendationsSessionStarted { context, .. } => {
                        println!("  {} Generating recommendations: {}", "💡".cyan(), context.dimmed());
                    }
                    OrchestrationEvent::RecommendationAdded { description, command, details, .. } => {
                        let cmd_str = command.as_deref().unwrap_or("");
                        let details_str = details.as_deref().unwrap_or("");
                        println!("  {} {} {} {}", "💡".yellow(), description, cmd_str.cyan(), details_str.dimmed());
                    }
                    OrchestrationEvent::RecommendationsExecutionRequested { .. } => {
                        println!("  {} Recommendations ready for execution", "💡".green());
                    }
                }
            }
            Ok(Err(broadcast::error::RecvError::Closed)) => {
                // Channel closed, exit
                break;
            }
            Ok(Err(broadcast::error::RecvError::Lagged(skipped))) => {
                eprintln!(
                    "  {} Warning: {} events were skipped (receiver lagged)",
                    "⚠".yellow(),
                    skipped
                );
            }
            Err(_) => {
                // Timeout - continue waiting (allows for periodic checks)
                continue;
            }
        }
    }

    Ok(())
}

/// Spawn a background task to render events.
///
/// Returns a handle that can be awaited to wait for event stream completion.
#[allow(dead_code)]
pub fn spawn_event_renderer(
    rx: broadcast::Receiver<OrchestrationEvent>,
    correlation_id: String,
) -> tokio::task::JoinHandle<Result<(), String>> {
    tokio::spawn(async move {
        render_event_stream(rx, &correlation_id)
            .await
            .map_err(|e| e.to_string())
    })
}
