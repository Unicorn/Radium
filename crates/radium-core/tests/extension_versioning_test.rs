//! Tests for extension versioning and update system.

use radium_core::extensions::installer::{ExtensionManager, InstallOptions};
use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::MANIFEST_FILE;
use radium_core::extensions::versioning::{UpdateChecker, VersionComparator};
use radium_core::extensions::Extension;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_manifest(name: &str, version: &str) -> ExtensionManifest {
    ExtensionManifest {
        name: name.to_string(),
        version: version.to_string(),
        description: format!("Test extension: {}", name),
        author: "Test Author".to_string(),
        components: ExtensionComponents::default(),
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    }
}

fn create_test_package(temp_dir: &TempDir, name: &str, version: &str) -> PathBuf {
    let package_dir = temp_dir.path().join(format!("package-{}-{}", name, version));
    std::fs::create_dir_all(&package_dir).unwrap();

    // Create manifest
    let manifest_path = package_dir.join(MANIFEST_FILE);
    let manifest = create_test_manifest(name, version);
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    std::fs::write(&manifest_path, manifest_json).unwrap();

    package_dir
}

#[test]
fn test_version_comparison() {
    use std::cmp::Ordering;

    // Test basic version comparison
    assert_eq!(
        VersionComparator::compare("1.0.0", "2.0.0").unwrap(),
        Ordering::Less
    );
    assert_eq!(
        VersionComparator::compare("2.0.0", "1.0.0").unwrap(),
        Ordering::Greater
    );
    assert_eq!(
        VersionComparator::compare("1.0.0", "1.0.0").unwrap(),
        Ordering::Equal
    );

    // Test pre-release versions
    assert_eq!(
        VersionComparator::compare("1.0.0-beta", "1.0.0").unwrap(),
        Ordering::Less
    );
}

#[test]
fn test_version_is_newer() {
    assert!(VersionComparator::is_newer("2.0.0", "1.0.0").unwrap());
    assert!(!VersionComparator::is_newer("1.0.0", "2.0.0").unwrap());
    assert!(!VersionComparator::is_newer("1.0.0", "1.0.0").unwrap());
}

#[test]
fn test_version_constraints() {
    // Caret constraint (^1.2.0 matches 1.2.0, 1.3.0, but not 2.0.0)
    assert!(VersionComparator::is_compatible_with("1.2.0", "^1.2.0").unwrap());
    assert!(VersionComparator::is_compatible_with("1.3.0", "^1.2.0").unwrap());
    assert!(!VersionComparator::is_compatible_with("2.0.0", "^1.2.0").unwrap());

    // Tilde constraint (~2.0.0 matches 2.0.0, 2.0.1, but not 2.1.0)
    assert!(VersionComparator::is_compatible_with("2.0.0", "~2.0.0").unwrap());
    assert!(VersionComparator::is_compatible_with("2.0.1", "~2.0.0").unwrap());
    assert!(!VersionComparator::is_compatible_with("2.1.0", "~2.0.0").unwrap());

    // Greater than or equal
    assert!(VersionComparator::is_compatible_with("1.0.0", ">=1.0.0").unwrap());
    assert!(VersionComparator::is_compatible_with("2.0.0", ">=1.0.0").unwrap());
    assert!(!VersionComparator::is_compatible_with("0.9.0", ">=1.0.0").unwrap());
}

#[test]
fn test_update_checker() {
    let manifest = create_test_manifest("test-ext", "1.0.0");
    let extension = Extension::new(manifest, PathBuf::from("/tmp"));

    // Newer version should be detected
    assert!(UpdateChecker::check_for_update(&extension, "2.0.0").unwrap());
    
    // Same version should not be detected
    assert!(!UpdateChecker::check_for_update(&extension, "1.0.0").unwrap());
    
    // Older version should not be detected
    assert!(!UpdateChecker::check_for_update(&extension, "0.9.0").unwrap());
}

#[test]
fn test_update_with_rollback() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_dir).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = InstallOptions::default();

    // Install initial version
    let package_v1 = create_test_package(&temp_dir, "test-ext", "1.0.0");
    let extension_v1 = manager.install(&package_v1, options.clone()).unwrap();
    assert_eq!(extension_v1.version, "1.0.0");

    // Create invalid package for v2 (missing manifest to simulate failure)
    let package_v2_invalid = temp_dir.path().join("package-invalid");
    std::fs::create_dir_all(&package_v2_invalid).unwrap();
    // Don't create manifest - this will cause install to fail

    // Attempt update (should fail and rollback)
    let result = manager.update("test-ext", &package_v2_invalid, options.clone());
    assert!(result.is_err());

    // Verify extension was rolled back (still exists with v1.0.0)
    let extension_after_rollback = manager.get("test-ext").unwrap();
    assert!(extension_after_rollback.is_some());
    assert_eq!(extension_after_rollback.unwrap().version, "1.0.0");
}

#[test]
fn test_update_successful() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_dir).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = InstallOptions::default();

    // Install initial version
    let package_v1 = create_test_package(&temp_dir, "test-ext", "1.0.0");
    manager.install(&package_v1, options.clone()).unwrap();

    // Create valid v2 package
    let package_v2 = create_test_package(&temp_dir, "test-ext", "2.0.0");

    // Update to v2
    let extension_v2 = manager.update("test-ext", &package_v2, options.clone()).unwrap();
    assert_eq!(extension_v2.version, "2.0.0");

    // Verify v2 is installed
    let extension = manager.get("test-ext").unwrap();
    assert!(extension.is_some());
    assert_eq!(extension.unwrap().version, "2.0.0");
}

#[test]
fn test_update_rejects_older_version() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_dir).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = InstallOptions::default();

    // Install v2.0.0
    let package_v2 = create_test_package(&temp_dir, "test-ext", "2.0.0");
    manager.install(&package_v2, options.clone()).unwrap();

    // Try to update to v1.0.0 (should fail)
    let package_v1 = create_test_package(&temp_dir, "test-ext", "1.0.0");
    let result = manager.update("test-ext", &package_v1, options.clone());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not newer"));
}

#[test]
fn test_rollback_method() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_dir).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = InstallOptions::default();

    // Install v1.0.0
    let package_v1 = create_test_package(&temp_dir, "test-ext", "1.0.0");
    manager.install(&package_v1, options.clone()).unwrap();

    // Update to v2.0.0 (creates backup)
    let package_v2 = create_test_package(&temp_dir, "test-ext", "2.0.0");
    manager.update("test-ext", &package_v2, options.clone()).unwrap();

    // Rollback to v1.0.0
    manager.rollback("test-ext").unwrap();

    // Verify v1.0.0 is restored
    let extension = manager.get("test-ext").unwrap();
    assert!(extension.is_some());
    assert_eq!(extension.unwrap().version, "1.0.0");
}

#[test]
fn test_rollback_no_backup() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_dir).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = InstallOptions::default();

    // Install extension (no backup created since no update happened)
    let package = create_test_package(&temp_dir, "test-ext", "1.0.0");
    manager.install(&package, options).unwrap();

    // Try to rollback (should fail - no backup)
    let result = manager.rollback("test-ext");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No backup found"));
}

