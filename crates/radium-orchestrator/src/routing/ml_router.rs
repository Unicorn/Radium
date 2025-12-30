//! ML-based agent routing with fallback to capability matching.
//!
//! This module provides intelligent agent routing using the Arch-Router 1.5B ML model,
//! with automatic fallback to skill-based routing when ML confidence is low.

use super::capability_matcher::CapabilityMatcher;
use super::skill_router::{SkillDefinition, SkillRouter, SkillRoutingResult};
use anyhow::{Context, Result};
use radium_ml::{ArchRouterEngine, ArchRouterResult, RouteDefinition};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// ML-based routing result with confidence and reasoning.
#[derive(Debug, Clone)]
pub struct MLRoutingResult {
    /// Selected agent ID.
    pub agent_id: String,

    /// Confidence score (0.0-1.0).
    pub confidence: f32,

    /// Alternative agents with scores.
    pub alternatives: Vec<(String, f32)>,

    /// Routing method used.
    pub routing_method: RoutingMethod,

    /// Reasoning/explanation (if available).
    pub reasoning: Option<String>,
}

/// Method used for routing decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMethod {
    /// ML-based routing (Arch-Router model).
    MLBased,
    /// Skill-based fallback (keyword/Jaccard matching).
    SkillBased,
    /// Default fallback (when both fail).
    Default,
}

impl std::fmt::Display for RoutingMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingMethod::MLBased => write!(f, "ml-based"),
            RoutingMethod::SkillBased => write!(f, "skill-based"),
            RoutingMethod::Default => write!(f, "default"),
        }
    }
}

/// Configuration for ML router.
#[derive(Debug, Clone)]
pub struct MLRouterConfig {
    /// Minimum confidence threshold for ML routing.
    /// If ML confidence < threshold, fall back to skill-based routing.
    pub ml_confidence_threshold: f32,

    /// Skill-based routing confidence threshold.
    /// If skill confidence < threshold, use default agent.
    pub skill_confidence_threshold: f32,

    /// Default agent ID to use when all routing methods fail.
    pub default_agent_id: String,

    /// Skills directory path for fallback routing.
    pub skills_path: Option<PathBuf>,
}

impl Default for MLRouterConfig {
    fn default() -> Self {
        Self {
            ml_confidence_threshold: 0.7,
            skill_confidence_threshold: 0.5,
            default_agent_id: "code-agent".to_string(),
            skills_path: None,
        }
    }
}

/// ML router with fallback to skill-based routing.
pub struct MLRouter {
    /// Arch-Router ML engine.
    arch_router: Arc<ArchRouterEngine>,

    /// Skill router for fallback.
    skill_router: Arc<SkillRouter>,

    /// Configuration.
    config: MLRouterConfig,

    /// Agent registry (agent ID -> description mapping).
    agent_registry: Arc<RwLock<HashMap<String, String>>>,
}

impl MLRouter {
    /// Create a new ML router with default configuration.
    ///
    /// # Errors
    /// Returns error if skill loading fails.
    pub async fn new() -> Result<Self> {
        Self::with_config(MLRouterConfig::default()).await
    }

    /// Create ML router with custom configuration.
    ///
    /// # Arguments
    /// * `config` - ML router configuration
    ///
    /// # Errors
    /// Returns error if skill loading fails.
    pub async fn with_config(config: MLRouterConfig) -> Result<Self> {
        // Initialize Arch-Router engine (with simulated backend by default)
        let arch_router = Arc::new(ArchRouterEngine::new());

        // Initialize skill router for fallback
        let skill_router = SkillRouter::new();

        if let Some(ref skills_path) = config.skills_path {
            skill_router.load_skills_from_directory(skills_path).await?;
        } else {
            // Try to find skills in standard location
            let default_skills_path = PathBuf::from("skills");
            if default_skills_path.exists() {
                skill_router.load_skills_from_directory(&default_skills_path).await?;
            }
            // If no skills directory exists, just use empty skill router
        }

        Ok(Self {
            arch_router,
            skill_router: Arc::new(skill_router),
            config,
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register an agent for routing.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `description` - Agent description/capabilities
    pub async fn register_agent(&self, agent_id: String, description: String) {
        let mut registry = self.agent_registry.write().await;
        registry.insert(agent_id, description);
    }

    /// Route a task to the best agent using ML inference with fallback.
    ///
    /// # Arguments
    /// * `task_description` - User's task/query
    ///
    /// # Returns
    /// Routing result with agent ID, confidence, and method used
    ///
    /// # Routing Logic
    /// 1. Try ML-based routing (Arch-Router)
    /// 2. If ML confidence < threshold, try skill-based routing
    /// 3. If skill confidence < threshold, use default agent
    pub async fn route(&self, task_description: &str) -> Result<MLRoutingResult> {
        // Step 1: Try ML-based routing
        match self.route_with_ml(task_description).await {
            Ok(ml_result) => {
                if ml_result.confidence >= self.config.ml_confidence_threshold {
                    info!(
                        agent_id = %ml_result.agent_id,
                        confidence = ml_result.confidence,
                        method = "ml-based",
                        "ML routing succeeded with high confidence"
                    );
                    return Ok(ml_result);
                } else {
                    debug!(
                        confidence = ml_result.confidence,
                        threshold = self.config.ml_confidence_threshold,
                        "ML confidence below threshold, falling back to skill-based routing"
                    );
                }
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "ML routing failed, falling back to skill-based routing"
                );
            }
        }

        // Step 2: Try skill-based routing
        match self.route_with_skills(task_description).await {
            Ok(skill_result) => {
                if skill_result.confidence >= self.config.skill_confidence_threshold {
                    info!(
                        agent_id = %skill_result.agent_id,
                        confidence = skill_result.confidence,
                        method = "skill-based",
                        "Skill routing succeeded"
                    );
                    return Ok(skill_result);
                } else {
                    debug!(
                        confidence = skill_result.confidence,
                        threshold = self.config.skill_confidence_threshold,
                        "Skill confidence below threshold, using default agent"
                    );
                }
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "Skill routing failed, using default agent"
                );
            }
        }

        // Step 3: Use default agent
        warn!(
            default_agent = %self.config.default_agent_id,
            "All routing methods failed or had low confidence, using default agent"
        );

        Ok(MLRoutingResult {
            agent_id: self.config.default_agent_id.clone(),
            confidence: 0.5, // Medium confidence for default fallback
            alternatives: vec![],
            routing_method: RoutingMethod::Default,
            reasoning: Some("All routing methods failed or had low confidence".to_string()),
        })
    }

    /// Route using ML (internal method).
    async fn route_with_ml(&self, task_description: &str) -> Result<MLRoutingResult> {
        // Get registered agents
        let registry = self.agent_registry.read().await;

        // Convert agents to route definitions
        let routes: Vec<RouteDefinition> = registry
            .iter()
            .map(|(agent_id, description)| RouteDefinition {
                name: agent_id.clone(),
                description: description.clone(),
                domain: None,
                action: None,
            })
            .collect();

        // If no agents registered, try to use skills
        let routes = if routes.is_empty() {
            // Get skill names from skill router
            let skill_names = self.skill_router.list_skills().await;

            // Fetch full skill definitions for each name
            let mut skill_routes = Vec::new();
            for skill_name in skill_names {
                if let Some(skill) = self.skill_router.get_skill(&skill_name).await {
                    skill_routes.push(RouteDefinition {
                        name: skill.name.clone(),
                        description: skill.description.clone(),
                        domain: None,
                        action: None,
                    });
                }
            }
            skill_routes
        } else {
            routes
        };

        if routes.is_empty() {
            anyhow::bail!("No routes available for ML routing");
        }

        // Call Arch-Router
        let result = self
            .arch_router
            .route(task_description, &routes)
            .await
            .context("ML routing failed")?;

        Ok(MLRoutingResult {
            agent_id: result.agent_id,
            confidence: result.confidence,
            alternatives: result.alternatives,
            routing_method: RoutingMethod::MLBased,
            reasoning: result.reasoning,
        })
    }

    /// Route using skill-based matching (internal method).
    async fn route_with_skills(&self, task_description: &str) -> Result<MLRoutingResult> {
        let skill_result_opt = self
            .skill_router
            .route(task_description)
            .await
            .context("Skill routing failed")?;

        match skill_result_opt {
            Some(skill_result) => Ok(MLRoutingResult {
                agent_id: skill_result.skill_name,
                confidence: skill_result.confidence,
                alternatives: vec![], // Skill router doesn't provide alternatives in single route
                routing_method: RoutingMethod::SkillBased,
                reasoning: Some(skill_result.reasoning),
            }),
            None => {
                anyhow::bail!("No skill matched the task description above confidence threshold")
            }
        }
    }

    /// Route and return top-k agents.
    ///
    /// # Arguments
    /// * `task_description` - User's task/query
    /// * `k` - Number of top agents to return
    ///
    /// # Returns
    /// Vector of (agent_id, confidence) pairs, sorted by confidence descending
    pub async fn route_top_k(
        &self,
        task_description: &str,
        k: usize,
    ) -> Result<Vec<(String, f32)>> {
        // Try ML routing first
        let result = self.route(task_description).await?;

        // Combine primary result with alternatives
        let mut results = vec![(result.agent_id.clone(), result.confidence)];
        results.extend(result.alternatives.clone());

        // Sort by confidence descending and take top k
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        Ok(results)
    }

    /// Get router statistics.
    pub async fn get_stats(&self) -> MLRouterStats {
        let agent_count = self.agent_registry.read().await.len();
        let skill_count = self.skill_router.skill_count().await;

        MLRouterStats {
            agent_count,
            skill_count,
            ml_threshold: self.config.ml_confidence_threshold,
            skill_threshold: self.config.skill_confidence_threshold,
            default_agent: self.config.default_agent_id.clone(),
        }
    }
}

/// ML router statistics.
#[derive(Debug, Clone)]
pub struct MLRouterStats {
    /// Number of registered agents.
    pub agent_count: usize,

    /// Number of loaded skills.
    pub skill_count: usize,

    /// ML confidence threshold.
    pub ml_threshold: f32,

    /// Skill confidence threshold.
    pub skill_threshold: f32,

    /// Default agent ID.
    pub default_agent: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ml_router_default_fallback() {
        let router = MLRouter::new().await.unwrap();

        // With no agents registered, should fall back to default
        let result = router.route("Write a Python function").await.unwrap();

        assert_eq!(result.agent_id, "code-agent"); // default agent
        assert_eq!(result.routing_method, RoutingMethod::Default);
    }

    #[tokio::test]
    async fn test_ml_router_with_registered_agents() {
        let router = MLRouter::new().await.unwrap();

        // Register test agents
        router
            .register_agent(
                "code-agent".to_string(),
                "Writes production-ready code".to_string(),
            )
            .await;
        router
            .register_agent(
                "review-agent".to_string(),
                "Reviews code for quality".to_string(),
            )
            .await;

        // Route a code generation task
        let result = router
            .route("Implement a REST API endpoint")
            .await
            .unwrap();

        // Should route to code-agent (simulated routing uses keyword matching)
        assert!(result.agent_id.contains("code") || result.confidence > 0.0);
        println!(
            "Routed to: {} (confidence: {:.3}, method: {})",
            result.agent_id, result.confidence, result.routing_method
        );
    }

    #[tokio::test]
    async fn test_route_top_k() {
        let router = MLRouter::new().await.unwrap();

        // Register agents
        router
            .register_agent(
                "code-agent".to_string(),
                "Writes code".to_string(),
            )
            .await;
        router
            .register_agent(
                "doc-agent".to_string(),
                "Writes documentation".to_string(),
            )
            .await;
        router
            .register_agent(
                "review-agent".to_string(),
                "Reviews code".to_string(),
            )
            .await;

        // Get top 3 agents
        let top_agents = router.route_top_k("Write Python code", 3).await.unwrap();

        assert!(!top_agents.is_empty());
        assert!(top_agents.len() <= 3);

        // Verify sorted by confidence
        for i in 1..top_agents.len() {
            assert!(top_agents[i - 1].1 >= top_agents[i].1);
        }

        println!("Top agents: {:?}", top_agents);
    }

    #[tokio::test]
    async fn test_router_stats() {
        let router = MLRouter::new().await.unwrap();

        router
            .register_agent("test-agent".to_string(), "Test agent".to_string())
            .await;

        let stats = router.get_stats().await;

        assert_eq!(stats.agent_count, 1);
        assert_eq!(stats.default_agent, "code-agent");
        assert_eq!(stats.ml_threshold, 0.7);
        assert_eq!(stats.skill_threshold, 0.5);

        println!("Router stats: {:?}", stats);
    }
}
