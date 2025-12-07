//! Tests for extension update functionality.

use radium_core::extensions::{
    ExtensionManager, ExtensionDiscovery, InstallOptions, UpdateChecker, UpdateInfo,
    MarketplaceClient, VersionComparator,
};
use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::Extension;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_check_all_updates() {
    // Create test extensions
    let ext1 = create_test_extension("ext1", "1.0.0");
    let ext2 = create_test_extension("ext2", "2.0.0");
    let extensions = vec![ext1, ext2];

    // Mock get_latest_version function
    let get_latest = |name: &str| -> Option<(String, Option<String>, Option<String>)> {
        match name {
            "ext1" => Some(("1.1.0".to_string(), None, None)),
            "ext2" => Some(("2.0.0".to_string(), None, None)), // No update
            _ => None,
        }
    };

    let updates = UpdateChecker::check_all_updates(&extensions, get_latest).unwrap();

    // Only ext1 should have an update
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].name, "ext1");
    assert_eq!(updates[0].current_version, "1.0.0");
    assert_eq!(updates[0].new_version, "1.1.0");
}

#[test]
fn test_check_all_updates_no_updates() {
    let ext1 = create_test_extension("ext1", "1.0.0");
    let extensions = vec![ext1];

    let get_latest = |_name: &str| -> Option<(String, Option<String>, Option<String>)> {
        Some(("1.0.0".to_string(), None, None)) // Same version
    };

    let updates = UpdateChecker::check_all_updates(&extensions, get_latest).unwrap();
    assert_eq!(updates.len(), 0);
}

#[test]
fn test_update_info_structure() {
    let update = UpdateInfo {
        name: "test-ext".to_string(),
        current_version: "1.0.0".to_string(),
        new_version: "2.0.0".to_string(),
        description: Some("Bug fixes and improvements".to_string()),
        download_url: Some("https://example.com/ext.tar.gz".to_string()),
    };

    assert_eq!(update.name, "test-ext");
    assert_eq!(update.current_version, "1.0.0");
    assert_eq!(update.new_version, "2.0.0");
    assert!(update.description.is_some());
    assert!(update.download_url.is_some());
}

#[test]
fn test_version_comparison_for_updates() {
    // Test that version comparison works correctly for updates
    assert!(VersionComparator::is_newer("2.0.0", "1.0.0").unwrap());
    assert!(VersionComparator::is_newer("1.1.0", "1.0.0").unwrap());
    assert!(VersionComparator::is_newer("1.0.1", "1.0.0").unwrap());
    assert!(!VersionComparator::is_newer("1.0.0", "1.0.0").unwrap());
    assert!(!VersionComparator::is_newer("0.9.0", "1.0.0").unwrap());
}

fn create_test_extension(name: &str, version: &str) -> Extension {
    let manifest = ExtensionManifest {
        name: name.to_string(),
        version: version.to_string(),
        description: "Test extension".to_string(),
        author: "Test Author".to_string(),
        components: ExtensionComponents::default(),
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    };

    Extension::new(manifest, PathBuf::from("/tmp"))
}

#[test]
fn test_update_checker_with_marketplace() {
    // This test verifies the integration between UpdateChecker and marketplace
    let ext = create_test_extension("test-ext", "1.0.0");
    
    // Simulate marketplace returning newer version
    let get_latest = |_name: &str| -> Option<(String, Option<String>, Option<String>)> {
        Some(("2.0.0".to_string(), 
              Some("Major update with new features".to_string()),
              Some("https://marketplace.example.com/ext.tar.gz".to_string())))
    };

    let updates = UpdateChecker::check_all_updates(&[ext], get_latest).unwrap();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].new_version, "2.0.0");
    assert!(updates[0].description.is_some());
    assert!(updates[0].download_url.is_some());
}

