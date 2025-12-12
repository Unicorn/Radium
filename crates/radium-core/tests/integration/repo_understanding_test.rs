//! Integration tests for repo understanding features.

use radium_core::search::{search_code, SearchOptions, filters::FileTypeFilter};
use radium_core::workspace::IgnoreWalker;
use radium_core::analysis::{RustAnalyzer, TypeScriptAnalyzer, find_definition};
use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_end_to_end_scan_search_find() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create test files
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn calculate(x: i32) -> i32 { x * 2 }").unwrap();
    fs::write(root.join("src/main.rs"), "fn main() { let result = calculate(5); }").unwrap();

    // Test 1: Scan with ignore support
    let walker = IgnoreWalker::new(root);
    let files: Vec<PathBuf> = walker.build().collect();
    assert!(files.iter().any(|f| f.ends_with("src/lib.rs")));
    assert!(files.iter().any(|f| f.ends_with("src/main.rs")));

    // Test 2: Search code
    let search_options = SearchOptions {
        pattern: "calculate".to_string(),
        context_before: 1,
        context_after: 1,
        file_filter: FileTypeFilter::Language("rust".to_string()),
        max_results: 10,
        root: root.to_path_buf(),
    };
    let results = search_code(search_options).unwrap();
    assert_eq!(results.len(), 2); // Definition and usage

    // Test 3: Find definition
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let definition = find_definition(&source, root.join("src/lib.rs"), "calculate", "rust").unwrap();
    assert!(definition.is_some());
    assert_eq!(definition.unwrap().name, "calculate");
}

#[test]
fn test_multi_language_repository() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::write(root.join("lib.rs"), "pub fn rust_fn() {}").unwrap();
    fs::write(root.join("lib.ts"), "export function ts_fn() {}").unwrap();

    // Search Rust
    let rust_options = SearchOptions {
        pattern: "rust_fn".to_string(),
        context_before: 0,
        context_after: 0,
        file_filter: FileTypeFilter::Language("rust".to_string()),
        max_results: 10,
        root: root.to_path_buf(),
    };
    let rust_results = search_code(rust_options).unwrap();
    assert_eq!(rust_results.len(), 1);

    // Search TypeScript
    let ts_options = SearchOptions {
        pattern: "ts_fn".to_string(),
        context_before: 0,
        context_after: 0,
        file_filter: FileTypeFilter::Language("typescript".to_string()),
        max_results: 10,
        root: root.to_path_buf(),
    };
    let ts_results = search_code(ts_options).unwrap();
    assert_eq!(ts_results.len(), 1);
}

#[test]
fn test_symbol_extraction_rust() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let source = r#"
pub struct User {
    name: String,
}

pub fn create_user(name: String) -> User {
    User { name }
}
"#;

    fs::write(root.join("test.rs"), source).unwrap();

    let mut analyzer = RustAnalyzer::new();
    let symbols = analyzer.extract_symbols(source, root.join("test.rs")).unwrap();
    
    assert!(symbols.iter().any(|s| s.name == "User" && matches!(s.kind, radium_core::analysis::SymbolKind::Struct)));
    assert!(symbols.iter().any(|s| s.name == "create_user" && matches!(s.kind, radium_core::analysis::SymbolKind::Function)));
}

#[test]
fn test_symbol_extraction_typescript() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let source = r#"
export class User {
    constructor(public name: string) {}
}

export function createUser(name: string): User {
    return new User(name);
}
"#;

    fs::write(root.join("test.ts"), source).unwrap();

    let mut analyzer = TypeScriptAnalyzer::new();
    let symbols = analyzer.extract_symbols(source, root.join("test.ts"), false).unwrap();
    
    assert!(symbols.iter().any(|s| s.name == "User"));
    assert!(symbols.iter().any(|s| s.name == "createUser"));
}
