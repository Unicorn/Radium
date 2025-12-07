//! Integration tests for extension discovery with AgentDiscovery, TemplateDiscovery, and CommandRegistry.

use radium_core::agents::discovery::AgentDiscovery;
use radium_core::commands::CommandRegistry;
use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::MANIFEST_FILE;
use radium_core::extensions::ExtensionManager;
use radium_core::workflow::template_discovery::TemplateDiscovery;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn create_test_manifest(name: &str) -> ExtensionManifest {
    ExtensionManifest {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: format!("Test extension: {}", name),
        author: "Test Author".to_string(),
        components: ExtensionComponents::default(),
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    }
}

fn create_test_extension_package(
    temp_dir: &TempDir,
    name: &str,
    has_agents: bool,
    has_templates: bool,
    has_commands: bool,
) -> std::path::PathBuf {
    let package_dir = temp_dir.path().join(format!("package-{}", name));
    fs::create_dir_all(&package_dir).unwrap();

    // Create manifest
    let manifest_path = package_dir.join(MANIFEST_FILE);
    let manifest = create_test_manifest(name);
    
    if has_agents {
        fs::create_dir_all(package_dir.join("agents")).unwrap();
        // Create prompts directory for the prompt file
        fs::create_dir_all(package_dir.join("prompts")).unwrap();
        let prompt_content = "# Test Agent\n\nTest agent prompt.";
        fs::write(package_dir.join("prompts").join("test-agent.md"), prompt_content).unwrap();
        
        // Create a test agent config with required prompt_path
        let agent_config = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent from extension"
prompt_path = "prompts/test-agent.md"
"#;
        fs::write(package_dir.join("agents").join("test-agent.toml"), agent_config).unwrap();
    }
    
    if has_templates {
        fs::create_dir_all(package_dir.join("templates")).unwrap();
        // Create a test template
        let template = r#"{
  "name": "test-template",
  "description": "Test template from extension",
  "steps": []
}"#;
        fs::write(package_dir.join("templates").join("test-template.json"), template).unwrap();
    }
    
    if has_commands {
        fs::create_dir_all(package_dir.join("commands")).unwrap();
        // Create a test command
        let command = r#"
name = "test-command"
description = "Test command from extension"
template = "echo 'test'"
"#;
        fs::write(package_dir.join("commands").join("test-command.toml"), command).unwrap();
    }
    
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    package_dir
}

#[test]
fn test_agent_discovery_from_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Set HOME to temp_dir so default paths work
    let original_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());

    // Create extension with agent
    let package_path = create_test_extension_package(&temp_dir, "test-ext", true, false, false);
    
    // Install extension
    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();
    manager.install(&package_path, options).unwrap();

    // Change to temp directory to simulate project root
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Discover agents
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().unwrap();

    // Should find the extension agent
    assert!(agents.contains_key("test-agent"), "Agent not found. Found agents: {:?}", agents.keys().collect::<Vec<_>>());
    let agent = agents.get("test-agent").unwrap();
    assert_eq!(agent.id, "test-agent");
    assert_eq!(agent.name, "Test Agent");

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
fn test_template_discovery_from_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Set HOME to temp_dir so default paths work
    let original_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());

    // Create extension with template
    let package_path = create_test_extension_package(&temp_dir, "test-ext", false, true, false);
    
    // Install extension
    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();
    manager.install(&package_path, options).unwrap();

    // Change to temp directory to simulate project root
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Discover templates
    let discovery = TemplateDiscovery::new();
    let templates = discovery.discover_all().unwrap();

    // Should find the extension template
    assert!(templates.contains_key("test-template"), "Template not found. Found templates: {:?}", templates.keys().collect::<Vec<_>>());
    let template = templates.get("test-template").unwrap();
    assert_eq!(template.name, "test-template");

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
fn test_command_discovery_from_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Set HOME to temp_dir so default paths work
    let original_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());

    // Create extension with command
    let package_path = create_test_extension_package(&temp_dir, "test-ext", false, false, true);
    
    // Install extension
    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();
    manager.install(&package_path, options).unwrap();

    // Discover commands
    let mut registry = CommandRegistry::new();
    registry.discover().unwrap();

    // Should find the extension command with namespace
    let command_opt = registry.get("test-ext:test-command");
    assert!(command_opt.is_some(), "Command not found. Available commands: {:?}", 
        registry.list_commands().iter().map(|c| c.as_str()).collect::<Vec<_>>());
    let command = command_opt.unwrap();
    assert_eq!(command.name, "test-command");
    assert_eq!(command.namespace, Some("test-ext".to_string()));

    // Restore
    if let Some(home) = original_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
fn test_search_path_priority_project_over_extension() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Set HOME to temp_dir so default paths work
    let original_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());

    // Create extension with agent
    let package_path = create_test_extension_package(&temp_dir, "test-ext", true, false, false);
    
    // Install extension
    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();
    manager.install(&package_path, options).unwrap();

    // Create project-local agent with same ID (should take precedence)
    let project_agents_dir = temp_dir.path().join("agents");
    fs::create_dir_all(&project_agents_dir).unwrap();
    // Create prompt file for project agent
    fs::create_dir_all(temp_dir.path().join("prompts")).unwrap();
    fs::write(temp_dir.path().join("prompts").join("project-agent.md"), "# Project Agent\n\nProject agent prompt.").unwrap();
    
    let project_agent_config = r#"
[agent]
id = "test-agent"
name = "Project Agent"
description = "Project agent takes precedence"
prompt_path = "prompts/project-agent.md"
"#;
    fs::write(project_agents_dir.join("test-agent.toml"), project_agent_config).unwrap();

    // Change to temp directory to simulate project root
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Discover agents
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().unwrap();

    // Project agent should take precedence
    assert!(agents.contains_key("test-agent"));
    let agent = agents.get("test-agent").unwrap();
    assert_eq!(agent.name, "Project Agent");

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
fn test_command_namespace_from_extension() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Create extension with command
    let package_path = create_test_extension_package(&temp_dir, "my-extension", false, false, true);
    
    // Install extension
    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();
    manager.install(&package_path, options).unwrap();

    // Discover commands
    let mut registry = CommandRegistry::new();
    registry.discover().unwrap();

    // Command should be namespaced with extension name
    assert!(registry.get("my-extension:test-command").is_some());
    let command = registry.get("my-extension:test-command").unwrap();
    assert_eq!(command.namespace, Some("my-extension".to_string()));
}

