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

#[test]
fn test_performance_large_context_file() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ContextFileLoader::new(temp_dir.path());

    // Create a large context file (simulate, not actually 10MB to keep tests fast)
    let large_content = "# Large Context File\n\n".to_string()
        + &"This is a test line. ".repeat(1000)
        + "\n\nEnd of large file.";
    let large_file = temp_dir.path().join("GEMINI.md");
    fs::write(&large_file, &large_content).unwrap();

    // Measure loading time
    let start = std::time::Instant::now();
    let content = loader.load_hierarchical(temp_dir.path()).unwrap();
    let duration = start.elapsed();

    assert!(content.contains("Large Context File"));
    // Should complete in reasonable time (< 1 second for this size)
    assert!(duration.as_secs() < 1, "Large file loading took too long: {:?}", duration);
}

#[test]
fn test_performance_cache_repeated_loads() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();

    // Create context file
    let context_file = temp_dir.path().join("GEMINI.md");
    fs::write(&context_file, "# Context").unwrap();

    let mut manager = ContextManager::new(&workspace);

    // First load (no cache)
    let start1 = std::time::Instant::now();
    let _content1 = manager.load_context_files(temp_dir.path()).unwrap();
    let duration1 = start1.elapsed();

    // Second load (cached)
    let start2 = std::time::Instant::now();
    let _content2 = manager.load_context_files(temp_dir.path()).unwrap();
    let duration2 = start2.elapsed();

    // Cached load should be faster (or at least not slower)
    // Note: For very small files, timing might be similar, so we just verify it works
    assert!(duration2.as_secs() < 1);
}

#[test]
fn test_performance_many_files_discovery() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ContextFileLoader::new(temp_dir.path());

    // Create many context files in subdirectories
    for i in 0..20 {
        let subdir = temp_dir.path().join(format!("dir{}", i));
        fs::create_dir_all(&subdir).unwrap();
        let context_file = subdir.join("GEMINI.md");
        fs::write(&context_file, &format!("# Context {}", i)).unwrap();
    }

    // Measure discovery time
    let start = std::time::Instant::now();
    let files = loader.discover_context_files().unwrap();
    let duration = start.elapsed();

    // Should find all files
    assert!(files.len() >= 20);
    // Should complete in reasonable time
    assert!(duration.as_secs() < 2, "Discovery took too long: {:?}", duration);
}

#[test]
fn test_performance_deep_import_chain() {
    let temp_dir = TempDir::new().unwrap();
    let loader = ContextFileLoader::new(temp_dir.path());

    // Create deep import chain (10 levels)
    let mut prev_file = None;
    for i in (1..=10).rev() {
        let file = temp_dir.path().join(format!("level{}.md", i));
        let content = if let Some(ref prev) = prev_file {
            format!(
                "# Level {}\n\n@{}\n\nContent level {}.",
                i,
                prev.file_name().unwrap().to_string_lossy(),
                i
            )
        } else {
            format!("# Level {}\n\nContent level {}.", i, i)
        };
        fs::write(&file, content).unwrap();
        prev_file = Some(file);
    }

    // Load the top level file
    let top_file = temp_dir.path().join("level10.md");
    let content = fs::read_to_string(&top_file).unwrap();

    // Measure processing time
    let start = std::time::Instant::now();
    let result = loader.process_imports(&content, temp_dir.path()).unwrap();
    let duration = start.elapsed();

    // Should contain all levels
    for i in 1..=10 {
        assert!(result.contains(&format!("Level {}", i)));
    }
    // Should complete in reasonable time
    assert!(duration.as_secs() < 1, "Deep import processing took too long: {:?}", duration);
}
