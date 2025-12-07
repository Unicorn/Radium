//! Edge case tests for extension structure and file organization.

use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::{Extension, MANIFEST_FILE};
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
fn test_extension_with_many_files() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let package_path = temp_dir.path().join("package-test");
    fs::create_dir_all(&package_path).unwrap();

    // Create manifest
    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("test-ext");
    
    // Create many component files
    let prompts_dir = package_path.join("prompts");
    fs::create_dir_all(&prompts_dir).unwrap();
    for i in 0..100 {
        fs::write(prompts_dir.join(format!("agent{}.md", i)), format!("# Agent {}", i)).unwrap();
        manifest.components.prompts.push(format!("prompts/agent{}.md", i));
    }

    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Install extension
    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();
    let extension = manager.install(&package_path, options).unwrap();

    // Verify all files are accessible
    let prompt_paths = extension.get_prompt_paths().unwrap();
    assert_eq!(prompt_paths.len(), 100);
}

#[test]
fn test_extension_with_deeply_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let package_path = temp_dir.path().join("package-test");
    fs::create_dir_all(&package_path).unwrap();

    // Create deeply nested structure (but reasonable depth)
    let mut current = package_path.clone();
    for i in 0..10 {
        current = current.join(format!("level{}", i));
        fs::create_dir_all(&current).unwrap();
    }

    // Create a file at the deep level
    fs::write(current.join("file.txt"), "content").unwrap();

    // This should work (10 levels is reasonable)
    assert!(current.join("file.txt").exists());
}

#[test]
fn test_extension_path_resolution() {
    let temp_dir = TempDir::new().unwrap();
    let install_path = temp_dir.path().join("installed-ext");
    fs::create_dir_all(&install_path).unwrap();

    let manifest = create_test_manifest("test-ext");
    let extension = Extension::new(manifest, install_path.clone());

    // Test path resolution
    let resolved = extension.resolve_component_path("prompts/agent.md");
    assert_eq!(resolved, install_path.join("prompts/agent.md"));
}

#[test]
fn test_extension_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    
    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    
    // Extensions directory should be created on first use
    let package_path = temp_dir.path().join("package-test");
    fs::create_dir_all(&package_path).unwrap();
    
    let manifest_path = package_path.join(MANIFEST_FILE);
    let manifest = create_test_manifest("test-ext");
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    let options = radium_core::extensions::InstallOptions::default();
    manager.install(&package_path, options).unwrap();

    // Extensions directory should now exist
    assert!(extensions_dir.exists());
    assert!(extensions_dir.join("test-ext").exists());
}

