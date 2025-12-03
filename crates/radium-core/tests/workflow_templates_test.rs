//! Tests for workflow template system.

use radium_core::workflow::behaviors::{LoopBehaviorConfig, TriggerBehaviorConfig};
use radium_core::workflow::templates::{
    ModuleBehavior, ModuleBehaviorAction, ModuleBehaviorType, WorkflowStep, WorkflowStepConfig,
    WorkflowStepType, WorkflowTemplate, WorkflowTemplateError,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_workflow_template_load_from_file_valid() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("template.json");

    let original = WorkflowTemplate::new("Test Template")
        .with_description("A test template")
        .add_step(WorkflowStep::agent_step("agent-1"))
        .add_step(WorkflowStep::ui_step("Phase 1"));

    original.save_to_file(&template_path).unwrap();
    let loaded = WorkflowTemplate::load_from_file(&template_path).unwrap();

    assert_eq!(loaded.name, "Test Template");
    assert_eq!(loaded.description.as_deref(), Some("A test template"));
    assert_eq!(loaded.steps.len(), 2);
}

#[test]
fn test_workflow_template_load_from_file_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("invalid.json");

    // Write invalid JSON
    std::fs::write(&template_path, "{ invalid json }").unwrap();

    let result = WorkflowTemplate::load_from_file(&template_path);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkflowTemplateError::ParseError { .. }));
    }
}

#[test]
fn test_workflow_template_load_from_file_missing_required_fields() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("incomplete.json");

    // Write JSON missing required "name" field
    std::fs::write(&template_path, r#"{"steps": []}"#).unwrap();

    let result = WorkflowTemplate::load_from_file(&template_path);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkflowTemplateError::ParseError { .. }));
    }
}

#[test]
fn test_workflow_template_load_from_file_not_found() {
    let template_path = PathBuf::from("/nonexistent/path/template.json");
    let result = WorkflowTemplate::load_from_file(&template_path);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkflowTemplateError::IoError { .. }));
    }
}

#[test]
fn test_workflow_template_validate_valid() {
    let template =
        WorkflowTemplate::new("Valid Template").add_step(WorkflowStep::agent_step("agent-1"));

    let result = template.validate();
    assert!(result.is_ok());
}

#[test]
fn test_workflow_template_validate_empty_name() {
    let template = WorkflowTemplate::new("").add_step(WorkflowStep::agent_step("agent-1"));

    let result = template.validate();
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkflowTemplateError::ValidationError(_)));
        assert!(e.to_string().contains("name cannot be empty"));
    }
}

#[test]
fn test_workflow_template_validate_empty_steps() {
    let template = WorkflowTemplate::new("Template Name");
    // No steps added

    let result = template.validate();
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkflowTemplateError::ValidationError(_)));
        assert!(e.to_string().contains("at least one step"));
    }
}

#[test]
fn test_workflow_template_validate_duplicate_step_ids() {
    // Note: WorkflowTemplate doesn't validate duplicate step IDs in current implementation
    // This test verifies the current behavior
    let template = WorkflowTemplate::new("Template")
        .add_step(WorkflowStep::agent_step("agent-1"))
        .add_step(WorkflowStep::agent_step("agent-1")); // Same agent, but different step instances

    // Current implementation allows this - steps are identified by their position/index
    let result = template.validate();
    // Should pass validation (duplicate step ID checking not implemented)
    assert!(result.is_ok());
}

#[test]
fn test_workflow_step_config_all_optional_fields() {
    let step = WorkflowStep::agent_step("test-agent")
        .with_agent_name("Test Agent")
        .with_execute_once(true)
        .with_engine("claude")
        .with_model("claude-3-opus");

    assert_eq!(step.config.agent_id, "test-agent");
    assert_eq!(step.config.agent_name.as_deref(), Some("Test Agent"));
    assert!(step.config.execute_once);
    assert_eq!(step.config.engine.as_deref(), Some("claude"));
    assert_eq!(step.config.model.as_deref(), Some("claude-3-opus"));
}

#[test]
fn test_workflow_step_config_module_behavior() {
    let loop_config =
        LoopBehaviorConfig { steps: 3, max_iterations: Some(5), skip: vec!["init".to_string()] };

    let behavior = ModuleBehavior {
        behavior_type: ModuleBehaviorType::Loop,
        action: ModuleBehaviorAction::StepBack,
        loop_config: Some(loop_config),
        trigger_config: None,
    };

    let step = WorkflowStep::module_step("loop-agent", behavior.clone());

    assert_eq!(step.config.step_type, WorkflowStepType::Module);
    assert!(step.config.module.is_some());
    let module = step.config.module.unwrap();
    assert_eq!(module.behavior_type, ModuleBehaviorType::Loop);
    assert_eq!(module.action, ModuleBehaviorAction::StepBack);
    assert!(module.loop_config.is_some());
}

#[test]
fn test_workflow_step_config_trigger_behavior() {
    let trigger_config =
        TriggerBehaviorConfig { trigger_agent_id: Some("trigger-agent".to_string()) };

    let behavior = ModuleBehavior {
        behavior_type: ModuleBehaviorType::Trigger,
        action: ModuleBehaviorAction::MainAgentCall,
        loop_config: None,
        trigger_config: Some(trigger_config),
    };

    let step = WorkflowStep::module_step("main-agent", behavior.clone());

    assert_eq!(step.config.step_type, WorkflowStepType::Module);
    assert!(step.config.module.is_some());
    let module = step.config.module.unwrap();
    assert_eq!(module.behavior_type, ModuleBehaviorType::Trigger);
    assert!(module.trigger_config.is_some());
}

#[test]
fn test_workflow_step_config_ui_step() {
    let step = WorkflowStep::ui_step("Phase 1: Setup");

    assert_eq!(step.config.step_type, WorkflowStepType::Ui);
    assert_eq!(step.config.label.as_deref(), Some("Phase 1: Setup"));
    assert_eq!(step.config.agent_id, ""); // UI steps don't need agent
}

#[test]
fn test_workflow_template_error_types() {
    // Test IoError
    let path = PathBuf::from("/nonexistent/path");
    let result = WorkflowTemplate::load_from_file(&path);
    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            WorkflowTemplateError::IoError { .. } => {}
            _ => panic!("Expected IoError"),
        }
    }

    // Test ParseError
    let temp_dir = TempDir::new().unwrap();
    let invalid_path = temp_dir.path().join("invalid.json");
    std::fs::write(&invalid_path, "{ invalid }").unwrap();

    let result = WorkflowTemplate::load_from_file(&invalid_path);
    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            WorkflowTemplateError::ParseError { .. } => {}
            _ => panic!("Expected ParseError"),
        }
    }

    // Test ValidationError
    let invalid_template = WorkflowTemplate::new(""); // Empty name
    let result = invalid_template.validate();
    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            WorkflowTemplateError::ValidationError(_) => {}
            _ => panic!("Expected ValidationError"),
        }
    }
}

#[test]
fn test_workflow_template_save_to_file_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("nested").join("dir").join("template.json");

    let template = WorkflowTemplate::new("Test").add_step(WorkflowStep::agent_step("agent-1"));

    // Should create parent directories
    let result = template.save_to_file(&nested_path);
    assert!(result.is_ok());

    // Verify file was created
    assert!(nested_path.exists());

    // Verify we can load it back
    let loaded = WorkflowTemplate::load_from_file(&nested_path).unwrap();
    assert_eq!(loaded.name, "Test");
}

#[test]
fn test_workflow_template_with_sub_agents() {
    let template = WorkflowTemplate::new("Template")
        .with_sub_agents(vec!["agent-1".to_string(), "agent-2".to_string()])
        .add_step(WorkflowStep::agent_step("agent-1"));

    assert_eq!(template.sub_agent_ids.len(), 2);
    assert!(template.sub_agent_ids.contains(&"agent-1".to_string()));
    assert!(template.sub_agent_ids.contains(&"agent-2".to_string()));
}

#[test]
fn test_workflow_template_with_steps() {
    let steps = vec![
        WorkflowStep::agent_step("agent-1"),
        WorkflowStep::agent_step("agent-2"),
        WorkflowStep::ui_step("Separator"),
    ];

    let template = WorkflowTemplate::new("Template").with_steps(steps.clone());

    assert_eq!(template.steps.len(), 3);
    assert_eq!(template.steps[0].config.agent_id, "agent-1");
    assert_eq!(template.steps[1].config.agent_id, "agent-2");
    assert_eq!(template.steps[2].config.step_type, WorkflowStepType::Ui);
}

#[test]
fn test_workflow_template_serialization_roundtrip() {
    let original = WorkflowTemplate::new("Test Template")
        .with_description("A test")
        .with_sub_agents(vec!["agent-1".to_string()])
        .add_step(WorkflowStep::agent_step("agent-1").with_execute_once(true))
        .add_step(WorkflowStep::ui_step("Phase 1"));

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: WorkflowTemplate = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, original.name);
    assert_eq!(deserialized.description, original.description);
    assert_eq!(deserialized.steps.len(), original.steps.len());
    assert_eq!(deserialized.sub_agent_ids, original.sub_agent_ids);
}

#[test]
fn test_workflow_template_save_serialize_error() {
    // This is hard to test without mocking serde_json
    // The SerializeError would occur if there's a serialization issue
    // For now, we test that normal serialization works
    let template = WorkflowTemplate::new("Test").add_step(WorkflowStep::agent_step("agent-1"));

    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("template.json");

    let result = template.save_to_file(&template_path);
    assert!(result.is_ok());
}
