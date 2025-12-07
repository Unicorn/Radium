//! Simple agent implementation.
//!
//! This agent processes text input using a model and returns the response.

use crate::{Agent, AgentContext, AgentOutput};
use async_trait::async_trait;
use radium_abstraction::ModelError;
use tracing::{debug, error};

/// A simple agent that processes text using a model.
#[derive(Debug, Clone)]
pub struct SimpleAgent {
    /// The agent's unique ID.
    id: String,
    /// The agent's description.
    description: String,
}

impl SimpleAgent {
    /// Creates a new `SimpleAgent` with the given ID and description.
    ///
    /// # Arguments
    /// * `id` - The agent ID
    /// * `description` - The agent description
    #[must_use]
    pub fn new(id: String, description: String) -> Self {
        Self { id, description }
    }
}

#[async_trait]
impl Agent for SimpleAgent {
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
        debug!(
            agent_id = %self.id,
            input_len = input.len(),
            "SimpleAgent executing"
        );

        // Use the model to generate a response
        let response = context.model.generate_text(input, None).await.map_err(|e| {
            error!(agent_id = %self.id, error = %e, "Model generation failed");
            e
        })?;

        debug!(
            agent_id = %self.id,
            response_len = response.content.len(),
            "SimpleAgent completed"
        );

        Ok(AgentOutput::Text(response.content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_models::ModelFactory;

    #[tokio::test]
    async fn test_simple_agent_execution() {
        let agent = SimpleAgent::new("test-simple".to_string(), "Test simple agent".to_string());
        let model = ModelFactory::create_from_str("mock", "mock-model".to_string()).unwrap();

        let context = AgentContext {
            model: model.as_ref(),
            collaboration: None,
        };
        let result = agent.execute("Hello, world!", context).await;

        assert!(result.is_ok());
        match result.unwrap() {
            AgentOutput::Text(text) => {
                assert!(!text.is_empty());
            }
            _ => panic!("Expected Text output"),
        }
    }
}
