//! Cancellation Handling
//!
//! Implement graceful cancellation support:
//! - Cancellation scopes for grouping operations
//! - Cleanup logic on cancellation
//! - Shielded scopes for critical operations
//! - Activity and child workflow cancellation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Cancellation scope for grouping cancellable operations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CancellationScope {
    /// Scope name for identification
    #[validate(length(min = 1))]
    pub name: String,

    /// Whether this scope is shielded (not cancelled when parent is)
    #[serde(default)]
    pub shielded: bool,

    /// Cleanup configuration to run on cancellation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup: Option<CleanupConfig>,

    /// Timeout for cleanup operations (ms)
    #[serde(default = "default_cleanup_timeout")]
    pub cleanup_timeout_ms: u64,
}

fn default_cleanup_timeout() -> u64 {
    30000
}

impl CancellationScope {
    /// Create a new cancellable scope
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            shielded: false,
            cleanup: None,
            cleanup_timeout_ms: default_cleanup_timeout(),
        }
    }

    /// Create a shielded (non-cancellable) scope
    pub fn shielded(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            shielded: true,
            cleanup: None,
            cleanup_timeout_ms: default_cleanup_timeout(),
        }
    }

    /// Add cleanup configuration
    pub fn with_cleanup(mut self, cleanup: CleanupConfig) -> Self {
        self.cleanup = Some(cleanup);
        self
    }

    /// Set cleanup timeout
    pub fn with_cleanup_timeout(mut self, ms: u64) -> Self {
        self.cleanup_timeout_ms = ms;
        self
    }

    /// Generate TypeScript code for this scope
    pub fn to_typescript(&self, body: &str) -> String {
        let scope_type = if self.shielded {
            "CancellationScope.nonCancellable"
        } else {
            "CancellationScope.cancellable"
        };

        let cleanup_code = self
            .cleanup
            .as_ref()
            .map(|c| c.to_typescript())
            .unwrap_or_default();

        if cleanup_code.is_empty() {
            format!(
                r#"// Cancellation Scope: {}
await {}(async () => {{
  {}
}});
"#,
                self.name, scope_type, body
            )
        } else {
            format!(
                r#"// Cancellation Scope: {}
try {{
  await {}(async () => {{
    {}
  }});
}} catch (err) {{
  if (isCancellation(err)) {{
    // Run cleanup with timeout
    await CancellationScope.nonCancellable(async () => {{
      await Promise.race([
        (async () => {{
          {}
        }})(),
        sleep({}),
      ]);
    }});
    throw err;
  }}
  throw err;
}}
"#,
                self.name, scope_type, body, cleanup_code, self.cleanup_timeout_ms
            )
        }
    }
}

/// Configuration for cleanup on cancellation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanupConfig {
    /// Activities to run for cleanup
    #[serde(default)]
    pub activities: Vec<CleanupActivity>,

    /// State updates on cleanup
    #[serde(default)]
    pub state_updates: Vec<StateUpdate>,

    /// Custom TypeScript cleanup code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_code: Option<String>,
}

impl CleanupConfig {
    /// Create empty cleanup config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a cleanup activity
    pub fn with_activity(mut self, activity: CleanupActivity) -> Self {
        self.activities.push(activity);
        self
    }

    /// Add a state update
    pub fn with_state_update(mut self, update: StateUpdate) -> Self {
        self.state_updates.push(update);
        self
    }

    /// Set custom cleanup code
    pub fn with_custom_code(mut self, code: impl Into<String>) -> Self {
        self.custom_code = Some(code.into());
        self
    }

    /// Generate TypeScript cleanup code
    pub fn to_typescript(&self) -> String {
        let mut code = String::new();

        // Run cleanup activities
        for activity in &self.activities {
            code.push_str(&format!(
                "try {{\n  await activities.{}({});\n}} catch (e) {{\n  console.warn('Cleanup activity {} failed:', e);\n}}\n",
                activity.activity_name,
                serde_json::to_string(&activity.input).unwrap_or_else(|_| "{}".to_string()),
                activity.activity_name
            ));
        }

        // Apply state updates
        for update in &self.state_updates {
            code.push_str(&format!(
                "state.variables.{} = {};\n",
                update.variable,
                serde_json::to_string(&update.value).unwrap_or_else(|_| "null".to_string())
            ));
        }

        // Add custom code
        if let Some(custom) = &self.custom_code {
            code.push_str(custom);
            code.push('\n');
        }

        code
    }
}

/// A cleanup activity to run on cancellation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupActivity {
    /// Activity name to call
    pub activity_name: String,

    /// Input for the activity
    #[serde(default)]
    pub input: serde_json::Value,

    /// Maximum attempts for cleanup
    #[serde(default = "default_one")]
    pub max_attempts: u32,

    /// Whether to continue cleanup if this fails
    #[serde(default = "default_true")]
    pub continue_on_failure: bool,
}

fn default_one() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

impl CleanupActivity {
    /// Create a new cleanup activity
    pub fn new(activity_name: impl Into<String>) -> Self {
        Self {
            activity_name: activity_name.into(),
            input: serde_json::Value::Object(Default::default()),
            max_attempts: 1,
            continue_on_failure: true,
        }
    }

    /// Set input for the activity
    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = input;
        self
    }

    /// Set max attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set to fail cleanup if this activity fails
    pub fn fail_on_error(mut self) -> Self {
        self.continue_on_failure = false;
        self
    }
}

/// A state update on cancellation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateUpdate {
    /// Variable name to update
    pub variable: String,

    /// Value to set
    pub value: serde_json::Value,
}

impl StateUpdate {
    /// Create a new state update
    pub fn new(variable: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            variable: variable.into(),
            value,
        }
    }
}

/// Cancellation handler for the entire workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCancellationHandler {
    /// Whether cancellation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Global cleanup to run on workflow cancellation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup: Option<CleanupConfig>,

    /// Timeout for workflow-level cleanup (ms)
    #[serde(default = "default_cleanup_timeout")]
    pub cleanup_timeout_ms: u64,

    /// Named cancellation scopes
    #[serde(default)]
    pub scopes: HashMap<String, CancellationScope>,
}

impl Default for WorkflowCancellationHandler {
    fn default() -> Self {
        Self {
            enabled: true,
            cleanup: None,
            cleanup_timeout_ms: default_cleanup_timeout(),
            scopes: HashMap::new(),
        }
    }
}

impl WorkflowCancellationHandler {
    /// Create a new cancellation handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable cancellation handling
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }

    /// Set global cleanup
    pub fn with_cleanup(mut self, cleanup: CleanupConfig) -> Self {
        self.cleanup = Some(cleanup);
        self
    }

    /// Add a cancellation scope
    pub fn with_scope(mut self, scope: CancellationScope) -> Self {
        self.scopes.insert(scope.name.clone(), scope);
        self
    }

    /// Get a scope by name
    pub fn get_scope(&self, name: &str) -> Option<&CancellationScope> {
        self.scopes.get(name)
    }

    /// Generate TypeScript imports for cancellation handling
    pub fn typescript_imports() -> &'static str {
        "import { CancellationScope, isCancellation, sleep } from '@temporalio/workflow';"
    }

    /// Generate TypeScript setup code
    pub fn to_typescript_setup(&self) -> String {
        if !self.enabled {
            return "// Cancellation handling disabled\n".to_string();
        }

        let cleanup_code = self
            .cleanup
            .as_ref()
            .map(|c| c.to_typescript())
            .unwrap_or_default();

        if cleanup_code.is_empty() {
            String::new()
        } else {
            format!(
                r#"// Global cancellation handler
const workflowCleanup = async () => {{
  {}
}};
"#,
                cleanup_code
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_scope_basic() {
        let scope = CancellationScope::new("orderProcessing");

        assert_eq!(scope.name, "orderProcessing");
        assert!(!scope.shielded);
        assert!(scope.cleanup.is_none());
    }

    #[test]
    fn test_cancellation_scope_shielded() {
        let scope = CancellationScope::shielded("criticalOperation");

        assert!(scope.shielded);
    }

    #[test]
    fn test_cancellation_scope_with_cleanup() {
        let cleanup = CleanupConfig::new()
            .with_activity(CleanupActivity::new("releaseResources"))
            .with_state_update(StateUpdate::new("status", serde_json::json!("cancelled")));

        let scope = CancellationScope::new("resourceScope")
            .with_cleanup(cleanup)
            .with_cleanup_timeout(60000);

        assert!(scope.cleanup.is_some());
        assert_eq!(scope.cleanup_timeout_ms, 60000);
    }

    #[test]
    fn test_cancellation_scope_to_typescript() {
        let scope = CancellationScope::new("processOrder");
        let ts = scope.to_typescript("await processOrder();");

        assert!(ts.contains("CancellationScope.cancellable"));
        assert!(ts.contains("await processOrder()"));
    }

    #[test]
    fn test_shielded_scope_to_typescript() {
        let scope = CancellationScope::shielded("saveState");
        let ts = scope.to_typescript("await saveState();");

        assert!(ts.contains("CancellationScope.nonCancellable"));
    }

    #[test]
    fn test_scope_with_cleanup_to_typescript() {
        let cleanup = CleanupConfig::new()
            .with_state_update(StateUpdate::new("cancelled", serde_json::json!(true)));

        let scope = CancellationScope::new("operation").with_cleanup(cleanup);
        let ts = scope.to_typescript("await doWork();");

        assert!(ts.contains("isCancellation(err)"));
        assert!(ts.contains("state.variables.cancelled"));
    }

    #[test]
    fn test_cleanup_config_to_typescript() {
        let cleanup = CleanupConfig::new()
            .with_activity(CleanupActivity::new("cleanup"))
            .with_state_update(StateUpdate::new("status", serde_json::json!("cleaned")))
            .with_custom_code("console.log('Cleanup complete');");

        let ts = cleanup.to_typescript();

        assert!(ts.contains("activities.cleanup"));
        assert!(ts.contains("state.variables.status"));
        assert!(ts.contains("console.log"));
    }

    #[test]
    fn test_cleanup_activity() {
        let activity = CleanupActivity::new("releaseResource")
            .with_input(serde_json::json!({"resourceId": "123"}))
            .with_max_attempts(3)
            .fail_on_error();

        assert_eq!(activity.activity_name, "releaseResource");
        assert_eq!(activity.max_attempts, 3);
        assert!(!activity.continue_on_failure);
    }

    #[test]
    fn test_workflow_cancellation_handler() {
        let handler = WorkflowCancellationHandler::new()
            .with_cleanup(CleanupConfig::new())
            .with_scope(CancellationScope::new("scope1"))
            .with_scope(CancellationScope::shielded("scope2"));

        assert!(handler.enabled);
        assert!(handler.cleanup.is_some());
        assert_eq!(handler.scopes.len(), 2);
        assert!(handler.get_scope("scope1").is_some());
    }

    #[test]
    fn test_disabled_cancellation_handler() {
        let handler = WorkflowCancellationHandler::disabled();
        let ts = handler.to_typescript_setup();

        assert!(ts.contains("disabled"));
    }

    #[test]
    fn test_typescript_imports() {
        let imports = WorkflowCancellationHandler::typescript_imports();
        assert!(imports.contains("CancellationScope"));
        assert!(imports.contains("isCancellation"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let scope = CancellationScope::new("test")
            .with_cleanup(CleanupConfig::new())
            .with_cleanup_timeout(10000);

        let json = serde_json::to_string(&scope).unwrap();
        let restored: CancellationScope = serde_json::from_str(&json).unwrap();

        assert_eq!(scope.name, restored.name);
        assert_eq!(scope.cleanup_timeout_ms, restored.cleanup_timeout_ms);
    }
}
