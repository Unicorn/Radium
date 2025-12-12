//! Unit tests for search functionality.

use super::*;
use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_code_basic() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("file1.rs"), "fn test() {\n    println!(\"hello\");\n}").unwrap();
        fs::write(root.join("file2.ts"), "function test() {\n    console.log('hello');\n}").unwrap();

        let options = SearchOptions {
            pattern: "test".to_string(),
            context_before: 0,
            context_after: 0,
            file_filter: filters::FileTypeFilter::All,
            max_results: 10,
            root: root.to_path_buf(),
        };

        let results = search_code(options).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_code_with_filter() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("file1.rs"), "fn test() {}").unwrap();
        fs::write(root.join("file2.ts"), "function test() {}").unwrap();

        let options = SearchOptions {
            pattern: "test".to_string(),
            context_before: 0,
            context_after: 0,
            file_filter: filters::FileTypeFilter::Language("rust".to_string()),
            max_results: 10,
            root: root.to_path_buf(),
        };

        let results = search_code(options).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].file_path.ends_with("file1.rs"));
    }

    #[test]
    fn test_search_code_max_results() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        for i in 0..5 {
            fs::write(root.join(format!("file{}.rs", i)), "fn test() {}").unwrap();
        }

        let options = SearchOptions {
            pattern: "test".to_string(),
            context_before: 0,
            context_after: 0,
            file_filter: filters::FileTypeFilter::All,
            max_results: 3,
            root: root.to_path_buf(),
        };

        let results = search_code(options).unwrap();
        assert!(results.len() <= 3);
    }

    #[test]
    fn test_search_code_context_lines() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("test.rs"), "line1\nline2\nfn test() {}\nline4\nline5").unwrap();

        let options = SearchOptions {
            pattern: "test".to_string(),
            context_before: 2,
            context_after: 2,
            file_filter: filters::FileTypeFilter::All,
            max_results: 10,
            root: root.to_path_buf(),
        };

        let results = search_code(options).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].context_before.len(), 2);
        assert_eq!(results[0].context_after.len(), 2);
    }
}
