//! CLI session manager for persisting session IDs.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// CLI session metadata persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CliSession {
    /// Session ID from daemon
    pub session_id: String,
    /// Daemon URL this session is associated with
    pub daemon_url: String,
    /// When this session was created
    pub created_at: DateTime<Utc>,
}

/// Manages CLI session persistence.
#[allow(dead_code)]
pub struct CliSessionManager {
    /// Path to session file
    session_file: PathBuf,
}

#[allow(dead_code)]
impl CliSessionManager {
    /// Create a new CLI session manager.
    ///
    /// # Arguments
    /// * `workspace_root` - Workspace root directory
    ///
    /// # Returns
    /// New CliSessionManager instance.
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let internals_dir = workspace_root
            .join(".radium")
            .join("_internals");

        fs::create_dir_all(&internals_dir)
            .context("Failed to create _internals directory")?;

        let session_file = internals_dir.join("cli-session");

        Ok(Self { session_file })
    }

    /// Load the current session ID if it exists.
    ///
    /// # Returns
    /// Session ID if found, None otherwise.
    #[allow(dead_code)]
    pub fn load_session(&self) -> Result<Option<CliSession>> {
        if !self.session_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.session_file)
            .context("Failed to read session file")?;

        let session: CliSession = serde_json::from_str(&content)
            .context("Failed to parse session file")?;

        Ok(Some(session))
    }

    /// Save a session ID to disk.
    ///
    /// # Arguments
    /// * `session_id` - Session ID to save
    /// * `daemon_url` - Daemon URL this session is associated with
    #[allow(dead_code)]
    pub fn save_session(&self, session_id: String, daemon_url: String) -> Result<()> {
        let session = CliSession {
            session_id,
            daemon_url,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string_pretty(&session)
            .context("Failed to serialize session")?;

        fs::write(&self.session_file, json)
            .context("Failed to write session file")?;

        Ok(())
    }

    /// Clear the saved session.
    #[allow(dead_code)]
    pub fn clear_session(&self) -> Result<()> {
        if self.session_file.exists() {
            fs::remove_file(&self.session_file)
                .context("Failed to remove session file")?;
        }
        Ok(())
    }
}
