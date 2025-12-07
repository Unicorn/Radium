// Orchestration context for maintaining conversation state
//
// The context tracks conversation history, user preferences, and session state
// across multiple orchestration calls.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Message in conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role (user, assistant, system)
    pub role: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into(), timestamp: Utc::now() }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".to_string(), content: content.into(), timestamp: Utc::now() }
    }

    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into(), timestamp: Utc::now() }
    }
}

/// User preferences for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred orchestration provider
    pub preferred_provider: Option<String>,
    /// Maximum tool iterations
    pub max_tool_iterations: u32,
    /// Temperature for generation
    pub temperature: f32,
    /// Whether to show thinking process
    pub show_thinking: bool,
    /// Custom preferences
    #[serde(default)]
    pub custom: HashMap<String, Value>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_provider: None,
            max_tool_iterations: 5,
            temperature: 0.7,
            show_thinking: true,
            custom: HashMap::new(),
        }
    }
}

impl UserPreferences {
    /// Create new user preferences with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set preferred provider
    #[must_use]
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.preferred_provider = Some(provider.into());
        self
    }

    /// Set max tool iterations
    #[must_use]
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_tool_iterations = max;
        self
    }

    /// Set temperature
    #[must_use]
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Set custom preference
    #[must_use]
    pub fn with_custom(mut self, key: impl Into<String>, value: Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }
}

/// Orchestration context for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationContext {
    /// Session identifier
    pub session_id: String,
    /// Conversation history
    pub conversation_history: Vec<Message>,
    /// User preferences
    pub user_preferences: UserPreferences,
    /// Session state for storing arbitrary data
    #[serde(default)]
    pub session_state: HashMap<String, Value>,
    /// When the session was created
    pub created_at: DateTime<Utc>,
}

impl OrchestrationContext {
    /// Create a new orchestration context
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            conversation_history: Vec::new(),
            user_preferences: UserPreferences::default(),
            session_state: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Add a message to conversation history
    pub fn add_message(&mut self, message: Message) {
        self.conversation_history.push(message);
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::user(content));
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::assistant(content));
    }

    /// Get recent conversation history (last N messages)
    pub fn recent_history(&self, n: usize) -> &[Message] {
        let start = self.conversation_history.len().saturating_sub(n);
        &self.conversation_history[start..]
    }

    /// Set session state
    pub fn set_state(&mut self, key: impl Into<String>, value: Value) {
        self.session_state.insert(key.into(), value);
    }

    /// Get session state
    pub fn get_state(&self, key: &str) -> Option<&Value> {
        self.session_state.get(key)
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }

    /// Get number of messages in history
    pub fn history_length(&self) -> usize {
        self.conversation_history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let user_msg = Message::user("Hello");
        assert_eq!(user_msg.role, "user");
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = Message::assistant("Hi there");
        assert_eq!(assistant_msg.role, "assistant");
        assert_eq!(assistant_msg.content, "Hi there");

        let system_msg = Message::system("System message");
        assert_eq!(system_msg.role, "system");
        assert_eq!(system_msg.content, "System message");
    }

    #[test]
    fn test_user_preferences_builder() {
        let prefs = UserPreferences::new()
            .with_provider("gemini")
            .with_max_iterations(10)
            .with_temperature(0.8)
            .with_custom("key", serde_json::json!("value"));

        assert_eq!(prefs.preferred_provider, Some("gemini".to_string()));
        assert_eq!(prefs.max_tool_iterations, 10);
        assert!((prefs.temperature - 0.8).abs() < f32::EPSILON);
        assert_eq!(prefs.custom.get("key"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn test_orchestration_context() {
        let mut ctx = OrchestrationContext::new("session_123");
        assert_eq!(ctx.session_id, "session_123");
        assert_eq!(ctx.history_length(), 0);

        ctx.add_user_message("Hello");
        ctx.add_assistant_message("Hi");
        assert_eq!(ctx.history_length(), 2);

        let recent = ctx.recent_history(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].role, "assistant");
    }

    #[test]
    fn test_session_state() {
        let mut ctx = OrchestrationContext::new("session_123");

        ctx.set_state("key1", serde_json::json!("value1"));
        ctx.set_state("key2", serde_json::json!(42));

        assert_eq!(ctx.get_state("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(ctx.get_state("key2"), Some(&serde_json::json!(42)));
        assert_eq!(ctx.get_state("missing"), None);
    }

    #[test]
    fn test_clear_history() {
        let mut ctx = OrchestrationContext::new("session_123");
        ctx.add_user_message("Message 1");
        ctx.add_user_message("Message 2");
        assert_eq!(ctx.history_length(), 2);

        ctx.clear_history();
        assert_eq!(ctx.history_length(), 0);
    }
}
