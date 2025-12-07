//! Unit tests for core orchestration types and abstractions

use radium_orchestrator::{
    FinishReason, OrchestrationResult, OrchestrationContext, Message,
    ToolCall, ToolParameters, ToolArguments, ToolResult,
};
use serde_json::json;

#[test]
fn test_orchestration_result_creation() {
    let result = OrchestrationResult::new(
        "Test response".to_string(),
        vec![],
        FinishReason::Stop,
    );
    
    assert_eq!(result.response, "Test response");
    assert!(result.tool_calls.is_empty());
    assert_eq!(result.finish_reason, FinishReason::Stop);
    assert!(result.is_success());
    assert!(!result.has_tool_calls());
}

#[test]
fn test_orchestration_result_with_tool_calls() {
    let tool_call = ToolCall {
        id: "call_1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({"task": "test"}),
    };
    
    let result = OrchestrationResult::new(
        "Calling tool".to_string(),
        vec![tool_call.clone()],
        FinishReason::Stop,
    );
    
    assert!(result.has_tool_calls());
    assert_eq!(result.tool_calls.len(), 1);
    assert_eq!(result.tool_calls[0].name, "test_tool");
}

#[test]
fn test_finish_reason_variants() {
    let stop = FinishReason::Stop;
    let max_iter = FinishReason::MaxIterations;
    let tool_error = FinishReason::ToolError;
    let cancelled = FinishReason::Cancelled;
    let error = FinishReason::Error;
    
    assert_eq!(stop.to_string(), "stop");
    assert_eq!(max_iter.to_string(), "max_iterations");
    assert_eq!(tool_error.to_string(), "tool_error");
    assert_eq!(cancelled.to_string(), "cancelled");
    assert_eq!(error.to_string(), "error");
}

#[test]
fn test_orchestration_result_finish_reasons() {
    let success = OrchestrationResult::new("Done".to_string(), vec![], FinishReason::Stop);
    assert!(success.is_success());
    
    let max_iter = OrchestrationResult::new("Max".to_string(), vec![], FinishReason::MaxIterations);
    assert!(!max_iter.is_success());
    
    let error = OrchestrationResult::new("Error".to_string(), vec![], FinishReason::Error);
    assert!(!error.is_success());
}

#[test]
fn test_orchestration_context_creation() {
    let context = OrchestrationContext::new("test-session");
    assert_eq!(context.session_id, "test-session");
    assert!(context.conversation_history.is_empty());
}

#[test]
fn test_orchestration_context_add_messages() {
    let mut context = OrchestrationContext::new("test-session");
    
    context.add_user_message("Hello");
    context.add_assistant_message("Hi there");
    
    assert_eq!(context.conversation_history.len(), 2);
    assert_eq!(context.conversation_history[0].role, "user");
    assert_eq!(context.conversation_history[0].content, "Hello");
    assert_eq!(context.conversation_history[1].role, "assistant");
    assert_eq!(context.conversation_history[1].content, "Hi there");
}

#[test]
fn test_orchestration_context_clone() {
    let mut context = OrchestrationContext::new("test-session");
    context.add_user_message("Test");
    
    let cloned = context.clone();
    assert_eq!(cloned.session_id, context.session_id);
    assert_eq!(cloned.conversation_history.len(), context.conversation_history.len());
}

#[test]
fn test_tool_call_creation() {
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "agent_name".to_string(),
        arguments: json!({"task": "refactor code"}),
    };
    
    assert_eq!(tool_call.id, "call_123");
    assert_eq!(tool_call.name, "agent_name");
    assert_eq!(tool_call.arguments["task"], "refactor code");
}

#[test]
fn test_tool_call_serialization() {
    let tool_call = ToolCall {
        id: "call_1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({"param": "value"}),
    };
    
    let serialized = serde_json::to_string(&tool_call).unwrap();
    assert!(serialized.contains("call_1"));
    assert!(serialized.contains("test_tool"));
    
    let deserialized: ToolCall = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, tool_call.id);
    assert_eq!(deserialized.name, tool_call.name);
}

#[test]
fn test_tool_arguments_get_string() {
    let args = ToolArguments::new(json!({
        "task": "test task",
        "priority": 5,
        "enabled": true
    }));
    
    assert_eq!(args.get_string("task"), Some("test task".to_string()));
    assert_eq!(args.get_string("missing"), None);
}

#[test]
fn test_tool_arguments_get_i64() {
    let args = ToolArguments::new(json!({"count": 42}));
    assert_eq!(args.get_i64("count"), Some(42));
    assert_eq!(args.get_i64("missing"), None);
}

#[test]
fn test_tool_arguments_get_bool() {
    let args = ToolArguments::new(json!({"enabled": true}));
    assert_eq!(args.get_bool("enabled"), Some(true));
    assert_eq!(args.get_bool("missing"), None);
}

#[test]
fn test_tool_result_success() {
    let result = ToolResult::success("Task completed");
    assert!(result.success);
    assert_eq!(result.output, "Task completed");
    assert!(result.metadata.is_empty());
}

#[test]
fn test_tool_result_error() {
    let result = ToolResult::error("Task failed");
    assert!(!result.success);
    assert_eq!(result.output, "Task failed");
}

#[test]
fn test_tool_result_with_metadata() {
    let result = ToolResult::success("Done")
        .with_metadata("duration", "1.5s")
        .with_metadata("agent", "test-agent");
    
    assert_eq!(result.metadata.get("duration"), Some(&"1.5s".to_string()));
    assert_eq!(result.metadata.get("agent"), Some(&"test-agent".to_string()));
}

#[test]
fn test_tool_parameters_builder() {
    let params = ToolParameters::new()
        .add_property("task", "string", "The task to perform", true)
        .add_property("priority", "number", "Task priority", false)
        .add_property("enabled", "boolean", "Whether enabled", false);
    
    assert_eq!(params.properties.len(), 3);
    assert_eq!(params.required.len(), 1);
    assert!(params.required.contains(&"task".to_string()));
}

#[test]
fn test_tool_parameters_serialization() {
    let params = ToolParameters::new()
        .add_property("task", "string", "Task description", true);
    
    let serialized = serde_json::to_string(&params).unwrap();
    assert!(serialized.contains("task"));
    assert!(serialized.contains("string"));
    
    let deserialized: ToolParameters = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.properties.len(), 1);
}

#[test]
fn test_message_creation() {
    let message = Message {
        role: "user".to_string(),
        content: "Hello".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello");
}

#[test]
fn test_message_serialization() {
    let message = Message {
        role: "assistant".to_string(),
        content: "Response".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    let serialized = serde_json::to_string(&message).unwrap();
    assert!(serialized.contains("assistant"));
    assert!(serialized.contains("Response"));
}

#[test]
fn test_empty_tool_calls() {
    let result = OrchestrationResult::new("Response".to_string(), vec![], FinishReason::Stop);
    assert!(!result.has_tool_calls());
}

#[test]
fn test_multiple_tool_calls() {
    let tool_calls = vec![
        ToolCall {
            id: "call_1".to_string(),
            name: "tool_1".to_string(),
            arguments: json!({}),
        },
        ToolCall {
            id: "call_2".to_string(),
            name: "tool_2".to_string(),
            arguments: json!({}),
        },
    ];
    
    let result = OrchestrationResult::new("Calling tools".to_string(), tool_calls, FinishReason::Stop);
    assert!(result.has_tool_calls());
    assert_eq!(result.tool_calls.len(), 2);
}

#[test]
fn test_tool_call_with_complex_arguments() {
    let tool_call = ToolCall {
        id: "call_1".to_string(),
        name: "complex_tool".to_string(),
        arguments: json!({
            "task": "refactor",
            "options": {
                "dry_run": true,
                "verbose": false
            },
            "files": ["file1.rs", "file2.rs"]
        }),
    };
    
    assert_eq!(tool_call.arguments["task"], "refactor");
    assert_eq!(tool_call.arguments["options"]["dry_run"], true);
    assert_eq!(tool_call.arguments["files"][0], "file1.rs");
}

