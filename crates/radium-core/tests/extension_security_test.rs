//! Security tests for extension system.

use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::MANIFEST_FILE;
use radium_core::extensions::validator::ExtensionValidator;
use radium_core::extensions::ExtensionManager;
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
fn test_path_traversal_prevention() {
    let temp_dir = TempDir::new().unwrap();
    let package_path = temp_dir.path().join("package-test");
    fs::create_dir_all(&package_path).unwrap();

    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("test-ext");

    // Test various path traversal attempts
    let malicious_paths = vec![
        "../etc/passwd",
        "../../etc/passwd",
        "..\\..\\windows\\system32\\config\\sam",
        "/etc/shadow",
        "C:\\Windows\\System32\\config\\SAM",
    ];

    for malicious_path in malicious_paths {
        manifest.components.prompts.clear();
        manifest.components.prompts.push(malicious_path.to_string());

        let manifest_json = serde_json::to_string(&manifest).unwrap();
        fs::write(&manifest_path, manifest_json).unwrap();

        // Validation should reject path traversal
        let result = ExtensionValidator::validate(&package_path, &manifest);
        assert!(result.is_err(), "Path traversal should be rejected: {}", malicious_path);
    }
}

#[test]
fn test_absolute_path_prevention() {
    let temp_dir = TempDir::new().unwrap();
    let package_path = temp_dir.path().join("package-test");
    fs::create_dir_all(&package_path).unwrap();

    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("test-ext");

    // Test absolute paths
    let absolute_paths = vec!["/etc/passwd", "C:\\Windows\\System32"];

    for abs_path in absolute_paths {
        manifest.components.prompts.clear();
        manifest.components.prompts.push(abs_path.to_string());

        let manifest_json = serde_json::to_string(&manifest).unwrap();
        fs::write(&manifest_path, manifest_json).unwrap();

        let result = ExtensionValidator::validate(&package_path, &manifest);
        assert!(result.is_err(), "Absolute path should be rejected: {}", abs_path);
    }
}

#[test]
fn test_null_byte_prevention() {
    let temp_dir = TempDir::new().unwrap();
    let package_path = temp_dir.path().join("package-test");
    fs::create_dir_all(&package_path).unwrap();

    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("test-ext");

    // Test null byte injection
    manifest.components.prompts.push("prompts\0/../etc/passwd".to_string());

    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    let result = ExtensionValidator::validate(&package_path, &manifest);
    assert!(result.is_err(), "Null byte should be rejected");
}

#[test]
fn test_malicious_manifest_injection() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(MANIFEST_FILE);

    // Test manifest with potential command injection attempts
    let malicious_manifests = vec![
        r#"{
            "name": "test; rm -rf /",
            "version": "1.0.0",
            "description": "Test",
            "author": "Author",
            "components": {}
        }"#,
        r#"{
            "name": "test",
            "version": "1.0.0",
            "description": "Test $(rm -rf /)",
            "author": "Author",
            "components": {}
        }"#,
    ];

    for malicious_json in malicious_manifests {
        fs::write(&manifest_path, malicious_json).unwrap();

        // Should parse as JSON (treats as literal strings)
        let result = ExtensionManifest::load(&manifest_path);
        // Parsing might succeed, but validation should catch invalid names
        if let Ok(manifest) = result {
            let validation_result = manifest.validate();
            // Invalid names should fail validation
            if manifest.name.contains(';') || manifest.name.contains('$') {
                assert!(validation_result.is_err(), "Invalid name should be rejected");
            }
        }
    }
}

#[test]
fn test_component_id_conflict_detection() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Create existing agent
    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir_all(&agents_dir).unwrap();
    fs::create_dir_all(temp_dir.path().join("prompts")).unwrap();
    fs::write(temp_dir.path().join("prompts").join("existing.md"), "# Existing").unwrap();
    
    let existing_agent = r#"
[agent]
id = "conflicting-agent"
name = "Existing Agent"
description = "Existing"
prompt_path = "prompts/existing.md"
"#;
    fs::write(agents_dir.join("conflicting-agent.toml"), existing_agent).unwrap();

    // Create extension with conflicting agent
    let package_path = temp_dir.path().join("package-conflict");
    fs::create_dir_all(&package_path).unwrap();
    let ext_agents_dir = package_path.join("agents");
    fs::create_dir_all(&ext_agents_dir).unwrap();
    fs::create_dir_all(package_path.join("prompts")).unwrap();
    fs::write(package_path.join("prompts").join("conflict.md"), "# Conflict").unwrap();
    
    let conflicting_agent = r#"
[agent]
id = "conflicting-agent"
name = "Conflicting Agent"
description = "Conflict"
prompt_path = "prompts/conflict.md"
"#;
    fs::write(ext_agents_dir.join("conflicting-agent.toml"), conflicting_agent).unwrap();

    let manifest_path = package_path.join(MANIFEST_FILE);
    let manifest = create_test_manifest("conflict-ext");
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Change to temp directory
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();

    // Installation should fail due to conflict
    let result = manager.install(&package_path, options);
    assert!(result.is_err(), "Conflict should be detected");

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
}

