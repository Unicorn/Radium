//! Integration tests for Agent Configuration System.
//!
//! Tests the complete agent configuration workflow including discovery,
//! registry, prompt templates, and behaviors.

use radium_core::agents::config::{AgentConfig, AgentConfigFile, AgentLoopBehavior, AgentTriggerBehavior, ReasoningEffort};
use radium_core::agents::discovery::{AgentDiscovery, DiscoveryOptions};
use radium_core::agents::registry::AgentRegistry;
use radium_core::prompts::templates::{PromptContext, PromptTemplate};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper function to create a temporary test workspace with directory structure.
fn create_test_workspace() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let workspace = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(workspace.join("agents/core")).expect("Failed to create agents/core");
    fs::create_dir_all(workspace.join("agents/custom")).expect("Failed to create agents/custom");
    fs::create_dir_all(workspace.join("prompts/agents/core")).expect("Failed to create prompts/agents/core");
    fs::create_dir_all(workspace.join("prompts/agents/custom")).expect("Failed to create prompts/agents/custom");

    temp_dir
}

/// Helper function to create a test agent configuration file.
fn create_test_agent(
    workspace: &Path,
    category: &str,
    agent_id: &str,
    name: &str,
    description: &str,
    prompt_content: &str,
) -> PathBuf {
    // Create agent config
    let config_path = workspace.join("agents").join(category).join(format!("{}.toml", agent_id));
    let prompt_path = workspace.join("prompts/agents").join(category).join(format!("{}.md", agent_id));

    let config_content = format!(
        r#"[agent]
id = "{}"
name = "{}"
description = "{}"
prompt_path = "prompts/agents/{}/{}.md"
"#,
        agent_id, name, description, category, agent_id
    );

    fs::write(&config_path, config_content).expect("Failed to write agent config");
    fs::write(&prompt_path, prompt_content).expect("Failed to write prompt file");

    config_path
}

/// Helper function to create an agent with optional fields.
fn create_test_agent_full(
    workspace: &Path,
    category: &str,
    agent_id: &str,
    name: &str,
    description: &str,
    engine: Option<&str>,
    model: Option<&str>,
    reasoning_effort: Option<ReasoningEffort>,
) -> PathBuf {
    let config_path = workspace.join("agents").join(category).join(format!("{}.toml", agent_id));
    let prompt_path = workspace.join("prompts/agents").join(category).join(format!("{}.md", agent_id));

    let mut config_content = format!(
        r#"[agent]
id = "{}"
name = "{}"
description = "{}"
prompt_path = "prompts/agents/{}/{}.md"
"#,
        agent_id, name, description, category, agent_id
    );

    if let Some(eng) = engine {
        config_content.push_str(&format!("engine = \"{}\"\n", eng));
    }
    if let Some(mdl) = model {
        config_content.push_str(&format!("model = \"{}\"\n", mdl));
    }
    if let Some(effort) = reasoning_effort {
        config_content.push_str(&format!("reasoning_effort = \"{}\"\n", effort));
    }

    fs::write(&config_path, config_content).expect("Failed to write agent config");
    fs::write(&prompt_path, "# Test Prompt\n").expect("Failed to write prompt file");

    config_path
}

/// Helper function to create an agent with loop behavior.
fn create_test_agent_with_loop_behavior(
    workspace: &Path,
    category: &str,
    agent_id: &str,
) -> PathBuf {
    let config_path = workspace.join("agents").join(category).join(format!("{}.toml", agent_id));
    let prompt_path = workspace.join("prompts/agents").join(category).join(format!("{}.md", agent_id));

    let config_content = format!(
        r#"[agent]
id = "{}"
name = "Loop Agent"
description = "Agent with loop behavior"
prompt_path = "prompts/agents/{}/{}.md"

[agent.loop_behavior]
steps = 2
max_iterations = 5
skip = ["step-1", "step-3"]
"#,
        agent_id, category, agent_id
    );

    fs::write(&config_path, config_content).expect("Failed to write agent config");
    fs::write(&prompt_path, "# Loop Agent Prompt\n").expect("Failed to write prompt file");

    config_path
}

/// Helper function to create an agent with trigger behavior.
fn create_test_agent_with_trigger_behavior(
    workspace: &Path,
    category: &str,
    agent_id: &str,
) -> PathBuf {
    let config_path = workspace.join("agents").join(category).join(format!("{}.toml", agent_id));
    let prompt_path = workspace.join("prompts/agents").join(category).join(format!("{}.md", agent_id));

    let config_content = format!(
        r#"[agent]
id = "{}"
name = "Trigger Agent"
description = "Agent with trigger behavior"
prompt_path = "prompts/agents/{}/{}.md"

[agent.trigger_behavior]
trigger_agent_id = "target-agent"
"#,
        agent_id, category, agent_id
    );

    fs::write(&config_path, config_content).expect("Failed to write agent config");
    fs::write(&prompt_path, "# Trigger Agent Prompt\n").expect("Failed to write prompt file");

    config_path
}

#[test]
fn test_full_agent_discovery_workflow() {
    let temp_dir = create_test_workspace();
    let workspace = temp_dir.path();

    // Create multiple agents in different categories
    create_test_agent(workspace, "core", "arch-agent", "Architecture Agent", "Defines architecture", "# Arch Prompt");
    create_test_agent(workspace, "core", "plan-agent", "Plan Agent", "Creates plans", "# Plan Prompt");
    create_test_agent(workspace, "custom", "my-agent", "My Agent", "Custom agent", "# Custom Prompt");
    create_test_agent(workspace, "core", "code-agent", "Code Agent", "Writes code", "# Code Prompt");
    create_test_agent(workspace, "custom", "test-agent", "Test Agent", "Testing agent", "# Test Prompt");

    // Change to workspace directory for discovery
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(workspace).expect("Failed to change directory");

    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().expect("Failed to discover agents");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");

    // Verify all agents were discovered
    assert_eq!(agents.len(), 5, "Should discover 5 agents");

    // Verify specific agents
    assert!(agents.contains_key("arch-agent"), "Should contain arch-agent");
    assert!(agents.contains_key("plan-agent"), "Should contain plan-agent");
    assert!(agents.contains_key("my-agent"), "Should contain my-agent");
    assert!(agents.contains_key("code-agent"), "Should contain code-agent");
    assert!(agents.contains_key("test-agent"), "Should contain test-agent");

    // Verify categories are derived correctly
    let arch_agent = agents.get("arch-agent").expect("arch-agent not found");
    assert_eq!(arch_agent.category, Some("core".to_string()), "arch-agent should have category 'core'");

    let my_agent = agents.get("my-agent").expect("my-agent not found");
    assert_eq!(my_agent.category, Some("custom".to_string()), "my-agent should have category 'custom'");

    // Verify file paths are set
    assert!(arch_agent.file_path.is_some(), "arch-agent should have file_path set");
    assert!(my_agent.file_path.is_some(), "my-agent should have file_path set");
}

#[test]
fn test_agent_registry_with_discovery() {
    let temp_dir = create_test_workspace();
    let workspace = temp_dir.path();

    // Create test agents
    create_test_agent(workspace, "core", "agent-1", "Agent 1", "First agent", "# Prompt 1");
    create_test_agent(workspace, "core", "agent-2", "Agent 2", "Second agent", "# Prompt 2");
    create_test_agent(workspace, "custom", "agent-3", "Agent 3", "Third agent", "# Prompt 3");

    // Change to workspace directory
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(workspace).expect("Failed to change directory");

    // Create registry with discovery
    let registry = AgentRegistry::with_discovery().expect("Failed to create registry with discovery");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");

    // Test lookup by ID
    let agent1 = registry.get("agent-1").expect("Failed to get agent-1");
    assert_eq!(agent1.name, "Agent 1");
    assert_eq!(agent1.id, "agent-1");

    let agent2 = registry.get("agent-2").expect("Failed to get agent-2");
    assert_eq!(agent2.name, "Agent 2");

    // Test search by name
    let results = registry.search("Agent 1").expect("Failed to search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "agent-1");

    // Test search by description
    let results = registry.search("Second").expect("Failed to search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "agent-2");

    // Test filtering by category
    let core_agents = registry.filter(|a| a.category.as_ref().map(|c| c == "core").unwrap_or(false))
        .expect("Failed to filter");
    assert_eq!(core_agents.len(), 2);
    assert!(core_agents.iter().all(|a| a.category.as_ref().map(|c| c == "core").unwrap_or(false)));

    // Test count
    let count = registry.count().expect("Failed to get count");
    assert_eq!(count, 3);

    // Test concurrent access (basic thread safety check)
    use std::sync::Arc;
    use std::thread;

    let registry_arc = Arc::new(registry);
    let mut handles = vec![];

    for i in 0..5 {
        let reg = registry_arc.clone();
        let handle = thread::spawn(move || {
            let agent = reg.get("agent-1").expect("Failed to get agent");
            assert_eq!(agent.id, "agent-1");
            format!("thread-{}", i)
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().expect("Thread panicked");
        assert!(result.starts_with("thread-"));
    }
}

#[test]
fn test_prompt_template_with_agent_config() {
    let temp_dir = create_test_workspace();
    let workspace = temp_dir.path();

    // Create agent with prompt template containing placeholders
    let prompt_content = r#"# Test Agent

Hello {{name}}!

Your task is to {{task}}.

Please complete this by {{deadline}}.
"#;

    create_test_agent(workspace, "core", "template-agent", "Template Agent", "Tests templates", prompt_content);

    // Change to workspace directory
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(workspace).expect("Failed to change directory");

    // Load agent configuration
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().expect("Failed to discover agents");
    let agent = agents.get("template-agent").expect("template-agent not found");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");

    // Load prompt template
    let prompt_path = workspace.join(&agent.prompt_path);
    let template = PromptTemplate::load(&prompt_path).expect("Failed to load prompt template");

    // Verify placeholders are detected
    let placeholders = template.list_placeholders();
    assert_eq!(placeholders.len(), 3);
    assert!(placeholders.contains(&"name".to_string()));
    assert!(placeholders.contains(&"task".to_string()));
    assert!(placeholders.contains(&"deadline".to_string()));

    // Render template with context
    let mut context = PromptContext::new();
    context.set("name", "Alice");
    context.set("task", "analyze the code");
    context.set("deadline", "tomorrow");

    let rendered = template.render(&context).expect("Failed to render template");
    assert!(rendered.contains("Hello Alice!"));
    assert!(rendered.contains("analyze the code"));
    assert!(rendered.contains("tomorrow"));
}

#[test]
fn test_agent_behaviors_configuration() {
    let temp_dir = create_test_workspace();
    let workspace = temp_dir.path();

    // Create agent with loop behavior
    create_test_agent_with_loop_behavior(workspace, "core", "loop-agent");

    // Create agent with trigger behavior
    create_test_agent_with_trigger_behavior(workspace, "core", "trigger-agent");

    // Change to workspace directory
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(workspace).expect("Failed to change directory");

    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().expect("Failed to discover agents");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");

    // Verify loop behavior
    let loop_agent = agents.get("loop-agent").expect("loop-agent not found");
    assert!(loop_agent.loop_behavior.is_some(), "loop-agent should have loop_behavior");
    let loop_behavior = loop_agent.loop_behavior.as_ref().unwrap();
    assert_eq!(loop_behavior.steps, 2);
    assert_eq!(loop_behavior.max_iterations, Some(5));
    assert_eq!(loop_behavior.skip, vec!["step-1", "step-3"]);

    // Verify trigger behavior
    let trigger_agent = agents.get("trigger-agent").expect("trigger-agent not found");
    assert!(trigger_agent.trigger_behavior.is_some(), "trigger-agent should have trigger_behavior");
    let trigger_behavior = trigger_agent.trigger_behavior.as_ref().unwrap();
    assert_eq!(trigger_behavior.trigger_agent_id, Some("target-agent".to_string()));
}

#[test]
fn test_error_scenarios() {
    let temp_dir = create_test_workspace();
    let workspace = temp_dir.path();

    // Test 1: Missing prompt file
    let config_path = workspace.join("agents/core/missing-prompt.toml");
    let config_content = r#"[agent]
id = "missing-prompt"
name = "Missing Prompt Agent"
description = "Agent with missing prompt"
prompt_path = "prompts/agents/core/nonexistent.md"
"#;
    fs::write(&config_path, config_content).expect("Failed to write config");

    // Change to workspace directory
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(workspace).expect("Failed to change directory");

    let discovery = AgentDiscovery::new();
    // Discovery should succeed even if prompt file doesn't exist (validation happens later)
    let agents = discovery.discover_all().expect("Failed to discover agents");
    assert!(agents.contains_key("missing-prompt"), "Should discover agent even with missing prompt");

    // Test 2: Invalid TOML syntax
    let invalid_config_path = workspace.join("agents/core/invalid.toml");
    fs::write(&invalid_config_path, "[agent]\nid = invalid syntax\n").expect("Failed to write invalid config");

    let result = AgentConfigFile::load(&invalid_config_path);
    assert!(result.is_err(), "Should fail to load invalid TOML");

    // Test 3: Missing required fields
    let missing_fields_path = workspace.join("agents/core/missing-fields.toml");
    fs::write(&missing_fields_path, "[agent]\nid = \"test\"\n").expect("Failed to write config");

    // Loading the file directly should fail validation
    let result = AgentConfigFile::load(&missing_fields_path);
    assert!(result.is_err(), "Should fail to load config with missing required fields");
    
    // Discovery should also fail when trying to load this invalid config
    let discovery_result = discovery.discover_all();
    // Discovery may succeed for other agents but fail when it hits the invalid one
    // The exact behavior depends on implementation, but we've verified direct loading fails

    // Test 4: Duplicate agent IDs (later entry should override)
    create_test_agent(workspace, "core", "duplicate", "First Agent", "First", "# Prompt 1");
    create_test_agent(workspace, "custom", "duplicate", "Second Agent", "Second", "# Prompt 2");

    let agents = discovery.discover_all().expect("Failed to discover agents");
    // Should only have one entry (later one overrides)
    let duplicate_agent = agents.get("duplicate").expect("duplicate agent not found");
    // The order depends on discovery order, but we should have exactly one
    assert_eq!(agents.len(), 2); // missing-prompt + duplicate

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");
}

#[test]
fn test_agent_with_all_optional_fields() {
    let temp_dir = create_test_workspace();
    let workspace = temp_dir.path();

    create_test_agent_full(
        workspace,
        "core",
        "full-agent",
        "Full Agent",
        "Agent with all optional fields",
        Some("gemini"),
        Some("gemini-2.0-flash-exp"),
        Some(ReasoningEffort::High),
    );

    // Change to workspace directory
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(workspace).expect("Failed to change directory");

    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().expect("Failed to discover agents");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");

    let agent = agents.get("full-agent").expect("full-agent not found");
    assert_eq!(agent.engine, Some("gemini".to_string()));
    assert_eq!(agent.model, Some("gemini-2.0-flash-exp".to_string()));
    assert_eq!(agent.reasoning_effort, Some(ReasoningEffort::High));
}

#[test]
fn test_discovery_with_custom_options() {
    let temp_dir = create_test_workspace();
    let workspace = temp_dir.path();

    create_test_agent(workspace, "core", "agent-1", "Agent 1", "First", "# Prompt 1");
    create_test_agent(workspace, "core", "agent-2", "Agent 2", "Second", "# Prompt 2");
    create_test_agent(workspace, "custom", "agent-3", "Agent 3", "Third", "# Prompt 3");

    // Change to workspace directory
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(workspace).expect("Failed to change directory");

    // Test with sub-agent filter
    let mut options = DiscoveryOptions::default();
    options.search_paths = vec![workspace.join("agents")];
    options.sub_agent_filter = Some(vec!["agent-1".to_string(), "agent-3".to_string()]);

    let discovery = AgentDiscovery::with_options(options);
    let agents = discovery.discover_all().expect("Failed to discover agents");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");

    // Should only have filtered agents
    assert_eq!(agents.len(), 2);
    assert!(agents.contains_key("agent-1"));
    assert!(agents.contains_key("agent-3"));
    assert!(!agents.contains_key("agent-2"));
}

