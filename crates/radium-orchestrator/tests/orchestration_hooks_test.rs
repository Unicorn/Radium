//! Integration tests for hook execution in orchestration

use radium_orchestrator::orchestration::hooks::{NoOpToolHookExecutor, ToolHookExecutor};
use serde_json::json;

#[tokio::test]
async fn test_no_op_hook_executor() {
    let executor = NoOpToolHookExecutor;
    let args = json!({"key": "value"});
    let result = json!({"output": "test"});

    let before_result = executor.before_tool_execution("test_tool", &args).await.unwrap();
    assert_eq!(before_result, args);

    let after_result = executor.after_tool_execution("test_tool", &args, &result).await.unwrap();
    assert_eq!(after_result, result);
}

