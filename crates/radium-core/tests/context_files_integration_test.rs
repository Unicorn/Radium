//! Integration tests for context files system.

use radium_core::context::ContextFileLoader;
use radium_core::context::ContextManager;
use radium_core::workspace::Workspace;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_hierarchical_loading_integration() {
    let temp_dir = TempDir::new().unwrap();
    let _workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create global context file (simulated)
    // Note: In real usage, this would be at ~/.radium/GEMINI.md
    // For testing, we'll just test project and subdirectory

    // Create project root context file
    let project_file = temp_dir.path().join("GEMINI.md");
    fs::write(&project_file, "# Project Context\n\nProject-level instructions.").unwrap();

    // Create subdirectory with context file
    let subdir = temp_dir.path().join("src");
    fs::create_dir_all(&subdir).unwrap();
    let subdir_file = subdir.join("GEMINI.md");
    fs::write(&subdir_file, "# Subdirectory Context\n\nSubdirectory-level instructions.").unwrap();

    let loader = ContextFileLoader::new(temp_dir.path());
    let content = loader.load_hierarchical(&subdir).unwrap();

    // Should contain both, with project first (lower precedence)
    assert!(content.contains("Project Context"));
    assert!(content.contains("Subdirectory Context"));
    assert!(content.contains("Project-level"));
    assert!(content.contains("Subdirectory-level"));

    // Verify order: project should come before subdirectory
    let project_pos = content.find("Project Context").unwrap();
    let subdir_pos = content.find("Subdirectory Context").unwrap();
    assert!(project_pos < subdir_pos);
}

#[test]
fn test_context_files_with_imports_integration() {
    let temp_dir = TempDir::new().unwrap();
    let _workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create imported file
    let imported_file = temp_dir.path().join("rules.md");
    fs::write(&imported_file, "# Rules\n\n- Use Rust\n- Follow conventions").unwrap();

    // Create main context file with import
    let main_file = temp_dir.path().join("GEMINI.md");
    fs::write(&main_file, "# Main Context\n\n@rules.md\n\nMore instructions.").unwrap();

    let loader = ContextFileLoader::new(temp_dir.path());
    let content = loader.load_hierarchical(temp_dir.path()).unwrap();

    assert!(content.contains("Main Context"));
    assert!(content.contains("Rules"));
    assert!(content.contains("Use Rust"));
    assert!(content.contains("Follow conventions"));
    assert!(content.contains("More instructions"));
}

#[test]
fn test_context_manager_with_context_files() {
    let temp_dir = TempDir::new().unwrap();
    let _workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create context file
    let context_file = temp_dir.path().join("GEMINI.md");
    fs::write(&context_file, "# Project Guidelines\n\nAlways write tests.").unwrap();

    let mut manager = ContextManager::new(&workspace);
    let context_files = manager.load_context_files(temp_dir.path()).unwrap();

    assert!(context_files.is_some());
    let content = context_files.unwrap();
    assert!(content.contains("Project Guidelines"));
    assert!(content.contains("Always write tests"));
}

#[test]
fn test_context_files_missing_handling() {
    let temp_dir = TempDir::new().unwrap();
    let _workspace = Workspace::create(temp_dir.path()).unwrap();

    // No context files exist
    let loader = ContextFileLoader::new(temp_dir.path());
    let content = loader.load_hierarchical(temp_dir.path()).unwrap();
    assert!(content.is_empty());

    let mut manager = ContextManager::new(&workspace);
    let context_files = manager.load_context_files(temp_dir.path()).unwrap();
    assert!(context_files.is_none());
}

#[test]
fn test_circular_import_detection_integration() {
    let temp_dir = TempDir::new().unwrap();
    let _workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create file1 that imports file2
    let file1 = temp_dir.path().join("file1.md");
    fs::write(&file1, "# File 1\n\n@file2.md").unwrap();

    // Create file2 that imports file1 (circular)
    let file2 = temp_dir.path().join("file2.md");
    fs::write(&file2, "# File 2\n\n@file1.md").unwrap();

    let loader = ContextFileLoader::new(temp_dir.path());
    let content = fs::read_to_string(&file1).unwrap();
    let result = loader.process_imports(&content, temp_dir.path());

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(format!("{:?}", e).contains("Circular"));
    }
}

#[test]
fn test_nested_imports_integration() {
    let temp_dir = TempDir::new().unwrap();
    let _workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create deeply nested imports
    let file3 = temp_dir.path().join("file3.md");
    fs::write(&file3, "# File 3\n\nDeep content.").unwrap();

    let file2 = temp_dir.path().join("file2.md");
    fs::write(&file2, "# File 2\n\n@file3.md").unwrap();

    let file1 = temp_dir.path().join("file1.md");
    fs::write(&file1, "# File 1\n\n@file2.md").unwrap();

    let loader = ContextFileLoader::new(temp_dir.path());
    let content = fs::read_to_string(&file1).unwrap();
    let result = loader.process_imports(&content, temp_dir.path()).unwrap();

    assert!(result.contains("File 1"));
    assert!(result.contains("File 2"));
    assert!(result.contains("File 3"));
    assert!(result.contains("Deep content"));
}

#[test]
fn test_context_files_in_build_context() {
    let temp_dir = TempDir::new().unwrap();
    let _workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create context file
    let context_file = temp_dir.path().join("GEMINI.md");
    fs::write(&context_file, "# Context\n\nProject context here.").unwrap();

    // Create a test file for injection
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();

    let mut manager = ContextManager::new(&workspace);
    // Note: build_context uses current_dir, so we can't easily test it in integration
    // But we can test load_context_files directly
    let context_files = manager.load_context_files(temp_dir.path()).unwrap();
    assert!(context_files.is_some());
    assert!(context_files.unwrap().contains("Project context"));
}

