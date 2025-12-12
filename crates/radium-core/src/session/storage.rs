//! Session storage for persistence.

use crate::session::state::{Approval, Artifact, Message, Session, ToolCall};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};
use uuid::Uuid;

/// Session storage manager for file-based persistence.
pub struct SessionStorage {
    sessions_dir: PathBuf,
}

impl SessionStorage {
    /// Create a new session storage manager.
    ///
    /// Creates the sessions directory structure if it doesn't exist.
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let sessions_dir = workspace_root
            .join(".radium")
            .join("_internals")
            .join("sessions");

        fs::create_dir_all(&sessions_dir)
            .with_context(|| format!("Failed to create sessions directory: {}", sessions_dir.display()))?;

        Ok(Self { sessions_dir })
    }

    /// Get the directory path for a specific session.
    fn session_dir(&self, session_id: &str) -> PathBuf {
        self.sessions_dir.join(session_id)
    }

    /// Save session metadata to session.json.
    pub fn save_session_metadata(&self, session: &Session) -> Result<()> {
        let session_dir = self.session_dir(&session.id);
        fs::create_dir_all(&session_dir)
            .with_context(|| format!("Failed to create session directory: {}", session_dir.display()))?;

        let metadata_path = session_dir.join("session.json");
        let json = serde_json::to_string_pretty(session)
            .context("Failed to serialize session metadata")?;

        self.atomic_write(&metadata_path, &json)
            .with_context(|| format!("Failed to write session metadata: {}", metadata_path.display()))?;

        Ok(())
    }

    /// Append a message to messages.jsonl.
    pub fn append_message(&self, session_id: &str, message: &Message) -> Result<()> {
        let messages_path = self.session_dir(session_id).join("messages.jsonl");
        self.append_jsonl(&messages_path, message)
    }

    /// Append a tool call to tools.jsonl.
    pub fn append_tool_call(&self, session_id: &str, tool_call: &ToolCall) -> Result<()> {
        let tools_path = self.session_dir(session_id).join("tools.jsonl");
        self.append_jsonl(&tools_path, tool_call)
    }

    /// Append an approval to approvals.jsonl.
    pub fn append_approval(&self, session_id: &str, approval: &Approval) -> Result<()> {
        let approvals_path = self.session_dir(session_id).join("approvals.jsonl");
        self.append_jsonl(&approvals_path, approval)
    }

    /// Save an artifact to the artifacts directory.
    pub fn save_artifact(&self, session_id: &str, artifact_id: &str, content: &[u8]) -> Result<PathBuf> {
        let artifacts_dir = self.session_dir(session_id).join("artifacts");
        fs::create_dir_all(&artifacts_dir)
            .with_context(|| format!("Failed to create artifacts directory: {}", artifacts_dir.display()))?;

        let artifact_path = artifacts_dir.join(artifact_id);
        fs::write(&artifact_path, content)
            .with_context(|| format!("Failed to write artifact: {}", artifact_path.display()))?;

        Ok(artifact_path)
    }

    /// Load a session from disk, reconstructing full state from all log files.
    pub fn load_session(&self, session_id: &str) -> Result<Session> {
        let session_dir = self.session_dir(session_id);
        let metadata_path = session_dir.join("session.json");

        // Load base session metadata
        let content = fs::read_to_string(&metadata_path)
            .with_context(|| format!("Session not found: {}", session_id))?;
        let mut session: Session = serde_json::from_str(&content)
            .context("Failed to deserialize session metadata")?;

        // Reconstruct messages from messages.jsonl
        let messages_path = session_dir.join("messages.jsonl");
        if messages_path.exists() {
            session.messages = self.load_jsonl::<Message>(&messages_path)?;
        }

        // Reconstruct tool calls from tools.jsonl
        let tools_path = session_dir.join("tools.jsonl");
        if tools_path.exists() {
            session.tool_calls = self.load_jsonl::<ToolCall>(&tools_path)?;
        }

        // Reconstruct approvals from approvals.jsonl
        let approvals_path = session_dir.join("approvals.jsonl");
        if approvals_path.exists() {
            session.approvals = self.load_jsonl::<Approval>(&approvals_path)?;
        }

        // Load artifacts metadata
        let artifacts_dir = session_dir.join("artifacts");
        if artifacts_dir.exists() {
            session.artifacts = self.load_artifacts(&artifacts_dir)?;
        }

        Ok(session)
    }

    /// List all session IDs by scanning the sessions directory.
    pub fn list_session_ids(&self) -> Result<Vec<String>> {
        let mut session_ids = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(session_ids);
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check if it has a session.json file
                let metadata_path = path.join("session.json");
                if metadata_path.exists() {
                    if let Some(session_id) = path.file_name().and_then(|n| n.to_str()) {
                        session_ids.push(session_id.to_string());
                    }
                }
            }
        }

        Ok(session_ids)
    }

    /// Delete a session and all its data.
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let session_dir = self.session_dir(session_id);
        if session_dir.exists() {
            fs::remove_dir_all(&session_dir)
                .with_context(|| format!("Failed to delete session directory: {}", session_dir.display()))?;
        }
        Ok(())
    }

    /// Write content to a file atomically.
    fn atomic_write(&self, file_path: &Path, content: &str) -> Result<()> {
        let temp_suffix = Uuid::new_v4().to_string();
        let temp_filename = format!(
            "{}.tmp.{}",
            file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file"),
            temp_suffix
        );
        let temp_path = file_path
            .parent()
            .unwrap_or(&self.sessions_dir)
            .join(&temp_filename);

        // Write to temporary file
        fs::write(&temp_path, content).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            anyhow::anyhow!("Failed to write temporary file: {}", e)
        })?;

        // Atomically rename temp file to final destination
        fs::rename(&temp_path, file_path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            anyhow::anyhow!("Failed to atomically rename file: {}", e)
        })?;

        Ok(())
    }

    /// Append a JSON-serializable object to a JSONL file.
    fn append_jsonl<T: Serialize>(&self, path: &Path, item: &T) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Open file in append mode
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("Failed to open JSONL file: {}", path.display()))?;

        // Serialize and write
        let json = serde_json::to_string(item)
            .context("Failed to serialize item to JSON")?;
        writeln!(file, "{}", json)
            .with_context(|| format!("Failed to write to JSONL file: {}", path.display()))?;

        Ok(())
    }

    /// Load all items from a JSONL file.
    fn load_jsonl<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<Vec<T>> {
        let mut items = Vec::new();

        if !path.exists() {
            return Ok(items);
        }

        let file = fs::File::open(path)
            .with_context(|| format!("Failed to open JSONL file: {}", path.display()))?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line
                .with_context(|| format!("Failed to read line {} from {}", line_num + 1, path.display()))?;

            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<T>(&line) {
                Ok(item) => items.push(item),
                Err(e) => {
                    warn!(
                        "Failed to parse line {} in {}: {}",
                        line_num + 1,
                        path.display(),
                        e
                    );
                }
            }
        }

        Ok(items)
    }

    /// Load artifacts metadata from the artifacts directory.
    fn load_artifacts(&self, artifacts_dir: &Path) -> Result<Vec<crate::session::state::Artifact>> {
        let mut artifacts = Vec::new();

        if !artifacts_dir.exists() {
            return Ok(artifacts);
        }

        for entry in fs::read_dir(artifacts_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = fs::metadata(&path)?;
                let created = metadata
                    .created()
                    .or_else(|_| metadata.modified())
                    .ok()
                    .map(|t| DateTime::<Utc>::from(t))
                    .unwrap_or_else(Utc::now);

                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    artifacts.push(crate::session::state::Artifact {
                        id: filename.to_string(),
                        path: path
                            .strip_prefix(artifacts_dir.parent().unwrap())
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string(),
                        artifact_type: "file".to_string(),
                        timestamp: created,
                    });
                }
            }
        }

        Ok(artifacts)
    }
}
