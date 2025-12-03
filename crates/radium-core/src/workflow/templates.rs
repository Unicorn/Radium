//! Workflow template system.
//!
//! Defines workflow templates with steps, modules, and behavior configurations
//! for Radium's workflow execution engine.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

use super::behaviors::{LoopBehaviorConfig, TriggerBehaviorConfig};

/// Type of workflow step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStepType {
    /// Regular agent step.
    Step,
    /// Module step with behavior.
    Module,
    /// UI separator label.
    Ui,
}

/// Configuration for a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStepConfig {
    /// Agent ID to execute.
    pub agent_id: String,
    /// Human-readable agent name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    /// Type of step.
    #[serde(rename = "type")]
    pub step_type: WorkflowStepType,
    /// Execute only once (skip on resume).
    #[serde(default)]
    pub execute_once: bool,
    /// Engine override (e.g., "claude", "codex", "cursor").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// Model override (e.g., "gpt-4", "claude-3-opus").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Reasoning effort level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,
    /// Fallback agent if step not completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_completed_fallback: Option<String>,
    /// Module behavior configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<ModuleBehavior>,
    /// UI label (for UI steps only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Module behavior configuration (combines loop and trigger).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleBehavior {
    /// Behavior type.
    #[serde(rename = "type")]
    pub behavior_type: ModuleBehaviorType,
    /// Action to perform.
    pub action: ModuleBehaviorAction,
    /// Loop configuration (if type is loop).
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub loop_config: Option<LoopBehaviorConfig>,
    /// Trigger configuration (if type is trigger).
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub trigger_config: Option<TriggerBehaviorConfig>,
}

/// Type of module behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleBehaviorType {
    /// Loop behavior (repeat steps).
    Loop,
    /// Trigger behavior (dynamic agent call).
    Trigger,
}

/// Action for module behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModuleBehaviorAction {
    /// Step back N steps (for loop).
    StepBack,
    /// Call main agent (for trigger).
    MainAgentCall,
}

/// A step in a workflow template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step configuration.
    #[serde(flatten)]
    pub config: WorkflowStepConfig,
}

impl WorkflowStep {
    /// Creates a new workflow step.
    pub fn new(config: WorkflowStepConfig) -> Self {
        Self { config }
    }

    /// Creates a regular agent step.
    pub fn agent_step(agent_id: impl Into<String>) -> Self {
        Self::new(WorkflowStepConfig {
            agent_id: agent_id.into(),
            agent_name: None,
            step_type: WorkflowStepType::Step,
            execute_once: false,
            engine: None,
            model: None,
            model_reasoning_effort: None,
            not_completed_fallback: None,
            module: None,
            label: None,
        })
    }

    /// Creates a module step with behavior.
    pub fn module_step(agent_id: impl Into<String>, behavior: ModuleBehavior) -> Self {
        Self::new(WorkflowStepConfig {
            agent_id: agent_id.into(),
            agent_name: None,
            step_type: WorkflowStepType::Module,
            execute_once: false,
            engine: None,
            model: None,
            model_reasoning_effort: None,
            not_completed_fallback: None,
            module: Some(behavior),
            label: None,
        })
    }

    /// Creates a UI separator step.
    pub fn ui_step(label: impl Into<String>) -> Self {
        Self::new(WorkflowStepConfig {
            agent_id: String::new(),
            agent_name: None,
            step_type: WorkflowStepType::Ui,
            execute_once: false,
            engine: None,
            model: None,
            model_reasoning_effort: None,
            not_completed_fallback: None,
            module: None,
            label: Some(label.into()),
        })
    }

    /// Sets execute_once flag.
    pub fn with_execute_once(mut self, execute_once: bool) -> Self {
        self.config.execute_once = execute_once;
        self
    }

    /// Sets engine override.
    pub fn with_engine(mut self, engine: impl Into<String>) -> Self {
        self.config.engine = Some(engine.into());
        self
    }

    /// Sets model override.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = Some(model.into());
        self
    }

    /// Sets agent name.
    pub fn with_agent_name(mut self, name: impl Into<String>) -> Self {
        self.config.agent_name = Some(name.into());
        self
    }
}

/// Workflow template definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTemplate {
    /// Template name.
    pub name: String,
    /// Template description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Workflow steps.
    pub steps: Vec<WorkflowStep>,
    /// Sub-agent IDs available in this workflow.
    #[serde(default)]
    pub sub_agent_ids: Vec<String>,
}

impl WorkflowTemplate {
    /// Creates a new workflow template.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), description: None, steps: Vec::new(), sub_agent_ids: Vec::new() }
    }

    /// Adds a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a step.
    pub fn add_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Adds multiple steps.
    pub fn with_steps(mut self, steps: Vec<WorkflowStep>) -> Self {
        self.steps = steps;
        self
    }

    /// Adds sub-agent IDs.
    pub fn with_sub_agents(mut self, sub_agent_ids: Vec<String>) -> Self {
        self.sub_agent_ids = sub_agent_ids;
        self
    }

    /// Loads a workflow template from a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to the template JSON file
    ///
    /// # Returns
    /// `Ok(WorkflowTemplate)` if successful, `Err(WorkflowTemplateError)` otherwise.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, WorkflowTemplateError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| WorkflowTemplateError::IoError { path: path.to_path_buf(), source: e })?;

        let template: WorkflowTemplate = serde_json::from_str(&content).map_err(|e| {
            WorkflowTemplateError::ParseError { path: path.to_path_buf(), source: e }
        })?;

        Ok(template)
    }

    /// Saves the workflow template to a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to save the template
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err(WorkflowTemplateError)` otherwise.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), WorkflowTemplateError> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| WorkflowTemplateError::IoError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| WorkflowTemplateError::SerializeError { source: e })?;

        std::fs::write(path, content)
            .map_err(|e| WorkflowTemplateError::IoError { path: path.to_path_buf(), source: e })
    }

    /// Validates the workflow template.
    ///
    /// # Returns
    /// `Ok(())` if valid, `Err(WorkflowTemplateError)` otherwise.
    pub fn validate(&self) -> Result<(), WorkflowTemplateError> {
        if self.name.is_empty() {
            return Err(WorkflowTemplateError::ValidationError(
                "template name cannot be empty".to_string(),
            ));
        }

        if self.steps.is_empty() {
            return Err(WorkflowTemplateError::ValidationError(
                "template must have at least one step".to_string(),
            ));
        }

        // Validate each step
        for (idx, step) in self.steps.iter().enumerate() {
            match step.config.step_type {
                WorkflowStepType::Step | WorkflowStepType::Module => {
                    if step.config.agent_id.is_empty() {
                        return Err(WorkflowTemplateError::ValidationError(format!(
                            "step {} has empty agent_id",
                            idx
                        )));
                    }
                }
                WorkflowStepType::Ui => {
                    if step.config.label.is_none() {
                        return Err(WorkflowTemplateError::ValidationError(format!(
                            "UI step {} has no label",
                            idx
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Errors that can occur when working with workflow templates.
#[derive(Error, Debug)]
pub enum WorkflowTemplateError {
    /// Failed to read or write template file.
    #[error("I/O error at {path}: {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse template JSON.
    #[error("Failed to parse template at {path}: {source}")]
    ParseError {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    /// Failed to serialize template.
    #[error("Failed to serialize template: {source}")]
    SerializeError {
        #[source]
        source: serde_json::Error,
    },

    /// Template validation error.
    #[error("Template validation error: {0}")]
    ValidationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workflow_step_agent() {
        let step = WorkflowStep::agent_step("test-agent")
            .with_execute_once(true)
            .with_engine("claude")
            .with_model("claude-3-opus");

        assert_eq!(step.config.agent_id, "test-agent");
        assert_eq!(step.config.step_type, WorkflowStepType::Step);
        assert!(step.config.execute_once);
        assert_eq!(step.config.engine.as_deref(), Some("claude"));
        assert_eq!(step.config.model.as_deref(), Some("claude-3-opus"));
    }

    #[test]
    fn test_workflow_step_module() {
        let behavior = ModuleBehavior {
            behavior_type: ModuleBehaviorType::Loop,
            action: ModuleBehaviorAction::StepBack,
            loop_config: Some(LoopBehaviorConfig {
                steps: 3,
                max_iterations: Some(5),
                skip: vec!["init".to_string()],
            }),
            trigger_config: None,
        };

        let step = WorkflowStep::module_step("loop-agent", behavior);

        assert_eq!(step.config.agent_id, "loop-agent");
        assert_eq!(step.config.step_type, WorkflowStepType::Module);
        assert!(step.config.module.is_some());
    }

    #[test]
    fn test_workflow_step_ui() {
        let step = WorkflowStep::ui_step("Development Phase");

        assert_eq!(step.config.step_type, WorkflowStepType::Ui);
        assert_eq!(step.config.label.as_deref(), Some("Development Phase"));
    }

    #[test]
    fn test_workflow_template_new() {
        let template = WorkflowTemplate::new("Test Template").with_description("A test template");

        assert_eq!(template.name, "Test Template");
        assert_eq!(template.description.as_deref(), Some("A test template"));
        assert!(template.steps.is_empty());
    }

    #[test]
    fn test_workflow_template_with_steps() {
        let template = WorkflowTemplate::new("Test")
            .add_step(WorkflowStep::agent_step("agent-1"))
            .add_step(WorkflowStep::agent_step("agent-2"));

        assert_eq!(template.steps.len(), 2);
        assert_eq!(template.steps[0].config.agent_id, "agent-1");
        assert_eq!(template.steps[1].config.agent_id, "agent-2");
    }

    #[test]
    fn test_workflow_template_validate_empty_name() {
        let template = WorkflowTemplate::new("").add_step(WorkflowStep::agent_step("agent-1"));

        let result = template.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WorkflowTemplateError::ValidationError(_)));
    }

    #[test]
    fn test_workflow_template_validate_no_steps() {
        let template = WorkflowTemplate::new("Test");

        let result = template.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_template_validate_empty_agent_id() {
        let mut template = WorkflowTemplate::new("Test");
        template.steps.push(WorkflowStep::new(WorkflowStepConfig {
            agent_id: String::new(),
            agent_name: None,
            step_type: WorkflowStepType::Step,
            execute_once: false,
            engine: None,
            model: None,
            model_reasoning_effort: None,
            not_completed_fallback: None,
            module: None,
            label: None,
        }));

        let result = template.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_template_validate_ui_no_label() {
        let mut template = WorkflowTemplate::new("Test");
        template.steps.push(WorkflowStep::new(WorkflowStepConfig {
            agent_id: String::new(),
            agent_name: None,
            step_type: WorkflowStepType::Ui,
            execute_once: false,
            engine: None,
            model: None,
            model_reasoning_effort: None,
            not_completed_fallback: None,
            module: None,
            label: None,
        }));

        let result = template.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_template_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("template.json");

        let original = WorkflowTemplate::new("Test Template")
            .with_description("A test")
            .add_step(WorkflowStep::agent_step("agent-1").with_execute_once(true))
            .add_step(WorkflowStep::ui_step("Phase 1"));

        // Save
        original.save_to_file(&template_path).unwrap();

        // Load
        let loaded = WorkflowTemplate::load_from_file(&template_path).unwrap();

        assert_eq!(loaded.name, "Test Template");
        assert_eq!(loaded.description.as_deref(), Some("A test"));
        assert_eq!(loaded.steps.len(), 2);
        assert_eq!(loaded.steps[0].config.agent_id, "agent-1");
        assert!(loaded.steps[0].config.execute_once);
    }

    #[test]
    fn test_workflow_template_serialization() {
        let template = WorkflowTemplate::new("Test").add_step(WorkflowStep::agent_step("agent-1"));

        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"agent-1\""));

        let deserialized: WorkflowTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test");
        assert_eq!(deserialized.steps.len(), 1);
    }
}
