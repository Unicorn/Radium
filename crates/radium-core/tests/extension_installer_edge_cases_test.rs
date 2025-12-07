//! Edge case tests for extension installer.

use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::MANIFEST_FILE;
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
fn test_concurrent_installation_simulation() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let package1 = create_test_package(&temp_dir, "ext1");
    let package2 = create_test_package(&temp_dir, "ext2");

    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();

    // Install both extensions
    manager.install(&package1, options.clone()).unwrap();
    manager.install(&package2, options).unwrap();

    // Both should be installed
    let extensions = manager.list().unwrap();
    assert_eq!(extensions.len(), 2);
}

#[test]
fn test_installation_with_partial_failure() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let package_path = temp_dir.path().join("package-test");
    fs::create_dir_all(&package_path).unwrap();

    // Create manifest with missing component directory
    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("test-ext");
    manifest.components.prompts.push("prompts/missing.md".to_string());
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Don't create prompts directory - should fail validation
    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();

    let result = manager.install(&package_path, options);
    assert!(result.is_err()); // Should fail due to missing component
}

#[test]
fn test_uninstall_with_dependent_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Install base extension
    let base_package = create_test_package(&temp_dir, "base-ext");
    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();
    manager.install(&base_package, options.clone()).unwrap();

    // Install dependent extension
    let dep_package = temp_dir.path().join("package-dep");
    fs::create_dir_all(&dep_package).unwrap();
    let mut dep_manifest = create_test_manifest("dep-ext");
    dep_manifest.dependencies.push("base-ext".to_string());
    let dep_manifest_path = dep_package.join(MANIFEST_FILE);
    let dep_manifest_json = serde_json::to_string(&dep_manifest).unwrap();
    fs::write(&dep_manifest_path, dep_manifest_json).unwrap();
    manager.install(&dep_package, options).unwrap();

    // Try to uninstall base extension (should fail due to dependency)
    let result = manager.uninstall("base-ext");
    assert!(result.is_err());
}

#[test]
fn test_update_extension() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let package_path = create_test_package(&temp_dir, "test-ext");
    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();

    // Install initial version
    manager.install(&package_path, options.clone()).unwrap();

    // Update (same package, but tests the update path)
    let extension = manager.update("test-ext", &package_path, options).unwrap();
    assert_eq!(extension.name, "test-ext");
}

