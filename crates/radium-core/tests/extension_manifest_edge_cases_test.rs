//! Edge case tests for extension manifest parsing and validation.

use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::MANIFEST_FILE;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn create_test_manifest(name: &str) -> ExtensionManifest {
    ExtensionManifest {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Author".to_string(),
        components: ExtensionComponents::default(),
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    }
}

#[test]
fn test_manifest_with_unicode() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(MANIFEST_FILE);

    let manifest_json = r#"{
        "name": "test-extension",
        "version": "1.0.0",
        "description": "测试扩展 🚀",
        "author": "作者 <author@example.com>",
        "components": {}
    }"#;

    fs::write(&manifest_path, manifest_json).unwrap();
    let manifest = ExtensionManifest::load(&manifest_path).unwrap();

    assert_eq!(manifest.description, "测试扩展 🚀");
    assert_eq!(manifest.author, "作者 <author@example.com>");
}

#[test]
fn test_manifest_with_very_long_strings() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(MANIFEST_FILE);

    let long_description = "a".repeat(10000);
    let manifest_json = format!(
        r#"{{
        "name": "test-extension",
        "version": "1.0.0",
        "description": "{}",
        "author": "Author",
        "components": {{}}
    }}"#,
        long_description
    );

    fs::write(&manifest_path, manifest_json).unwrap();
    let manifest = ExtensionManifest::load(&manifest_path).unwrap();

    assert_eq!(manifest.description.len(), 10000);
}

#[test]
fn test_manifest_with_duplicate_keys() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(MANIFEST_FILE);

    // JSON with duplicate "name" key (last one wins in JSON)
    let manifest_json = r#"{
        "name": "first-name",
        "version": "1.0.0",
        "description": "Test",
        "author": "Author",
        "name": "second-name",
        "components": {}
    }"#;

    fs::write(&manifest_path, manifest_json).unwrap();
    let manifest = ExtensionManifest::load(&manifest_path).unwrap();

    // JSON parser will use the last value
    assert_eq!(manifest.name, "second-name");
}

#[test]
fn test_manifest_with_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(MANIFEST_FILE);

    let invalid_json = r#"{
        "name": "test",
        "version": "1.0.0",
        "description": "Test",
        "author": "Author",
        "components": {
    }"#; // Missing closing brace

    fs::write(&manifest_path, invalid_json).unwrap();
    let result = ExtensionManifest::load(&manifest_path);
    assert!(result.is_err());
}

#[test]
fn test_manifest_with_special_characters_in_name() {
    let invalid_names = vec!["name with spaces", "name@special", "name/slash"];

    for name in invalid_names {
        let mut manifest = create_test_manifest(name);
        let result = manifest.validate();
        assert!(result.is_err(), "Name '{}' should be invalid", name);
    }
}

#[test]
fn test_manifest_with_valid_version_formats() {
    let valid_versions = vec!["1", "1.0", "1.0.0", "2.1.3", "10.20.30"];

    for version in valid_versions {
        let mut manifest = create_test_manifest("test");
        manifest.version = version.to_string();
        let result = manifest.validate();
        assert!(result.is_ok(), "Version '{}' should be valid", version);
    }
}

#[test]
fn test_manifest_with_empty_components() {
    let manifest = create_test_manifest("test");
    assert!(manifest.components.prompts.is_empty());
    assert!(manifest.components.mcp_servers.is_empty());
    assert!(manifest.components.commands.is_empty());
    assert!(manifest.validate().is_ok());
}

#[test]
fn test_manifest_with_many_components() {
    let mut manifest = create_test_manifest("test");
    
    // Add many component paths
    for i in 0..100 {
        manifest.components.prompts.push(format!("prompts/agent{}.md", i));
        manifest.components.commands.push(format!("commands/cmd{}.toml", i));
    }

    assert_eq!(manifest.components.prompts.len(), 100);
    assert_eq!(manifest.components.commands.len(), 100);
    assert!(manifest.validate().is_ok());
}

