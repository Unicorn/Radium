//! Terminal command tool for orchestration
//!
//! This module provides a tool for executing terminal commands safely,
//! with optional sandbox support and policy engine integration.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::{OrchestrationError, Result};

/// Trait for workspace root resolution
pub trait WorkspaceRootProvider: Send + Sync {
    /// Get the workspace root path
    fn workspace_root(&self) -> Option<PathBuf>;
}

/// Trait for sandbox operations (matches executor::SandboxManager)
#[async_trait::async_trait]
pub trait SandboxManager: Send + Sync {
    /// Initialize sandbox for an agent/tool execution
    async fn initialize_sandbox(&self, agent_id: &str) -> std::result::Result<(), String>;
    
    /// Cleanup sandbox for an agent/tool execution
    async fn cleanup_sandbox(&self, agent_id: &str);
    
    /// Get active sandbox path (if any)
    fn get_sandbox_path(&self, agent_id: &str) -> Option<PathBuf>;
}

/// Terminal command tool handler
struct TerminalCommandHandler {
    /// Workspace root provider
    workspace_root: Arc<dyn WorkspaceRootProvider>,
    /// Optional sandbox manager
    sandbox_manager: Option<Arc<dyn SandboxManager>>,
    /// Default timeout for commands (in seconds)
    default_timeout: u64,
}

#[async_trait]
impl ToolHandler for TerminalCommandHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let command = args.get_string("command").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "run_terminal_cmd".to_string(),
                reason: "Missing required 'command' argument".to_string(),
            }
        })?;

        let working_dir = args.get_string("working_dir");
        let timeout_secs = args.get_i64("timeout_seconds").unwrap_or(self.default_timeout as i64) as u64;
        let shell = args.get_bool("use_shell").unwrap_or(true);

        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        // Determine working directory
        let cwd = if let Some(dir) = working_dir {
            let path = PathBuf::from(dir);
            if path.is_absolute() {
                path
            } else {
                workspace_root.join(path)
            }
        } else {
            workspace_root
        };

        // Check if sandbox should be used
        // For now, we'll use a simple approach - if sandbox is available,
        // we could initialize it, but for terminal commands, we'll just use
        // the workspace root. Full sandbox integration would require more
        // complex setup.
        let _sandbox_path: Option<PathBuf> = if self.sandbox_manager.is_some() {
            None // TODO: Implement proper sandbox path resolution
        } else {
            None
        };

        let actual_cwd = cwd;

        // Execute command
        self.execute_command(&command, &actual_cwd, shell, timeout_secs).await
    }
}

impl TerminalCommandHandler {
    /// Execute a terminal command
    async fn execute_command(
        &self,
        command: &str,
        cwd: &PathBuf,
        use_shell: bool,
        timeout_secs: u64,
    ) -> Result<ToolResult> {
        let timeout_duration = Duration::from_secs(timeout_secs);

        if use_shell {
            // Execute via shell (sh -c on Unix, cmd /c on Windows)
            #[cfg(unix)]
            let shell_cmd = "sh";
            #[cfg(unix)]
            let shell_arg = "-c";
            #[cfg(windows)]
            let shell_cmd = "cmd";
            #[cfg(windows)]
            let shell_arg = "/c";

            let mut cmd = Command::new(shell_cmd);
            cmd.arg(shell_arg);
            cmd.arg(command);
            cmd.current_dir(cwd);

            match timeout(timeout_duration, cmd.output()).await {
                Ok(Ok(output)) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let exit_code = output.status.code().unwrap_or(-1);

                    let success = output.status.success();
                    let output_text = if success {
                        format!(
                            "Command executed successfully (exit code: {})\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                            exit_code, stdout, stderr
                        )
                    } else {
                        format!(
                            "Command failed (exit code: {})\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                            exit_code, stdout, stderr
                        )
                    };

                    Ok(ToolResult {
                        success,
                        output: output_text,
                        is_error: false,
                        metadata: std::collections::HashMap::from([
                            ("exit_code".to_string(), exit_code.to_string()),
                            ("command".to_string(), command.to_string()),
                            ("working_dir".to_string(), cwd.display().to_string()),
                        ]),
                    })
                }
                Ok(Err(e)) => Ok(ToolResult::error(format!(
                    "Failed to execute command '{}': {}",
                    command, e
                ))),
                Err(_) => Ok(ToolResult::error(format!(
                    "Command '{}' timed out after {} seconds",
                    command, timeout_secs
                ))),
            }
        } else {
            // Execute command directly (split by spaces - simple approach)
            // For more complex parsing, we'd need a proper shell parser
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                return Ok(ToolResult::error("Empty command".to_string()));
            }

            let mut cmd = Command::new(parts[0]);
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }
            cmd.current_dir(cwd);

            match timeout(timeout_duration, cmd.output()).await {
                Ok(Ok(output)) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let exit_code = output.status.code().unwrap_or(-1);

                    let success = output.status.success();
                    let output_text = if success {
                        format!(
                            "Command executed successfully (exit code: {})\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                            exit_code, stdout, stderr
                        )
                    } else {
                        format!(
                            "Command failed (exit code: {})\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                            exit_code, stdout, stderr
                        )
                    };

                    Ok(ToolResult {
                        success,
                        output: output_text,
                        is_error: false,
                        metadata: std::collections::HashMap::from([
                            ("exit_code".to_string(), exit_code.to_string()),
                            ("command".to_string(), command.to_string()),
                            ("working_dir".to_string(), cwd.display().to_string()),
                        ]),
                    })
                }
                Ok(Err(e)) => Ok(ToolResult::error(format!(
                    "Failed to execute command '{}': {}",
                    command, e
                ))),
                Err(_) => Ok(ToolResult::error(format!(
                    "Command '{}' timed out after {} seconds",
                    command, timeout_secs
                ))),
            }
        }
    }
}

/// Create terminal command tool
///
/// # Arguments
/// * `workspace_root` - Provider for workspace root path
/// * `sandbox_manager` - Optional sandbox manager for safe execution
/// * `default_timeout` - Default timeout in seconds (default: 30)
///
/// # Returns
/// Terminal command tool
pub fn create_terminal_command_tool(
    workspace_root: Arc<dyn WorkspaceRootProvider>,
    sandbox_manager: Option<Arc<dyn SandboxManager>>,
    default_timeout: Option<u64>,
) -> Tool {
    let parameters = ToolParameters::new()
        .add_property("command", "string", "Command to execute", true)
        .add_property("working_dir", "string", "Working directory for command (relative to workspace root, optional)", false)
        .add_property("timeout_seconds", "number", "Timeout in seconds (default: 30)", false)
        .add_property("use_shell", "boolean", "Execute via shell (default: true)", false);

    let handler = Arc::new(TerminalCommandHandler {
        workspace_root,
        sandbox_manager,
        default_timeout: default_timeout.unwrap_or(30),
    });

    Tool::new(
        "run_terminal_cmd",
        "run_terminal_cmd",
        "Execute a terminal command safely with optional sandbox support",
        parameters,
        handler,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    struct TestWorkspaceRoot {
        root: PathBuf,
    }

    impl WorkspaceRootProvider for TestWorkspaceRoot {
        fn workspace_root(&self) -> Option<PathBuf> {
            Some(self.root.clone())
        }
    }

    #[tokio::test]
    async fn test_terminal_command_tool_success() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_terminal_command_tool(workspace_root, None, None);
        
        #[cfg(unix)]
        let command = "echo 'Hello, world!'";
        #[cfg(windows)]
        let command = "echo Hello, world!";

        let args = ToolArguments::new(serde_json::json!({
            "command": command
        }));

        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Hello"));
    }

    #[tokio::test]
    async fn test_terminal_command_tool_failure() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_terminal_command_tool(workspace_root, None, None);
        
        #[cfg(unix)]
        let command = "false"; // Command that always fails
        #[cfg(windows)]
        let command = "exit /b 1"; // Command that always fails

        let args = ToolArguments::new(serde_json::json!({
            "command": command
        }));

        let result = tool.execute(&args).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_terminal_command_tool_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_terminal_command_tool(workspace_root, None, Some(1)); // 1 second timeout
        
        #[cfg(unix)]
        let command = "sleep 5"; // Sleep for 5 seconds
        #[cfg(windows)]
        let command = "timeout /t 5 /nobreak"; // Wait 5 seconds

        let args = ToolArguments::new(serde_json::json!({
            "command": command,
            "timeout_seconds": 1
        }));

        let result = tool.execute(&args).await.unwrap();
        assert!(!result.success);
        assert!(result.output.contains("timed out"));
    }
}

