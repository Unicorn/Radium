//! Tests for extension validation.

use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::validator::ExtensionValidator;
use radium_core::extensions::structure::MANIFEST_FILE;
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
fn test_validate_valid_extension() {
    let temp_dir = TempDir::new().unwrap();
    let package_path = temp_dir.path();

    // Create manifest
    let manifest_path = package_path.join(MANIFEST_FILE);
    let manifest = create_test_manifest("test-ext");
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Create component directories
    fs::create_dir_all(package_path.join("prompts")).unwrap();
    fs::create_dir_all(package_path.join("commands")).unwrap();

    // Create component files
    fs::write(package_path.join("prompts").join("test.md"), "# Test").unwrap();
    let command_toml = r#"
name = "test-command"
description = "Test command"
template = "echo test"
"#;
    fs::write(package_path.join("commands").join("test.toml"), command_toml).unwrap();

    // Update manifest with components
    let mut manifest = manifest;
    manifest.components.prompts.push("prompts/test.md".to_string());
    manifest.components.commands.push("commands/test.toml".to_string());

    // Validate
    let result = ExtensionValidator::validate(package_path, &manifest);
    assert!(result.is_ok());
}

#[test]
fn test_validate_missing_component_file() {
    let temp_dir = TempDir::new().unwrap();
    let package_path = temp_dir.path();

    // Create manifest
    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("test-ext");
    manifest.components.prompts.push("prompts/missing.md".to_string());
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Validate (should fail because file doesn't exist)
    let result = ExtensionValidator::validate(package_path, &manifest);
    assert!(result.is_err());
}

#[test]
fn test_validate_path_traversal() {
    let temp_dir = TempDir::new().unwrap();
    let package_path = temp_dir.path();

    // Create manifest with path traversal
    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("test-ext");
    manifest.components.prompts.push("../etc/passwd".to_string());
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Validate (should fail due to path traversal)
    let result = ExtensionValidator::validate(package_path, &manifest);
    assert!(result.is_err());
}

#[test]
fn test_validate_invalid_command_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let package_path = temp_dir.path();

    // Create manifest
    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("test-ext");
    manifest.components.commands.push("commands/invalid.toml".to_string());
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Create invalid TOML file
    fs::create_dir_all(package_path.join("commands")).unwrap();
    fs::write(package_path.join("commands").join("invalid.toml"), "invalid toml content {").unwrap();

    // Validate (should fail due to syntax error)
    let result = ExtensionValidator::validate(package_path, &manifest);
    assert!(result.is_err());
}

