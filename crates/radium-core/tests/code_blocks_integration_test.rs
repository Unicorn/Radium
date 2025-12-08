//! Integration tests for code block system.

use radium_core::code_blocks::{CodeBlockParser, CodeBlockStore};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_end_to_end_parsing_and_storage() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Sample markdown with multiple code blocks
    let markdown = r#"Here's some Rust code:

```rust
fn main() {
    println!("Hello, world!");
}
```

And some Python:

```python
def greet():
    print("Hello, world!")
```

And plain text:

```
just some text
```"#;

    // Parse blocks
    let blocks = CodeBlockParser::parse(markdown);
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].index, 1);
    assert_eq!(blocks[0].language, Some("rust".to_string()));
    assert_eq!(blocks[1].index, 2);
    assert_eq!(blocks[1].language, Some("python".to_string()));
    assert_eq!(blocks[2].index, 3);
    assert_eq!(blocks[2].language, None);

    // Store blocks
    let mut store = CodeBlockStore::new(workspace_root, "test-session".to_string()).unwrap();
    store.store_blocks("test-agent", blocks.clone()).unwrap();

    // Retrieve blocks
    let retrieved = store.list_blocks(None).unwrap();
    assert_eq!(retrieved.len(), 3);

    // Verify content
    assert!(retrieved[0].content.contains("fn main()"));
    assert!(retrieved[1].content.contains("def greet()"));
    assert_eq!(retrieved[2].content, "just some text");
}

#[test]
fn test_block_selection_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Store 10 blocks
    let blocks: Vec<_> = (1..=10)
        .map(|i| radium_core::code_blocks::CodeBlock {
            index: i,
            language: Some("rust".to_string()),
            content: format!("fn block_{}() {{}}", i),
            file_hint: None,
            start_line: i * 2,
        })
        .collect();

    let mut store = CodeBlockStore::new(workspace_root, "test-session".to_string()).unwrap();
    store.store_blocks("test-agent", blocks).unwrap();

    // Test single selection
    let selected = store
        .get_blocks(radium_core::code_blocks::BlockSelector::Single(5))
        .unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].index, 5);

    // Test range selection
    let selected = store
        .get_blocks(radium_core::code_blocks::BlockSelector::Range(2, 5))
        .unwrap();
    assert_eq!(selected.len(), 4);
    assert_eq!(selected[0].index, 2);
    assert_eq!(selected[3].index, 5);

    // Test multiple selection
    let selected = store
        .get_blocks(radium_core::code_blocks::BlockSelector::Multiple(vec![1, 3, 5, 7]))
        .unwrap();
    assert_eq!(selected.len(), 4);
    assert_eq!(selected[0].index, 1);
    assert_eq!(selected[1].index, 3);
    assert_eq!(selected[2].index, 5);
    assert_eq!(selected[3].index, 7);
}

#[test]
fn test_performance_large_response() {
    // Create markdown with 50 code blocks
    let mut markdown = String::new();
    for i in 1..=50 {
        markdown.push_str(&format!("Block {}:\n\n```rust\nfn block_{}() {{}}\n```\n\n", i, i));
    }

    let start = std::time::Instant::now();
    let blocks = CodeBlockParser::parse(&markdown);
    let elapsed = start.elapsed();

    assert_eq!(blocks.len(), 50);
    // Performance check: should parse 50 blocks in <100ms
    assert!(
        elapsed.as_millis() < 100,
        "Parsing took {}ms, expected <100ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_performance_large_storage() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create 1000 blocks
    let blocks: Vec<_> = (1..=1000)
        .map(|i| radium_core::code_blocks::CodeBlock {
            index: i,
            language: Some("rust".to_string()),
            content: format!("fn block_{}() {{}}", i),
            file_hint: None,
            start_line: i * 2,
        })
        .collect();

    let mut store = CodeBlockStore::new(workspace_root, "test-session".to_string()).unwrap();

    let start = std::time::Instant::now();
    store.store_blocks("test-agent", blocks).unwrap();
    let store_elapsed = start.elapsed();

    let start = std::time::Instant::now();
    let retrieved = store.list_blocks(None).unwrap();
    let list_elapsed = start.elapsed();

    assert_eq!(retrieved.len(), 1000);
    // Performance checks
    assert!(
        store_elapsed.as_millis() < 500,
        "Storage took {}ms, expected <500ms",
        store_elapsed.as_millis()
    );
    assert!(
        list_elapsed.as_millis() < 200,
        "Listing took {}ms, expected <200ms",
        list_elapsed.as_millis()
    );
}

#[test]
fn test_malformed_blocks() {
    // Block missing closing fence
    let markdown = r#"Code block:

```rust
fn main() {
    println!("Hello");
}
// Missing closing fence"#;

    let blocks = CodeBlockParser::parse(markdown);
    // Should still capture the block even without closing fence
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].content.contains("fn main()"));
}

#[test]
fn test_empty_response() {
    let markdown = "No code blocks here.";
    let blocks = CodeBlockParser::parse(markdown);
    assert_eq!(blocks.len(), 0);
}

#[test]
fn test_file_hint_detection() {
    // Rust style comment
    let content = "// src/main.rs\nfn main() {}";
    let hint = CodeBlockParser::detect_file_hints(content);
    assert_eq!(hint, Some(PathBuf::from("src/main.rs")));

    // Python style comment
    let content = "# app/models.py\nclass Model: pass";
    let hint = CodeBlockParser::detect_file_hints(content);
    assert_eq!(hint, Some(PathBuf::from("app/models.py")));

    // Explicit file annotation
    let content = "// file: path/to/file.ts\nconst x = 1;";
    let hint = CodeBlockParser::detect_file_hints(content);
    assert_eq!(hint, Some(PathBuf::from("path/to/file.ts")));
}

#[test]
fn test_agent_filtering() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    let mut store = CodeBlockStore::new(workspace_root, "test-session".to_string()).unwrap();

    let blocks1 = vec![radium_core::code_blocks::CodeBlock {
        index: 1,
        language: Some("rust".to_string()),
        content: "fn main() {}".to_string(),
        file_hint: None,
        start_line: 1,
    }];

    let blocks2 = vec![radium_core::code_blocks::CodeBlock {
        index: 2,
        language: Some("python".to_string()),
        content: "print('hi')".to_string(),
        file_hint: None,
        start_line: 5,
    }];

    store.store_blocks("agent-1", blocks1).unwrap();
    store.store_blocks("agent-2", blocks2).unwrap();

    let agent1_blocks = store.list_blocks(Some("agent-1")).unwrap();
    assert_eq!(agent1_blocks.len(), 1);
    assert_eq!(agent1_blocks[0].index, 1);

    let all_blocks = store.list_blocks(None).unwrap();
    assert_eq!(all_blocks.len(), 2);
}

#[test]
fn test_not_found_error() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    let store = CodeBlockStore::new(workspace_root, "test-session".to_string()).unwrap();

    let result = store.get_block(999);
    assert!(matches!(
        result,
        Err(radium_core::code_blocks::CodeBlockError::NotFound(999))
    ));
}

#[test]
fn test_invalid_range() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    let store = CodeBlockStore::new(workspace_root, "test-session".to_string()).unwrap();

    let result = store.get_blocks(radium_core::code_blocks::BlockSelector::Range(5, 2));
    assert!(result.is_err());
}

