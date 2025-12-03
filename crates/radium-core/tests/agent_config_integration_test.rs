//! Integration tests for agent configuration system.

use radium_core::agents::config::{AgentConfig, AgentConfigFile, ReasoningEffort};
use radium_core::agents::discovery::{AgentDiscovery, DiscoveryOptions};
use radium_core::agents::prompt_loader::AgentPromptLoader;
use radium_core::prompts::PromptContext;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_agent_config_discovery_and_prompt_loading() {
    let temp_dir = TempDir::new().unwrap();

    // Create agent directory structure
    let agents_dir = temp_dir.path().join("agents");
    let category_dir = agents_dir.join("test-agents");
    fs::create_dir_all(&category_dir).unwrap();

    // Create prompt file
    let prompt_dir = temp_dir.path().join("prompts");
    fs::create_dir_all(&prompt_dir).unwrap();
    let prompt_file = prompt_dir.join("test-agent.md");
    fs::write(&prompt_file, "Hello {{name}}! Your task is {{task}}.").unwrap();

    // Create agent config
    let config = AgentConfigFile {
        agent: AgentConfig::new("test-agent", "Test Agent", PathBuf::from("prompts/test-agent.md"))
            .with_description("A test agent")
            .with_engine("gemini")
            .with_model("gemini-2.0-flash-exp")
            .with_reasoning_effort(ReasoningEffort::Medium),
    };

    let config_path = category_dir.join("test-agent.toml");
    config.save(&config_path).unwrap();

    // Discover agent
    let options = DiscoveryOptions {
        search_paths: vec![agents_dir.clone()],
        sub_agent_filter: None,
    };

    let discovery = AgentDiscovery::with_options(options);
    let agents = discovery.discover_all().unwrap();

    assert_eq!(agents.len(), 1);
    assert!(agents.contains_key("test-agent"));

    let agent_config = agents.get("test-agent").unwrap();
    assert_eq!(agent_config.id, "test-agent");
    assert_eq!(agent_config.name, "Test Agent");
    assert_eq!(agent_config.engine, Some("gemini".to_string()));

    // Load and render prompt
    let loader = AgentPromptLoader::with_base_path(temp_dir.path());
    let mut context = PromptContext::new();
    context.set("name", "World");
    context.set("task", "test the system");

    let rendered = loader.render_prompt(agent_config, &context).unwrap();
    assert!(rendered.contains("Hello World!"));
    assert!(rendered.contains("test the system"));
}

#[test]
fn test_agent_config_with_file_injection() {
    let temp_dir = TempDir::new().unwrap();

    // Create files
    let file1 = temp_dir.path().join("file1.md");
    let file2 = temp_dir.path().join("file2.md");
    fs::write(&file1, "Content from file 1").unwrap();
    fs::write(&file2, "Content from file 2").unwrap();

    // Create prompt with file injection
    let prompt_dir = temp_dir.path().join("prompts");
    fs::create_dir_all(&prompt_dir).unwrap();
    let prompt_file = prompt_dir.join("test-agent.md");
    fs::write(
        &prompt_file,
        "Please review:\nagent[input:file1.md,file2.md]\n\nThen complete {{task}}.",
    )
    .unwrap();

    // Create agent config
    let config = AgentConfigFile {
        agent: AgentConfig::new("test-agent", "Test Agent", PathBuf::from("prompts/test-agent.md")),
    };

    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir_all(&agents_dir).unwrap();
    let config_path = agents_dir.join("test-agent.toml");
    config.save(&config_path).unwrap();

    // Load and render
    let loader = AgentPromptLoader::with_base_path(temp_dir.path());
    let mut context = PromptContext::new();
    context.set("task", "the review");

    let agent_config = config.agent;
    let rendered = loader.render_prompt(&agent_config, &context).unwrap();

    assert!(rendered.contains("Content from file 1"));
    assert!(rendered.contains("Content from file 2"));
    assert!(rendered.contains("the review"));
}
