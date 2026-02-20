//! Session history persistence for TUI.
//!
//! Manages session state with automatic persistence to disk,
//! enabling session resume across app restarts.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::time::SystemTime;
use tracing::debug;

/// Historical message in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMessage {
    /// Message role ("user", "assistant", "system").
    pub role: String,
    /// Message content.
    pub content: String,
    /// Timestamp when message was created.
    pub timestamp: SystemTime,
    /// Whether streaming is complete for this message.
    pub streaming_complete: bool,
}

/// Session history with persistence support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistory {
    /// Unique session identifier.
    pub session_id: String,
    /// Agent ID for this session.
    pub agent_id: String,
    /// All messages in the session.
    pub messages: Vec<HistoricalMessage>,
    /// Timestamp when session was created.
    pub created_at: SystemTime,
    /// Timestamp of last activity.
    pub last_active: SystemTime,
    /// Number of tokens streamed since last save.
    #[serde(skip)]
    token_count_since_save: u32,
}

impl SessionHistory {
    /// Creates a new session history.
    pub fn new(session_id: impl Into<String>, agent_id: impl Into<String>) -> Self {
        let now = SystemTime::now();
        Self {
            session_id: session_id.into(),
            agent_id: agent_id.into(),
            messages: Vec::new(),
            created_at: now,
            last_active: now,
            token_count_since_save: 0,
        }
    }

    /// Appends a streaming token to the current message.
    ///
    /// If no message is currently streaming, starts a new assistant message.
    pub fn append_streaming_token(&mut self, content: &str) {
        // Check if last message is an incomplete assistant message
        if let Some(last_msg) = self.messages.last_mut() {
            if last_msg.role == "assistant" && !last_msg.streaming_complete {
                last_msg.content.push_str(content);
                self.token_count_since_save += 1;
                return;
            }
        }

        // Start new streaming message
        self.messages.push(HistoricalMessage {
            role: "assistant".to_string(),
            content: content.to_string(),
            timestamp: SystemTime::now(),
            streaming_complete: false,
        });
        self.token_count_since_save += 1;
    }

    /// Marks the current streaming message as complete.
    pub fn mark_streaming_complete(&mut self) {
        if let Some(last_msg) = self.messages.last_mut() {
            if !last_msg.streaming_complete {
                last_msg.streaming_complete = true;
                self.last_active = SystemTime::now();
                self.token_count_since_save = 0;
            }
        }
    }

    /// Adds a user message to the session.
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(HistoricalMessage {
            role: "user".to_string(),
            content: content.into(),
            timestamp: SystemTime::now(),
            streaming_complete: true,
        });
        self.last_active = SystemTime::now();
    }

    /// Adds a system message to the session.
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.messages.push(HistoricalMessage {
            role: "system".to_string(),
            content: content.into(),
            timestamp: SystemTime::now(),
            streaming_complete: true,
        });
        self.last_active = SystemTime::now();
    }

    /// Returns whether auto-save is needed (every 10 tokens).
    pub fn should_auto_save(&self) -> bool {
        self.token_count_since_save >= 10
    }

    /// Saves session history to disk.
    ///
    /// Persists to `{workspace_root}/.radium/history/{session_id}.json`.
    pub fn save_to_disk(&mut self, workspace_root: &Path) -> Result<(), std::io::Error> {
        let history_dir = workspace_root.join(".radium/history");
        fs::create_dir_all(&history_dir)?;

        let file_path = history_dir.join(format!("{}.json", self.session_id));
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)?;

        // Reset auto-save counter
        self.token_count_since_save = 0;

        debug!("Saved session history: {}", self.session_id);
        Ok(())
    }

    /// Loads session history from disk.
    pub fn load_from_disk(
        session_id: &str,
        workspace_root: &Path,
    ) -> Result<Self, std::io::Error> {
        let file_path = workspace_root
            .join(".radium/history")
            .join(format!("{}.json", session_id));

        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);
        let mut history: SessionHistory = serde_json::from_reader(reader)?;

        // Reset transient state
        history.token_count_since_save = 0;

        debug!("Loaded session history: {}", session_id);
        Ok(history)
    }

    /// Lists all available session history files.
    pub fn list_sessions(workspace_root: &Path) -> Result<Vec<String>, std::io::Error> {
        let history_dir = workspace_root.join(".radium/history");

        if !history_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        for entry in fs::read_dir(history_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    sessions.push(stem.to_string());
                }
            }
        }

        Ok(sessions)
    }

    /// Deletes a session history file from disk.
    pub fn delete_from_disk(session_id: &str, workspace_root: &Path) -> Result<(), std::io::Error> {
        let file_path = workspace_root
            .join(".radium/history")
            .join(format!("{}.json", session_id));

        if file_path.exists() {
            fs::remove_file(file_path)?;
            debug!("Deleted session history: {}", session_id);
        }

        Ok(())
    }

    /// Returns a preview of the session (first user message or session ID).
    pub fn preview(&self) -> String {
        self.messages
            .iter()
            .find(|m| m.role == "user")
            .map(|m| {
                let preview_len = 50;
                if m.content.len() > preview_len {
                    format!("{}...", &m.content[..preview_len])
                } else {
                    m.content.clone()
                }
            })
            .unwrap_or_else(|| self.session_id.clone())
    }

    /// Returns the number of messages in the session.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Returns whether the session has any messages.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Default for SessionHistory {
    fn default() -> Self {
        Self::new("default", "chat-assistant")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_streaming_token_append() {
        let mut session = SessionHistory::new("test-session", "test-agent");

        session.append_streaming_token("Hello ");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "Hello ");
        assert!(!session.messages[0].streaming_complete);

        session.append_streaming_token("world!");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "Hello world!");
    }

    #[test]
    fn test_mark_streaming_complete() {
        let mut session = SessionHistory::new("test-session", "test-agent");

        session.append_streaming_token("Test message");
        session.mark_streaming_complete();

        assert!(session.messages[0].streaming_complete);
        assert_eq!(session.token_count_since_save, 0);
    }

    #[test]
    fn test_auto_save_threshold() {
        let mut session = SessionHistory::new("test-session", "test-agent");

        for _ in 0..9 {
            session.append_streaming_token("token ");
        }
        assert!(!session.should_auto_save());

        session.append_streaming_token("token ");
        assert!(session.should_auto_save());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = env::temp_dir().join("radium-test");
        fs::create_dir_all(&temp_dir).unwrap();

        let mut session = SessionHistory::new("test-save-load", "test-agent");
        session.add_user_message("Hello");
        session.append_streaming_token("Hi there");
        session.mark_streaming_complete();

        session.save_to_disk(&temp_dir).unwrap();

        let loaded = SessionHistory::load_from_disk("test-save-load", &temp_dir).unwrap();
        assert_eq!(loaded.session_id, "test-save-load");
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.messages[0].content, "Hello");
        assert_eq!(loaded.messages[1].content, "Hi there");

        // Cleanup
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_preview() {
        let mut session = SessionHistory::new("test-preview", "test-agent");
        session.add_user_message("This is a very long message that should be truncated in the preview to show only the first fifty characters");

        let preview = session.preview();
        assert_eq!(preview.len(), 53); // 50 chars + "..."
        assert!(preview.starts_with("This is a very long message"));
        assert!(preview.ends_with("..."));
    }
}
