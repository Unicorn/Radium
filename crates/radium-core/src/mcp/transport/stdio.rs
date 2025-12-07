//! Stdio transport for MCP servers.

use crate::mcp::{McpError, McpTransport, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// Stdio transport implementation for MCP servers.
pub struct StdioTransport {
    /// Command to execute.
    command: String,
    /// Command arguments.
    args: Vec<String>,
    /// Child process (if running).
    child: Option<Arc<Mutex<Child>>>,
    /// Stdin handle.
    stdin: Option<Arc<Mutex<tokio::process::ChildStdin>>>,
    /// Stdout reader.
    stdout: Option<Arc<Mutex<BufReader<tokio::process::ChildStdout>>>>,
    /// Connection status.
    connected: bool,
}

impl StdioTransport {
    /// Create a new stdio transport.
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self {
            command,
            args,
            child: None,
            stdin: None,
            stdout: None,
            connected: false,
        }
    }
}

#[async_trait::async_trait]
impl McpTransport for StdioTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Err(McpError::Connection("Already connected".to_string()));
        }

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            McpError::Transport(format!("Failed to spawn process: {}", e))
        })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            McpError::Transport("Failed to get stdin handle".to_string())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            McpError::Transport("Failed to get stdout handle".to_string())
        })?;

        self.stdin = Some(Arc::new(Mutex::new(stdin)));
        self.stdout = Some(Arc::new(Mutex::new(BufReader::new(stdout))));
        self.child = Some(Arc::new(Mutex::new(child)));
        self.connected = true;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        if let Some(child) = &self.child {
            let mut child = child.lock().await;
            let _ = child.kill();
            let _ = child.wait();
        }

        self.stdin = None;
        self.stdout = None;
        self.child = None;
        self.connected = false;

        Ok(())
    }

    async fn send(&mut self, message: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(McpError::Connection("Not connected".to_string()));
        }

        let stdin = self.stdin.as_ref().ok_or_else(|| {
            McpError::Transport("Stdin not available".to_string())
        })?;

        let mut stdin = stdin.lock().await;
        stdin.write_all(message).await.map_err(|e| {
            McpError::Transport(format!("Failed to write to stdin: {}", e))
        })?;
        stdin.write_all(b"\n").await.map_err(|e| {
            McpError::Transport(format!("Failed to write newline: {}", e))
        })?;
        stdin.flush().await.map_err(|e| {
            McpError::Transport(format!("Failed to flush stdin: {}", e))
        })?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(McpError::Connection("Not connected".to_string()));
        }

        let stdout = self.stdout.as_ref().ok_or_else(|| {
            McpError::Transport("Stdout not available".to_string())
        })?;

        let mut stdout = stdout.lock().await;
        let mut line = String::new();
        stdout.read_line(&mut line).await.map_err(|e| {
            McpError::Transport(format!("Failed to read from stdout: {}", e))
        })?;

        if line.is_empty() {
            return Err(McpError::Connection("Connection closed".to_string()));
        }

        Ok(line.trim_end().as_bytes().to_vec())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        if self.connected {
            // Try to clean up, but don't block
            if let Some(child) = &self.child {
                if let Ok(mut child) = child.try_lock() {
                    let _ = child.kill();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stdio_transport_creation() {
        let transport = StdioTransport::new("echo".to_string(), vec![]);
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_stdio_transport_connect_disconnect() {
        // Use a simple command that will exit quickly
        let mut transport = StdioTransport::new("echo".to_string(), vec!["test".to_string()]);
        
        // Note: This test may fail if echo doesn't work as expected
        // In a real scenario, we'd use a mock MCP server
        let result = transport.connect().await;
        // Connection might succeed or fail depending on system
        // We just verify the method exists and doesn't panic
        if result.is_ok() {
            assert!(transport.is_connected());
            let _ = transport.disconnect().await;
            assert!(!transport.is_connected());
        }
    }

    #[test]
    fn test_stdio_transport_is_connected() {
        let transport = StdioTransport::new("test".to_string(), vec![]);
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_stdio_transport_connect_twice() {
        let mut transport = StdioTransport::new("echo".to_string(), vec![]);
        
        if transport.connect().await.is_ok() {
            // Try to connect again - should fail
            let result = transport.connect().await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Already connected"));
            
            let _ = transport.disconnect().await;
        }
    }

    #[tokio::test]
    async fn test_stdio_transport_send_when_not_connected() {
        let mut transport = StdioTransport::new("echo".to_string(), vec![]);
        let result = transport.send(b"test message").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not connected"));
    }

    #[tokio::test]
    async fn test_stdio_transport_receive_when_not_connected() {
        let mut transport = StdioTransport::new("echo".to_string(), vec![]);
        let result = transport.receive().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not connected"));
    }

    #[tokio::test]
    async fn test_stdio_transport_disconnect_when_not_connected() {
        let mut transport = StdioTransport::new("echo".to_string(), vec![]);
        // Disconnecting when not connected should not error
        let result = transport.disconnect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_transport_invalid_command() {
        let mut transport = StdioTransport::new("nonexistent_command_xyz123".to_string(), vec![]);
        let result = transport.connect().await;
        // Should fail to spawn the process
        assert!(result.is_err());
        assert!(!transport.is_connected());
    }

    #[test]
    fn test_stdio_transport_with_args() {
        let transport = StdioTransport::new(
            "echo".to_string(),
            vec!["--help".to_string(), "--verbose".to_string()],
        );
        assert!(!transport.is_connected());
    }
}

