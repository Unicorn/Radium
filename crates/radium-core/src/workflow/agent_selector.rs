//! Intelligent agent selection based on task type and keywords.
//!
//! This module provides functionality to automatically select the most appropriate
//! agent for executing a task based on task description keywords and explicit agent type hints.

use crate::context::braingrid_client::BraingridTask;
use crate::agents::registry::AgentRegistry;
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
}

/// Agent selector for intelligent task-to-agent mapping.
///
/// Maps task keywords to agent types and validates agent availability.
pub struct AgentSelector {
    /// Agent registry for validation.
    registry: Arc<AgentRegistry>,
}

impl AgentSelector {
    /// Creates a new agent selector.
    ///
    /// # Arguments
    /// * `registry` - The agent registry to use for validation
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
    }

    /// Selects the appropriate agent for a task.
    ///
    /// Selection priority:
    /// 1. Explicit `agent_type` field in task (if present and valid)
    /// 2. Keyword matching in task title and description
    /// 3. Default to code-agent
    ///
    /// # Arguments
    /// * `task` - The Braingrid task to select an agent for
    ///
    /// # Returns
    /// The selected agent ID, or error if agent not found
    pub async fn select_agent(&self, task: &BraingridTask) -> Result<String, AgentSelectionError> {
        // Check if task has explicit agent_type hint
        // Note: BraingridTask doesn't currently have agent_type field, but we'll check
        // for it in the future. For now, we'll rely on keyword matching.

        // Extract keywords from task title and description
        let text = format!(
            "{} {}",
            task.title.to_lowercase(),
            task.description
                .as_ref()
                .map(|d| d.to_lowercase())
                .unwrap_or_default()
        );

        // Match keywords to agent types
        let agent_id = Self::match_keywords(&text);

        // Validate agent exists
        self.validate_agent(&agent_id).await?;

        Ok(agent_id)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::braingrid_client::TaskStatus;

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

