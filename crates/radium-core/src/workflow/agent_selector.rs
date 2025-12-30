//! Intelligent agent selection based on task type and keywords.
//!
//! This module provides functionality to automatically select the most appropriate
//! agent for executing a task based on task description keywords and explicit agent type hints.

use crate::context::braingrid_client::BraingridTask;
use radium_orchestrator::{AgentRegistry, SkillRouter};
use std::sync::Arc;

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
pub struct AgentSelector {
    /// Agent registry for validation.
    registry: Arc<AgentRegistry>,

    /// Optional skill router for intelligent agent selection (REQ-245).
    /// When present, skill routing is used instead of keyword matching.
    skill_router: Option<Arc<SkillRouter>>,
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
        }
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
    /// The selected agent ID, or error if agent not found
    pub async fn select_agent(&self, task: &BraingridTask) -> Result<String, AgentSelectionError> {
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

        // Log routing metrics
        tracing::info!(
            routing_method = %routing_method,
            routing_confidence = ?confidence,
            routing_latency_ns = routing_latency_ns,
            selected_agent = %agent_id,
            "Agent routing decision"
        );

        // Validate agent exists
        self.validate_agent(&agent_id).await?;

        Ok(agent_id.to_string())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::braingrid_client::TaskStatus;
    use radium_orchestrator::{Agent, AgentInfo, SkillRoutingResult};
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
        fn info(&self) -> AgentInfo {
            AgentInfo {
                id: self.id.clone(),
                name: format!("Mock {}", self.id),
                description: "Mock agent for testing".to_string(),
                capabilities: vec![],
            }
        }

        async fn execute(
            &self,
            _context: radium_orchestrator::OrchestrationContext,
        ) -> Result<radium_orchestrator::AgentResult, radium_orchestrator::AgentError> {
            Ok(radium_orchestrator::AgentResult {
                success: true,
                output: "Mock execution".to_string(),
                metadata: std::collections::HashMap::new(),
            })
        }
    }

    struct MockAgentRegistry {
        agents: Vec<AgentInfo>,
    }

    impl MockAgentRegistry {
        fn new(agent_ids: Vec<&str>) -> Arc<AgentRegistry> {
            let agents: Vec<AgentInfo> = agent_ids
                .iter()
                .map(|id| AgentInfo {
                    id: id.to_string(),
                    name: format!("Mock {}", id),
                    description: "Mock agent".to_string(),
                    capabilities: vec![],
                })
                .collect();

            let registry = AgentRegistry::new();
            for agent_id in agent_ids {
                let agent = MockAgent {
                    id: agent_id.to_string(),
                };
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        registry.register(Arc::new(agent)).await;
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
        assert_eq!(result.unwrap(), "code-agent");
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
        assert_eq!(result.unwrap(), "review-agent");
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

