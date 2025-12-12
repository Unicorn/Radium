//! Session state definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session state enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SessionState {
    /// Session is active and processing requests.
    Active,
    /// Session is paused (not processing but can be resumed).
    Paused,
    /// Session completed successfully.
    Completed,
    /// Session failed with an error.
    Failed,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Active
    }
}

impl ToString for SessionState {
    fn to_string(&self) -> String {
        match self {
            SessionState::Active => "ACTIVE".to_string(),
            SessionState::Paused => "PAUSED".to_string(),
            SessionState::Completed => "COMPLETED".to_string(),
            SessionState::Failed => "FAILED".to_string(),
        }
    }
}

/// A message in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID.
    pub id: String,
    /// Message content.
    pub content: String,
    /// Message role: "user", "assistant", "system".
    pub role: String,
    /// Timestamp when message was created.
    pub timestamp: DateTime<Utc>,
}

/// A tool call in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique tool call ID.
    pub id: String,
    /// Tool name.
    pub tool_name: String,
    /// Tool arguments as JSON string.
    pub arguments_json: String,
    /// Tool result as JSON string (if completed).
    pub result_json: Option<String>,
    /// Whether the tool call succeeded.
    pub success: bool,
    /// Error message if tool call failed.
    pub error: Option<String>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Timestamp when tool call was made.
    pub timestamp: DateTime<Utc>,
}

/// An approval decision in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    /// Unique approval ID.
    pub id: String,
    /// Tool name that required approval.
    pub tool_name: String,
    /// Tool arguments as JSON string.
    pub arguments_json: String,
    /// Policy rule that triggered the approval.
    pub policy_rule: String,
    /// Whether the approval was granted.
    pub approved: bool,
    /// Reason for approval decision.
    pub reason: Option<String>,
    /// Timestamp when approval was requested.
    pub timestamp: DateTime<Utc>,
}

/// An artifact stored in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact ID (filename or identifier).
    pub id: String,
    /// Artifact path relative to session directory.
    pub path: String,
    /// Artifact type (e.g., "file", "report", "log").
    pub artifact_type: String,
    /// Timestamp when artifact was created.
    pub timestamp: DateTime<Utc>,
}

/// A session with full state and history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier.
    pub id: String,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// Last time the session was active.
    pub last_active: DateTime<Utc>,
    /// Current session state.
    pub state: SessionState,
    /// Optional agent ID associated with this session.
    pub agent_id: Option<String>,
    /// Optional workspace root for this session.
    pub workspace_root: Option<String>,
    /// Optional session name.
    pub name: Option<String>,
    /// Message history.
    pub messages: Vec<Message>,
    /// Tool call history.
    pub tool_calls: Vec<ToolCall>,
    /// Approval history.
    pub approvals: Vec<Approval>,
    /// Artifacts stored in this session.
    pub artifacts: Vec<Artifact>,
    /// Additional metadata (key-value pairs).
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// Create a new session with the given ID.
    pub fn new(id: String, agent_id: Option<String>, workspace_root: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            created_at: now,
            last_active: now,
            state: SessionState::Active,
            agent_id,
            workspace_root,
            name: None,
            messages: Vec::new(),
            tool_calls: Vec::new(),
            approvals: Vec::new(),
            artifacts: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Update the last active timestamp.
    pub fn touch(&mut self) {
        self.last_active = Utc::now();
    }

    /// Add a message to the session.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.touch();
    }

    /// Add a tool call to the session.
    pub fn add_tool_call(&mut self, tool_call: ToolCall) {
        self.tool_calls.push(tool_call);
        self.touch();
    }

    /// Add an approval to the session.
    pub fn add_approval(&mut self, approval: Approval) {
        self.approvals.push(approval);
        self.touch();
    }

    /// Add an artifact to the session.
    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
        self.touch();
    }

    /// Set session state.
    pub fn set_state(&mut self, state: SessionState) {
        self.state = state;
        self.touch();
    }
}
