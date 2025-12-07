//! Sandbox manager implementation for agent execution.
//!
//! This module provides a SandboxManager implementation that connects
//! AgentConfig sandbox settings to AgentExecutor sandbox operations.

use crate::agents::config::AgentConfig;
use crate::sandbox::{SandboxConfig, SandboxFactory, Sandbox as SandboxTrait, SandboxError};
use radium_orchestrator::SandboxManager as SandboxManagerTrait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Sandbox manager that manages sandbox instances for agents.
pub struct AgentSandboxManager {
    /// Sandbox configurations by agent ID.
    configs: Arc<RwLock<HashMap<String, SandboxConfig>>>,
    /// Active sandbox instances by agent ID.
    active_sandboxes: Arc<RwLock<HashMap<String, Box<dyn SandboxTrait + Send + Sync>>>>,
}

impl AgentSandboxManager {
    /// Creates a new sandbox manager.
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            active_sandboxes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a sandbox configuration for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    /// * `config` - The sandbox configuration (from AgentConfig)
    pub async fn register_config(&self, agent_id: String, config: SandboxConfig) {
        let mut configs = self.configs.write().await;
        configs.insert(agent_id, config);
    }

    /// Registers sandbox configuration from an AgentConfig.
    ///
    /// # Arguments
    /// * `agent_config` - The agent configuration
    pub async fn register_from_agent_config(&self, agent_config: &AgentConfig) {
        if let Some(ref sandbox_config) = agent_config.sandbox {
            self.register_config(agent_config.id.clone(), sandbox_config.clone()).await;
        }
    }

    /// Gets the sandbox configuration for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    ///
    /// # Returns
    /// The sandbox configuration if registered, None otherwise
    pub async fn get_config(&self, agent_id: &str) -> Option<SandboxConfig> {
        let configs = self.configs.read().await;
        configs.get(agent_id).cloned()
    }
}

impl Default for AgentSandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SandboxManagerTrait for AgentSandboxManager {
    async fn initialize_sandbox(&self, agent_id: &str) -> Result<(), String> {
        let sandbox_config = {
            let configs = self.configs.read().await;
            configs.get(agent_id).cloned()
        };

        if let Some(config) = sandbox_config {
            match SandboxFactory::create(&config) {
                Ok(mut sandbox) => {
                    // Initialize the sandbox
                    if let Err(e) = sandbox.initialize().await {
                        // If sandbox is not available, fall back to NoSandbox with warning
                        if matches!(e, SandboxError::NotAvailable(_)) {
                            warn!(
                                agent_id = %agent_id,
                                error = %e,
                                "Sandbox not available, falling back to NoSandbox"
                            );
                            // Create NoSandbox as fallback
                            let no_sandbox_config = SandboxConfig::default();
                            if let Ok(mut no_sandbox) = SandboxFactory::create(&no_sandbox_config) {
                                if no_sandbox.initialize().await.is_ok() {
                                    let mut active = self.active_sandboxes.write().await;
                                    active.insert(agent_id.to_string(), no_sandbox);
                                }
                            }
                        } else {
                            return Err(format!("Failed to initialize sandbox: {}", e));
                        }
                    } else {
                        // Successfully initialized, store it
                        let mut active = self.active_sandboxes.write().await;
                        active.insert(agent_id.to_string(), sandbox);
                        debug!(agent_id = %agent_id, "Sandbox initialized for agent");
                    }
                }
                Err(e) => {
                    // If sandbox creation fails (e.g., NotAvailable), fall back to NoSandbox
                    if matches!(e, SandboxError::NotAvailable(_)) {
                        warn!(
                            agent_id = %agent_id,
                            error = %e,
                            "Sandbox not available, falling back to NoSandbox"
                        );
                        let no_sandbox_config = SandboxConfig::default();
                        if let Ok(mut no_sandbox) = SandboxFactory::create(&no_sandbox_config) {
                            if no_sandbox.initialize().await.is_ok() {
                                let mut active = self.active_sandboxes.write().await;
                                active.insert(agent_id.to_string(), no_sandbox);
                            }
                        }
                    } else {
                        return Err(format!("Failed to create sandbox: {}", e));
                    }
                }
            }
        }

        Ok(())
    }

    async fn cleanup_sandbox(&self, agent_id: &str) {
        let mut sandboxes = self.active_sandboxes.write().await;
        if let Some(mut sandbox) = sandboxes.remove(agent_id) {
            if let Err(e) = sandbox.cleanup().await {
                warn!(
                    agent_id = %agent_id,
                    error = %e,
                    "Failed to cleanup sandbox"
                );
            } else {
                debug!(agent_id = %agent_id, "Sandbox cleaned up for agent");
            }
        }
    }

    fn get_active_sandbox(&self, _agent_id: &str) -> Option<Box<dyn std::any::Any + Send + Sync>> {
        // Note: This method returns the sandbox as Any, which can be downcast if needed
        // For now, we return None as sandboxes are managed internally
        // The sandbox is accessed through the active_sandboxes map during command execution
        // This method exists to satisfy the trait, but actual sandbox access happens
        // through a different mechanism (e.g., via command execution context)
        None
    }
}

