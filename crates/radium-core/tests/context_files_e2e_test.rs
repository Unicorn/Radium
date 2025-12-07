//! End-to-end tests for context files system.

use radium_core::context::ContextFileLoader;
use radium_core::context::ContextManager;
use radium_core::workspace::Workspace;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_e2e_agent_execution_with_context_files() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create context file with project guidelines
    let context_file = temp_dir.path().join("GEMINI.md");
    fs::write(
        &context_file,
        "# Project Guidelines\n\n- Always write tests\n- Use Rust best practices\n- Document code",
    )
    .unwrap();

    // Simulate agent execution by building context
    let mut manager = ContextManager::new(&workspace);
    let context_files = manager.load_context_files(temp_dir.path()).unwrap();

    assert!(context_files.is_some());
    let content = context_files.unwrap();
    assert!(content.contains("Project Guidelines"));
    assert!(content.contains("Always write tests"));
    assert!(content.contains("Rust best practices"));
    assert!(content.contains("Document code"));
}

#[test]
fn test_e2e_hierarchical_context_real_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create project root context file
    let project_file = temp_dir.path().join("GEMINI.md");
    fs::write(&project_file, "# Project Context\n\nProject-level instructions.").unwrap();

    // Create subdirectory with context file
    let subdir = temp_dir.path().join("src");
    fs::create_dir_all(&subdir).unwrap();
    let subdir_file = subdir.join("GEMINI.md");
    fs::write(&subdir_file, "# Subdirectory Context\n\nSubdirectory-specific instructions.")
        .unwrap();

    // Load context from subdirectory (simulating agent running in subdirectory)
    let loader = ContextFileLoader::new(temp_dir.path());
    let content = loader.load_hierarchical(&subdir).unwrap();

    // Should contain both, with project first (lower precedence), then subdirectory
    assert!(content.contains("Project Context"));
    assert!(content.contains("Subdirectory Context"));
    assert!(content.contains("Project-level"));
    assert!(content.contains("Subdirectory-specific"));

    // Verify order: project should come before subdirectory
    let project_pos = content.find("Project Context").unwrap();
    let subdir_pos = content.find("Subdirectory Context").unwrap();
    assert!(project_pos < subdir_pos);
}

#[test]
fn test_e2e_context_files_with_imports_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create imported rules file
    let rules_file = temp_dir.path().join("RULES.md");
    fs::write(&rules_file, "# Coding Rules\n\n1. Use cargo fmt\n2. Write tests\n3. Document APIs")
        .unwrap();

    // Create main context file with import
    let main_file = temp_dir.path().join("GEMINI.md");
    fs::write(
        &main_file,
        "# Project Context\n\nFollow these rules:\n\n@RULES.md\n\nAdditional guidelines here.",
    )
    .unwrap();

    // Load context (simulating workflow execution)
    let loader = ContextFileLoader::new(temp_dir.path());
    let content = loader.load_hierarchical(temp_dir.path()).unwrap();

    // Should contain main content and imported content
    assert!(content.contains("Project Context"));
    assert!(content.contains("Coding Rules"));
    assert!(content.contains("Use cargo fmt"));
    assert!(content.contains("Write tests"));
    assert!(content.contains("Document APIs"));
    assert!(content.contains("Additional guidelines"));
}

#[test]
fn test_e2e_context_file_changes_during_execution() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create initial context file
    let context_file = temp_dir.path().join("GEMINI.md");
    fs::write(&context_file, "# Original Context\n\nOriginal content.").unwrap();

    let mut manager = ContextManager::new(&workspace);

    // First load
    let content1 = manager.load_context_files(temp_dir.path()).unwrap();
    assert!(content1.as_ref().unwrap().contains("Original Context"));
    assert!(content1.as_ref().unwrap().contains("Original content"));

    // Simulate file change during execution
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different mtime
    fs::write(&context_file, "# Updated Context\n\nUpdated content.").unwrap();

    // Second load should detect change and reload (cache invalidation)
    let content2 = manager.load_context_files(temp_dir.path()).unwrap();
    assert!(content2.as_ref().unwrap().contains("Updated Context"));
    assert!(content2.as_ref().unwrap().contains("Updated content"));
    assert!(!content2.as_ref().unwrap().contains("Original Context"));
}

#[test]
fn test_e2e_context_files_with_complex_imports() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create deeply nested import structure
    let level3 = temp_dir.path().join("level3.md");
    fs::write(&level3, "# Level 3\n\nDeep content.").unwrap();

    let level2 = temp_dir.path().join("level2.md");
    fs::write(&level2, "# Level 2\n\n@level3.md\n\nMid content.").unwrap();

    let level1 = temp_dir.path().join("level1.md");
    fs::write(&level1, "# Level 1\n\n@level2.md\n\nTop content.").unwrap();

    // Main context file with nested imports
    let main_file = temp_dir.path().join("GEMINI.md");
    fs::write(&main_file, "# Main Context\n\n@level1.md\n\nMain content.").unwrap();

    // Load context
    let loader = ContextFileLoader::new(temp_dir.path());
    let content = loader.load_hierarchical(temp_dir.path()).unwrap();

    // Should contain all levels
    assert!(content.contains("Main Context"));
    assert!(content.contains("Level 1"));
    assert!(content.contains("Level 2"));
    assert!(content.contains("Level 3"));
    assert!(content.contains("Deep content"));
    assert!(content.contains("Mid content"));
    assert!(content.contains("Top content"));
    assert!(content.contains("Main content"));
}
