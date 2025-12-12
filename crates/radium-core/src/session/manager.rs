//! Session manager for in-memory state and persistence coordination.

use crate::session::state::{Approval, Artifact, Message, Session, SessionState, ToolCall};
use crate::session::storage::SessionStorage;
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Session manager for creating, managing, and persisting sessions.
pub struct SessionManager {
    /// In-memory session cache.
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// Storage backend for persistence.
    storage: SessionStorage,
}

impl SessionManager {
    /// Create a new session manager.
    ///
    /// # Arguments
    /// * `workspace_root` - Workspace root directory for session storage
    ///
    /// # Errors
    /// Returns an error if the storage directory cannot be created.
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let storage = SessionStorage::new(workspace_root)?;
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            storage,
        })
    }

    /// Create a new session.
    ///
    /// # Arguments
    /// * `agent_id` - Optional agent ID for this session
    /// * `workspace_root` - Optional workspace root path
    /// * `name` - Optional session name
    ///
    /// # Returns
    /// The created session with a new UUID as the ID.
    pub async fn create_session(
        &self,
        agent_id: Option<String>,
        workspace_root: Option<String>,
        name: Option<String>,
    ) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        let mut session = Session::new(session_id.clone(), agent_id, workspace_root);
        session.name = name;

        // Persist session metadata
        self.storage.save_session_metadata(&session)?;

        // Add to in-memory cache
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        info!(session_id = %session_id, "Created new session");
        Ok(session)
    }

    /// Get a session by ID, loading from disk if not in memory.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    ///
    /// # Returns
    /// The session if found, or an error if not found.
    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        // Check in-memory cache first
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(session_id) {
                return Ok(session.clone());
            }
        }

        // Load from disk
        debug!(session_id = %session_id, "Loading session from disk");
        let session = self.storage.load_session(session_id)?;

        // Add to cache
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.to_string(), session.clone());

        Ok(session)
    }

    /// List all sessions with pagination and filtering.
    ///
    /// # Arguments
    /// * `page` - Page number (1-indexed)
    /// * `page_size` - Items per page
    /// * `filter_state` - Optional state filter
    /// * `filter_agent_id` - Optional agent ID filter
    ///
    /// # Returns
    /// Tuple of (sessions, total_count, page, page_size)
    pub async fn list_sessions(
        &self,
        page: Option<u32>,
        page_size: Option<u32>,
        filter_state: Option<SessionState>,
        filter_agent_id: Option<String>,
    ) -> Result<(Vec<Session>, u32, u32, u32)> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(50);

        // Get all session IDs from storage
        let session_ids = self.storage.list_session_ids()?;
        let mut sessions = Vec::new();

        // Load sessions (from cache or disk) and apply filters
        for session_id in session_ids {
            match self.get_session(&session_id).await {
                Ok(session) => {
                    // Apply filters
                    if let Some(ref filter_state) = filter_state {
                        if session.state != *filter_state {
                            continue;
                        }
                    }
                    if let Some(ref filter_agent_id) = filter_agent_id {
                        if session.agent_id.as_ref() != Some(filter_agent_id) {
                            continue;
                        }
                    }
                    sessions.push(session);
                }
                Err(e) => {
                    warn!(session_id = %session_id, error = %e, "Failed to load session for listing");
                }
            }
        }

        // Sort by last_active (most recent first)
        sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));

        let total = sessions.len() as u32;
        let offset = ((page - 1) * page_size) as usize;
        let limit = (page * page_size) as usize;

        let paginated_sessions: Vec<Session> = sessions
            .into_iter()
            .skip(offset)
            .take(limit - offset)
            .collect();

        Ok((paginated_sessions, total, page, page_size))
    }

    /// Attach to an existing session, loading full state.
    ///
    /// This is similar to `get_session` but explicitly indicates the client
    /// is attaching to continue work on the session.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    ///
    /// # Returns
    /// The full session state with all history.
    pub async fn attach_session(&self, session_id: &str) -> Result<Session> {
        let mut session = self.get_session(session_id).await?;
        session.touch(); // Update last_active
        self.storage.save_session_metadata(&session)?;

        // Update cache
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.to_string(), session.clone());

        info!(session_id = %session_id, "Client attached to session");
        Ok(session)
    }

    /// Append a message to a session.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    /// * `message` - Message to append
    ///
    /// # Errors
    /// Returns an error if the session doesn't exist or persistence fails.
    pub async fn append_message(&self, session_id: &str, message: Message) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .context(format!("Session not found: {}", session_id))?;

        session.add_message(message.clone());
        self.storage.save_session_metadata(session)?;
        self.storage.append_message(session_id, &message)?;

        Ok(())
    }

    /// Append a tool call to a session.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    /// * `tool_call` - Tool call to append
    ///
    /// # Errors
    /// Returns an error if the session doesn't exist or persistence fails.
    pub async fn append_tool_call(&self, session_id: &str, tool_call: ToolCall) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .context(format!("Session not found: {}", session_id))?;

        session.add_tool_call(tool_call.clone());
        self.storage.save_session_metadata(session)?;
        self.storage.append_tool_call(session_id, &tool_call)?;

        Ok(())
    }

    /// Append an approval to a session.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    /// * `approval` - Approval to append
    ///
    /// # Errors
    /// Returns an error if the session doesn't exist or persistence fails.
    pub async fn append_approval(&self, session_id: &str, approval: Approval) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .context(format!("Session not found: {}", session_id))?;

        session.add_approval(approval.clone());
        self.storage.save_session_metadata(session)?;
        self.storage.append_approval(session_id, &approval)?;

        Ok(())
    }

    /// Save an artifact to a session.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    /// * `artifact_id` - Artifact identifier (filename)
    /// * `content` - Artifact content bytes
    ///
    /// # Returns
    /// The path where the artifact was saved.
    ///
    /// # Errors
    /// Returns an error if the session doesn't exist or persistence fails.
    pub async fn save_artifact(
        &self,
        session_id: &str,
        artifact_id: &str,
        content: &[u8],
    ) -> Result<PathBuf> {
        let artifact_path = self.storage.save_artifact(session_id, artifact_id, content)?;

        // Update session metadata
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .context(format!("Session not found: {}", session_id))?;

        let artifact = Artifact {
            id: artifact_id.to_string(),
            path: artifact_path.to_string_lossy().to_string(),
            artifact_type: "file".to_string(),
            timestamp: Utc::now(),
        };

        session.add_artifact(artifact);
        self.storage.save_session_metadata(session)?;

        Ok(artifact_path)
    }

    /// Update session state.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    /// * `state` - New session state
    ///
    /// # Errors
    /// Returns an error if the session doesn't exist or persistence fails.
    pub async fn update_session_state(&self, session_id: &str, state: SessionState) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .context(format!("Session not found: {}", session_id))?;

        session.set_state(state);
        self.storage.save_session_metadata(session)?;

        Ok(())
    }

    /// Delete a session and all its data.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    ///
    /// # Errors
    /// Returns an error if deletion fails.
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        // Remove from cache
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);

        // Delete from storage
        self.storage.delete_session(session_id)?;

        info!(session_id = %session_id, "Deleted session");
        Ok(())
    }
}

use std::path::PathBuf;
impl SessionManager {
    /// Get the storage instance (for testing).
    #[cfg(test)]
    pub fn storage(&self) -> &SessionStorage {
        &self.storage
    }
}
