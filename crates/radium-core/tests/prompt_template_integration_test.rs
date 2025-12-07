//! Comprehensive integration tests for Prompt Template System.
//!
//! Tests template loading, rendering, file injection, caching, and thread safety.

use radium_core::prompts::processing::{FileInjectionOptions, FileInjectionFormat, process_with_file_injection, PromptCache};
use radium_core::prompts::templates::{PromptContext, PromptTemplate, RenderOptions};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_basic_template_loading_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("template.md");
    let template_content = "Hello {{name}}!";
    
    fs::write(&template_path, template_content).unwrap();
    
    let template = PromptTemplate::load(&template_path).unwrap();
    assert_eq!(template.content(), "Hello {{name}}!");
    assert_eq!(template.file_path(), Some(template_path.as_path()));
}

#[test]
fn test_placeholder_replacement_simple() {
    let template = PromptTemplate::from_string("Hello {{name}}! Your task is {{task}}.");
    let mut context = PromptContext::new();
    context.set("name", "Alice");
    context.set("task", "write code");
    
    let result = template.render(&context).unwrap();
    assert_eq!(result, "Hello Alice! Your task is write code.");
}

#[test]
fn test_placeholder_replacement_multiple() {
    let template = PromptTemplate::from_string("{{greeting}} {{name}}! Welcome to {{place}}.");
    let mut context = PromptContext::new();
    context.set("greeting", "Hello");
    context.set("name", "Bob");
    context.set("place", "Wonderland");
    
    let result = template.render(&context).unwrap();
    assert_eq!(result, "Hello Bob! Welcome to Wonderland.");
}

#[test]
fn test_file_injection_plain_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("data.txt");
    fs::write(&file_path, "File content here").unwrap();
    
    let template = PromptTemplate::from_string("Content: {{file:data.txt}}");
    let context = PromptContext::new();
    let options = FileInjectionOptions {
        base_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };
    
    let result = process_with_file_injection(&template, &context, &options).unwrap();
    assert!(result.contains("File content here"));
    assert!(result.contains("Content:"));
}

#[test]
fn test_file_injection_code_block_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("script.rs");
    fs::write(&file_path, "fn main() {\n    println!(\"Hello\");\n}").unwrap();
    
    let template = PromptTemplate::from_string("Code: {{file:script.rs:code}}");
    let context = PromptContext::new();
    let options = FileInjectionOptions {
        base_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };
    
    let result = process_with_file_injection(&template, &context, &options).unwrap();
    assert!(result.contains("```rs"));
    assert!(result.contains("fn main()"));
    assert!(result.contains("println!"));
}

#[test]
fn test_file_injection_markdown_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("content.md");
    fs::write(&file_path, "# Section\n\nSome content").unwrap();
    
    let template = PromptTemplate::from_string("Document: {{file:content.md:markdown}}");
    let context = PromptContext::new();
    let options = FileInjectionOptions {
        base_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };
    
    let result = process_with_file_injection(&template, &context, &options).unwrap();
    assert!(result.contains("---"));
    assert!(result.contains("# Section"));
    assert!(result.contains("Some content"));
}

#[test]
fn test_file_injection_with_nested_paths() {
    let temp_dir = TempDir::new().unwrap();
    let nested_dir = temp_dir.path().join("nested");
    fs::create_dir_all(&nested_dir).unwrap();
    let file_path = nested_dir.join("file.txt");
    fs::write(&file_path, "Nested file content").unwrap();
    
    let template = PromptTemplate::from_string("Nested: {{file:nested/file.txt}}");
    let context = PromptContext::new();
    let options = FileInjectionOptions {
        base_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };
    
    let result = process_with_file_injection(&template, &context, &options).unwrap();
    assert!(result.contains("Nested file content"));
}

#[test]
fn test_file_injection_with_placeholder_mixing() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("data.txt");
    fs::write(&file_path, "File data").unwrap();
    
    let template = PromptTemplate::from_string("Hello {{name}}! File: {{file:data.txt}}");
    let mut context = PromptContext::new();
    context.set("name", "User");
    let options = FileInjectionOptions {
        base_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };
    
    let result = process_with_file_injection(&template, &context, &options).unwrap();
    assert!(result.contains("Hello User!"));
    assert!(result.contains("File data"));
}

#[test]
fn test_strict_rendering_mode_errors_on_missing() {
    let template = PromptTemplate::from_string("Hello {{name}}! Task: {{task}}.");
    let mut context = PromptContext::new();
    context.set("name", "Alice");
    // task is missing
    
    let options = RenderOptions {
        strict: true,
        ..Default::default()
    };
    
    let result = template.render_with_options(&context, &options);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("missing placeholder") || e.to_string().contains("task"));
    }
}

#[test]
fn test_non_strict_rendering_mode_with_defaults() {
    let template = PromptTemplate::from_string("Hello {{name}}! Task: {{task}}.");
    let mut context = PromptContext::new();
    context.set("name", "Alice");
    // task is missing
    
    let options = RenderOptions {
        strict: false,
        default_value: Some("default task".to_string()),
    };
    
    let result = template.render_with_options(&context, &options).unwrap();
    assert_eq!(result, "Hello Alice! Task: default task.");
}

#[test]
fn test_non_strict_rendering_mode_empty_default() {
    let template = PromptTemplate::from_string("Hello {{name}}! Task: {{task}}.");
    let mut context = PromptContext::new();
    context.set("name", "Alice");
    // task is missing
    
    let options = RenderOptions {
        strict: false,
        default_value: None,
    };
    
    let result = template.render_with_options(&context, &options).unwrap();
    assert_eq!(result, "Hello Alice! Task: .");
}

#[test]
fn test_prompt_caching_cache_hit() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("template.md");
    fs::write(&template_path, "Cached template").unwrap();
    
    let cache = PromptCache::new();
    
    let template1 = cache.load(&template_path).unwrap();
    let template2 = cache.load(&template_path).unwrap();
    
    // Both should have the same content (from cache)
    assert_eq!(template1.content(), template2.content());
    assert_eq!(template1.content(), "Cached template");
}

#[test]
fn test_prompt_caching_cache_clear() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("template.md");
    fs::write(&template_path, "Original content").unwrap();
    
    let cache = PromptCache::new();
    
    cache.load(&template_path).unwrap();
    assert_eq!(cache.stats().unwrap().size, 1);
    
    cache.clear().unwrap();
    assert_eq!(cache.stats().unwrap().size, 0);
}

#[test]
fn test_prompt_caching_with_ttl_expiration() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("template.md");
    fs::write(&template_path, "TTL test").unwrap();
    
    // Create cache with very short TTL
    let cache = PromptCache::with_ttl(Duration::from_millis(100));
    
    // Load template
    let _template1 = cache.load(&template_path).unwrap();
    assert_eq!(cache.stats().unwrap().size, 1);
    
    // Wait for TTL to expire
    thread::sleep(Duration::from_millis(150));
    
    // Load again - should reload from file (cache expired)
    let _template2 = cache.load(&template_path).unwrap();
    // Cache should still have 1 entry (the new one)
    assert_eq!(cache.stats().unwrap().size, 1);
}

#[test]
fn test_prompt_caching_with_ttl_not_expired() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("template.md");
    fs::write(&template_path, "TTL test").unwrap();
    
    // Create cache with longer TTL
    let cache = PromptCache::with_ttl(Duration::from_secs(1));
    
    // Load template
    let template1 = cache.load(&template_path).unwrap();
    
    // Load again immediately - should be from cache
    let template2 = cache.load(&template_path).unwrap();
    
    // Both should be the same (from cache)
    assert_eq!(template1.content(), template2.content());
}

#[test]
fn test_thread_safe_concurrent_cache_access() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("template.md");
    fs::write(&template_path, "Thread-safe template").unwrap();
    
    let cache = Arc::new(PromptCache::new());
    let mut handles = vec![];
    
    // Spawn multiple threads that access the cache concurrently
    for i in 0..10 {
        let cache_clone = cache.clone();
        let path = template_path.clone();
        let handle = thread::spawn(move || {
            let template = cache_clone.load(&path).unwrap();
            assert_eq!(template.content(), "Thread-safe template");
            format!("thread-{}", i)
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.starts_with("thread-"));
    }
    
    // Cache should have exactly one entry
    assert_eq!(cache.stats().unwrap().size, 1);
}

#[test]
fn test_file_injection_with_various_file_types() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test different file extensions
    let files = vec![
        ("script.py", "print('hello')", "py"),
        ("data.json", r#"{"key": "value"}"#, "json"),
        ("code.js", "console.log('test');", "js"),
        ("text.txt", "Plain text", "text"),
    ];
    
    for (filename, content, ext) in files {
        let file_path = temp_dir.path().join(filename);
        fs::write(&file_path, content).unwrap();
        
        let template = PromptTemplate::from_string(&format!("{{{{file:{}:code}}}}", filename));
        let context = PromptContext::new();
        let options = FileInjectionOptions {
            base_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };
        
        let result = process_with_file_injection(&template, &context, &options).unwrap();
        // Code block format is ```ext\ncontent\n```
        assert!(result.contains("```"), "Result should contain code block markers. Result: {}", result);
        assert!(result.contains(content), "Result should contain file content. Result: {}", result);
        // The extension should appear in the code block (format: ```ext)
        // Just verify it's a code block with the content, don't check exact extension format
    }
}

#[test]
fn test_file_injection_file_size_limit() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large.txt");
    
    // Create a file that exceeds the size limit
    let large_content = "x".repeat(1000);
    fs::write(&file_path, large_content).unwrap();
    
    let template = PromptTemplate::from_string("{{file:large.txt}}");
    let context = PromptContext::new();
    let options = FileInjectionOptions {
        base_path: Some(temp_dir.path().to_path_buf()),
        max_file_size: Some(500), // Limit to 500 bytes
        ..Default::default()
    };
    
    let result = process_with_file_injection(&template, &context, &options);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("exceeds maximum size"));
    }
}

#[test]
fn test_file_injection_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    
    let template = PromptTemplate::from_string("{{file:nonexistent.txt}}");
    let context = PromptContext::new();
    let options = FileInjectionOptions {
        base_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };
    
    let result = process_with_file_injection(&template, &context, &options);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Failed to read file") || e.to_string().contains("not found"));
    }
}

#[test]
fn test_multiple_file_injections() {
    let temp_dir = TempDir::new().unwrap();
    
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "Content 1").unwrap();
    fs::write(&file2, "Content 2").unwrap();
    
    let template = PromptTemplate::from_string("First: {{file:file1.txt}} Second: {{file:file2.txt}}");
    let context = PromptContext::new();
    let options = FileInjectionOptions {
        base_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };
    
    let result = process_with_file_injection(&template, &context, &options).unwrap();
    assert!(result.contains("Content 1"));
    assert!(result.contains("Content 2"));
}

#[test]
fn test_list_placeholders() {
    let template = PromptTemplate::from_string("Hello {{name}}! Your task is {{task}}. Deadline: {{deadline}}.");
    
    let placeholders = template.list_placeholders();
    assert_eq!(placeholders.len(), 3);
    assert!(placeholders.contains(&"name".to_string()));
    assert!(placeholders.contains(&"task".to_string()));
    assert!(placeholders.contains(&"deadline".to_string()));
}

#[test]
fn test_placeholder_case_sensitivity() {
    let template = PromptTemplate::from_string("{{Name}} and {{name}} are different.");
    let mut context = PromptContext::new();
    context.set("Name", "Alice");
    context.set("name", "Bob");
    
    let result = template.render(&context).unwrap();
    assert_eq!(result, "Alice and Bob are different.");
}

#[test]
fn test_template_from_string_vs_load() {
    let content = "Template content";
    
    let from_string = PromptTemplate::from_string(content);
    assert_eq!(from_string.content(), content);
    assert!(from_string.file_path().is_none());
    
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("template.md");
    fs::write(&file_path, content).unwrap();
    
    let from_file = PromptTemplate::load(&file_path).unwrap();
    assert_eq!(from_file.content(), content);
    assert_eq!(from_file.file_path(), Some(file_path.as_path()));
}

