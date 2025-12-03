//! Integration tests for the rad step command.
//!
//! Tests the single agent execution functionality including:
//! - Agent discovery
//! - Prompt template loading and rendering
//! - Model integration with graceful fallback
//! - Token usage tracking

use radium_core::{AgentConfigFile, AgentDiscovery, PromptContext, PromptTemplate};
use radium_models::{ModelFactory, ModelType};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test agent configuration.
fn create_test_agent_config(temp_dir: &TempDir, agent_id: &str) -> PathBuf {
    let agents_dir = temp_dir.path().join("agents").join("test");
    fs::create_dir_all(&agents_dir).unwrap();

    let config_path = agents_dir.join(format!("{}.toml", agent_id));
    let config_content = format!(
        r#"[agent]
id = "{}"
name = "Test Agent"
description = "A test agent for integration testing"
prompt_path = "prompts/test-agent.md"
engine = "mock"
model = "test-model"
reasoning_effort = "medium"
"#,
        agent_id
    );

    fs::write(&config_path, config_content).unwrap();
    config_path
}

/// Helper to create a test prompt template.
fn create_test_prompt_template(temp_dir: &TempDir) -> PathBuf {
    let prompts_dir = temp_dir.path().join("prompts");
    fs::create_dir_all(&prompts_dir).unwrap();

    let prompt_path = prompts_dir.join("test-agent.md");
    let prompt_content = r#"# Test Agent Prompt

You are a test agent for integration testing.

## User Input

{{user_input}}
"#;

    fs::write(&prompt_path, prompt_content).unwrap();
    prompt_path
}

#[test]
fn test_agent_discovery_from_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_test_agent_config(&temp_dir, "test-agent");

    // Note: In real usage, discovery would search standard paths
    let _discovery = AgentDiscovery::new();
    // For testing, we verify the config can be loaded directly
    let config_path = temp_dir.path().join("agents/test/test-agent.toml");
    let config_file = AgentConfigFile::load(&config_path).unwrap();
    let config = &config_file.agent;

    assert_eq!(config.id, "test-agent");
    assert_eq!(config.name, "Test Agent");
    assert_eq!(config.engine.as_deref(), Some("mock"));
}

#[test]
fn test_prompt_template_rendering() {
    let temp_dir = TempDir::new().unwrap();
    let prompt_path = create_test_prompt_template(&temp_dir);

    // Load and render template
    let template_content = fs::read_to_string(&prompt_path).unwrap();
    let template = PromptTemplate::from_str(template_content);

    let mut context = PromptContext::new();
    context.set("user_input", "Test input message");

    let rendered = template.render(&context).unwrap();

    assert!(rendered.contains("Test Agent Prompt"));
    assert!(rendered.contains("Test input message"));
    assert!(!rendered.contains("{{user_input}}"));
}

#[test]
fn test_model_factory_creates_mock_model() {
    // Test that ModelFactory can create a mock model
    let result = ModelFactory::create_from_str("mock", "test-model".to_string());

    // Mock model should be created successfully without API keys
    assert!(result.is_ok());
}

#[test]
fn test_model_factory_gemini_without_api_key() {
    // Clear any existing API key
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::remove_var("GEMINI_API_KEY") };

    // Attempt to create Gemini model without API key
    let result = ModelFactory::create_from_str("gemini", "gemini-2.0-flash-exp".to_string());

    // Should fail with appropriate error
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("API_KEY") || error_msg.contains("environment variable"),
            "Error message should mention API key: {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn test_mock_model_execution() {
    // Create a mock model and execute a simple prompt
    let model = ModelFactory::create_from_str("mock", "test-model".to_string()).unwrap();

    let prompt = "Test prompt for mock model";
    let response = model.generate_text(prompt, None).await.unwrap();

    // Mock model should return a response
    assert!(!response.content.is_empty());
    assert!(response.content.contains("Mock response"));
}

#[test]
fn test_agent_config_validation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_test_agent_config(&temp_dir, "validation-test");

    let config_file = AgentConfigFile::load(&config_path).unwrap();
    let config = &config_file.agent;

    // Verify all required fields are present
    assert!(!config.id.is_empty());
    assert!(!config.name.is_empty());
    assert!(!config.description.is_empty());
    assert!(!config.prompt_path.as_os_str().is_empty());
}

#[test]
fn test_prompt_template_with_missing_variable() {
    let template_content = "Hello {{name}}, welcome to {{place}}!";
    let template = PromptTemplate::from_str(template_content);

    let mut context = PromptContext::new();
    context.set("name", "Alice");
    // Note: 'place' is missing

    let rendered = template.render(&context).unwrap();

    // Should render with the available variable
    assert!(rendered.contains("Alice"));
    // Missing variable should be left as-is or replaced with empty string
    // depending on implementation
}

#[test]
fn test_prompt_template_list_placeholders() {
    let template_content = "Variables: {{var1}}, {{var2}}, {{var3}}";
    let template = PromptTemplate::from_str(template_content);

    let placeholders = template.list_placeholders();

    assert_eq!(placeholders.len(), 3);
    assert!(placeholders.contains(&"var1".to_string()));
    assert!(placeholders.contains(&"var2".to_string()));
    assert!(placeholders.contains(&"var3".to_string()));
}

#[test]
fn test_model_type_from_string() {
    use std::str::FromStr;

    assert!(matches!(ModelType::from_str("mock"), Ok(ModelType::Mock)));
    assert!(matches!(ModelType::from_str("gemini"), Ok(ModelType::Gemini)));
    assert!(matches!(ModelType::from_str("openai"), Ok(ModelType::OpenAI)));
    assert!(ModelType::from_str("invalid").is_err());
}

#[test]
fn test_agent_config_with_all_fields() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path().join("agents").join("test");
    fs::create_dir_all(&agents_dir).unwrap();

    let config_path = agents_dir.join("full-config.toml");
    let config_content = r#"[agent]
id = "full-agent"
name = "Full Agent"
description = "Agent with all configuration fields"
prompt_path = "prompts/full-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "high"
category = "development"
tags = ["test", "full"]
"#;

    fs::write(&config_path, config_content).unwrap();

    let config_file = AgentConfigFile::load(&config_path).unwrap();
    let config = &config_file.agent;

    assert_eq!(config.id, "full-agent");
    assert_eq!(config.engine.as_deref(), Some("gemini"));
    assert_eq!(config.model.as_deref(), Some("gemini-2.0-flash-exp"));
    assert_eq!(config.reasoning_effort, Some(radium_core::ReasoningEffort::High));
}

#[tokio::test]
async fn test_end_to_end_step_execution_mock() {
    // Create temporary workspace with agent and prompt
    let temp_dir = TempDir::new().unwrap();
    create_test_agent_config(&temp_dir, "e2e-test");
    create_test_prompt_template(&temp_dir);

    // Load agent config
    let config_path = temp_dir.path().join("agents/test/e2e-test.toml");
    let config_file = AgentConfigFile::load(&config_path).unwrap();
    let config = &config_file.agent;

    // Load and render prompt
    let prompt_path = temp_dir.path().join(&config.prompt_path);
    let prompt_content = fs::read_to_string(&prompt_path).unwrap();
    let template = PromptTemplate::from_str(prompt_content);

    let mut context = PromptContext::new();
    context.set("user_input", "End-to-end test input");

    let rendered = template.render(&context).unwrap();

    // Create model and execute
    let model = ModelFactory::create_from_str(
        config.engine.as_deref().unwrap_or("mock"),
        config.model.clone().unwrap_or_default(),
    )
    .unwrap();

    let response = model.generate_text(&rendered, None).await.unwrap();

    // Verify response
    assert!(!response.content.is_empty());
}
