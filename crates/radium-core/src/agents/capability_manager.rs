//! Dynamic capability adjustment system for runtime permission elevation.

use crate::agents::config::AgentCapabilities;
use crate::policy::constitution::ConstitutionManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// Capability elevation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationRequest {
    /// Agent ID requesting elevation.
    pub agent_id: String,
    /// Requested capabilities.
    pub requested_capabilities: AgentCapabilities,
    /// Justification for the elevation.
    pub justification: String,
    /// Duration in seconds (None = until manually revoked).
    pub duration_secs: Option<u64>,
    /// Timestamp when request was created.
    pub created_at: DateTime<Utc>,
}

/// Active capability elevation.
#[derive(Debug, Clone)]
struct ActiveElevation {
    /// Original request.
    request: ElevationRequest,
    /// Expiration time (None = never expires).
    expires_at: Option<Instant>,
}

/// Capability manager for handling runtime elevation requests.
pub struct CapabilityManager {
    /// Active elevations by agent ID.
    active_elevations: Arc<RwLock<HashMap<String, ActiveElevation>>>,
    /// Constitution manager for session rules.
    constitution_manager: Arc<ConstitutionManager>,
    /// Elevation history.
    elevation_history: Arc<RwLock<Vec<ElevationRecord>>>,
}

/// Elevation history record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationRecord {
    /// Agent ID.
    pub agent_id: String,
    /// Requested capabilities.
    pub requested_capabilities: AgentCapabilities,
    /// Justification.
    pub justification: String,
    /// When elevation was granted.
    pub granted_at: DateTime<Utc>,
    /// When elevation was revoked/expired.
    pub revoked_at: Option<DateTime<Utc>>,
    /// Whether it was manually revoked.
    pub manually_revoked: bool,
}

impl CapabilityManager {
    /// Creates a new capability manager.
    pub fn new(constitution_manager: Arc<ConstitutionManager>) -> Self {
        Self {
            active_elevations: Arc::new(RwLock::new(HashMap::new())),
            constitution_manager,
            elevation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Requests capability elevation for an agent.
    ///
    /// This creates an elevation request that requires user approval.
    /// The actual elevation is granted via `grant_elevation`.
    pub fn create_elevation_request(
        &self,
        agent_id: String,
        requested_capabilities: AgentCapabilities,
        justification: String,
        duration_secs: Option<u64>,
    ) -> ElevationRequest {
        ElevationRequest {
            agent_id,
            requested_capabilities,
            justification,
            duration_secs,
            created_at: Utc::now(),
        }
    }

    /// Grants capability elevation (after user approval).
    pub async fn grant_elevation(&self, request: ElevationRequest) -> Result<(), CapabilityError> {
        let expires_at = request.duration_secs.map(|secs| {
            Instant::now() + Duration::from_secs(secs)
        });

        let elevation = ActiveElevation {
            request: request.clone(),
            expires_at,
        };

        // Store active elevation
        {
            let mut elevations = self.active_elevations.write().await;
            elevations.insert(request.agent_id.clone(), elevation);
        }

        // Add to constitution for session rules
        let session_id = format!("elevation-{}", request.agent_id);
        let constitution_rule = format!(
            "Agent {} has elevated capabilities: {:?}",
            request.agent_id, request.requested_capabilities
        );
        self.constitution_manager.update_constitution(&session_id, constitution_rule);

        // Record in history
        {
            let mut history = self.elevation_history.write().await;
            history.push(ElevationRecord {
                agent_id: request.agent_id.clone(),
                requested_capabilities: request.requested_capabilities.clone(),
                justification: request.justification.clone(),
                granted_at: Utc::now(),
                revoked_at: None,
                manually_revoked: false,
            });
        }

        Ok(())
    }

    /// Revokes capability elevation for an agent.
    pub async fn revoke_elevation(&self, agent_id: &str) -> Result<(), CapabilityError> {
        // Remove from active elevations
        let was_active = {
            let mut elevations = self.active_elevations.write().await;
            elevations.remove(agent_id).is_some()
        };

        if was_active {
            // Remove from constitution
            let session_id = format!("elevation-{}", agent_id);
            self.constitution_manager.remove_constitution(&session_id);

            // Update history
            {
                let mut history = self.elevation_history.write().await;
                if let Some(record) = history.iter_mut().find(|r| {
                    r.agent_id == agent_id && r.revoked_at.is_none()
                }) {
                    record.revoked_at = Some(Utc::now());
                    record.manually_revoked = true;
                }
            }
        }

        Ok(())
    }

    /// Gets active elevation for an agent.
    pub async fn get_active_elevation(&self, agent_id: &str) -> Option<ElevationRequest> {
        let elevations = self.active_elevations.read().await;
        elevations.get(agent_id).map(|e| e.request.clone())
    }

    /// Lists all active elevations.
    pub async fn list_active_elevations(&self) -> Vec<ElevationRequest> {
        let elevations = self.active_elevations.read().await;
        elevations.values().map(|e| e.request.clone()).collect()
    }

    /// Gets elevation history for an agent.
    pub async fn get_elevation_history(&self, agent_id: &str) -> Vec<ElevationRecord> {
        let history = self.elevation_history.read().await;
        history.iter()
            .filter(|r| r.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Gets all elevation history.
    pub async fn get_all_elevation_history(&self) -> Vec<ElevationRecord> {
        let history = self.elevation_history.read().await;
        history.clone()
    }

    /// Checks and expires old elevations.
    pub async fn check_expirations(&self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        {
            let elevations = self.active_elevations.read().await;
            for (agent_id, elevation) in elevations.iter() {
                if let Some(expires_at) = elevation.expires_at {
                    if now >= expires_at {
                        to_remove.push(agent_id.clone());
                    }
                }
            }
        }

        for agent_id in to_remove {
            let _ = self.revoke_elevation(&agent_id).await;
            
            // Update history to mark as expired (not manually revoked)
            {
                let mut history = self.elevation_history.write().await;
                if let Some(record) = history.iter_mut().find(|r| {
                    r.agent_id == agent_id && r.revoked_at.is_none()
                }) {
                    record.revoked_at = Some(Utc::now());
                    record.manually_revoked = false;
                }
            }
        }
    }

    /// Gets effective capabilities for an agent (base + elevation if any).
    pub async fn get_effective_capabilities(
        &self,
        agent_id: &str,
        base_capabilities: &AgentCapabilities,
    ) -> AgentCapabilities {
        if let Some(elevation) = self.get_active_elevation(agent_id).await {
            // Merge: elevation overrides base
            elevation.requested_capabilities
        } else {
            base_capabilities.clone()
        }
    }
}

/// Capability management errors.
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    /// Agent not found.
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Elevation not found.
    #[error("No active elevation for agent: {0}")]
    ElevationNotFound(String),

    /// Invalid duration.
    #[error("Invalid duration: {0}")]
    InvalidDuration(String),
}

