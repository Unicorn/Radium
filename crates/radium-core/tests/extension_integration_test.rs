#![cfg(feature = "workflow")]

//! Integration tests for complete extension system workflow.

#![allow(unsafe_code)]

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

fn create_complete_extension_package(temp_dir: &TempDir, name: &str) -> std::path::PathBuf {
    let package_dir = temp_dir.path().join(format!("package-{}", name));
    fs::create_dir_all(&package_dir).unwrap();

    // Create manifest
    let manifest_path = package_dir.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest(name);

    // Create agents directory and agent
    let agents_dir = package_dir.join("agents");
    fs::create_dir_all(&agents_dir).unwrap();
    fs::create_dir_all(package_dir.join("prompts")).unwrap();
    fs::write(package_dir.join("prompts").join("test-agent.md"), "# Test Agent").unwrap();
    
    let agent_config = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent from extension"
prompt_path = "prompts/test-agent.md"
"#;
    fs::write(agents_dir.join("test-agent.toml"), agent_config).unwrap();

    // Create templates directory and template
    let templates_dir = package_dir.join("templates");
    fs::create_dir_all(&templates_dir).unwrap();
    let template = r#"{
  "name": "test-template",
  "description": "Test template from extension",
  "steps": []
}"#;
    fs::write(templates_dir.join("test-template.json"), template).unwrap();

    // Create commands directory and command
    let commands_dir = package_dir.join("commands");
    fs::create_dir_all(&commands_dir).unwrap();
    let command = r#"
name = "test-command"
description = "Test command from extension"
template = "echo 'test'"
"#;
    fs::write(commands_dir.join("test-command.toml"), command).unwrap();

    // Update manifest
    manifest.components.prompts.push("prompts/test-agent.md".to_string());
    manifest.components.commands.push("commands/test-command.toml".to_string());

    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    package_dir
}

#[test]
fn test_complete_extension_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Set HOME to temp_dir
    let original_home = std::env::var("HOME").ok();
    unsafe {
        std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());
    }

    // Create extension package
    let package_path = create_complete_extension_package(&temp_dir, "lifecycle-test");

    // Change to temp directory
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 1. Install extension
    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();
    let extension = manager.install(&package_path, options).unwrap();
    assert_eq!(extension.name, "lifecycle-test");

    // 2. Discover extension components
    let agents = AgentDiscovery::new().discover_all().unwrap();
    assert!(agents.contains_key("test-agent"));

    let templates = TemplateDiscovery::new().discover_all().unwrap();
    assert!(templates.contains_key("test-template"));

    let mut registry = CommandRegistry::new();
    registry.discover().unwrap();
    assert!(registry.get("lifecycle-test:test-command").is_some());

    // 3. Use components (verify they work)
    let agent = agents.get("test-agent").unwrap();
    assert_eq!(agent.name, "Test Agent");

    let template = templates.get("test-template").unwrap();
    assert_eq!(template.name, "test-template");

    let command = registry.get("lifecycle-test:test-command").unwrap();
    assert_eq!(command.name, "test-command");

    // 4. Uninstall extension
    manager.uninstall("lifecycle-test").unwrap();

    // 5. Verify components are no longer available
    let agents_after = AgentDiscovery::new().discover_all().unwrap();
    assert!(!agents_after.contains_key("test-agent"));

    let templates_after = TemplateDiscovery::new().discover_all().unwrap();
    assert!(!templates_after.contains_key("test-template"));

    let mut registry_after = CommandRegistry::new();
    registry_after.discover().unwrap();
    assert!(registry_after.get("lifecycle-test:test-command").is_none());

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_multi_extension_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Set HOME
    let original_home = std::env::var("HOME").ok();
    unsafe {
        std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());
    }

    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();

    // Install base extension
    let base_package = create_test_package(&temp_dir, "base-ext");
    manager.install(&base_package, options.clone()).unwrap();

    // Install extension that depends on base
    let dep_package = temp_dir.path().join("package-dep-ext");
    fs::create_dir_all(&dep_package).unwrap();
    let mut dep_manifest = create_test_manifest("dep-ext");
    dep_manifest.dependencies.push("base-ext".to_string());
    let dep_manifest_path = dep_package.join(MANIFEST_FILE);
    let dep_manifest_json = serde_json::to_string(&dep_manifest).unwrap();
    fs::write(&dep_manifest_path, dep_manifest_json).unwrap();

    // Should succeed since base-ext is installed
    manager.install(&dep_package, options).unwrap();

    // Verify both are installed
    let extensions = manager.list().unwrap();
    assert_eq!(extensions.len(), 2);
    assert!(extensions.iter().any(|e| e.name == "base-ext"));
    assert!(extensions.iter().any(|e| e.name == "dep-ext"));

    // Restore
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

fn create_test_package(temp_dir: &TempDir, name: &str) -> std::path::PathBuf {
    let package_dir = temp_dir.path().join(format!("package-{}", name));
    fs::create_dir_all(&package_dir).unwrap();

    let manifest_path = package_dir.join(MANIFEST_FILE);
    let manifest = create_test_manifest(name);
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    package_dir
}

#[test]
fn test_extension_upgrade() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Set HOME
    let original_home = std::env::var("HOME").ok();
    unsafe {
        std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());
    }

    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();

    // Install v1.0.0
    let v1_package = create_test_package(&temp_dir, "upgrade-test");
    manager.install(&v1_package, options.clone()).unwrap();

    // Create v2.0.0 package
    let v2_package = temp_dir.path().join("package-upgrade-test-v2");
    fs::create_dir_all(&v2_package).unwrap();
    let mut v2_manifest = create_test_manifest("upgrade-test");
    v2_manifest.version = "2.0.0".to_string();
    let v2_manifest_path = v2_package.join(MANIFEST_FILE);
    let v2_manifest_json = serde_json::to_string(&v2_manifest).unwrap();
    fs::write(&v2_manifest_path, v2_manifest_json).unwrap();

    // Update to v2.0.0
    let updated = manager.update("upgrade-test", &v2_package, options).unwrap();
    assert_eq!(updated.version, "2.0.0");

    // Verify old version is gone
    let extension = manager.get("upgrade-test").unwrap().unwrap();
    assert_eq!(extension.version, "2.0.0");

    // Restore
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

