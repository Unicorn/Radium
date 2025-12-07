// Orchestration engine for multi-turn tool execution
//
// This engine coordinates between orchestration providers and tool execution,
// handling the full loop of: input -> model decision -> tool execution -> result -> repeat.

use std::sync::Arc;
use tokio::time::{timeout, Duration};

use super::{
    FinishReason, OrchestrationProvider, OrchestrationResult,
    context::{Message, OrchestrationContext},
    tool::{Tool, ToolArguments, ToolCall},
};
use crate::error::{OrchestrationError, Result};

/// Configuration for orchestration engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum number of tool execution iterations
    pub max_iterations: usize,
    /// Maximum time (in seconds) for entire orchestration
    pub timeout_seconds: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { max_iterations: 5, timeout_seconds: 120 }
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
}

impl OrchestrationEngine {
    /// Create a new orchestration engine
    pub fn new(
        provider: Arc<dyn OrchestrationProvider>,
        tools: Vec<Tool>,
        config: EngineConfig,
    ) -> Self {
        Self { provider, tools, config }
    }

    /// Create engine with default configuration
    pub fn with_defaults(provider: Arc<dyn OrchestrationProvider>, tools: Vec<Tool>) -> Self {
        Self::new(provider, tools, EngineConfig::default())
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
        
        // Wrap the entire execution in a timeout
        match timeout(timeout_duration, self.execute_internal(input, context)).await {
            Ok(result) => result,
            Err(_) => {
                // Timeout occurred
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

        loop {
            // Check iteration limit
            if iterations >= self.config.max_iterations {
                return Ok(OrchestrationResult::new(
                    format!("Reached maximum iterations ({})", self.config.max_iterations),
                    vec![],
                    FinishReason::MaxIterations,
                ));
            }

            // Get orchestration decision from provider
            let result = match self.provider.execute_with_tools(&current_input, &self.tools, context).await {
                Ok(r) => r,
                Err(e) => {
                    // Provider error - check if it's a function calling error that should trigger fallback
                    // This will be handled by the service layer if fallback is enabled
                    return Err(e);
                }
            };

            // If no tool calls, we're done
            if result.tool_calls.is_empty() {
                // Add final assistant message to conversation
                if !result.response.is_empty() {
                    context.add_assistant_message(&result.response);
                }
                return Ok(result);
            }

            // Execute tool calls
            let tool_results = self.execute_tools(&result.tool_calls).await?;

            // Add assistant message with tool calls to conversation
            if !result.response.is_empty() {
                context.add_assistant_message(&result.response);
            }

            // Add tool results to conversation
            for (i, tool_result) in tool_results.iter().enumerate() {
                let tool_call = &result.tool_calls[i];
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
                let error_msg = tool_results
                    .iter()
                    .filter(|r| !r.success)
                    .map(|r| r.output.as_str())
                    .collect::<Vec<_>>()
                    .join("; ");
                return Ok(OrchestrationResult::new(error_msg, vec![], FinishReason::ToolError));
            }

            // Prepare next iteration input (tool results summary)
            current_input =
                tool_results.iter().map(|r| r.output.as_str()).collect::<Vec<_>>().join("\n");

            iterations += 1;
        }
    }

    /// Execute all tool calls and collect results
    async fn execute_tools(
        &self,
        tool_calls: &[ToolCall],
    ) -> Result<Vec<crate::orchestration::tool::ToolResult>> {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            // Find matching tool
            let tool = self.tools.iter().find(|t| t.name == tool_call.name).ok_or_else(|| {
                OrchestrationError::Other(format!("Tool '{}' not found", tool_call.name))
            })?;

            // Execute tool
            let args = ToolArguments::new(tool_call.arguments.clone());
            let result = tool.execute(&args).await?;
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

        let engine = OrchestrationEngine::with_defaults(provider, vec![]);
        let mut context = OrchestrationContext::new("test-session");

        let result = engine.execute("Test input", &mut context).await.unwrap();
        assert_eq!(result.response, "Done");
        assert_eq!(result.finish_reason, FinishReason::Stop);
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
            EngineConfig { max_iterations: 3, timeout_seconds: 120 },
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
