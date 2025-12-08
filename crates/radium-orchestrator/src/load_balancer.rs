//! Load balancer for intelligent agent selection.
//!
//! This module provides functionality for distributing tasks across available agents
//! based on their current load and capacity limits.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Load balancer for agent task distribution.
pub struct LoadBalancer {
    /// Current load per agent (number of active tasks).
    agent_loads: Arc<Mutex<HashMap<String, usize>>>,
    /// Maximum concurrent tasks per agent.
    max_concurrent_per_agent: usize,
}

impl std::fmt::Debug for LoadBalancer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadBalancer")
            .field("max_concurrent_per_agent", &self.max_concurrent_per_agent)
            .finish_non_exhaustive()
    }
}

impl LoadBalancer {
    /// Creates a new load balancer.
    ///
    /// # Arguments
    /// * `max_concurrent_per_agent` - Maximum number of concurrent tasks per agent
    #[must_use]
    pub fn new(max_concurrent_per_agent: usize) -> Self {
        Self {
            agent_loads: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent_per_agent,
        }
    }

    /// Gets an available agent with the lowest current load.
    ///
    /// # Arguments
    /// * `available_agents` - List of available agent IDs
    ///
    /// # Returns
    /// Returns `Some(agent_id)` if an available agent is found, `None` otherwise.
    pub async fn get_available_agent(&self, available_agents: &[String]) -> Option<String> {
        if available_agents.is_empty() {
            return None;
        }

        let loads = self.agent_loads.lock().await;

        // Find agent with lowest load that hasn't exceeded capacity
        let mut best_agent: Option<(&String, usize)> = None;

        for agent_id in available_agents {
            let current_load = loads.get(agent_id).copied().unwrap_or(0);

            // Skip agents that have reached capacity
            if current_load >= self.max_concurrent_per_agent {
                continue;
            }

            // Select agent with lowest load
            match best_agent {
                None => best_agent = Some((agent_id, current_load)),
                Some((_, best_load)) if current_load < best_load => {
                    best_agent = Some((agent_id, current_load));
                }
                _ => {}
            }
        }

        best_agent.map(|(agent_id, _)| agent_id.clone())
    }

    /// Increments the load for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    pub async fn increment_load(&self, agent_id: &str) {
        let mut loads = self.agent_loads.lock().await;
        let current_load = loads.entry(agent_id.to_string()).or_insert(0);
        *current_load += 1;
        debug!(
            agent_id = %agent_id,
            load = *current_load,
            "Incremented agent load"
        );
    }

    /// Decrements the load for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    pub async fn decrement_load(&self, agent_id: &str) {
        let mut loads = self.agent_loads.lock().await;
        if let Some(load) = loads.get_mut(agent_id) {
            if *load > 0 {
                *load -= 1;
                debug!(
                    agent_id = %agent_id,
                    load = *load,
                    "Decremented agent load"
                );
            } else {
                warn!(
                    agent_id = %agent_id,
                    "Attempted to decrement load below zero"
                );
            }
        } else {
            warn!(
                agent_id = %agent_id,
                "Attempted to decrement load for unknown agent"
            );
        }
    }

    /// Gets the current utilization for all agents.
    ///
    /// # Returns
    /// Returns a map of agent ID to utilization (0.0-1.0).
    pub async fn get_agent_utilization(&self) -> HashMap<String, f32> {
        let loads = self.agent_loads.lock().await;
        loads
            .iter()
            .map(|(agent_id, load)| {
                let utilization = (*load as f32) / (self.max_concurrent_per_agent as f32);
                (agent_id.clone(), utilization.min(1.0))
            })
            .collect()
    }

    /// Gets the current load for a specific agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    ///
    /// # Returns
    /// Returns the current load for the agent.
    pub async fn get_agent_load(&self, agent_id: &str) -> usize {
        let loads = self.agent_loads.lock().await;
        loads.get(agent_id).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_balancer_new() {
        let balancer = LoadBalancer::new(5);
        assert_eq!(balancer.max_concurrent_per_agent, 5);
    }

    #[tokio::test]
    async fn test_load_balancer_get_available_agent() {
        let balancer = LoadBalancer::new(2);
        let agents = vec!["agent-1".to_string(), "agent-2".to_string()];

        // Initially, both agents should be available
        let selected = balancer.get_available_agent(&agents).await;
        assert!(selected.is_some());
        assert!(agents.contains(&selected.unwrap()));
    }

    #[tokio::test]
    async fn test_load_balancer_increment_decrement() {
        let balancer = LoadBalancer::new(5);

        // Increment load
        balancer.increment_load("agent-1").await;
        assert_eq!(balancer.get_agent_load("agent-1").await, 1);

        // Increment again
        balancer.increment_load("agent-1").await;
        assert_eq!(balancer.get_agent_load("agent-1").await, 2);

        // Decrement
        balancer.decrement_load("agent-1").await;
        assert_eq!(balancer.get_agent_load("agent-1").await, 1);

        // Decrement again
        balancer.decrement_load("agent-1").await;
        assert_eq!(balancer.get_agent_load("agent-1").await, 0);
    }

    #[tokio::test]
    async fn test_load_balancer_capacity_limits() {
        let balancer = LoadBalancer::new(2);
        let agents = vec!["agent-1".to_string()];

        // Fill agent to capacity
        balancer.increment_load("agent-1").await;
        balancer.increment_load("agent-1").await;

        // Agent should be at capacity
        assert_eq!(balancer.get_agent_load("agent-1").await, 2);

        // Agent should not be available
        let selected = balancer.get_available_agent(&agents).await;
        assert!(selected.is_none());

        // Free up one slot
        balancer.decrement_load("agent-1").await;

        // Agent should be available again
        let selected = balancer.get_available_agent(&agents).await;
        assert_eq!(selected, Some("agent-1".to_string()));
    }

    #[tokio::test]
    async fn test_load_balancer_utilization() {
        let balancer = LoadBalancer::new(10);

        // No load initially
        let utilization = balancer.get_agent_utilization().await;
        assert!(utilization.is_empty());

        // Add some load
        balancer.increment_load("agent-1").await;
        balancer.increment_load("agent-1").await;
        balancer.increment_load("agent-1").await;

        let utilization = balancer.get_agent_utilization().await;
        assert_eq!(utilization.get("agent-1"), Some(&0.3)); // 3/10 = 0.3

        // Fill to capacity
        for _ in 0..7 {
            balancer.increment_load("agent-1").await;
        }

        let utilization = balancer.get_agent_utilization().await;
        assert_eq!(utilization.get("agent-1"), Some(&1.0)); // 10/10 = 1.0
    }

    #[tokio::test]
    async fn test_load_balancer_selects_lowest_load() {
        let balancer = LoadBalancer::new(5);
        let agents = vec!["agent-1".to_string(), "agent-2".to_string(), "agent-3".to_string()];

        // Add load to agent-1
        balancer.increment_load("agent-1").await;
        balancer.increment_load("agent-1").await;

        // Add load to agent-2
        balancer.increment_load("agent-2").await;

        // agent-3 should be selected (lowest load = 0)
        let selected = balancer.get_available_agent(&agents).await;
        assert_eq!(selected, Some("agent-3".to_string()));

        // Add load to agent-3
        balancer.increment_load("agent-3").await;

        // agent-2 should be selected (lowest load = 1)
        let selected = balancer.get_available_agent(&agents).await;
        assert_eq!(selected, Some("agent-2".to_string()));
    }

    #[tokio::test]
    async fn test_load_balancer_empty_agents() {
        let balancer = LoadBalancer::new(5);
        let agents: Vec<String> = vec![];

        let selected = balancer.get_available_agent(&agents).await;
        assert!(selected.is_none());
    }
}

