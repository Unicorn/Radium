//! Learning integration helpers for workflow execution.
//!
//! Provides utilities to integrate ACE learning into workflow execution,
//! automatically applying skillbook updates after task completion.

use std::sync::Arc;

use crate::learning::{LearningStore, SkillManager};
use crate::oversight::{MetacognitiveService, OversightRequest, OversightResponse};
use crate::workflow::behaviors::vibe_check::WorkflowPhase;

/// Configuration for learning integration.
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// Whether learning is enabled.
    pub enabled: bool,
    /// Maximum skills per section to include in context.
    pub max_skills_per_section: usize,
    /// Maximum learning entries per category to include in context.
    pub max_entries_per_category: usize,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_skills_per_section: 5,
            max_entries_per_category: 3,
        }
    }
}

impl LearningConfig {
    /// Creates a new learning configuration.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Default::default()
        }
    }

    /// Disables learning.
    pub fn disabled() -> Self {
        Self::new(false)
    }
}

/// Learning integration helper for workflow execution.
///
/// This helper can be used to automatically apply learning updates
/// after workflow steps complete. It integrates:
/// - MetacognitiveService for oversight feedback
/// - SkillManager for generating skillbook updates
/// - LearningStore for applying updates
pub struct LearningIntegration {
    /// Learning configuration.
    config: LearningConfig,
    /// Metacognitive service for oversight.
    metacognitive: Arc<MetacognitiveService>,
    /// Skill manager for generating updates.
    skill_manager: Arc<SkillManager>,
    /// Learning store for applying updates.
    learning_store: Arc<std::sync::Mutex<LearningStore>>,
}

impl LearningIntegration {
    /// Creates a new learning integration helper.
    ///
    /// # Arguments
    /// * `config` - Learning configuration
    /// * `metacognitive` - Metacognitive service
    /// * `skill_manager` - Skill manager
    /// * `learning_store` - Learning store
    pub fn new(
        config: LearningConfig,
        metacognitive: Arc<MetacognitiveService>,
        skill_manager: Arc<SkillManager>,
        learning_store: Arc<std::sync::Mutex<LearningStore>>,
    ) -> Self {
        Self { config, metacognitive, skill_manager, learning_store }
    }

    /// Processes learning from a completed task.
    ///
    /// This method:
    /// 1. Generates oversight feedback for the task
    /// 2. Extracts helpful/harmful patterns
    /// 3. Generates skillbook updates
    /// 4. Applies updates to the learning store
    ///
    /// # Arguments
    /// * `phase` - Current workflow phase
    /// * `goal` - Task goal
    /// * `plan` - Task plan
    /// * `progress` - Current progress
    /// * `task_context` - Task context or output
    /// * `question_context` - Description of the task domain
    ///
    /// # Returns
    /// `Ok(Some(OversightResponse))` if learning was processed,
    /// `Ok(None)` if learning is disabled,
    /// `Err` if processing failed
    pub async fn process_task_learning(
        &self,
        phase: WorkflowPhase,
        goal: &str,
        plan: &str,
        progress: &str,
        task_context: &str,
        question_context: &str,
    ) -> Result<Option<OversightResponse>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Ok(None);
        }

        // 1. Generate oversight feedback
        // Gather learning context first
        let learning_context = {
            let store = self.learning_store.lock().map_err(|e| {
                format!("Failed to lock learning store: {}", e)
            })?;
            store.generate_context(self.config.max_entries_per_category)
        };

        let mut oversight_request = OversightRequest::new(phase, goal.to_string(), plan.to_string())
            .with_progress(progress)
            .with_task_context(task_context);

        if !learning_context.is_empty() {
            oversight_request = oversight_request.with_learning_context(learning_context);
        }

        let oversight_response = self
            .metacognitive
            .generate_oversight(&oversight_request)
            .await
            .map_err(|e| format!("Oversight generation failed: {}", e))?;

        // 2. Generate skillbook updates from oversight
        let learning_store_guard = self.learning_store.lock().map_err(|e| {
            format!("Failed to lock learning store: {}", e)
        })?;

        let update_batch = self
            .skill_manager
            .generate_updates(
                &oversight_response,
                &learning_store_guard,
                question_context,
                progress,
            )
            .await
            .map_err(|e| format!("Skill manager update generation failed: {}", e))?;

        drop(learning_store_guard);

        // 3. Apply updates to learning store
        if !update_batch.is_empty() {
            let mut store = self.learning_store.lock().map_err(|e| {
                format!("Failed to lock learning store: {}", e)
            })?;
            store.apply_update(&update_batch).map_err(|e| {
                format!("Failed to apply learning updates: {}", e)
            })?;
        }

        Ok(Some(oversight_response))
    }

    /// Gets the learning store reference.
    pub fn learning_store(&self) -> &Arc<std::sync::Mutex<LearningStore>> {
        &self.learning_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::LearningStore;
    use crate::oversight::MetacognitiveService;
    use crate::learning::SkillManager;
    use radium_abstraction::{Model, ModelResponse, ModelParameters};
    use std::sync::Arc;
    use tempfile::TempDir;

    // Mock model for testing
    struct MockModel;
    #[async_trait::async_trait]
    impl Model for MockModel {
        async fn generate_text(
            &self,
            _prompt: &str,
            _params: Option<ModelParameters>,
        ) -> Result<ModelResponse, radium_abstraction::ModelError> {
            Ok(ModelResponse {
                content: "Mock text response".to_string(),
                model_id: Some("mock".to_string()),
                usage: None,
            })
        }

        async fn generate_chat_completion(
            &self,
            _messages: &[radium_abstraction::ChatMessage],
            _params: Option<ModelParameters>,
        ) -> Result<ModelResponse, radium_abstraction::ModelError> {
            Ok(ModelResponse {
                content: r#"{
                    "reasoning": "Test reasoning",
                    "operations": []
                }"#
                .to_string(),
                model_id: Some("mock".to_string()),
                usage: None,
            })
        }

        fn model_id(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_learning_integration_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let learning_store = Arc::new(std::sync::Mutex::new(
            LearningStore::new(temp_dir.path()).unwrap(),
        ));
        let model = Arc::new(MockModel);
        let metacognitive = Arc::new(MetacognitiveService::new(model.clone()));
        let skill_manager = Arc::new(SkillManager::new(model));

        let integration = LearningIntegration::new(
            LearningConfig::disabled(),
            metacognitive,
            skill_manager,
            learning_store,
        );

        let result = integration
            .process_task_learning(
                WorkflowPhase::Implementation,
                "Test goal",
                "Test plan",
                "50%",
                "Test context",
                "Test question",
            )
            .await
            .unwrap();

        assert!(result.is_none());
    }
}

