// Orchestration engine for multi-turn tool execution
//
// This engine coordinates between orchestration providers and tool execution,
// handling the full loop of: input -> model decision -> tool execution -> result -> repeat.

use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};

use super::{
    FinishReason, OrchestrationProvider, OrchestrationResult,
    context::{Message, OrchestrationContext},
    events::OrchestrationEvent,
    execution::execute_tool_calls,
    hooks::ToolHookExecutor,
    tool::{Tool, ToolArguments, ToolCall},
    ToolExecutionConfig,
};
use crate::error::{OrchestrationError, Result};
use tracing::warn;

/// Configuration for orchestration engine
///
/// The engine uses parallel execution by default. When multiple tool calls are requested,
/// they execute concurrently using `tokio::spawn`, significantly improving performance
/// compared to sequential execution. The `tool_execution` configuration controls this behavior.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum number of tool execution iterations
    pub max_iterations: usize,
    /// Maximum time (in seconds) for entire orchestration
    pub timeout_seconds: u64,
    /// Configuration for executing multiple tool calls (concurrency, batching, timeouts)
    ///
    /// Defaults to `FunctionExecutionStrategy::Concurrent`, which executes all tool calls
    /// in parallel. This provides significant performance improvements when multiple independent
    /// tools need to be executed (e.g., multiple agent tools working on different subtasks).
    pub tool_execution: ToolExecutionConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            timeout_seconds: 120,
            tool_execution: ToolExecutionConfig::default(),
        }
    }
}

/// Orchestration engine coordinating providers and tool execution
pub struct OrchestrationEngine {
    /// Provider for orchestration decisions
    provider: Arc<dyn OrchestrationProvider>,
    /// Available tools
    tools: Vec<Tool>,
    /// Engine configuration
    config: EngineConfig,
    /// Optional hook executor for tool execution hooks
    hook_executor: Option<Arc<dyn ToolHookExecutor>>,
    /// Optional event sender for streaming orchestration progress
    event_tx: Option<broadcast::Sender<OrchestrationEvent>>,
}

impl OrchestrationEngine {
    /// Create a new orchestration engine
    pub fn new(
        provider: Arc<dyn OrchestrationProvider>,
        tools: Vec<Tool>,
        config: EngineConfig,
    ) -> Self {
        Self {
            provider,
            tools,
            config,
            hook_executor: None,
            event_tx: None,
        }
    }

    /// Create engine with default configuration
    pub fn with_defaults(provider: Arc<dyn OrchestrationProvider>, tools: Vec<Tool>) -> Self {
        Self::new(provider, tools, EngineConfig::default())
    }

    /// Create engine with hook executor
    pub fn with_hook_executor(
        provider: Arc<dyn OrchestrationProvider>,
        tools: Vec<Tool>,
        config: EngineConfig,
        hook_executor: Option<Arc<dyn ToolHookExecutor>>,
    ) -> Self {
        Self {
            provider,
            tools,
            config,
            hook_executor,
            event_tx: None,
        }
    }

    /// Set the event sender used to emit orchestration events.
    pub fn set_event_sender(&mut self, event_tx: Option<broadcast::Sender<OrchestrationEvent>>) {
        self.event_tx = event_tx;
    }

    fn emit(&self, event: OrchestrationEvent) {
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(event);
        }
    }

    /// Set hook executor
    pub fn set_hook_executor(&mut self, hook_executor: Option<Arc<dyn ToolHookExecutor>>) {
        self.hook_executor = hook_executor;
    }

    /// Execute orchestration with multi-turn tool execution loop
    ///
    /// This handles the full orchestration loop:
    /// 1. Send user input to provider
    /// 2. Execute any requested tools
    /// 3. Add results to conversation
    /// 4. Repeat until complete or max iterations reached
    ///
    /// The execution is wrapped in a timeout to prevent indefinite hanging.
    pub async fn execute(
        &self,
        input: &str,
        context: &mut OrchestrationContext,
    ) -> Result<OrchestrationResult> {
        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        let correlation_id = context.session_id.clone();
        
        // Wrap the entire execution in a timeout
        match timeout(timeout_duration, self.execute_internal(input, context)).await {
            Ok(result) => result,
            Err(_) => {
                // Timeout occurred
                self.emit(OrchestrationEvent::Error {
                    correlation_id: correlation_id.clone(),
                    message: format!(
                        "Orchestration timed out after {} seconds",
                        self.config.timeout_seconds
                    ),
                });
                self.emit(OrchestrationEvent::Done {
                    correlation_id,
                    finish_reason: FinishReason::Error.to_string(),
                });
                Ok(OrchestrationResult::new(
                    format!("Orchestration timed out after {} seconds", self.config.timeout_seconds),
                    vec![],
                    FinishReason::Error,
                ))
            }
        }
    }

    /// Internal execution logic (without timeout wrapper)
    async fn execute_internal(
        &self,
        input: &str,
        context: &mut OrchestrationContext,
    ) -> Result<OrchestrationResult> {
        let mut iterations = 0;
        let mut current_input = input.to_string();
        let correlation_id = context.session_id.clone();

        self.emit(OrchestrationEvent::UserInput {
            correlation_id: correlation_id.clone(),
            content: current_input.clone(),
        });

        loop {
            // Check iteration limit
            if iterations >= self.config.max_iterations {
                let error_msg = format!(
                    "Reached maximum iterations ({}) while processing: {}\n\n\
                    This usually means:\n\
                    - The task requires more steps than allowed\n\
                    - There's a loop in tool execution\n\
                    - The orchestrator is having trouble completing the task\n\n\
                    Suggestions:\n\
                    - Break the task into smaller requests\n\
                    - Increase max_iterations in orchestration config\n\
                    - Check if tools are calling each other in a loop",
                    self.config.max_iterations,
                    if current_input.len() > 100 {
                        format!("{}...", &current_input[..100])
                    } else {
                        current_input.clone()
                    }
                );
                warn!("Orchestration reached max iterations: {}", iterations);
                self.emit(OrchestrationEvent::Done {
                    correlation_id: correlation_id.clone(),
                    finish_reason: FinishReason::MaxIterations.to_string(),
                });
                return Ok(OrchestrationResult::new(error_msg, vec![], FinishReason::MaxIterations));
            }

            // Get orchestration decision from provider
            let result = match self.provider.execute_with_tools(&current_input, &self.tools, context).await {
                Ok(r) => r,
                Err(e) => {
                    // Provider error - check if it's a function calling error that should trigger fallback
                    // This will be handled by the service layer if fallback is enabled
                    self.emit(OrchestrationEvent::Error {
                        correlation_id: correlation_id.clone(),
                        message: e.to_string(),
                    });
                    return Err(e);
                }
            };

            // If no tool calls, we're done
            if result.tool_calls.is_empty() {
                // Add final assistant message to conversation
                if !result.response.is_empty() {
                    context.add_assistant_message(&result.response);
                }
                self.emit(OrchestrationEvent::AssistantMessage {
                    correlation_id: correlation_id.clone(),
                    content: result.response.clone(),
                });
                self.emit(OrchestrationEvent::Done {
                    correlation_id: correlation_id.clone(),
                    finish_reason: result.finish_reason.to_string(),
                });
                return Ok(result);
            }

            // Execute tool calls
            if !result.response.is_empty() {
                self.emit(OrchestrationEvent::AssistantMessage {
                    correlation_id: correlation_id.clone(),
                    content: result.response.clone(),
                });
            }
            for call in &result.tool_calls {
                self.emit(OrchestrationEvent::ToolCallRequested {
                    correlation_id: correlation_id.clone(),
                    call: call.clone(),
                });
            }

            let tool_results = self.execute_tools(&correlation_id, &result.tool_calls).await?;

            // Add assistant message with tool calls to conversation
            if !result.response.is_empty() {
                context.add_assistant_message(&result.response);
            }

            // Add tool results to conversation
            for (i, tool_result) in tool_results.iter().enumerate() {
                let tool_call = &result.tool_calls[i];
                self.emit(OrchestrationEvent::ToolCallFinished {
                    correlation_id: correlation_id.clone(),
                    tool_name: tool_call.name.clone(),
                    result: tool_result.clone(),
                });
                let result_message =
                    format!("Tool '{}' returned: {}", tool_call.name, tool_result.output);
                context.add_message(Message {
                    role: "tool".to_string(),
                    content: result_message,
                    timestamp: chrono::Utc::now(),
                });
            }

            // Check if any tool failed
            if tool_results.iter().any(|r| !r.success) {
                let failed_tools: Vec<_> = tool_results
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| !r.success)
                    .map(|(i, r)| {
                        let tool_call = &result.tool_calls[i];
                        let file_path = r.metadata.get("file_path")
                            .map(|p| format!(" (file: {})", p))
                            .unwrap_or_default();
                        format!("Tool '{}'{} failed: {}", tool_call.name, file_path, r.output)
                    })
                    .collect();
                
                let error_msg = format!(
                    "Tool execution failed:\n\n{}\n\n\
                    Suggestions:\n\
                    - Check file paths and permissions\n\
                    - Verify tool arguments are correct\n\
                    - Review error messages above for specific issues",
                    failed_tools.join("\n")
                );
                warn!("Tool execution failed: {} tool(s) failed", failed_tools.len());
                self.emit(OrchestrationEvent::Done {
                    correlation_id: correlation_id.clone(),
                    finish_reason: FinishReason::ToolError.to_string(),
                });
                return Ok(OrchestrationResult::new(error_msg, vec![], FinishReason::ToolError));
            }

            // Prepare next iteration input (tool results summary)
            current_input =
                tool_results.iter().map(|r| r.output.as_str()).collect::<Vec<_>>().join("\n");

            iterations += 1;
        }
    }

    /// Execute all tool calls and collect results
    ///
    /// This method executes tool calls according to the configured execution strategy.
    /// By default, tool calls execute concurrently in parallel using `tokio::spawn`,
    /// which provides significant performance improvements when multiple independent tools
    /// are called (e.g., multiple agent tools working on different subtasks).
    ///
    /// When hooks are not installed, this uses the optimized parallel execution path.
    /// When hooks are installed, tools execute sequentially to ensure proper hook ordering.
    async fn execute_tools(
        &self,
        correlation_id: &str,
        tool_calls: &[ToolCall],
    ) -> Result<Vec<crate::orchestration::tool::ToolResult>> {
        // Fast path: when no hooks are installed, use the shared parallel execution engine.
        // This executes all tool calls concurrently, providing significant performance
        // improvements for multi-agent scenarios where multiple tools can run in parallel.
        if self.hook_executor.is_none() {
            // Emit "started" for each tool call upfront (execution may run concurrently).
            for call in tool_calls {
                self.emit(OrchestrationEvent::ToolCallStarted {
                    correlation_id: correlation_id.to_string(),
                    tool_name: call.name.clone(),
                });
            }

            // Execute tools concurrently according to tool_execution configuration.
            // Default strategy is Concurrent, which uses tokio::spawn for parallel execution.
            let raw_results = execute_tool_calls(tool_calls, &self.tools, &self.config.tool_execution).await;
            let mut results = Vec::with_capacity(raw_results.len());
            for res in raw_results {
                results.push(res?);
            }
            return Ok(results);
        }

        let mut results = Vec::new();

        for tool_call in tool_calls {
            // Find matching tool
            let tool = self.tools.iter().find(|t| t.name == tool_call.name).ok_or_else(|| {
                let available_tools: Vec<&str> = self.tools.iter().map(|t| t.name.as_str()).collect();
                let similar_tools: Vec<&str> = available_tools
                    .iter()
                    .filter(|name| name.contains(&tool_call.name) || tool_call.name.contains(**name))
                    .copied()
                    .collect();
                
                let mut error_msg = format!(
                    "Tool '{}' not found.\n\nAvailable tools ({}): {}",
                    tool_call.name,
                    available_tools.len(),
                    available_tools.join(", ")
                );
                
                if !similar_tools.is_empty() {
                    error_msg.push_str(&format!("\n\nDid you mean: {}", similar_tools.join(", ")));
                }
                
                OrchestrationError::Other(error_msg)
            })?;

            // Execute before_tool hooks if available
            let mut effective_arguments = tool_call.arguments.clone();
            if let Some(ref hook_executor) = self.hook_executor {
                match hook_executor.before_tool_execution(&tool_call.name, &effective_arguments).await {
                    Ok(modified_args) => {
                        effective_arguments = modified_args;
                        tracing::debug!("BeforeTool hooks modified arguments for tool: {}", tool_call.name);
                    }
                    Err(e) => {
                        // Check if this is an approval request (from policy hook)
                        if e.starts_with("APPROVAL_REQUIRED:") {
                            let reason = e.strip_prefix("APPROVAL_REQUIRED:").unwrap_or(&e).trim().to_string();
                            self.emit(OrchestrationEvent::ApprovalRequired {
                                correlation_id: correlation_id.to_string(),
                                tool_name: tool_call.name.clone(),
                                reason,
                            });
                            // Return error to pause execution - user must approve via CLI/TUI
                            return Err(OrchestrationError::Other(format!(
                                "Tool execution requires approval: {}",
                                tool_call.name
                            )));
                        }
                        // Hook requested to abort execution
                        tracing::warn!("BeforeTool hook aborted execution for tool {}: {}", tool_call.name, e);
                        return Err(OrchestrationError::Other(format!(
                            "Tool execution aborted by hook: {}",
                            e
                        )));
                    }
                }
            }

            // Execute tool
            self.emit(OrchestrationEvent::ToolCallStarted {
                correlation_id: correlation_id.to_string(),
                tool_name: tool_call.name.clone(),
            });
            let args = ToolArguments::new(effective_arguments.clone());
            let mut result = match tool.execute(&args).await {
                Ok(r) => r,
                Err(e) => {
                    // Enhance error with tool context
                    let error_msg = format!(
                        "Tool '{}' execution error: {}\n\n\
                        Tool arguments: {}\n\
                        Suggestions:\n\
                        - Verify all required arguments are provided\n\
                        - Check argument types and formats\n\
                        - Review tool description: {}",
                        tool_call.name,
                        e,
                        serde_json::to_string_pretty(&effective_arguments).unwrap_or_else(|_| "invalid JSON".to_string()),
                        tool.description
                    );
                    warn!("Tool execution error: {} - {}", tool_call.name, e);
                    return Err(OrchestrationError::Other(error_msg));
                }
            };

            // Execute after_tool hooks if available
            if let Some(ref hook_executor) = self.hook_executor {
                let result_json = serde_json::json!({
                    "success": result.success,
                    "output": result.output,
                    "metadata": result.metadata,
                });

                match hook_executor.after_tool_execution(&tool_call.name, &effective_arguments, &result_json).await {
                    Ok(modified_result) => {
                        // Update result if hooks modified it
                        if let Some(success) = modified_result.get("success").and_then(|v| v.as_bool()) {
                            result.success = success;
                        }
                        if let Some(output) = modified_result.get("output").and_then(|v| v.as_str()) {
                            result.output = output.to_string();
                        }
                        if let Some(metadata) = modified_result.get("metadata").and_then(|v| v.as_object()) {
                            for (key, value) in metadata {
                                if let Some(val_str) = value.as_str() {
                                    result.metadata.insert(key.clone(), val_str.to_string());
                                }
                            }
                        }
                        tracing::debug!("AfterTool hooks modified result for tool: {}", tool_call.name);
                    }
                    Err(e) => {
                        tracing::warn!("AfterTool hook error for tool {}: {}", tool_call.name, e);
                        // Continue with original result even if hook fails
                    }
                }
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Get provider name
    pub fn provider_name(&self) -> &'static str {
        self.provider.provider_name()
    }

    /// Check if provider supports function calling
    pub fn supports_function_calling(&self) -> bool {
        self.provider.supports_function_calling()
    }

    /// Get number of available tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::tool::{ToolHandler, ToolParameters, ToolResult};
    use async_trait::async_trait;
    use serde_json::json;

    // Mock provider for testing
    struct MockProvider {
        responses: Vec<OrchestrationResult>,
        call_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl MockProvider {
        fn new(responses: Vec<OrchestrationResult>) -> Self {
            Self {
                responses,
                call_count: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait]
    impl OrchestrationProvider for MockProvider {
        async fn execute_with_tools(
            &self,
            _input: &str,
            _tools: &[Tool],
            _context: &OrchestrationContext,
        ) -> Result<OrchestrationResult> {
            let count = self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if count < self.responses.len() {
                Ok(self.responses[count].clone())
            } else {
                Ok(OrchestrationResult::new("Done".to_string(), vec![], FinishReason::Stop))
            }
        }

        fn supports_function_calling(&self) -> bool {
            true
        }

        fn provider_name(&self) -> &'static str {
            "mock"
        }
    }

    // Mock tool handler for testing
    struct MockHandler;

    #[async_trait]
    impl ToolHandler for MockHandler {
        async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
            let task = args.get_string("task").unwrap_or_else(|| "unknown".to_string());
            Ok(ToolResult::success(format!("Executed: {}", task)))
        }
    }

    #[tokio::test]
    async fn test_engine_simple_execution() {
        let provider = Arc::new(MockProvider::new(vec![OrchestrationResult::new(
            "Done".to_string(),
            vec![],
            FinishReason::Stop,
        )]));

        let (tx, mut rx) = broadcast::channel(16);
        let mut engine = OrchestrationEngine::with_defaults(provider, vec![]);
        engine.set_event_sender(Some(tx));
        let mut context = OrchestrationContext::new("test-session");

        let result = engine.execute("Test input", &mut context).await.unwrap();
        assert_eq!(result.response, "Done");
        assert_eq!(result.finish_reason, FinishReason::Stop);

        // Verify we emitted basic lifecycle events.
        let mut seen_user_input = false;
        let mut seen_done = false;
        while let Ok(event) = rx.try_recv() {
            match event {
                OrchestrationEvent::UserInput { .. } => seen_user_input = true,
                OrchestrationEvent::Done { .. } => seen_done = true,
                _ => {}
            }
        }
        assert!(seen_user_input);
        assert!(seen_done);
    }

    #[tokio::test]
    async fn test_engine_tool_execution() {
        let tool = Tool::new(
            "test_agent",
            "test_tool",
            "Test tool",
            ToolParameters::new().add_property("task", "string", "Task to perform", true),
            Arc::new(MockHandler),
        );

        let provider = Arc::new(MockProvider::new(vec![
            // First call: request tool
            OrchestrationResult::new(
                "Calling tool".to_string(),
                vec![ToolCall {
                    id: "call_1".to_string(),
                    name: "test_tool".to_string(),
                    arguments: json!({"task": "test task"}),
                }],
                FinishReason::Stop,
            ),
            // Second call: finish
            OrchestrationResult::new("All done".to_string(), vec![], FinishReason::Stop),
        ]));

        let engine = OrchestrationEngine::with_defaults(provider, vec![tool]);
        let mut context = OrchestrationContext::new("test-session");

        let result = engine.execute("Test input", &mut context).await.unwrap();
        assert_eq!(result.response, "All done");
        assert_eq!(result.finish_reason, FinishReason::Stop);

        // Check conversation history
        assert!(context.conversation_history.len() >= 2); // At least assistant + tool messages
    }

    #[tokio::test]
    async fn test_engine_max_iterations() {
        // Provider that always requests tools
        let provider = Arc::new(MockProvider::new(vec![
            OrchestrationResult::new(
                "Calling tool".to_string(),
                vec![ToolCall {
                    id: "call_1".to_string(),
                    name: "test_tool".to_string(),
                    arguments: json!({"task": "test"}),
                }],
                FinishReason::Stop,
            );
            10
        ])); // More responses than max iterations

        let tool = Tool::new(
            "test_agent",
            "test_tool",
            "Test tool",
            ToolParameters::new().add_property("task", "string", "Task", true),
            Arc::new(MockHandler),
        );

        let engine = OrchestrationEngine::new(
            provider,
            vec![tool],
            EngineConfig {
                max_iterations: 3,
                timeout_seconds: 120,
                tool_execution: ToolExecutionConfig::default(),
            },
        );

        let mut context = OrchestrationContext::new("test-session");
        let result = engine.execute("Test input", &mut context).await.unwrap();

        assert_eq!(result.finish_reason, FinishReason::MaxIterations);
    }

    #[tokio::test]
    async fn test_engine_tool_not_found() {
        let provider = Arc::new(MockProvider::new(vec![OrchestrationResult::new(
            "Calling tool".to_string(),
            vec![ToolCall {
                id: "call_1".to_string(),
                name: "nonexistent_tool".to_string(),
                arguments: json!({}),
            }],
            FinishReason::Stop,
        )]));

        let engine = OrchestrationEngine::with_defaults(provider, vec![]);
        let mut context = OrchestrationContext::new("test-session");

        let result = engine.execute("Test input", &mut context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_engine_provider_info() {
        let provider = Arc::new(MockProvider::new(vec![]));
        let engine = OrchestrationEngine::with_defaults(provider, vec![]);

        assert_eq!(engine.provider_name(), "mock");
        assert!(engine.supports_function_calling());
        assert_eq!(engine.tool_count(), 0);
    }
}
