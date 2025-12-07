//! Performance benchmarks for context file operations.
//!
//! This benchmark suite establishes baseline performance metrics for:
//! - Hierarchical context file loading
//! - Import processing with various depths
//! - Context file discovery across directory trees
//!
//! Expected performance characteristics:
//! - Hierarchical loading: < 1ms for typical files (< 10KB)
//! - Import processing: < 5ms for 10-level deep imports
//! - Discovery: < 100ms for workspaces with 100+ files

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use radium_core::context::ContextFileLoader;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Creates a context file with the specified size (in bytes).
fn create_context_file(path: &Path, size: usize) {
    let content = "# Context File\n\n".to_string()
        + &"x".repeat(size.saturating_sub(20))
        + "\n";
    fs::write(path, content).unwrap();
}

/// Creates a workspace with the specified number of context files.
fn create_workspace_with_files(num_files: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create project root context file
    let project_file = workspace_root.join("GEMINI.md");
    create_context_file(&project_file, 1000);

    // Create subdirectories with context files
    for i in 0..num_files {
        let subdir = workspace_root.join(format!("dir{}", i));
        fs::create_dir_all(&subdir).unwrap();
        let subdir_file = subdir.join("GEMINI.md");
        create_context_file(&subdir_file, 500);
    }

    temp_dir
}

/// Creates a file with nested imports.
fn create_nested_imports(
    base_path: &Path,
    depth: usize,
    current_depth: usize,
    file_name: &str,
) -> std::path::PathBuf {
    let file_path = base_path.join(file_name);
    let content = if current_depth < depth {
        let next_file = format!("import{}.md", current_depth + 1);
        let _ = create_nested_imports(base_path, depth, current_depth + 1, &next_file);
        format!("# File {}\n\n@{}", current_depth, next_file)
    } else {
        format!("# File {} (leaf)", current_depth)
    };
    fs::write(&file_path, content).unwrap();
    file_path
}

fn benchmark_hierarchical_loading_small(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create small context file (< 1KB)
    let project_file = workspace_root.join("GEMINI.md");
    create_context_file(&project_file, 500);

    let loader = ContextFileLoader::new(workspace_root);

    c.bench_function("hierarchical_loading_small", |b| {
        b.iter(|| {
            black_box(loader.load_hierarchical(black_box(workspace_root))).unwrap();
        });
    });
}

fn benchmark_hierarchical_loading_medium(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create medium context file (1-10KB)
    let project_file = workspace_root.join("GEMINI.md");
    create_context_file(&project_file, 5000);

    let loader = ContextFileLoader::new(workspace_root);

    c.bench_function("hierarchical_loading_medium", |b| {
        b.iter(|| {
            black_box(loader.load_hierarchical(black_box(workspace_root))).unwrap();
        });
    });
}

fn benchmark_hierarchical_loading_large(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create large context file (10-100KB)
    let project_file = workspace_root.join("GEMINI.md");
    create_context_file(&project_file, 50000);

    let loader = ContextFileLoader::new(workspace_root);

    c.bench_function("hierarchical_loading_large", |b| {
        b.iter(|| {
            black_box(loader.load_hierarchical(black_box(workspace_root))).unwrap();
        });
    });
}

fn benchmark_hierarchical_loading_multiple_levels(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create project root context file
    let project_file = workspace_root.join("GEMINI.md");
    create_context_file(&project_file, 1000);

    // Create subdirectory with context file
    let subdir = workspace_root.join("src");
    fs::create_dir_all(&subdir).unwrap();
    let subdir_file = subdir.join("GEMINI.md");
    create_context_file(&subdir_file, 1000);

    let loader = ContextFileLoader::new(workspace_root);

    c.bench_function("hierarchical_loading_multiple_levels", |b| {
        b.iter(|| {
            black_box(loader.load_hierarchical(black_box(&subdir))).unwrap();
        });
    });
}

fn benchmark_import_processing_single(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create imported file
    let imported_file = workspace_root.join("imported.md");
    create_context_file(&imported_file, 1000);

    // Create main file with single import
    let main_file = workspace_root.join("main.md");
    fs::write(&main_file, "# Main\n\n@imported.md").unwrap();

    let loader = ContextFileLoader::new(workspace_root);
    let content = fs::read_to_string(&main_file).unwrap();

    c.bench_function("import_processing_single", |b| {
        b.iter(|| {
            black_box(loader.process_imports(black_box(&content), black_box(workspace_root)))
                .unwrap();
        });
    });
}

fn benchmark_import_processing_nested(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create 3-level nested imports
    let _ = create_nested_imports(workspace_root, 3, 1, "import1.md");
    let main_file = workspace_root.join("main.md");
    fs::write(&main_file, "# Main\n\n@import1.md").unwrap();

    let loader = ContextFileLoader::new(workspace_root);
    let content = fs::read_to_string(&main_file).unwrap();

    c.bench_function("import_processing_nested_3_levels", |b| {
        b.iter(|| {
            black_box(loader.process_imports(black_box(&content), black_box(workspace_root)))
                .unwrap();
        });
    });
}

fn benchmark_import_processing_deep(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create 10-level deep imports
    let _ = create_nested_imports(workspace_root, 10, 1, "import1.md");
    let main_file = workspace_root.join("main.md");
    fs::write(&main_file, "# Main\n\n@import1.md").unwrap();

    let loader = ContextFileLoader::new(workspace_root);
    let content = fs::read_to_string(&main_file).unwrap();

    c.bench_function("import_processing_deep_10_levels", |b| {
        b.iter(|| {
            black_box(loader.process_imports(black_box(&content), black_box(workspace_root)))
                .unwrap();
        });
    });
}

fn benchmark_import_processing_multiple(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create multiple imported files
    for i in 1..=5 {
        let imported_file = workspace_root.join(format!("import{}.md", i));
        create_context_file(&imported_file, 1000);
    }

    // Create main file with multiple imports
    let main_file = workspace_root.join("main.md");
    fs::write(
        &main_file,
        "# Main\n\n@import1.md\n\n@import2.md\n\n@import3.md\n\n@import4.md\n\n@import5.md",
    )
    .unwrap();

    let loader = ContextFileLoader::new(workspace_root);
    let content = fs::read_to_string(&main_file).unwrap();

    c.bench_function("import_processing_multiple", |b| {
        b.iter(|| {
            black_box(loader.process_imports(black_box(&content), black_box(workspace_root)))
                .unwrap();
        });
    });
}

fn benchmark_discovery_small(c: &mut Criterion) {
    let temp_dir = create_workspace_with_files(5);
    let workspace_root = temp_dir.path();

    let loader = ContextFileLoader::new(workspace_root);

    c.bench_function("discovery_small_workspace", |b| {
        b.iter(|| {
            black_box(loader.discover_context_files()).unwrap();
        });
    });
}

fn benchmark_discovery_medium(c: &mut Criterion) {
    let temp_dir = create_workspace_with_files(50);
    let workspace_root = temp_dir.path();

    let loader = ContextFileLoader::new(workspace_root);

    c.bench_function("discovery_medium_workspace", |b| {
        b.iter(|| {
            black_box(loader.discover_context_files()).unwrap();
        });
    });
}

fn benchmark_discovery_large(c: &mut Criterion) {
    let temp_dir = create_workspace_with_files(200);
    let workspace_root = temp_dir.path();

    let loader = ContextFileLoader::new(workspace_root);

    c.bench_function("discovery_large_workspace", |b| {
        b.iter(|| {
            black_box(loader.discover_context_files()).unwrap();
        });
    });
}

criterion_group!(
    benches,
    benchmark_hierarchical_loading_small,
    benchmark_hierarchical_loading_medium,
    benchmark_hierarchical_loading_large,
    benchmark_hierarchical_loading_multiple_levels,
    benchmark_import_processing_single,
    benchmark_import_processing_nested,
    benchmark_import_processing_deep,
    benchmark_import_processing_multiple,
    benchmark_discovery_small,
    benchmark_discovery_medium,
    benchmark_discovery_large
);
criterion_main!(benches);

