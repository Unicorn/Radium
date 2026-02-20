//! Agent discovery utilities for TUI.
//!
//! Provides helper functions to discover and list available agents.

use anyhow::Result;
use radium_core::AgentDiscovery;

/// Get list of available agents.
///
/// Returns a vector of tuples containing `(agent_id, agent_name)`.
pub fn get_available_agents() -> Result<Vec<(String, String)>> {
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()?;
    Ok(agents.into_iter().map(|(id, config)| (id, config.name)).collect())
}
