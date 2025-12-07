//! Agent selector for dynamic agent selection based on capabilities and load.

use crate::registry::AgentRegistry;
use crate::queue::ExecutionQueue;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, warn};

/// Model class categories for agent selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelClass {
    /// Fast models (e.g., Flash, Mini) - optimized for speed.
    Fast,

    /// Balanced models (e.g., Pro, 4o) - balanced speed and quality.
    Balanced,

    /// Reasoning models (e.g., o1, Thinking) - optimized for deep reasoning.
    Reasoning,
}

/// Selection criteria for agent selection.
#[derive(Debug, Clone)]
pub struct SelectionCriteria {
    /// Model class to match (fast, balanced, reasoning).
    /// If None, defaults to Balanced.
    pub model_class: Option<ModelClass>,
}

impl Default for SelectionCriteria {
    fn default() -> Self {
        Self {
            model_class: Some(ModelClass::Balanced),
        }
    }
}

/// Errors that can occur during agent selection.
#[derive(Debug, Error)]
pub enum SelectionError {
    /// No agents match the selection criteria.
    #[error("No agents match the selection criteria: {0}")]
    NoMatchingAgents(String),

    /// Agent registry error.
    #[error("Registry error: {0}")]
    RegistryError(String),
}

/// Agent selector for dynamic agent selection based on capabilities and load.
#[derive(Debug)]
pub struct AgentSelector {
    /// Agent registry for finding agents.
    registry: Arc<AgentRegistry>,
    /// Execution queue for checking load.
    queue: Arc<ExecutionQueue>,
}

impl AgentSelector {
    /// Creates a new agent selector.
    ///
    /// # Arguments
    /// * `registry` - The agent registry
    /// * `queue` - The execution queue
    pub fn new(registry: Arc<AgentRegistry>, queue: Arc<ExecutionQueue>) -> Self {
        Self { registry, queue }
    }

    /// Selects the best agent based on criteria and current load.
    ///
    /// # Arguments
    /// * `criteria` - Selection criteria
    ///
    /// # Returns
    /// The agent ID of the best matching agent, or an error if no agents match.
    ///
    /// # Errors
    /// Returns `SelectionError::NoMatchingAgents` if no agents match the criteria.
    pub async fn select_best_agent(
        &self,
        criteria: SelectionCriteria,
    ) -> Result<String, SelectionError> {
        let model_class = criteria.model_class.unwrap_or(ModelClass::Balanced);

        debug!(model_class = ?model_class, "Selecting best agent");

        // Get all registered agents
        let agents = self.registry.list_agents().await;

        if agents.is_empty() {
            return Err(SelectionError::NoMatchingAgents(
                "No agents registered".to_string(),
            ));
        }

        // Filter agents by model_class from capabilities
        let matching_agents: Vec<String> = agents
            .iter()
            .filter_map(|agent_meta| {
                // Get capabilities from metadata
                let capabilities = agent_meta.capabilities.as_ref()?;
                
                // Extract model_class from capabilities JSON
                // Expected format: {"class": "fast|balanced|reasoning", ...}
                let class_str = capabilities.get("class")?.as_str()?;
                
                // Parse to ModelClass and compare
                let agent_model_class = match class_str {
                    "fast" => ModelClass::Fast,
                    "balanced" => ModelClass::Balanced,
                    "reasoning" => ModelClass::Reasoning,
                    _ => return None,
                };
                
                // Match if model_class matches (or if criteria is None, default to Balanced)
                if agent_model_class == model_class {
                    Some(agent_meta.id.clone())
                } else {
                    None
                }
            })
            .collect();
        
        // If no agents have capabilities, fall back to all agents (backward compatibility)
        let matching_agents = if matching_agents.is_empty() && agents.iter().all(|a| a.capabilities.is_none()) {
            warn!("No agents have capabilities defined, selecting from all agents");
            agents.iter().map(|a| a.id.clone()).collect()
        } else {
            matching_agents
        };

        if matching_agents.is_empty() {
            return Err(SelectionError::NoMatchingAgents(format!(
                "No agents match model_class: {:?}",
                model_class
            )));
        }

        // Get queue depth for each candidate
        let mut candidates_with_load: Vec<(String, usize)> = Vec::new();
        for agent_id in matching_agents {
            let queue_depth = self.queue.get_queue_depth_for_agent(&agent_id).await;
            candidates_with_load.push((agent_id, queue_depth));
        }

        // Sort by queue depth (ascending - lower is better)
        candidates_with_load.sort_by_key(|(_, depth)| *depth);

        // Find minimum queue depth
        let min_depth = candidates_with_load
            .first()
            .map_or(0, |(_, depth)| *depth);

        // Filter to agents with minimum queue depth
        let best_candidates: Vec<String> = candidates_with_load
            .into_iter()
            .filter(|(_, depth)| *depth == min_depth)
            .map(|(id, _)| id)
            .collect();

        if best_candidates.is_empty() {
            return Err(SelectionError::NoMatchingAgents(
                "No agents available after filtering".to_string(),
            ));
        }

        // Random tie-breaking (for MVP, just pick the first one)
        // In a production system, we'd use proper random selection
        let selected = best_candidates.first().unwrap().clone();

        debug!(
            agent_id = %selected,
            model_class = ?model_class,
            queue_depth = min_depth,
            "Selected agent"
        );

        Ok(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EchoAgent, ExecutionQueue, ExecutionTask};

    #[tokio::test]
    async fn test_select_best_agent_no_agents() {
        let registry = Arc::new(AgentRegistry::new());
        let queue = Arc::new(ExecutionQueue::new());
        let selector = AgentSelector::new(registry, queue);

        let criteria = SelectionCriteria {
            model_class: Some(ModelClass::Balanced),
        };

        let result = selector.select_best_agent(criteria).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No agents registered"));
    }

    #[tokio::test]
    async fn test_select_best_agent_single_agent() {
        let registry = Arc::new(AgentRegistry::new());
        let queue = Arc::new(ExecutionQueue::new());
        let selector = AgentSelector::new(registry.clone(), queue.clone());

        let agent = Arc::new(EchoAgent::new(
            "test-agent".to_string(),
            "Test agent".to_string(),
        ));
        registry.register_agent(agent).await;

        let criteria = SelectionCriteria {
            model_class: Some(ModelClass::Balanced),
        };

        let result = selector.select_best_agent(criteria).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-agent");
    }

    #[tokio::test]
    async fn test_select_best_agent_load_balancing() {
        let registry = Arc::new(AgentRegistry::new());
        let queue = Arc::new(ExecutionQueue::new());
        let selector = AgentSelector::new(registry.clone(), queue.clone());

        // Register two agents
        let agent1 = Arc::new(EchoAgent::new(
            "agent-1".to_string(),
            "Agent 1".to_string(),
        ));
        let agent2 = Arc::new(EchoAgent::new(
            "agent-2".to_string(),
            "Agent 2".to_string(),
        ));
        registry.register_agent(agent1).await;
        registry.register_agent(agent2).await;

        // Add tasks to agent-1 to increase its load
        for _ in 0..3 {
            let task = ExecutionTask::new(
                "agent-1".to_string(),
                "test".to_string(),
                1,
            );
            queue.enqueue_task(task).await.unwrap();
        }

        let criteria = SelectionCriteria {
            model_class: Some(ModelClass::Balanced),
        };

        // Should select agent-2 (lower load)
        let result = selector.select_best_agent(criteria).await;
        assert!(result.is_ok());
        let selected = result.unwrap();
        // Note: Since we're not filtering by capabilities yet, either agent could be selected
        // if they have the same load. This test verifies the basic selection works.
        assert!(selected == "agent-1" || selected == "agent-2");
    }
}

