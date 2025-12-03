//! Agent lifecycle management.
//!
//! This module provides functionality for managing agent execution states and lifecycle.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error};

/// Agent execution state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent is idle and ready to execute.
    Idle,
    /// Agent is currently running.
    Running,
    /// Agent execution is paused.
    Paused,
    /// Agent execution has been stopped.
    Stopped,
    /// Agent execution encountered an error.
    Error,
}

impl AgentState {
    /// Checks if the agent can transition to the given state.
    ///
    /// # Arguments
    /// * `to` - The target state
    ///
    /// # Returns
    /// Returns `true` if the transition is valid, `false` otherwise.
    #[must_use]
    #[allow(clippy::match_same_arms)] // Each arm represents a distinct state transition rule
    pub fn can_transition_to(&self, to: Self) -> bool {
        match (self, to) {
            // From Idle: can go to Running or Stopped
            (Self::Idle, Self::Running | Self::Stopped) => true,
            // From Running: can go to Paused, Stopped, or Error
            (Self::Running, Self::Paused | Self::Stopped | Self::Error) => true,
            // From Paused: can go to Running or Stopped
            (Self::Paused, Self::Running | Self::Stopped) => true,
            // From Stopped: can go to Idle
            (Self::Stopped, Self::Idle) => true,
            // From Error: can go to Idle or Stopped
            (Self::Error, Self::Idle | Self::Stopped) => true,
            // Same state is always valid
            (a, b) if *a == b => true,
            // All other transitions are invalid
            _ => false,
        }
    }
}

/// Lifecycle manager for agents.
pub struct AgentLifecycle {
    /// Map of agent ID to current state.
    states: Arc<RwLock<HashMap<String, AgentState>>>,
}

impl fmt::Debug for AgentLifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentLifecycle")
            .field("agent_count", &self.states.try_read().map(|s| s.len()).unwrap_or(0))
            .finish_non_exhaustive()
    }
}

impl AgentLifecycle {
    /// Creates a new lifecycle manager.
    #[must_use]
    pub fn new() -> Self {
        Self { states: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Gets the current state of an agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    ///
    /// # Returns
    /// Returns the current state, or `Idle` if the agent is not tracked.
    pub async fn get_state(&self, agent_id: &str) -> AgentState {
        let states = self.states.read().await;
        states.get(agent_id).copied().unwrap_or(AgentState::Idle)
    }

    /// Sets the state of an agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    /// * `new_state` - The new state
    ///
    /// # Returns
    /// Returns `Ok(AgentState)` with the previous state if the transition is valid,
    /// or `Err(AgentState)` with the current state if the transition is invalid.
    pub async fn set_state(
        &self,
        agent_id: &str,
        new_state: AgentState,
    ) -> Result<AgentState, AgentState> {
        let current_state = self.get_state(agent_id).await;

        if !current_state.can_transition_to(new_state) {
            error!(
                agent_id = %agent_id,
                from = ?current_state,
                to = ?new_state,
                "Invalid state transition"
            );
            return Err(current_state);
        }

        debug!(
            agent_id = %agent_id,
            from = ?current_state,
            to = ?new_state,
            "State transition"
        );

        let mut states = self.states.write().await;
        states.insert(agent_id.to_string(), new_state);
        Ok(current_state)
    }

    /// Starts an agent (transitions from Idle to Running).
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    pub async fn start_agent(&self, agent_id: &str) -> Result<(), AgentState> {
        self.set_state(agent_id, AgentState::Running).await.map(|_| ())
    }

    /// Stops an agent (transitions to Stopped).
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    pub async fn stop_agent(&self, agent_id: &str) -> Result<(), AgentState> {
        self.set_state(agent_id, AgentState::Stopped).await.map(|_| ())
    }

    /// Pauses an agent (transitions from Running to Paused).
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    pub async fn pause_agent(&self, agent_id: &str) -> Result<(), AgentState> {
        self.set_state(agent_id, AgentState::Paused).await.map(|_| ())
    }

    /// Resumes an agent (transitions from Paused to Running).
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    ///
    /// # Note
    /// This method only works from Paused state. To start an agent from Idle, use `start_agent`.
    pub async fn resume_agent(&self, agent_id: &str) -> Result<(), AgentState> {
        let current_state = self.get_state(agent_id).await;
        if current_state != AgentState::Paused {
            return Err(current_state);
        }
        self.set_state(agent_id, AgentState::Running).await.map(|_| ())
    }

    /// Marks an agent as having an error (transitions to Error).
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    pub async fn mark_error(&self, agent_id: &str) -> Result<(), AgentState> {
        self.set_state(agent_id, AgentState::Error).await.map(|_| ())
    }

    /// Resets an agent to Idle state (from Stopped or Error).
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    pub async fn reset_agent(&self, agent_id: &str) -> Result<(), AgentState> {
        self.set_state(agent_id, AgentState::Idle).await.map(|_| ())
    }

    /// Removes an agent from lifecycle tracking.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    pub async fn remove_agent(&self, agent_id: &str) {
        let mut states = self.states.write().await;
        states.remove(agent_id);
        debug!(agent_id = %agent_id, "Removed agent from lifecycle tracking");
    }
}

impl Default for AgentLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        // Idle transitions
        assert!(AgentState::Idle.can_transition_to(AgentState::Running));
        assert!(AgentState::Idle.can_transition_to(AgentState::Stopped));
        assert!(!AgentState::Idle.can_transition_to(AgentState::Paused));

        // Running transitions
        assert!(AgentState::Running.can_transition_to(AgentState::Paused));
        assert!(AgentState::Running.can_transition_to(AgentState::Stopped));
        assert!(AgentState::Running.can_transition_to(AgentState::Error));
        assert!(!AgentState::Running.can_transition_to(AgentState::Idle));

        // Paused transitions
        assert!(AgentState::Paused.can_transition_to(AgentState::Running));
        assert!(AgentState::Paused.can_transition_to(AgentState::Stopped));
        assert!(!AgentState::Paused.can_transition_to(AgentState::Idle));

        // Stopped transitions
        assert!(AgentState::Stopped.can_transition_to(AgentState::Idle));
        assert!(!AgentState::Stopped.can_transition_to(AgentState::Running));

        // Error transitions
        assert!(AgentState::Error.can_transition_to(AgentState::Idle));
        assert!(AgentState::Error.can_transition_to(AgentState::Stopped));
        assert!(!AgentState::Error.can_transition_to(AgentState::Running));
    }

    #[tokio::test]
    async fn test_lifecycle_start_stop() {
        let lifecycle = AgentLifecycle::new();
        let agent_id = "test-agent";

        // Start from Idle
        assert_eq!(lifecycle.get_state(agent_id).await, AgentState::Idle);
        assert!(lifecycle.start_agent(agent_id).await.is_ok());
        assert_eq!(lifecycle.get_state(agent_id).await, AgentState::Running);

        // Stop from Running
        assert!(lifecycle.stop_agent(agent_id).await.is_ok());
        assert_eq!(lifecycle.get_state(agent_id).await, AgentState::Stopped);

        // Reset to Idle
        assert!(lifecycle.reset_agent(agent_id).await.is_ok());
        assert_eq!(lifecycle.get_state(agent_id).await, AgentState::Idle);
    }

    #[tokio::test]
    async fn test_lifecycle_pause_resume() {
        let lifecycle = AgentLifecycle::new();
        let agent_id = "test-agent";

        lifecycle.start_agent(agent_id).await.unwrap();
        assert_eq!(lifecycle.get_state(agent_id).await, AgentState::Running);

        // Pause from Running
        assert!(lifecycle.pause_agent(agent_id).await.is_ok());
        assert_eq!(lifecycle.get_state(agent_id).await, AgentState::Paused);

        // Resume from Paused
        assert!(lifecycle.resume_agent(agent_id).await.is_ok());
        assert_eq!(lifecycle.get_state(agent_id).await, AgentState::Running);
    }

    #[tokio::test]
    async fn test_lifecycle_invalid_transitions() {
        let lifecycle = AgentLifecycle::new();
        let agent_id = "test-agent";

        // Can't pause from Idle
        assert_eq!(lifecycle.pause_agent(agent_id).await, Err(AgentState::Idle));

        // Can't resume from Idle
        assert_eq!(lifecycle.resume_agent(agent_id).await, Err(AgentState::Idle));
    }

    #[tokio::test]
    async fn test_lifecycle_error() {
        let lifecycle = AgentLifecycle::new();
        let agent_id = "test-agent";

        lifecycle.start_agent(agent_id).await.unwrap();
        assert!(lifecycle.mark_error(agent_id).await.is_ok());
        assert_eq!(lifecycle.get_state(agent_id).await, AgentState::Error);

        // Can reset from Error
        assert!(lifecycle.reset_agent(agent_id).await.is_ok());
        assert_eq!(lifecycle.get_state(agent_id).await, AgentState::Idle);
    }
}
