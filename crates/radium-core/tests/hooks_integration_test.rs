//! Integration tests for the hooks system.

use radium_core::hooks::config::HookConfig;
use radium_core::hooks::integration::OrchestratorHooks;
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::registry::HookRegistry;
use radium_core::hooks::tool::{ToolHook, ToolHookContext};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use std::sync::Arc;

struct TestModelHook {
    name: String,
    priority: HookPriority,
    before_called: Arc<tokio::sync::Mutex<bool>>,
    after_called: Arc<tokio::sync::Mutex<bool>>,
}

#[async_trait::async_trait]
impl ModelHook for TestModelHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_model_call(
        &self,
        _context: &ModelHookContext,
    ) -> radium_core::hooks::error::Result<HookExecutionResult> {
        *self.before_called.lock().await = true;
        Ok(HookExecutionResult::success())
    }

    async fn after_model_call(
        &self,
        _context: &ModelHookContext,
    ) -> radium_core::hooks::error::Result<HookExecutionResult> {
        *self.after_called.lock().await = true;
        Ok(HookExecutionResult::success())
    }
}

#[tokio::test]
async fn test_model_hooks_integration() {
    let registry = Arc::new(HookRegistry::new());
    let before_called = Arc::new(tokio::sync::Mutex::new(false));
    let after_called = Arc::new(tokio::sync::Mutex::new(false));

    let hook = Arc::new(TestModelHook {
        name: "test-model-hook".to_string(),
        priority: HookPriority::default(),
        before_called: Arc::clone(&before_called),
        after_called: Arc::clone(&after_called),
    });

    // Register before and after hooks
    let hook_dyn: Arc<dyn ModelHook> = hook;
    let before_adapter = radium_core::hooks::model::ModelHookAdapter::before(Arc::clone(&hook_dyn));
    let after_adapter = radium_core::hooks::model::ModelHookAdapter::after(Arc::clone(&hook_dyn));

    registry.register(before_adapter).await.unwrap();
    registry.register(after_adapter).await.unwrap();

    let hooks = OrchestratorHooks::new(Arc::clone(&registry));

    // Test before model call
    let (modified_input, _) = hooks.before_model_call("test input", "test-model").await.unwrap();
    assert_eq!(modified_input, "test input");
    assert!(*before_called.lock().await);

    // Test after model call
    let modified_response =
        hooks.after_model_call("test input", "test-model", "test response").await.unwrap();
    assert_eq!(modified_response, "test response");
    assert!(*after_called.lock().await);
}

struct TestToolHook {
    name: String,
    priority: HookPriority,
    before_called: Arc<tokio::sync::Mutex<bool>>,
    after_called: Arc<tokio::sync::Mutex<bool>>,
}

#[async_trait::async_trait]
impl ToolHook for TestToolHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_tool_execution(
        &self,
        _context: &ToolHookContext,
    ) -> radium_core::hooks::error::Result<HookExecutionResult> {
        *self.before_called.lock().await = true;
        Ok(HookExecutionResult::success())
    }

    async fn after_tool_execution(
        &self,
        _context: &ToolHookContext,
    ) -> radium_core::hooks::error::Result<HookExecutionResult> {
        *self.after_called.lock().await = true;
        Ok(HookExecutionResult::success())
    }

    async fn tool_selection(
        &self,
        _context: &ToolHookContext,
    ) -> radium_core::hooks::error::Result<HookExecutionResult> {
        Ok(HookExecutionResult::success())
    }
}

#[tokio::test]
async fn test_tool_hooks_integration() {
    let registry = Arc::new(HookRegistry::new());
    let before_called = Arc::new(tokio::sync::Mutex::new(false));
    let after_called = Arc::new(tokio::sync::Mutex::new(false));

    let hook = Arc::new(TestToolHook {
        name: "test-tool-hook".to_string(),
        priority: HookPriority::default(),
        before_called: Arc::clone(&before_called),
        after_called: Arc::clone(&after_called),
    });

    let hook_dyn: Arc<dyn ToolHook> = hook;
    let before_adapter = radium_core::hooks::tool::ToolHookAdapter::before(Arc::clone(&hook_dyn));
    let after_adapter = radium_core::hooks::tool::ToolHookAdapter::after(Arc::clone(&hook_dyn));

    registry.register(before_adapter).await.unwrap();
    registry.register(after_adapter).await.unwrap();

    let hooks = OrchestratorHooks::new(Arc::clone(&registry));

    // Test before tool execution
    let modified_args = hooks
        .before_tool_execution("test-tool", &serde_json::json!({"arg": "value"}))
        .await
        .unwrap();
    assert_eq!(modified_args, serde_json::json!({"arg": "value"}));
    assert!(*before_called.lock().await);

    // Test after tool execution
    let modified_result = hooks
        .after_tool_execution(
            "test-tool",
            &serde_json::json!({"arg": "value"}),
            &serde_json::json!({"result": "success"}),
        )
        .await
        .unwrap();
    assert_eq!(modified_result, serde_json::json!({"result": "success"}));
    assert!(*after_called.lock().await);
}

#[tokio::test]
async fn test_hook_config_parsing() {
    let config_str = r#"
[[hooks]]
name = "test-hook"
type = "before_model"
priority = 100
script = "hooks/test.rs"
"#;

    let config = HookConfig::from_str(config_str).unwrap();
    assert_eq!(config.hooks.len(), 1);
    assert_eq!(config.hooks[0].name, "test-hook");
    assert_eq!(config.hooks[0].hook_type, "before_model");
    assert_eq!(config.hooks[0].priority, Some(100));
    assert_eq!(config.hooks[0].script, Some("hooks/test.rs".to_string()));

    // Test validation
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_hook_config_validation() {
    // Test invalid hook type
    let invalid_config = r#"
[[hooks]]
name = "test-hook"
type = "invalid_type"
script = "hooks/test.rs"
"#;

    let config = HookConfig::from_str(invalid_config).unwrap();
    assert!(config.validate().is_err());

    // Test missing script and config
    let missing_config = r#"
[[hooks]]
name = "test-hook"
type = "before_model"
"#;

    let config = HookConfig::from_str(missing_config).unwrap();
    assert!(config.validate().is_err());
}
