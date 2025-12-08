//! Session management for chat history persistence.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    /// Session ID
    pub session_id: String,
    /// Agent ID
    pub agent_id: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Message count
    pub message_count: usize,
    /// Last message preview
    pub last_message: Option<String>,
    /// Model ID used for this session
    #[serde(default)]
    pub model_id: Option<String>,
}

/// Session manager for loading and saving chat sessions.
pub struct SessionManager {
    sessions_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new(workspace_root: Option<PathBuf>) -> Result<Self> {
        let sessions_dir = if let Some(root) = workspace_root {
            root.join(".radium").join("sessions")
        } else {
            dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
                .join(".radium")
                .join("sessions")
        };

        // Create sessions directory if it doesn't exist
        fs::create_dir_all(&sessions_dir)?;

        Ok(Self { sessions_dir })
    }

    /// Load all sessions, grouped by date.
    pub fn load_sessions(&self) -> Result<HashMap<String, Vec<ChatSession>>> {
        let mut sessions_by_date: HashMap<String, Vec<ChatSession>> = HashMap::new();

        if !self.sessions_dir.exists() {
            return Ok(sessions_by_date);
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<ChatSession>(&content) {
                        let date_key = session.created_at.format("%Y-%m-%d").to_string();
                        sessions_by_date.entry(date_key).or_insert_with(Vec::new).push(session);
                    }
                }
            }
        }

        // Sort sessions within each date by updated_at (most recent first)
        for sessions in sessions_by_date.values_mut() {
            sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        }

        Ok(sessions_by_date)
    }

    /// Save a session.
    pub fn save_session(&self, session: &ChatSession) -> Result<()> {
        let file_path = self.sessions_dir.join(format!("{}.json", session.session_id));
        let content = serde_json::to_string_pretty(session)?;
        fs::write(file_path, content)?;
        Ok(())
    }

    /// Delete a session.
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let file_path = self.sessions_dir.join(format!("{}.json", session_id));
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    /// Update session with new message.
    pub fn update_session(&self, session_id: &str, agent_id: &str, message: &str, model_id: Option<String>) -> Result<()> {
        let file_path = self.sessions_dir.join(format!("{}.json", session_id));

        let mut session = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            serde_json::from_str::<ChatSession>(&content)?
        } else {
            ChatSession {
                session_id: session_id.to_string(),
                agent_id: agent_id.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                message_count: 0,
                last_message: None,
                model_id: None,
            }
        };

        session.updated_at = Utc::now();
        session.message_count += 1;
        session.last_message = Some(message.to_string());
        
        // Update model_id if provided
        if let Some(model) = model_id {
            session.model_id = Some(model);
        }

        self.save_session(&session)
    }
}
