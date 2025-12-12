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
pub async fn render_event_stream(
    mut rx: broadcast::Receiver<OrchestrationEvent>,
    correlation_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Track tool execution state for better UX
    let mut active_tools: Vec<String> = Vec::new();

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
                        active_tools.push(tool_name.clone());
                        println!("  {} Executing {}...", "⏳".yellow(), tool_name.cyan());
                    }
                    OrchestrationEvent::ToolCallFinished { tool_name, result, .. } => {
                        active_tools.retain(|name| name != &tool_name);
                        if result.success {
                            let output_preview = if result.output.len() > 100 {
                                format!("{}...", &result.output[..100])
                            } else {
                                result.output.clone()
                            };
                            println!(
                                "  {} {} completed {}",
                                "✓".green(),
                                tool_name.cyan(),
                                output_preview.dimmed()
                            );
                        } else {
                            println!(
                                "  {} {} failed: {}",
                                "✗".red(),
                                tool_name.red(),
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
                        // Note: Actual approval logic should be handled by the caller
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
