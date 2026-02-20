#![cfg(feature = "workflow")]

//! End-to-end tests for extension system workflows.
//!
//! Tests complete user workflows from installation to usage to uninstallation.

#![allow(unsafe_code)]

use radium_core::agents::discovery::AgentDiscovery;
use radium_core::commands::CommandRegistry;
use radium_core::extensions::{ExtensionManager, InstallOptions};
use radium_core::workflow::template_discovery::TemplateDiscovery;
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
fn test_hello_world_extension_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join(".radium").join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Set HOME to temp_dir
    let original_home = std::env::var("HOME").ok();
    unsafe {
        std::env::set_var("HOME", temp_dir.path().to_string_lossy().to_string());
    }

    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let hello_world_path = example_extensions_dir().join("hello-world");

    // 1. Install extension
    let options = InstallOptions {
        overwrite: false,
        install_dependencies: false,
        validate_after_install: true,
    };
    let extension = manager.install(&hello_world_path, options).unwrap();
    assert_eq!(extension.name, "hello-world");
    assert_eq!(extension.version, "1.0.0");

    // 2. List extensions
    let extensions = manager.list().unwrap();
    assert!(extensions.iter().any(|e| e.name == "hello-world"));

    // 3. Get extension info
    let extension_info = manager.get("hello-world").unwrap();
    assert!(extension_info.is_some());
    let ext = extension_info.unwrap();
    assert_eq!(ext.name, "hello-world");
    assert!(!ext.manifest.components.prompts.is_empty());

    // 4. Verify prompts are discoverable
    let agents = AgentDiscovery::new().discover_all().unwrap();
    // Note: Prompts from extensions need to be integrated with agent discovery
    // This test verifies the extension structure is correct

    // 5. Uninstall extension
    manager.uninstall("hello-world").unwrap();

    // 6. Verify extension is removed
    let extensions_after = manager.list().unwrap();
    assert!(!extensions_after.iter().any(|e| e.name == "hello-world"));

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_code_review_agents_extension() {
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
    let code_review_path = example_extensions_dir().join("code-review-agents");

    // Install extension
    let options = InstallOptions::default();
    let extension = manager.install(&code_review_path, options).unwrap();
    assert_eq!(extension.name, "code-review-agents");

    // Verify multiple prompts are included
    assert_eq!(extension.manifest.components.prompts.len(), 1); // One glob pattern
    assert!(extension.manifest.components.prompts[0].contains("prompts/review"));

    // Verify extension structure
    let prompts_dir = extension.install_path.join("prompts").join("review");
    assert!(prompts_dir.exists());
    assert!(prompts_dir.join("rust-reviewer.md").exists());
    assert!(prompts_dir.join("typescript-reviewer.md").exists());
    assert!(prompts_dir.join("python-reviewer.md").exists());

    // Cleanup
    manager.uninstall("code-review-agents").unwrap();

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_github_integration_extension() {
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
    let github_path = example_extensions_dir().join("github-integration");

    // Install extension
    let options = InstallOptions::default();
    let extension = manager.install(&github_path, options).unwrap();
    assert_eq!(extension.name, "github-integration");

    // Verify MCP server config exists
    let mcp_dir = extension.install_path.join("mcp");
    assert!(mcp_dir.exists());
    assert!(mcp_dir.join("github-api.json").exists());

    // Verify prompts exist
    let prompts_dir = extension.install_path.join("prompts");
    assert!(prompts_dir.exists());
    assert!(prompts_dir.join("github-pr-agent.md").exists());

    // Cleanup
    manager.uninstall("github-integration").unwrap();

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_extension_with_dependencies() {
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

    // Install dependency first
    let hello_world_path = example_extensions_dir().join("hello-world");
    let options = InstallOptions::default();
    manager.install(&hello_world_path, options.clone()).unwrap();

    // Install extension with dependency
    let complete_toolkit_path = example_extensions_dir().join("complete-toolkit");
    let extension = manager.install(&complete_toolkit_path, options).unwrap();
    assert_eq!(extension.name, "complete-toolkit");
    assert!(extension.manifest.dependencies.contains(&"hello-world".to_string()));

    // Verify both extensions are installed
    let extensions = manager.list().unwrap();
    assert!(extensions.iter().any(|e| e.name == "hello-world"));
    assert!(extensions.iter().any(|e| e.name == "complete-toolkit"));

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

#[test]
fn test_extension_overwrite() {
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
    let hello_world_path = example_extensions_dir().join("hello-world");

    // Install first version
    let options = InstallOptions {
        overwrite: false,
        install_dependencies: false,
        validate_after_install: true,
    };
    let extension1 = manager.install(&hello_world_path, options.clone()).unwrap();
    assert_eq!(extension1.version, "1.0.0");

    // Try to install again without overwrite (should fail)
    let result = manager.install(&hello_world_path, options);
    assert!(result.is_err());

    // Install with overwrite (should succeed)
    let options_overwrite = InstallOptions {
        overwrite: true,
        install_dependencies: false,
        validate_after_install: true,
    };
    let extension2 = manager.install(&hello_world_path, options_overwrite).unwrap();
    assert_eq!(extension2.version, "1.0.0");

    // Cleanup
    manager.uninstall("hello-world").unwrap();

    // Restore
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    }
}

