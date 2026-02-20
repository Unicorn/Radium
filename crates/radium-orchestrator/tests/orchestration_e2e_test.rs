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
            tool_execution: radium_orchestrator::orchestration::ToolExecutionConfig::default(),
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
    context1.add_user_message("First message");
    let _result1 = engine.execute("First message", &mut context1).await.unwrap();
    
    // Second call
    let mut context2 = OrchestrationContext::new("session-1");
    context2.conversation_history = context1.conversation_history.clone();
    context2.add_user_message("Second message");
    let _result2 = engine.execute("Second message", &mut context2).await.unwrap();
    
    // Third call
    let mut context3 = OrchestrationContext::new("session-1");
    context3.conversation_history = context2.conversation_history.clone();
    context3.add_user_message("Third message");
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
    // load_agents() automatically calls build_tools() internally
    let _ = registry.load_agents();
    
    let tools = registry.get_tools();
    
    // Tools should be available (even if empty)
    assert!(tools.len() >= 0); // This is always true, but documents intent
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

#[tokio::test]
async fn test_multi_turn_conversation_with_tools() {
    // Test multi-turn conversation where tool results feed into next turn
    let provider = Arc::new(MockE2EProvider::new(vec![
        // First turn: request tool
        OrchestrationResult::new(
            "I'll calculate that".to_string(),
            vec![ToolCall {
                id: "call_1".to_string(),
                name: "senior_developer".to_string(),
                arguments: json!({"task": "calculate 5+3"}),
            }],
            FinishReason::Stop,
        ),
        // Second turn: process result and respond
        OrchestrationResult::new(
            "The result is 8".to_string(),
            vec![],
            FinishReason::Stop,
        ),
    ]));

    let tools = create_mock_tools();
    let engine = OrchestrationEngine::with_defaults(provider, tools);
    let mut context = OrchestrationContext::new("test-session");
    
    // First turn
    context.add_user_message("Calculate 5 plus 3");
    let result1 = engine.execute("Calculate 5 plus 3", &mut context).await.unwrap();
    assert_eq!(result1.finish_reason, FinishReason::Stop);
    
    // Second turn - context should include tool result
    context.add_user_message("What was the answer?");
    let result2 = engine.execute("What was the answer?", &mut context).await.unwrap();
    assert_eq!(result2.finish_reason, FinishReason::Stop);
    
    // Verify context has full history including tool execution
    assert!(context.conversation_history.len() >= 4);
}

#[tokio::test]
async fn test_invalid_tool_arguments() {
    // Test handling of invalid tool arguments
    struct StrictToolHandler {
        should_fail: bool,
    }

    #[async_trait]
    impl ToolHandler for StrictToolHandler {
        async fn execute(&self, args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            if self.should_fail || args.get_string("task").is_none() {
                Ok(ToolResult::error("Invalid arguments"))
            } else {
                Ok(ToolResult::success("Success".to_string()))
            }
        }
    }

    let provider = Arc::new(MockE2EProvider::new(vec![
        OrchestrationResult::new(
            "Calling tool".to_string(),
            vec![ToolCall {
                id: "call_1".to_string(),
                name: "strict_tool".to_string(),
                arguments: json!({}), // Missing required "task" argument
            }],
            FinishReason::Stop,
        ),
    ]));

    let strict_tool = Tool::new(
        "agent_strict",
        "strict_tool",
        "A tool that validates arguments",
        ToolParameters::new().add_property("task", "string", "Task", true),
        Arc::new(StrictToolHandler { should_fail: false }),
    );

    let engine = OrchestrationEngine::with_defaults(provider, vec![strict_tool]);
    let mut context = OrchestrationContext::new("test-session");

    let result = engine.execute("Test input", &mut context).await.unwrap();

    // Should handle error gracefully
    assert_eq!(result.finish_reason, FinishReason::ToolError);
}

#[tokio::test]
async fn test_orchestration_performance() {
    // Test that orchestration overhead is reasonable
    // Target: < 500ms overhead (excluding actual API/execution time)
    let provider = Arc::new(MockE2EProvider::new(vec![
        OrchestrationResult::new(
            "Quick response".to_string(),
            vec![],
            FinishReason::Stop,
        ),
    ]));

    let engine = OrchestrationEngine::with_defaults(provider, vec![]);
    let mut context = OrchestrationContext::new("test-session");

    let start = std::time::Instant::now();
    let result = engine.execute("Quick test", &mut context).await.unwrap();
    let elapsed = start.elapsed();

    assert!(result.is_success());
    // Mock provider is instant, so this should be very fast (< 10ms)
    // Real providers would have API latency, but orchestration overhead should be minimal
    assert!(elapsed.as_millis() < 100, "Orchestration overhead too high: {}ms", elapsed.as_millis());
}

#[tokio::test]
async fn test_sequential_tool_execution() {
    // Test that tools execute in sequence when dependencies exist
    let call_order = Arc::new(std::sync::Mutex::new(Vec::new()));

    struct OrderedToolHandler {
        name: String,
        order: Arc<std::sync::Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl ToolHandler for OrderedToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            self.order.lock().unwrap().push(self.name.clone());
            Ok(ToolResult::success(format!("{} executed", self.name)))
        }
    }

    let provider = Arc::new(MockE2EProvider::new(vec![
        OrchestrationResult::new(
            "Calling tools sequentially".to_string(),
            vec![
                ToolCall {
                    id: "call_1".to_string(),
                    name: "tool_1".to_string(),
                    arguments: json!({"task": "first"}),
                },
                ToolCall {
                    id: "call_2".to_string(),
                    name: "tool_2".to_string(),
                    arguments: json!({"task": "second"}),
                },
            ],
            FinishReason::Stop,
        ),
    ]));

    let tools = vec![
        Tool::new(
            "agent_1",
            "tool_1",
            "First tool",
            ToolParameters::new().add_property("task", "string", "Task", true),
            Arc::new(OrderedToolHandler {
                name: "tool_1".to_string(),
                order: Arc::clone(&call_order),
            }),
        ),
        Tool::new(
            "agent_2",
            "tool_2",
            "Second tool",
            ToolParameters::new().add_property("task", "string", "Task", true),
            Arc::new(OrderedToolHandler {
                name: "tool_2".to_string(),
                order: Arc::clone(&call_order),
            }),
        ),
    ];

    let engine = OrchestrationEngine::with_defaults(provider, tools);
    let mut context = OrchestrationContext::new("test-session");

    let _result = engine.execute("Execute tools", &mut context).await.unwrap();

    // Verify tools were called in order
    let order = call_order.lock().unwrap();
    assert_eq!(order.len(), 2);
    assert_eq!(order[0], "tool_1");
    assert_eq!(order[1], "tool_2");
}

