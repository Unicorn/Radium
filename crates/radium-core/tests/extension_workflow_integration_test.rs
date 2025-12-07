//! Integration tests for extension workflow templates.
//!
//! Tests that workflow templates from extensions are properly discovered and usable.

#![allow(unsafe_code)]

use radium_core::extensions::{ExtensionManager, InstallOptions};
use radium_core::workflow::template_discovery::TemplateDiscovery;
use radium_core::workflow::templates::WorkflowTemplate;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to get the path to example extensions.
fn example_extensions_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/radium-core
    path.pop(); // crates
    path.pop(); // root
    path.push("examples");
    path.push("extensions");
    path
}

#[test]
fn test_custom_workflows_extension_discovery() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let original_home = std::env::var("HOME").ok();
    unsafe {
        std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());
    }

    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let workflows_path = example_extensions_dir().join("custom-workflows");

    // Install extension
    let options = InstallOptions::default();
    let extension = manager.install(&workflows_path, options).unwrap();
    assert_eq!(extension.name, "custom-workflows");

    // Verify templates directory exists
    let templates_dir = extension.install_path.join("templates");
    assert!(templates_dir.exists());
    assert!(templates_dir.join("code-review-workflow.json").exists());
    assert!(templates_dir.join("deployment-workflow.json").exists());

    // Discover workflow templates
    let discovery = TemplateDiscovery::new();
    let templates = discovery.discover_all().unwrap();

    // Verify templates are discoverable
    // Note: Template discovery looks in extension directories automatically
    assert!(templates.contains_key("code-review-workflow") || templates.contains_key("deployment-workflow"));

    // Cleanup
    manager.uninstall("custom-workflows").unwrap();

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_workflow_template_loading_from_extension() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let original_home = std::env::var("HOME").ok();
    unsafe {
        std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());
    }

    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir);
    let workflows_path = example_extensions_dir().join("custom-workflows");

    // Install extension
    let options = InstallOptions::default();
    manager.install(&workflows_path, options).unwrap();

    // Load workflow template directly
    let extension = manager.get("custom-workflows").unwrap().unwrap();
    let template_path = extension.install_path.join("templates").join("code-review-workflow.json");

    // Verify template can be loaded
    let template = WorkflowTemplate::load_from_file(&template_path);
    assert!(template.is_ok());
    let workflow = template.unwrap();
    assert_eq!(workflow.name, "code-review-workflow");
    assert!(!workflow.steps.is_empty());

    // Cleanup
    manager.uninstall("custom-workflows").unwrap();

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_complete_toolkit_workflow_template() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let original_home = std::env::var("HOME").ok();
    unsafe {
        std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());
    }

    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir);

    // Install dependencies first
    let hello_world_path = example_extensions_dir().join("hello-world");
    let options = InstallOptions {
        overwrite: false,
        install_dependencies: true,
        validate_after_install: true,
    };
    manager.install(&hello_world_path, options.clone()).unwrap();

    // Install complete-toolkit
    let toolkit_path = example_extensions_dir().join("complete-toolkit");
    let extension = manager.install(&toolkit_path, options).unwrap();
    assert_eq!(extension.name, "complete-toolkit");

    // Verify workflow template exists
    let template_path = extension.install_path.join("templates").join("full-stack-workflow.json");
    assert!(template_path.exists());

    // Load and verify template
    let template = WorkflowTemplate::load_from_file(&template_path).unwrap();
    assert_eq!(template.name, "full-stack-workflow");
    assert!(!template.steps.is_empty());

    // Cleanup
    manager.uninstall("complete-toolkit").unwrap();
    manager.uninstall("hello-world").unwrap();

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

