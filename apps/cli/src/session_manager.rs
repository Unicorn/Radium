//! Session management for CLI chat mode.
//!
//! Provides save/load functionality for conversation sessions,
//! allowing users to resume long-running conversations.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use radium_abstraction::ChatMessage;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::conversation_context::ConversationContext;

/// A saved chat session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,

    /// When this session was created
    pub created_at: DateTime<Utc>,

    /// Last time this session was updated
    pub updated_at: DateTime<Utc>,

    /// Agent ID used in this session
    pub agent_id: String,

    /// Conversation history
    pub history: Vec<ChatMessage>,

    /// Conversation context
    pub context: ConversationContext,

    /// Optional session name
    pub name: Option<String>,
}

/// Manages session persistence
pub struct SessionManager {
    /// Directory where sessions are stored
    sessions_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let sessions_dir = workspace_root.join(".radium").join("sessions");

        // Create sessions directory if it doesn't exist
        if !sessions_dir.exists() {
            fs::create_dir_all(&sessions_dir)
                .context("Failed to create sessions directory")?;
        }

        Ok(Self { sessions_dir })
    }

    /// Save a session to disk
    pub fn save_session(&self, session: &Session) -> Result<PathBuf> {
        let filename = format!("{}.json", session.id);
        let path = self.sessions_dir.join(&filename);

        let json = serde_json::to_string_pretty(session)
            .context("Failed to serialize session")?;

        fs::write(&path, json)
            .context("Failed to write session file")?;

        Ok(path)
    }

    /// Load a session from disk
    pub fn load_session(&self, session_id: &str) -> Result<Session> {
        let filename = format!("{}.json", session_id);
        let path = self.sessions_dir.join(&filename);

        if !path.exists() {
            anyhow::bail!("Session '{}' not found", session_id);
        }

        let json = fs::read_to_string(&path)
            .context("Failed to read session file")?;

        let session: Session = serde_json::from_str(&json)
            .context("Failed to deserialize session")?;

        Ok(session)
    }

    /// List all saved sessions
    pub fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let mut sessions = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_session_info(&path) {
                    Ok(info) => sessions.push(info),
                    Err(e) => {
                        eprintln!("Warning: Failed to load session {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by updated_at (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    /// Load just the metadata for a session (faster than loading full session)
    fn load_session_info(&self, path: &Path) -> Result<SessionInfo> {
        let json = fs::read_to_string(path)?;
        let session: Session = serde_json::from_str(&json)?;

        Ok(SessionInfo {
            id: session.id,
            name: session.name,
            agent_id: session.agent_id,
            created_at: session.created_at,
            updated_at: session.updated_at,
            message_count: session.history.len(),
        })
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let filename = format!("{}.json", session_id);
        let path = self.sessions_dir.join(&filename);

        if path.exists() {
            fs::remove_file(path)
                .context("Failed to delete session file")?;
        }

        Ok(())
    }

    /// Generate a new unique session ID
    pub fn generate_session_id(agent_id: &str) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        format!("{}_{}", agent_id, timestamp)
    }
}

/// Summary information about a session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SessionInfo {
    pub id: String,
    pub name: Option<String>,
    pub agent_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}

impl Session {
    /// Create a new session
    pub fn new(id: String, agent_id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            created_at: now,
            updated_at: now,
            agent_id,
            history: Vec::new(),
            context: ConversationContext::new(),
            name: None,
        }
    }

    /// Update the session's timestamp
    #[allow(dead_code)]
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Add a message to history
    pub fn add_message(&mut self, message: ChatMessage) {
        self.history.push(message);
        self.touch();
    }

    /// Update conversation context
    pub fn update_context(&mut self, context: ConversationContext) {
        self.context = context;
        self.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_abstraction::MessageContent;
    use std::env;

    #[test]
    fn test_session_creation() {
        let session = Session::new("test-123".to_string(), "chat-agent".to_string());

        assert_eq!(session.id, "test-123");
        assert_eq!(session.agent_id, "chat-agent");
        assert_eq!(session.history.len(), 0);
    }

    #[test]
    fn test_session_add_message() {
        let mut session = Session::new("test-123".to_string(), "chat-agent".to_string());

        let message = ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("Hello".to_string()),
        };

        session.add_message(message);

        assert_eq!(session.history.len(), 1);
    }

    #[test]
    fn test_session_manager() -> Result<()> {
        let temp_dir = env::temp_dir().join("radium_test_sessions");
        let manager = SessionManager::new(&temp_dir)?;

        let mut session = Session::new("test-456".to_string(), "chat-agent".to_string());
        session.add_message(ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("Test message".to_string()),
        });

        // Save
        manager.save_session(&session)?;

        // Load
        let loaded = manager.load_session("test-456")?;
        assert_eq!(loaded.id, "test-456");
        assert_eq!(loaded.history.len(), 1);

        // List
        let sessions = manager.list_sessions()?;
        assert!(sessions.iter().any(|s| s.id == "test-456"));

        // Delete
        manager.delete_session("test-456")?;

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(())
    }

    #[test]
    fn test_generate_session_id() {
        let id = SessionManager::generate_session_id("chat-gemini");
        assert!(id.starts_with("chat-gemini_"));
        assert!(id.len() > "chat-gemini_".len());
    }
}
