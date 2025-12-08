//! Error types for playbook operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during playbook operations.
#[derive(Error, Debug)]
pub enum PlaybookError {
    /// Failed to load playbook file.
    #[error("Failed to load playbook file at {path}: {source}")]
    LoadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse playbook YAML frontmatter.
    #[error("Failed to parse playbook frontmatter at {path}: {source}")]
    ParseError {
        path: Option<PathBuf>,
        #[source]
        source: serde_yaml::Error,
    },

    /// Invalid playbook configuration.
    #[error("Invalid playbook configuration: {0}")]
    InvalidConfig(String),

    /// Missing required field in playbook.
    #[error("Missing required field in playbook: {0}")]
    MissingField(String),

    /// Invalid URI format.
    #[error("Invalid URI format: {0}. URI must start with 'radium://'")]
    InvalidUri(String),

    /// Invalid frontmatter format.
    #[error("Invalid frontmatter format: {0}")]
    InvalidFrontmatter(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Playbook not found.
    #[error("Playbook not found: {0}")]
    NotFound(String),
}

/// Result type alias for playbook operations.
pub type Result<T> = std::result::Result<T, PlaybookError>;

