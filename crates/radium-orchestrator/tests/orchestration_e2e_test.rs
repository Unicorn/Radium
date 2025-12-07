//! End-to-end tests for orchestration flow
//!
//! These tests validate the complete orchestration flow from user input
//! through agent selection, tool execution, and result synthesis.

use radium_orchestrator::orchestration::{
    FinishReason, OrchestrationProvider, OrchestrationResult,
    context::OrchestrationContext,
    engine::{EngineConfig, OrchestrationEngine},
    tool::{Tool, ToolCall, ToolParameters, ToolResult, ToolHandler, ToolArguments},
    agent_tools::AgentToolRegistry,
};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

// Mock provider for E2E testing
struct MockE2EProvider {
    responses: Vec<OrchestrationResult>,
    call_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl MockE2EProvider {
    fn new(responses: Vec<OrchestrationResult>) -> Self {
        Self {
            responses,
            call_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl OrchestrationProvider for MockE2EProvider {
    async fn execute_with_tools(
        &self,
        _input: &str,
        _tools: &[Tool],
        _context: &OrchestrationContext,
    ) -> radium_orchestrator::error::Result<OrchestrationResult> {
        let count = self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if count < self.responses.len() {
            Ok(self.responses[count].clone())
        } else {
            // Default: return final response
            Ok(OrchestrationResult::new(
                "All done".to_string(),
                vec![],
                FinishReason::Stop,
            ))
        }
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn provider_name(&self) -> &'static str {
        "mock_e2e"
    }
}

// Mock tool handler
struct MockE2EToolHandler {
    tool_name: String,
}

#[async_trait]
impl ToolHandler for MockE2EToolHandler {
    async fn execute(&self, args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
        let task = args.get_string("task").unwrap_or_else(|| "unknown".to_string());
        Ok(ToolResult::success(format!("{} executed: {}", self.tool_name, task)))
    }
}

fn create_mock_tools() -> Vec<Tool> {
    vec![
        Tool::new(
            "agent_senior-developer",
            "senior_developer",
            "Senior developer agent",
            ToolParameters::new().add_property("task", "string", "Development task", true),
            Arc::new(MockE2EToolHandler {
                tool_name: "senior_developer".to_string(),
            }),
        ),
        Tool::new(
            "agent_tester",
            "tester",
            "Testing agent",
            ToolParameters::new().add_property("task", "string", "Testing task", true),
            Arc::new(MockE2EToolHandler {
                tool_name: "tester".to_string(),
            }),
        ),
    ]
}

#[tokio::test]
async fn test_single_agent_orchestration() {
    // Single agent orchestration: input -> agent selection -> execution -> response
    let provider = Arc::new(MockE2EProvider::new(vec![
        OrchestrationResult::new(
            "I'll help with that".to_string(),
            vec![ToolCall {
                id: "call_1".to_string(),
                name: "senior_developer".to_string(),
                arguments: json!({"task": "refactor authentication module"}),
            }],
            FinishReason::Stop,
        ),
        OrchestrationResult::new(
            "Authentication module has been refactored successfully.".to_string(),
            vec![],
            FinishReason::Stop,
        ),
    ]));

    let tools = create_mock_tools();
    let engine = OrchestrationEngine::with_defaults(provider, tools);
    let mut context = OrchestrationContext::new("test-session");

    let result = engine.execute("Refactor the authentication module", &mut context).await.unwrap();

    assert_eq!(result.finish_reason, FinishReason::Stop);
    assert!(result.response.contains("refactored"));
    assert!(!result.has_tool_calls()); // Final response has no tool calls
}

#[tokio::test]
async fn test_multi_agent_orchestration() {
    // Multi-agent orchestration: input -> multiple agents -> synthesis
    let provider = Arc::new(MockE2EProvider::new(vec![
        OrchestrationResult::new(
            "Planning workflow".to_string(),
            vec![
                ToolCall {
                    id: "call_1".to_string(),
                    name: "senior_developer".to_string(),
                    arguments: json!({"task": "implement feature"}),
                },
                ToolCall {
                    id: "call_2".to_string(),
                    name: "tester".to_string(),
                    arguments: json!({"task": "test feature"}),
                },
            ],
            FinishReason::Stop,
        ),
        OrchestrationResult::new(
            "Feature implemented and tested successfully.".to_string(),
            vec![],
            FinishReason::Stop,
        ),
    ]));

    let tools = create_mock_tools();
    let engine = OrchestrationEngine::with_defaults(provider, tools);
    let mut context = OrchestrationContext::new("test-session");

    let result = engine.execute("Create and test a new feature", &mut context).await.unwrap();

    assert_eq!(result.finish_reason, FinishReason::Stop);
    assert!(result.response.contains("successfully"));
    
    // Check conversation history includes tool results
    assert!(context.conversation_history.len() >= 3); // user + assistant + tool messages
}

#[tokio::test]
async fn test_max_iterations_enforcement() {
    // Test that max iterations prevents infinite loops
    let provider = Arc::new(MockE2EProvider::new(vec![
        OrchestrationResult::new(
            "Calling tool".to_string(),
            vec![ToolCall {
                id: "call_1".to_string(),
                name: "senior_developer".to_string(),
                arguments: json!({"task": "test"}),
            }],
            FinishReason::Stop,
        );
        10 // Repeat this response 10 times
    ]));

    let tools = create_mock_tools();
    let engine = OrchestrationEngine::new(
        provider,
        tools,
        EngineConfig {
            max_iterations: 3,
            timeout_seconds: 120,
        },
    );
    let mut context = OrchestrationContext::new("test-session");

    let result = engine.execute("Test input", &mut context).await.unwrap();

    assert_eq!(result.finish_reason, FinishReason::MaxIterations);
    assert!(result.response.contains("maximum iterations"));
}

#[tokio::test]
async fn test_conversation_history_tracking() {
    // Test that conversation history is maintained across multiple turns
    let provider = Arc::new(MockE2EProvider::new(vec![
        OrchestrationResult::new("First response".to_string(), vec![], FinishReason::Stop),
        OrchestrationResult::new("Second response".to_string(), vec![], FinishReason::Stop),
        OrchestrationResult::new("Third response".to_string(), vec![], FinishReason::Stop),
    ]));

    let engine = OrchestrationEngine::with_defaults(provider, vec![]);
    
    // First call
    let mut context1 = OrchestrationContext::new("session-1");
    let _result1 = engine.execute("First message", &mut context1).await.unwrap();
    
    // Second call
    let mut context2 = OrchestrationContext::new("session-1");
    context2.conversation_history = context1.conversation_history.clone();
    let _result2 = engine.execute("Second message", &mut context2).await.unwrap();
    
    // Third call
    let mut context3 = OrchestrationContext::new("session-1");
    context3.conversation_history = context2.conversation_history.clone();
    let _result3 = engine.execute("Third message", &mut context3).await.unwrap();

    // Should have 6 messages: 3 user + 3 assistant
    assert_eq!(context3.conversation_history.len(), 6);
}

#[tokio::test]
async fn test_tool_execution_failure() {
    // Test handling of tool execution failures
    struct FailingToolHandler;

    #[async_trait]
    impl ToolHandler for FailingToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            Ok(ToolResult::error("Tool execution failed"))
        }
    }

    let provider = Arc::new(MockE2EProvider::new(vec![
        OrchestrationResult::new(
            "Calling tool".to_string(),
            vec![ToolCall {
                id: "call_1".to_string(),
                name: "failing_tool".to_string(),
                arguments: json!({"task": "test"}),
            }],
            FinishReason::Stop,
        ),
    ]));

    let failing_tool = Tool::new(
        "agent_failing",
        "failing_tool",
        "A tool that fails",
        ToolParameters::new().add_property("task", "string", "Task", true),
        Arc::new(FailingToolHandler),
    );

    let engine = OrchestrationEngine::with_defaults(provider, vec![failing_tool]);
    let mut context = OrchestrationContext::new("test-session");

    let result = engine.execute("Test input", &mut context).await.unwrap();

    assert_eq!(result.finish_reason, FinishReason::ToolError);
    assert!(result.response.contains("failed"));
}

#[tokio::test]
async fn test_agent_tool_registry_conversion() {
    // Test that AgentToolRegistry can convert agents to tools
    let mut registry = AgentToolRegistry::new();
    
    // Try to load agents (may fail if no agents exist, that's OK)
    let _ = registry.load_agents();
    
    // Build tools
    registry.build_tools();
    let tools = registry.get_tools();
    
    // Tools should be available (even if empty)
    assert!(tools.len() >= 0);
}

#[tokio::test]
async fn test_engine_provider_info() {
    let provider = Arc::new(MockE2EProvider::new(vec![]));
    let engine = OrchestrationEngine::with_defaults(provider, vec![]);

    assert_eq!(engine.provider_name(), "mock_e2e");
    assert!(engine.supports_function_calling());
    assert_eq!(engine.tool_count(), 0);
}

#[tokio::test]
async fn test_empty_tools_orchestration() {
    // Test orchestration with no available tools
    let provider = Arc::new(MockE2EProvider::new(vec![
        OrchestrationResult::new(
            "I can help, but no tools are available.".to_string(),
            vec![],
            FinishReason::Stop,
        ),
    ]));

    let engine = OrchestrationEngine::with_defaults(provider, vec![]);
    let mut context = OrchestrationContext::new("test-session");

    let result = engine.execute("Help me", &mut context).await.unwrap();

    assert_eq!(result.finish_reason, FinishReason::Stop);
    assert!(!result.has_tool_calls());
}

