//! Chat agent implementation.
//!
//! This agent maintains conversation context across multiple interactions.

use crate::{Agent, AgentContext, AgentOutput};
use async_trait::async_trait;
use radium_abstraction::{ChatMessage, ModelError};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

/// A chat agent that maintains conversation context.
#[derive(Debug)]
pub struct ChatAgent {
    /// The agent's unique ID.
    id: String,
    /// The agent's description.
    description: String,
    /// Conversation history.
    history: Arc<RwLock<Vec<ChatMessage>>>,
    /// Maximum number of messages to keep in history.
    max_history: usize,
}

impl ChatAgent {
    /// Creates a new `ChatAgent` with the given ID and description.
    ///
    /// # Arguments
    /// * `id` - The agent ID
    /// * `description` - The agent description
    #[must_use]
    pub fn new(id: String, description: String) -> Self {
        Self { id, description, history: Arc::new(RwLock::new(Vec::new())), max_history: 100 }
    }

    /// Creates a new `ChatAgent` with a custom maximum history size.
    ///
    /// # Arguments
    /// * `id` - The agent ID
    /// * `description` - The agent description
    /// * `max_history` - Maximum number of messages to keep in history
    #[must_use]
    pub fn with_max_history(id: String, description: String, max_history: usize) -> Self {
        Self { id, description, history: Arc::new(RwLock::new(Vec::new())), max_history }
    }

    /// Clears the conversation history.
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
        debug!(agent_id = %self.id, "ChatAgent history cleared");
    }

    /// Returns the current conversation history length.
    ///
    /// # Returns
    /// The number of messages in the history.
    pub async fn history_len(&self) -> usize {
        let history = self.history.read().await;
        history.len()
    }
}

#[async_trait]
impl Agent for ChatAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(
        &self,
        input: &str,
        context: AgentContext<'_>,
    ) -> Result<AgentOutput, ModelError> {
        let history_len = self.history_len().await;
        debug!(
            agent_id = %self.id,
            input_len = input.len(),
            history_len = history_len,
            "ChatAgent executing"
        );

        // Add user message to history
        let mut history = self.history.write().await;
        history.push(ChatMessage { role: "user".to_string(), content: input.to_string() });

        // Convert history to slice for model
        let messages: Vec<ChatMessage> = history.clone();
        drop(history); // Release lock before async operation

        // Generate response using chat completion
        let response =
            context.model.generate_chat_completion(&messages, None).await.map_err(|e| {
                error!(agent_id = %self.id, error = %e, "Model generation failed");
                e
            })?;

        // Add assistant response to history and trim if necessary
        let mut history = self.history.write().await;
        history
            .push(ChatMessage { role: "assistant".to_string(), content: response.content.clone() });

        // Trim history if it exceeds max_history (keep most recent messages)
        if history.len() > self.max_history {
            let excess = history.len() - self.max_history;
            history.drain(..excess);
            warn!(
                agent_id = %self.id,
                trimmed = excess,
                "ChatAgent history trimmed"
            );
        }

        debug!(
            agent_id = %self.id,
            response_len = response.content.len(),
            history_len = history.len(),
            "ChatAgent completed"
        );

        Ok(AgentOutput::Text(response.content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_models::ModelFactory;

    #[tokio::test]
    async fn test_chat_agent_execution() {
        let agent = ChatAgent::new("test-chat".to_string(), "Test chat agent".to_string());
        let model = ModelFactory::create_from_str("mock", "mock-model".to_string()).unwrap();

        // First message
        let context1 = AgentContext {
            model: model.as_ref(),
            collaboration: None,
        };
        let result1 = agent.execute("Hello!", context1).await;
        assert!(result1.is_ok());
        assert_eq!(agent.history_len().await, 2); // user + assistant

        // Second message (should have context)
        let context2 = AgentContext {
            model: model.as_ref(),
            collaboration: None,
        };
        let result2 = agent.execute("What did I say?", context2).await;
        assert!(result2.is_ok());
        assert_eq!(agent.history_len().await, 4); // 2 previous + 2 new
    }

    #[tokio::test]
    async fn test_chat_agent_clear_history() {
        let agent = ChatAgent::new("test-chat".to_string(), "Test chat agent".to_string());
        let model = ModelFactory::create_from_str("mock", "mock-model".to_string()).unwrap();

        let context = AgentContext {
            model: model.as_ref(),
            collaboration: None,
        };
        agent.execute("Hello!", context).await.unwrap();
        assert_eq!(agent.history_len().await, 2);

        agent.clear_history().await;
        assert_eq!(agent.history_len().await, 0);
    }

    #[tokio::test]
    async fn test_chat_agent_max_history() {
        let agent = ChatAgent::with_max_history(
            "test-chat".to_string(),
            "Test chat agent".to_string(),
            4, // Keep only 4 messages
        );
        let model = ModelFactory::create_from_str("mock", "mock-model".to_string()).unwrap();

        // Send multiple messages to exceed max_history
        for i in 0..5 {
            let context = AgentContext {
            model: model.as_ref(),
            collaboration: None,
        };
            agent.execute(&format!("Message {}", i), context).await.unwrap();
        }

        // History should be trimmed to max_history (each execute adds 2 messages: user + assistant)
        // After 5 executes: 10 messages, trimmed to 4
        assert_eq!(agent.history_len().await, 4);
    }
}
