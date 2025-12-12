//! File filtering utilities for search operations.

use std::path::Path;
use glob::Pattern;

/// File type filter for search operations.
#[derive(Debug, Clone)]
pub enum FileTypeFilter {
    /// Match files by glob pattern (e.g., "*.rs", "*.ts")
    Glob(String),
    /// Match files by language name (e.g., "rust", "typescript")
    Language(String),
    /// Match all files
    All,
}

impl FileTypeFilter {
    /// Check if a file path matches this filter.
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            FileTypeFilter::All => true,
            FileTypeFilter::Glob(pattern) => {
                // Convert path to string for glob matching
                let path_str = path.to_string_lossy();
                Pattern::new(pattern)
                    .ok()
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            }
            FileTypeFilter::Language(lang) => {
                // Map language names to file extensions
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                match lang.to_lowercase().as_str() {
                    "rust" => ext == "rs",
                    "typescript" | "ts" => ext == "ts" || ext == "tsx",
                    "javascript" | "js" => ext == "js" || ext == "jsx",
                    "python" | "py" => ext == "py",
                    "go" => ext == "go",
                    "java" => ext == "java",
                    "cpp" | "c++" => ext == "cpp" || ext == "cc" || ext == "cxx",
                    "c" => ext == "c",
                    "ruby" | "rb" => ext == "rb",
                    "php" => ext == "php",
                    "swift" => ext == "swift",
                    "kotlin" => ext == "kt",
                    "scala" => ext == "scala",
                    _ => false,
                }
            }
        }
    }

    /// Parse a file type filter from a string.
    ///
    /// Supports:
    /// - `"*"` or `"all"` - match all files
    /// - `"*.rs"`, `"*.ts"` - glob patterns
    /// - `"language:rust"`, `"language:typescript"` - language filters
    /// - `"rust"`, `"typescript"` - shorthand for language filters
    pub fn from_str(s: &str) -> Self {
        let s = s.trim();
        
        if s == "*" || s.eq_ignore_ascii_case("all") {
            return FileTypeFilter::All;
        }
        
        if s.starts_with("language:") {
            let lang = s.strip_prefix("language:").unwrap_or("").trim();
            return FileTypeFilter::Language(lang.to_string());
        }
        
        // Check if it's a glob pattern
        if s.contains('*') || s.contains('?') || s.contains('[') {
            return FileTypeFilter::Glob(s.to_string());
        }
        
        // Assume it's a language name
        FileTypeFilter::Language(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_glob_filter() {
        let filter = FileTypeFilter::Glob("*.rs".to_string());
        assert!(filter.matches(Path::new("src/main.rs")));
        assert!(!filter.matches(Path::new("src/main.ts")));
    }

    #[test]
    fn test_language_filter() {
        let filter = FileTypeFilter::Language("rust".to_string());
        assert!(filter.matches(Path::new("src/main.rs")));
        assert!(!filter.matches(Path::new("src/main.ts")));
        
        let filter = FileTypeFilter::Language("typescript".to_string());
        assert!(filter.matches(Path::new("src/main.ts")));
        assert!(filter.matches(Path::new("src/main.tsx")));
        assert!(!filter.matches(Path::new("src/main.rs")));
    }

    #[test]
    fn test_all_filter() {
        let filter = FileTypeFilter::All;
        assert!(filter.matches(Path::new("src/main.rs")));
        assert!(filter.matches(Path::new("src/main.ts")));
        assert!(filter.matches(Path::new("README.md")));
    }

    #[test]
    fn test_from_str() {
        assert!(matches!(FileTypeFilter::from_str("*"), FileTypeFilter::All));
        assert!(matches!(FileTypeFilter::from_str("all"), FileTypeFilter::All));
        assert!(matches!(FileTypeFilter::from_str("*.rs"), FileTypeFilter::Glob(_)));
        assert!(matches!(FileTypeFilter::from_str("language:rust"), FileTypeFilter::Language(_)));
        assert!(matches!(FileTypeFilter::from_str("rust"), FileTypeFilter::Language(_)));
    }
}
