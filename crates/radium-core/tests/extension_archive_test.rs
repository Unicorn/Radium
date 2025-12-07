//! Tests for archive-based extension installation.

use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::MANIFEST_FILE;
use radium_core::extensions::ExtensionManager;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use flate2::Compression;
use flate2::write::GzEncoder;
use tar::Builder;

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

fn create_test_archive(temp_dir: &TempDir, name: &str) -> std::path::PathBuf {
    let package_dir = temp_dir.path().join(format!("package-{}", name));
    fs::create_dir_all(&package_dir).unwrap();

    // Create manifest
    let manifest_path = package_dir.join(MANIFEST_FILE);
    let manifest = create_test_manifest(name);
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Create component directories
    fs::create_dir_all(package_dir.join("prompts")).unwrap();
    fs::write(package_dir.join("prompts").join("test.md"), "# Test").unwrap();

    // Create archive
    let archive_path = temp_dir.path().join(format!("{}.tar.gz", name));
    let tar_gz = fs::File::create(&archive_path).unwrap();
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    
    // Add all files from package directory
    tar.append_dir_all(name, &package_dir).unwrap();
    tar.finish().unwrap();

    archive_path
}

#[test]
fn test_install_from_archive() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Create test archive
    let archive_path = create_test_archive(&temp_dir, "archive-ext");

    // Install from archive
    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();
    
    let extension = manager.install_from_archive(&archive_path, options).unwrap();
    assert_eq!(extension.name, "archive-ext");
    assert_eq!(extension.version, "1.0.0");
}

#[test]
fn test_install_from_source_archive() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Create test archive
    let archive_path = create_test_archive(&temp_dir, "source-archive-ext");

    // Install using install_from_source
    let manager = ExtensionManager::with_directory(extensions_dir);
    let options = radium_core::extensions::InstallOptions::default();
    
    let extension = manager.install_from_source(
        archive_path.to_str().unwrap(),
        options
    ).unwrap();
    assert_eq!(extension.name, "source-archive-ext");
}

