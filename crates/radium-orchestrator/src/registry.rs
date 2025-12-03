//! Agent registry for managing registered agents.
//!
//! This module provides functionality to register, retrieve, list, and unregister agents.

use crate::Agent;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Metadata about a registered agent.
#[derive(Debug, Clone)]
pub struct AgentMetadata {
    /// The agent's unique ID.
    pub id: String,
    /// The agent's description.
    pub description: String,
    /// Whether the agent is currently registered.
    pub registered: bool,
}

/// Registry for managing agents.
pub struct AgentRegistry {
    /// Map of agent ID to agent instance.
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent + Send + Sync>>>>,
}

impl fmt::Debug for AgentRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentRegistry")
            .field("agent_count", &self.agents.try_read().map(|a| a.len()).unwrap_or(0))
            .finish_non_exhaustive()
    }
}

impl AgentRegistry {
    /// Creates a new empty agent registry.
    #[must_use]
    pub fn new() -> Self {
        Self { agents: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Registers an agent in the registry.
    ///
    /// # Arguments
    /// * `agent` - The agent to register
    ///
    /// # Returns
    /// Returns `true` if the agent was newly registered, `false` if it replaced an existing agent.
    pub async fn register_agent(&self, agent: Arc<dyn Agent + Send + Sync>) -> bool {
        let id = agent.id().to_string();

        debug!(agent_id = %id, "Registering agent");

        let mut agents = self.agents.write().await;
        let was_new = !agents.contains_key(&id);
        agents.insert(id.clone(), agent);

        if !was_new {
            warn!(agent_id = %id, "Agent replaced in registry");
        }

        was_new
    }

    /// Retrieves an agent by ID.
    ///
    /// # Arguments
    /// * `id` - The agent ID to look up
    ///
    /// # Returns
    /// Returns `Some(Arc<dyn Agent>)` if found, `None` otherwise.
    pub async fn get_agent(&self, id: &str) -> Option<Arc<dyn Agent + Send + Sync>> {
        debug!(agent_id = %id, "Retrieving agent");

        let agents = self.agents.read().await;
        agents.get(id).cloned()
    }

    /// Lists all registered agents with their metadata.
    ///
    /// # Returns
    /// Returns a vector of agent metadata.
    pub async fn list_agents(&self) -> Vec<AgentMetadata> {
        debug!("Listing all agents");

        let agents = self.agents.read().await;
        agents
            .iter()
            .map(|(id, agent)| AgentMetadata {
                id: id.clone(),
                description: agent.description().to_string(),
                registered: true,
            })
            .collect()
    }

    /// Unregisters an agent from the registry.
    ///
    /// # Arguments
    /// * `id` - The agent ID to unregister
    ///
    /// # Returns
    /// Returns `true` if the agent was found and removed, `false` otherwise.
    pub async fn unregister_agent(&self, id: &str) -> bool {
        debug!(agent_id = %id, "Unregistering agent");

        let mut agents = self.agents.write().await;
        let removed = agents.remove(id).is_some();

        if !removed {
            warn!(agent_id = %id, "Attempted to unregister non-existent agent");
        }

        removed
    }

    /// Checks if an agent is registered.
    ///
    /// # Arguments
    /// * `id` - The agent ID to check
    ///
    /// # Returns
    /// Returns `true` if the agent is registered, `false` otherwise.
    pub async fn is_registered(&self, id: &str) -> bool {
        let agents = self.agents.read().await;
        agents.contains_key(id)
    }

    /// Returns the number of registered agents.
    ///
    /// # Returns
    /// The count of registered agents.
    pub async fn count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EchoAgent;

    #[tokio::test]
    async fn test_register_agent() {
        let registry = AgentRegistry::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));

        let was_new = registry.register_agent(agent).await;
        assert!(was_new);
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_register_duplicate_agent() {
        let registry = AgentRegistry::new();
        let agent1 = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        let agent2 =
            Arc::new(EchoAgent::new("test-agent".to_string(), "Updated agent".to_string()));

        let was_new1 = registry.register_agent(agent1).await;
        assert!(was_new1);

        let was_new2 = registry.register_agent(agent2).await;
        assert!(!was_new2); // Should replace existing
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_get_agent() {
        let registry = AgentRegistry::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));

        registry.register_agent(agent.clone()).await;

        let retrieved = registry.get_agent("test-agent").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), "test-agent");
    }

    #[tokio::test]
    async fn test_get_nonexistent_agent() {
        let registry = AgentRegistry::new();
        let retrieved = registry.get_agent("nonexistent").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_agents() {
        let registry = AgentRegistry::new();
        let agent1 = Arc::new(EchoAgent::new("agent-1".to_string(), "Agent 1".to_string()));
        let agent2 = Arc::new(EchoAgent::new("agent-2".to_string(), "Agent 2".to_string()));

        registry.register_agent(agent1).await;
        registry.register_agent(agent2).await;

        let agents = registry.list_agents().await;
        assert_eq!(agents.len(), 2);
        assert!(agents.iter().any(|a| a.id == "agent-1"));
        assert!(agents.iter().any(|a| a.id == "agent-2"));
    }

    #[tokio::test]
    async fn test_unregister_agent() {
        let registry = AgentRegistry::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));

        registry.register_agent(agent).await;
        assert_eq!(registry.count().await, 1);

        let removed = registry.unregister_agent("test-agent").await;
        assert!(removed);
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_agent() {
        let registry = AgentRegistry::new();
        let removed = registry.unregister_agent("nonexistent").await;
        assert!(!removed);
    }

    #[tokio::test]
    async fn test_is_registered() {
        let registry = AgentRegistry::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));

        assert!(!registry.is_registered("test-agent").await);

        registry.register_agent(agent).await;
        assert!(registry.is_registered("test-agent").await);
    }

    #[tokio::test]
    async fn test_count() {
        let registry = AgentRegistry::new();
        assert_eq!(registry.count().await, 0);

        let agent1 = Arc::new(EchoAgent::new("agent-1".to_string(), "Agent 1".to_string()));
        let agent2 = Arc::new(EchoAgent::new("agent-2".to_string(), "Agent 2".to_string()));

        registry.register_agent(agent1).await;
        assert_eq!(registry.count().await, 1);

        registry.register_agent(agent2).await;
        assert_eq!(registry.count().await, 2);
    }
}
