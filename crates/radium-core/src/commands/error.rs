//! Custom commands error types.

use std::io;

/// Custom commands errors.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// I/O error during command operations.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// TOML parsing error.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// Command not found.
    #[error("command not found: {0}")]
    NotFound(String),

    /// Invalid command definition.
    #[error("invalid command definition: {0}")]
    InvalidDefinition(String),

    /// Shell execution error.
    #[error("shell execution error: {0}")]
    ShellExecution(String),

    /// Template rendering error.
    #[error("template rendering error: {0}")]
    TemplateRender(String),

    /// File injection error.
    #[error("file injection error: {0}")]
    FileInjection(String),

    /// Tool execution denied by hook.
    #[error("tool execution denied: {0}")]
    ToolDenied(String),
}

/// Result type for command operations.
pub type Result<T> = std::result::Result<T, CommandError>;
