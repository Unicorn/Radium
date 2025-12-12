//! Shared tool execution utilities for CLI commands.
//!
//! This module provides common functionality for executing tools across
//! different CLI commands (step, chat, etc.).

use anyhow::{anyhow, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::colors::RadiumBrandColors;
use radium_abstraction::{
    ChatMessage, MessageContent, Model, ModelResponse, Tool as AbstractionTool, ToolCall,
    ToolConfig,
};
use radium_models::{ClaudeModel, GeminiModel, MockModel, OpenAIModel};
use radium_orchestrator::orchestration::tool::{Tool as OrchestrationTool, ToolArguments};
use std::path::PathBuf;
use std::time::Duration;

/// Create a Model instance based on engine ID and model name
pub fn create_model(
    engine_id: &str,
    model: &str,
    api_key: String,
) -> Result<Box<dyn Model>> {
    match engine_id {
        "claude" | "anthropic" => {
            Ok(Box::new(ClaudeModel::with_api_key(model.to_string(), api_key)))
        }
        "gemini" => {
            Ok(Box::new(GeminiModel::with_api_key(model.to_string(), api_key)))
        }
        "openai" => {
            Ok(Box::new(OpenAIModel::with_api_key(model.to_string(), api_key)))
        }
        "mock" => {
            Ok(Box::new(MockModel::new(model.to_string())))
        }
        _ => Err(anyhow!("Unsupported engine for tool execution: {}", engine_id))
    }
}

/// Convert orchestration Tool to abstraction Tool
pub fn convert_tools(tools: &[OrchestrationTool]) -> Vec<AbstractionTool> {
    tools.iter().map(|tool| {
        // Convert ToolParameters to JSON Value
        let parameters_json = serde_json::to_value(&tool.parameters)
            .unwrap_or_else(|_| serde_json::json!({}));

        AbstractionTool {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: parameters_json,
        }
    }).collect()
}

/// Convert ModelResponse to ExecutionResponse
pub fn convert_to_execution_response(
    response: ModelResponse,
    model: &str,
) -> radium_core::engines::ExecutionResponse {
    radium_core::engines::ExecutionResponse {
        content: response.content,
        usage: response.usage.map(|u| radium_core::engines::TokenUsage {
            input_tokens: u.prompt_tokens as u64,
            output_tokens: u.completion_tokens as u64,
            total_tokens: u.total_tokens as u64,
        }),
        model: response.model_id.unwrap_or_else(|| model.to_string()),
        raw: None,
        execution_duration: None,
        metadata: response.metadata,
    }
}

/// Execute a single tool call with progress indicator
pub async fn execute_tool_call(
    tool_call: &ToolCall,
    tools: &[OrchestrationTool],
    _workspace_root: &PathBuf,
) -> Result<String> {
    // Find the tool by name
    let tool = tools.iter()
        .find(|t| t.name == tool_call.name)
        .ok_or_else(|| anyhow!("Tool not found: {}", tool_call.name))?;

    // Create progress spinner with Radium brand colors
    // Note: indicatif uses named colors in templates; .cyan matches our primary brand color
    let colors = RadiumBrandColors::new();
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.cyan} {msg}") // .cyan matches Radium primary brand color (#00D9FF)
            .unwrap()
    );
    spinner.set_message(format!("Executing {}...", tool_call.name.color(colors.primary())));
    spinner.enable_steady_tick(Duration::from_millis(100));

    // Execute the tool using the execute method
    let args = ToolArguments::new(tool_call.arguments.clone());
    let result = tool.execute(&args).await
        .map_err(|e| {
            spinner.finish_and_clear();
            anyhow!("Tool execution failed: {}", e)
        })?;

    // Finish spinner with success message
    spinner.finish_with_message(format!(
        "{} {} ({} bytes)",
        "✓".green(),
        tool_call.name.cyan(),
        result.output.len().to_string().dimmed()
    ));

    Ok(result.output)
}

/// Multi-turn tool execution loop
///
/// This function handles the conversation loop where the model can call tools
/// and continue the conversation based on tool results.
pub async fn execute_with_tools_loop(
    model: &dyn Model,
    mut messages: Vec<ChatMessage>,
    tools: &[AbstractionTool],
    tool_config: &ToolConfig,
    orchestration_tools: &[OrchestrationTool],
    workspace_root: &PathBuf,
) -> Result<ModelResponse> {
    const MAX_ITERATIONS: usize = 10;

    for iteration in 0..MAX_ITERATIONS {
        // Show progress for iterations beyond the first
        if iteration > 0 {
            println!("\n  {} Processing iteration {}/{}...", "→".dimmed(), iteration + 1, MAX_ITERATIONS);
        }

        // Call model with tools
        let response = match model.generate_with_tools(
            &messages,
            tools,
            Some(tool_config),
        ).await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("\n  {} Model execution failed: {}", "✗".red(), e);
                return Err(anyhow!("Model execution failed: {}", e));
            }
        };

        // Check if model wants to call tools
        if let Some(ref tool_calls) = response.tool_calls {
            if tool_calls.is_empty() {
                // No tool calls, return final response
                return Ok(response);
            }

            // Show tool call count
            if tool_calls.len() == 1 {
                println!("\n  {} Calling 1 tool...", "🔧".dimmed());
            } else {
                println!("\n  {} Calling {} tools...", "🔧".dimmed(), tool_calls.len());
            }

            // Add assistant's message to conversation
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: MessageContent::Text(response.content.clone()),
            });

            // Execute each tool call and add results
            for tool_call in tool_calls {
                let result = execute_tool_call(tool_call, orchestration_tools, workspace_root).await?;

                // Add tool result as a user message (like TUI does)
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: MessageContent::Text(format!(
                        "[Tool result for {}]\n{}",
                        tool_call.name, result
                    )),
                });
            }

            // Continue loop to get next response
            continue;
        }

        // No tool calls - return final response
        return Ok(response);
    }

    Err(anyhow!("Tool execution loop exceeded maximum iterations ({})", MAX_ITERATIONS))
}
