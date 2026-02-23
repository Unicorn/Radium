//! Intelligent agent selection based on task type and keywords.
//!
//! This module provides functionality to automatically select the most appropriate
//! agent for executing a task based on task description keywords and explicit agent type hints.

use crate::context::braingrid_client::BraingridTask;
use radium_orchestrator::{AgentRegistry, SkillRouter};
use radium_orchestrator::routing::{FeedbackRating, RoutingFeedbackRecord, RoutingPreferences};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Routing decision metadata for feedback collection (Phase 2 - REQ-246).
#[derive(Debug, Clone)]
pub struct RoutingDecisionMetadata {
    /// Selected agent ID.
    pub agent_id: String,
    /// Routing method used ("skill", "keyword").
    pub routing_method: String,
    /// Routing confidence score (0.0-1.0), if available.
    pub confidence: Option<f32>,
    /// Task description that was routed.
    pub task_description: String,
}

/// Errors that can occur during agent selection.
#[derive(Debug, thiserror::Error)]
pub enum AgentSelectionError {
    /// Agent not found in registry.
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// No agents available in registry.
    #[error("No agents available in registry")]
    NoAgentsAvailable,

    /// Skill routing error.
    #[error("Skill routing error: {0}")]
    SkillRoutingError(String),
}

/// Agent selector for intelligent task-to-agent mapping.
///
/// Maps task keywords to agent types and validates agent availability.
/// Supports both keyword-based routing (legacy) and skill-based routing (REQ-245).
/// With Phase 2 (REQ-246), supports per-project routing preferences for adaptive learning.
pub struct AgentSelector {
    /// Agent registry for validation.
    registry: Arc<AgentRegistry>,

    /// Optional skill router for intelligent agent selection (REQ-245).
    /// When present, skill routing is used instead of keyword matching.
    skill_router: Option<Arc<SkillRouter>>,

    /// Optional routing preferences for adaptive routing (Phase 2 - REQ-246).
    /// Loaded from per-project `.radium/routing_preferences.json`.
    routing_preferences: Option<Arc<Mutex<RoutingPreferences>>>,
}

impl AgentSelector {
    /// Creates a new agent selector with keyword-based routing (legacy).
    ///
    /// # Arguments
    /// * `registry` - The agent registry to use for validation
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            registry,
            skill_router: None,
            routing_preferences: None,
        }
    }

    /// Creates a new agent selector with skill-based routing enabled (REQ-245).
    ///
    /// # Arguments
    /// * `registry` - The agent registry to use for validation
    /// * `skill_router` - The skill router for intelligent agent selection
    pub fn with_skill_router(registry: Arc<AgentRegistry>, skill_router: Arc<SkillRouter>) -> Self {
        Self {
            registry,
            skill_router: Some(skill_router),
            routing_preferences: None,
        }
    }

    /// Loads routing preferences from workspace root (Phase 2 - REQ-246).
    ///
    /// # Arguments
    /// * `workspace_root` - Path to the workspace root directory
    ///
    /// # Returns
    /// The AgentSelector instance with preferences loaded (if available)
    pub fn with_preferences(mut self, workspace_root: &PathBuf) -> Self {
        match RoutingPreferences::load(workspace_root) {
            Ok(prefs) => {
                tracing::info!(
                    workspace = %workspace_root.display(),
                    accuracy = %prefs.accuracy_estimate,
                    feedback_count = prefs.feedback_history.len(),
                    "Loaded routing preferences"
                );
                self.routing_preferences = Some(Arc::new(Mutex::new(prefs)));
            }
            Err(e) => {
                tracing::debug!(
                    workspace = %workspace_root.display(),
                    error = %e,
                    "No existing routing preferences found, starting fresh"
                );
                // Create new preferences for this workspace
                let prefs = RoutingPreferences::new(workspace_root.clone());
                self.routing_preferences = Some(Arc::new(Mutex::new(prefs)));
            }
        }
        self
    }

    /// Selects the appropriate agent for a task.
    ///
    /// Selection priority:
    /// 1. Skill-based routing (if enabled via REQ-245) with fallback to keywords
    /// 2. Keyword matching in task title and description (legacy)
    /// 3. Default to code-agent
    ///
    /// # Arguments
    /// * `task` - The Braingrid task to select an agent for
    ///
    /// # Returns
    /// Routing decision metadata including agent ID, method, confidence, and task description
    pub async fn select_agent(&self, task: &BraingridTask) -> Result<RoutingDecisionMetadata, AgentSelectionError> {
        use std::time::Instant;

        // Extract task text
        let text = format!(
            "{} {}",
            task.title,
            task.description.as_ref().unwrap_or(&String::new())
        );

        let start = Instant::now();

        // Try skill-based routing first (REQ-245)
        let (agent_id, routing_method, confidence) = if let Some(ref skill_router) = self.skill_router {
            match skill_router.route(&text).await {
                Ok(Some(result)) => {
                    // Skill routing succeeded
                    tracing::debug!(
                        "Skill routing selected agent '{}' with confidence {}",
                        result.skill_name,
                        result.confidence
                    );
                    (result.skill_name, "skill".to_string(), Some(result.confidence))
                }
                Ok(None) => {
                    // No skills matched above threshold, fall back to keyword matching
                    tracing::debug!("No skills matched above threshold, falling back to keyword matching");
                    (Self::match_keywords(&text.to_lowercase()), "keyword".to_string(), None)
                }
                Err(e) => {
                    // Skill routing failed, fall back to keyword matching
                    tracing::warn!(
                        "Skill routing failed: {}, falling back to keyword matching",
                        e
                    );
                    (Self::match_keywords(&text.to_lowercase()), "keyword".to_string(), None)
                }
            }
        } else {
            // Skill routing not enabled, use keyword matching
            (Self::match_keywords(&text.to_lowercase()), "keyword".to_string(), None)
        };

        let routing_latency_ns = start.elapsed().as_nanos() as u64;

        // Apply routing preferences (Phase 2 - REQ-246)
        let (final_agent_id, adjusted_confidence) = if let Some(ref prefs) = self.routing_preferences {
            let prefs_guard = prefs.lock().unwrap();

            // Check if agent is blocked
            if prefs_guard.blocked_agents.contains(&agent_id) {
                tracing::warn!(
                    blocked_agent = %agent_id,
                    "Agent is blocked by user preferences, using fallback"
                );
                // Fallback to code-agent (or could pick next best alternative)
                ("code-agent".to_string(), confidence)
            } else {
                // Apply confidence override if available
                let adjusted_conf = if let Some(base_conf) = confidence {
                    if let Some(&multiplier) = prefs_guard.skill_confidence_overrides.get(&agent_id) {
                        let adjusted = base_conf * multiplier;
                        tracing::debug!(
                            agent = %agent_id,
                            base_confidence = base_conf,
                            multiplier = multiplier,
                            adjusted_confidence = adjusted,
                            "Applied confidence override from preferences"
                        );
                        Some(adjusted)
                    } else {
                        Some(base_conf)
                    }
                } else {
                    None
                };

                (agent_id, adjusted_conf)
            }
        } else {
            (agent_id, confidence)
        };

        // Log routing metrics
        tracing::info!(
            routing_method = %routing_method,
            routing_confidence = ?adjusted_confidence,
            routing_latency_ns = routing_latency_ns,
            selected_agent = %final_agent_id,
            "Agent routing decision"
        );

        // Validate agent exists
        self.validate_agent(&final_agent_id).await?;

        // Return routing decision metadata (Phase 2 - REQ-246)
        Ok(RoutingDecisionMetadata {
            agent_id: final_agent_id.clone(),
            routing_method,
            confidence: adjusted_confidence,
            task_description: text,
        })
    }

    /// Validates that an agent exists in the registry.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID to validate
    ///
    /// # Returns
    /// Ok(()) if agent exists, error otherwise
    pub async fn validate_agent(&self, agent_id: &str) -> Result<(), AgentSelectionError> {
        let agents = self.registry.list_agents().await;
        
        if agents.is_empty() {
            return Err(AgentSelectionError::NoAgentsAvailable);
        }

        if agents.iter().any(|a| a.id == agent_id) {
            Ok(())
        } else {
            Err(AgentSelectionError::AgentNotFound(agent_id.to_string()))
        }
    }

    /// Matches keywords in text to agent types.
    ///
    /// Keyword mappings:
    /// - ["implement", "code", "build", "create", "develop"] → code-agent
    /// - ["test", "verify", "validate", "check"] → review-agent
    /// - ["document", "write", "readme", "docs"] → doc-agent
    /// - ["design", "architecture", "arch", "structure"] → arch-agent
    /// - Default: code-agent
    ///
    /// # Arguments
    /// * `text` - The text to search for keywords (should be lowercase)
    ///
    /// # Returns
    /// The agent ID that matches the keywords
    fn match_keywords(text: &str) -> String {
        // Code agent keywords
        let code_keywords = ["implement", "code", "build", "create", "develop", "write code"];
        // Review agent keywords
        let review_keywords = ["test", "verify", "validate", "check", "testing"];
        // Doc agent keywords
        let doc_keywords = ["document", "write docs", "readme", "docs", "documentation"];
        // Arch agent keywords
        let arch_keywords = ["design", "architecture", "arch", "structure", "design system"];

        // Check in priority order
        if code_keywords.iter().any(|kw| text.contains(kw)) {
            "code-agent".to_string()
        } else if review_keywords.iter().any(|kw| text.contains(kw)) {
            "review-agent".to_string()
        } else if doc_keywords.iter().any(|kw| text.contains(kw)) {
            "doc-agent".to_string()
        } else if arch_keywords.iter().any(|kw| text.contains(kw)) {
            "arch-agent".to_string()
        } else {
            // Default to code-agent
            "code-agent".to_string()
        }
    }

    /// Gets an agent from the registry by ID.
    ///
    /// # Arguments
    /// * `agent_id` - The ID of the agent to retrieve
    ///
    /// # Returns
    /// The agent if found, None otherwise
    pub async fn get_agent(&self, agent_id: &str) -> Option<std::sync::Arc<dyn radium_orchestrator::Agent + Send + Sync>> {
        self.registry.get_agent(agent_id).await
    }

    /// Records routing feedback to update preferences (Phase 2 - REQ-246).
    ///
    /// # Arguments
    /// * `metadata` - The routing decision metadata
    /// * `execution_success` - Whether task execution succeeded
    /// * `user_rating` - Optional explicit user feedback rating
    ///
    /// This method updates the routing preferences with execution outcomes and
    /// saves them to disk for future routing decisions.
    pub fn record_feedback(
        &self,
        metadata: &RoutingDecisionMetadata,
        execution_success: bool,
        user_rating: Option<FeedbackRating>,
    ) -> Result<(), AgentSelectionError> {
        if let Some(ref prefs) = self.routing_preferences {
            let mut prefs_guard = prefs.lock().unwrap();

            // Determine feedback rating (use explicit rating or infer from execution)
            let rating = user_rating.unwrap_or({
                if execution_success {
                    FeedbackRating::Positive
                } else {
                    FeedbackRating::Negative
                }
            });

            let feedback = RoutingFeedbackRecord {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                task_description: metadata.task_description.clone(),
                selected_agent: metadata.agent_id.clone(),
                confidence: metadata.confidence.unwrap_or(0.0),
                routing_method: metadata.routing_method.clone(),
                user_feedback: rating,
                execution_success,
                retry_count: 0, // Could be tracked in future iterations
                comment: None,  // Could add user comment support in future
            };

            prefs_guard.record_feedback(feedback);

            // Save to disk
            if let Err(e) = prefs_guard.save() {
                tracing::error!(
                    error = %e,
                    workspace = %prefs_guard.workspace_root.display(),
                    "Failed to save routing preferences"
                );
                // Don't fail - just log the error
            } else {
                tracing::debug!(
                    agent = %metadata.agent_id,
                    rating = ?prefs_guard.feedback_history.last().map(|f| &f.user_feedback),
                    accuracy = %prefs_guard.accuracy_estimate,
                    "Recorded routing feedback"
                );
            }
        }

        Ok(())
    }

    /// Gets a clone of the current routing preferences (if loaded).
    pub fn get_preferences(&self) -> Option<RoutingPreferences> {
        self.routing_preferences.as_ref().map(|prefs| {
            prefs.lock().unwrap().clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::braingrid_client::TaskStatus;
    use radium_orchestrator::{Agent, AgentContext, AgentOutput};
    use radium_orchestrator::routing::SkillRoutingResult;
    use radium_abstraction::ModelError;
    use std::sync::Arc;
    use async_trait::async_trait;

    // ===== Test Helpers =====

    fn create_test_task(title: &str, description: Option<&str>) -> BraingridTask {
        BraingridTask {
            id: "test-task".to_string(),
            short_id: Some("TASK-1".to_string()),
            number: "1".to_string(),
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            status: TaskStatus::Planned,
            assigned_to: None,
            dependencies: vec![],
        }
    }

    // ===== Mock Implementations =====

    struct MockAgent {
        id: String,
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn id(&self) -> &str {
            &self.id
        }

        fn description(&self) -> &str {
            "Mock agent for testing"
        }

        async fn execute(
            &self,
            _input: &str,
            _context: AgentContext<'_>,
        ) -> std::result::Result<AgentOutput, ModelError> {
            Ok(AgentOutput::Text("Mock execution".to_string()))
        }
    }

    struct MockAgentRegistry;

    impl MockAgentRegistry {
        fn new(agent_ids: Vec<&str>) -> Arc<AgentRegistry> {
            let registry = AgentRegistry::new();
            for agent_id in agent_ids {
                let agent = MockAgent {
                    id: agent_id.to_string(),
                };
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        registry.register_agent(Arc::new(agent)).await;
                    })
                });
            }

            Arc::new(registry)
        }
    }

    struct MockSkillRouter {
        result: Option<Result<SkillRoutingResult, anyhow::Error>>,
    }

    impl MockSkillRouter {
        fn with_success(skill_name: &str, confidence: f32) -> Arc<radium_orchestrator::SkillRouter> {
            // Create a real SkillRouter for testing
            Arc::new(radium_orchestrator::SkillRouter::new())
        }

        fn with_failure(error_msg: &str) -> Arc<radium_orchestrator::SkillRouter> {
            // Create an empty SkillRouter (will fail to route with no skills)
            Arc::new(radium_orchestrator::SkillRouter::new())
        }
    }

    // ===== Integration Tests (Skill Routing) =====

    #[tokio::test]
    async fn test_select_agent_keyword_routing_only() {
        // Test keyword-based routing without skill router
        let registry = MockAgentRegistry::new(vec!["code-agent", "review-agent", "doc-agent"]);
        let selector = AgentSelector::new(registry);

        let task = create_test_task("Implement authentication", Some("Build user login"));
        let result = selector.select_agent(&task).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().agent_id, "code-agent");
    }

    #[tokio::test]
    async fn test_select_agent_with_empty_skill_router_fallback() {
        // Test fallback to keyword matching when skill router has no skills
        let registry = MockAgentRegistry::new(vec!["code-agent", "review-agent"]);
        let skill_router = Arc::new(radium_orchestrator::SkillRouter::new()); // Empty router
        let selector = AgentSelector::with_skill_router(registry, skill_router);

        let task = create_test_task("Test the API", Some("Write integration tests"));
        let result = selector.select_agent(&task).await;

        // Should fall back to keyword matching and select review-agent
        assert!(result.is_ok());
        assert_eq!(result.unwrap().agent_id, "review-agent");
    }

    #[tokio::test]
    async fn test_validate_agent_success() {
        let registry = MockAgentRegistry::new(vec!["code-agent", "review-agent"]);
        let selector = AgentSelector::new(registry);

        let result = selector.validate_agent("code-agent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_agent_not_found() {
        let registry = MockAgentRegistry::new(vec!["code-agent"]);
        let selector = AgentSelector::new(registry);

        let result = selector.validate_agent("nonexistent-agent").await;
        assert!(result.is_err());
        match result {
            Err(AgentSelectionError::AgentNotFound(agent_id)) => {
                assert_eq!(agent_id, "nonexistent-agent");
            }
            _ => panic!("Expected AgentNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_validate_agent_no_agents_available() {
        let registry = Arc::new(AgentRegistry::new()); // Empty registry
        let selector = AgentSelector::new(registry);

        let result = selector.validate_agent("any-agent").await;
        assert!(result.is_err());
        match result {
            Err(AgentSelectionError::NoAgentsAvailable) => {}
            _ => panic!("Expected NoAgentsAvailable error"),
        }
    }

    #[tokio::test]
    async fn test_select_agent_validates_result() {
        // Test that select_agent validates the selected agent exists
        let registry = MockAgentRegistry::new(vec!["review-agent"]); // Only review-agent
        let selector = AgentSelector::new(registry);

        // This task should select "code-agent" by keywords, but it doesn't exist
        let task = create_test_task("Implement feature", None);
        let result = selector.select_agent(&task).await;

        assert!(result.is_err());
        match result {
            Err(AgentSelectionError::AgentNotFound(agent_id)) => {
                assert_eq!(agent_id, "code-agent");
            }
            _ => panic!("Expected AgentNotFound error"),
        }
    }

    // ===== Unit Tests (Keyword Matching) =====

    #[test]
    fn test_match_keywords_code_agent() {
        let task = create_test_task("Implement user authentication", None);
        let text = format!(
            "{} {}",
            task.title.to_lowercase(),
            task.description
                .as_ref()
                .map(|d| d.to_lowercase())
                .unwrap_or_default()
        );
        let agent = AgentSelector::match_keywords(&text);
        assert_eq!(agent, "code-agent");
    }

    #[test]
    fn test_match_keywords_review_agent() {
        let task = create_test_task("Test API endpoints", None);
        let text = format!(
            "{} {}",
            task.title.to_lowercase(),
            task.description
                .as_ref()
                .map(|d| d.to_lowercase())
                .unwrap_or_default()
        );
        let agent = AgentSelector::match_keywords(&text);
        assert_eq!(agent, "review-agent");
    }

    #[test]
    fn test_match_keywords_doc_agent() {
        let task = create_test_task("Write documentation", None);
        let text = format!(
            "{} {}",
            task.title.to_lowercase(),
            task.description
                .as_ref()
                .map(|d| d.to_lowercase())
                .unwrap_or_default()
        );
        let agent = AgentSelector::match_keywords(&text);
        assert_eq!(agent, "doc-agent");
    }

    #[test]
    fn test_match_keywords_arch_agent() {
        let task = create_test_task("Design system architecture", None);
        let text = format!(
            "{} {}",
            task.title.to_lowercase(),
            task.description
                .as_ref()
                .map(|d| d.to_lowercase())
                .unwrap_or_default()
        );
        let agent = AgentSelector::match_keywords(&text);
        assert_eq!(agent, "arch-agent");
    }

    #[test]
    fn test_match_keywords_default() {
        let task = create_test_task("Update configuration", None);
        let text = format!(
            "{} {}",
            task.title.to_lowercase(),
            task.description
                .as_ref()
                .map(|d| d.to_lowercase())
                .unwrap_or_default()
        );
        let agent = AgentSelector::match_keywords(&text);
        assert_eq!(agent, "code-agent"); // Default
    }

    #[test]
    fn test_match_keywords_priority() {
        // If multiple keywords match, code-agent should win (first in priority)
        let task = create_test_task("Implement and test feature", None);
        let text = format!(
            "{} {}",
            task.title.to_lowercase(),
            task.description
                .as_ref()
                .map(|d| d.to_lowercase())
                .unwrap_or_default()
        );
        let agent = AgentSelector::match_keywords(&text);
        assert_eq!(agent, "code-agent"); // "implement" comes before "test" in priority
    }
}

