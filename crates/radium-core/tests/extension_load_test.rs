//! Load testing for extension system.

use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::MANIFEST_FILE;
use radium_core::extensions::{ExtensionDiscovery, ExtensionManager};
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
fn test_install_many_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();

    // Install 100 extensions sequentially
    for i in 0..100 {
        let package_path = create_test_package(&temp_dir, &format!("ext{}", i));
        manager.install(&package_path, options.clone()).unwrap();
    }

    // Verify all are installed
    let extensions = manager.list().unwrap();
    assert_eq!(extensions.len(), 100);
}

#[test]
fn test_discover_many_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();

    // Install 500 extensions
    for i in 0..500 {
        let package_path = create_test_package(&temp_dir, &format!("ext{}", i));
        manager.install(&package_path, options.clone()).unwrap();
    }

    // Discover all extensions
    let discovery = ExtensionDiscovery::with_options(
        radium_core::extensions::DiscoveryOptions {
            search_paths: vec![extensions_dir],
            validate_structure: false,
        },
    );

    let extensions = discovery.discover_all().unwrap();
    assert_eq!(extensions.len(), 500);
}

#[test]
fn test_extension_with_many_components() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Create extension with 200 component files
    let package_path = temp_dir.path().join("package-large");
    fs::create_dir_all(&package_path).unwrap();

    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("large-ext");

    // Create many component files
    let prompts_dir = package_path.join("prompts");
    fs::create_dir_all(&prompts_dir).unwrap();
    for i in 0..200 {
        fs::write(prompts_dir.join(format!("agent{}.md", i)), format!("# Agent {}", i)).unwrap();
        manifest.components.prompts.push(format!("prompts/agent{}.md", i));
    }

    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Install and verify
    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();
    let extension = manager.install(&package_path, options).unwrap();

    // Verify all components are accessible
    let prompt_paths = extension.get_prompt_paths().unwrap();
    assert_eq!(prompt_paths.len(), 200);
}

