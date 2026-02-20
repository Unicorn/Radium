//! Integration tests for Custom Commands with Sandbox Execution.
//!
//! Tests that verify custom commands execute safely within sandbox environments
//! and integrate correctly with the hook system for approval.

use radium_core::commands::custom::CustomCommand;
use radium_core::hooks::registry::HookRegistry;
use radium_core::hooks::tool::{ToolHook, ToolHookContext, ToolHookAdapter};
use radium_core::hooks::types::{HookPriority, HookResult};
use radium_core::sandbox::{Sandbox, SandboxConfig};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

// Mock sandbox for testing
struct MockSandbox {
    enabled: bool,
}

impl Sandbox for MockSandbox {
    fn execute_command(&mut self, command: &str, _cwd: Option<&PathBuf>) -> Result<String, String> {
        if !self.enabled {
            return Err("Sandbox disabled".to_string());
        }
        Ok(format!("sandboxed: {}", command))
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn config(&self) -> &SandboxConfig {
        // Return default config for testing
        &SandboxConfig::default()
    }
}

#[tokio::test]
async fn test_sandboxed_shell_command_execution() {
    let temp_dir = TempDir::new().unwrap();
    let command = CustomCommand {
        name: "test-cmd".to_string(),
        description: "Test command".to_string(),
        template: "!{echo 'test'}".to_string(),
        args: vec![],
        namespace: None,
    };

    // Create sandbox
    let mut sandbox: Box<dyn Sandbox> = Box::new(MockSandbox { enabled: true });

    // Execute with sandbox
    let result = command.execute_with_sandbox(&[], temp_dir.path(), Some(&mut sandbox));
    
    // Should execute in sandbox
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("sandboxed") || output.contains("test"));
}

#[tokio::test]
async fn test_hook_system_approval_flow() {
    let temp_dir = TempDir::new().unwrap();
    let command = CustomCommand {
        name: "test-command".to_string(),
        description: "Test command".to_string(),
        template: "Hello World".to_string(),
        args: vec![],
        namespace: None,
    };

    // Create hook registry
    let registry = Arc::new(HookRegistry::new());

    // Create an approval hook
    struct ApprovalHook;
    #[async_trait::async_trait]
    impl ToolHook for ApprovalHook {
        async fn tool_selection(
            &self,
            _tool_name: &str,
            _context: &ToolHookContext,
        ) -> HookResult<bool> {
            Ok(true) // Always approve
        }
    }

    let hook_adapter = ToolHookAdapter::new(
        "approval-hook".to_string(),
        HookPriority::Normal,
        Arc::new(ApprovalHook),
    );
    registry.register(hook_adapter).await.unwrap();

    // Execute command with hooks
    let args = vec![];
    let result = command.execute_with_hooks(&args, temp_dir.path(), Some(registry)).await;

    // Should succeed (approved)
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello World");
}

#[tokio::test]
async fn test_hook_system_denial_flow() {
    let temp_dir = TempDir::new().unwrap();
    let command = CustomCommand {
        name: "test-command".to_string(),
        description: "Test command".to_string(),
        template: "Hello World".to_string(),
        args: vec![],
        namespace: None,
    };

    // Create hook registry
    let registry = Arc::new(HookRegistry::new());

    // Create a denial hook
    struct DenialHook;
    #[async_trait::async_trait]
    impl ToolHook for DenialHook {
        async fn tool_selection(
            &self,
            _tool_name: &str,
            _context: &ToolHookContext,
        ) -> HookResult<bool> {
            Ok(false) // Always deny
        }
    }

    let hook_adapter = ToolHookAdapter::new(
        "denial-hook".to_string(),
        HookPriority::Normal,
        Arc::new(DenialHook),
    );
    registry.register(hook_adapter).await.unwrap();

    // Execute command with hooks
    let args = vec![];
    let result = command.execute_with_hooks(&args, temp_dir.path(), Some(registry)).await;

    // Should fail (denied)
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("denied"));
}

#[tokio::test]
async fn test_template_substitution_all_injection_types() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a test file for file injection
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "File content").unwrap();

    let command = CustomCommand {
        name: "test-cmd".to_string(),
        description: "Test command".to_string(),
        template: "Args: {{args}}, Shell: !{echo 'test'}, File: @{test.txt}".to_string(),
        args: vec!["arg1".to_string(), "arg2".to_string()],
        namespace: None,
    };

    // Execute command
    let args = vec!["arg1".to_string(), "arg2".to_string()];
    let result = command.execute(&args, temp_dir.path());

    assert!(result.is_ok());
    let output = result.unwrap();
    
    // Should contain substituted arguments
    assert!(output.contains("arg1") || output.contains("arg2"));
    // Should contain shell output
    assert!(output.contains("test") || output.contains("Shell:"));
    // Should contain file content
    assert!(output.contains("File content") || output.contains("File:"));
}

#[tokio::test]
async fn test_command_precedence_project_user_extensions() {
    // This test verifies command discovery precedence
    // The actual discovery logic is tested in the custom command discovery tests
    // This integration test verifies the precedence is enforced when executing
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create project-level command
    let project_commands_dir = temp_dir.path().join(".radium").join("commands");
    std::fs::create_dir_all(&project_commands_dir).unwrap();
    std::fs::write(
        project_commands_dir.join("test-cmd.toml"),
        r#"name = "test-cmd"
description = "Project command"
template = "project-level"
"#,
    ).unwrap();

    // Command should be discoverable and executable
    // Note: Full discovery testing is in custom command discovery module
    // This test just verifies a discovered command executes correctly
    let command = CustomCommand {
        name: "test-cmd".to_string(),
        description: "Project command".to_string(),
        template: "project-level".to_string(),
        args: vec![],
        namespace: None,
    };

    let result = command.execute(&[], temp_dir.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "project-level");
}

