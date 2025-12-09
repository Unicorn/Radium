//! Tests for extension conflict detection.

#![cfg(feature = "workflow")]

use radium_core::extensions::conflict::ConflictDetector;
use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn create_test_manifest(name: &str) -> ExtensionManifest {
    ExtensionManifest {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: "Test extension".to_string(),
        author: "Test Author".to_string(),
        components: ExtensionComponents::default(),
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    }
}

#[test]
fn test_detect_agent_conflict() {
    let temp_dir = TempDir::new().unwrap();
    let package_path = temp_dir.path();

    // Create existing agent
    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir_all(&agents_dir).unwrap();
    let existing_agent = r#"
[agent]
id = "existing-agent"
name = "Existing Agent"
description = "Existing"
prompt_path = "prompts/existing.md"
"#;
    fs::write(agents_dir.join("existing-agent.toml"), existing_agent).unwrap();
    fs::create_dir_all(temp_dir.path().join("prompts")).unwrap();
    fs::write(temp_dir.path().join("prompts").join("existing.md"), "# Existing").unwrap();

    // Create extension with conflicting agent
    let ext_agents_dir = package_path.join("agents");
    fs::create_dir_all(&ext_agents_dir).unwrap();
    let conflicting_agent = r#"
[agent]
id = "existing-agent"
name = "Conflicting Agent"
description = "Conflict"
prompt_path = "prompts/conflict.md"
"#;
    fs::write(ext_agents_dir.join("existing-agent.toml"), conflicting_agent).unwrap();

    let manifest = create_test_manifest("test-ext");

    // Change to temp directory to simulate project root
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Check conflicts (should fail)
    let result = ConflictDetector::check_conflicts(&manifest, package_path);
    assert!(result.is_err());

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_detect_dependency_cycle() {
    let mut all_extensions = HashMap::new();
    all_extensions.insert("A".to_string(), vec!["B".to_string()]);
    all_extensions.insert("B".to_string(), vec!["C".to_string()]);
    all_extensions.insert("C".to_string(), vec!["A".to_string()]); // Cycle!

    let mut manifest = create_test_manifest("A");
    manifest.dependencies.push("B".to_string());

    let result = ConflictDetector::detect_dependency_cycles("A", &manifest, &all_extensions);
    assert!(result.is_err());
}

#[test]
fn test_no_conflicts() {
    let temp_dir = TempDir::new().unwrap();
    // Use a subdirectory for the package to avoid conflicts with temp_dir root
    let package_path = temp_dir.path().join("package-test-ext");
    fs::create_dir_all(&package_path).unwrap();

    // Create extension with unique agent
    let ext_agents_dir = package_path.join("agents");
    fs::create_dir_all(&ext_agents_dir).unwrap();
    fs::create_dir_all(package_path.join("prompts")).unwrap();
    fs::write(package_path.join("prompts").join("unique.md"), "# Unique").unwrap();
    
    let unique_agent = r#"
[agent]
id = "unique-agent"
name = "Unique Agent"
description = "Unique"
prompt_path = "prompts/unique.md"
"#;
    fs::write(ext_agents_dir.join("unique-agent.toml"), unique_agent).unwrap();

    let manifest = create_test_manifest("test-ext");

    // Change to temp directory
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Check conflicts (should pass - no existing agents to conflict with)
    let result = ConflictDetector::check_conflicts(&manifest, &package_path);
    if let Err(e) = &result {
        eprintln!("Conflict check failed: {}", e);
    }
    assert!(result.is_ok(), "Expected no conflicts but got: {:?}", result);

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
}

