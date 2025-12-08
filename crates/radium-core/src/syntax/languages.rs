//! Language registry for syntax highlighting.
//!
//! Maps language names and file extensions to syntect syntax definitions.

use std::collections::HashMap;
use syntect::parsing::SyntaxSet;

/// Registry for mapping language identifiers to syntect syntax definitions.
pub struct LanguageRegistry {
    syntax_set: SyntaxSet,
    language_map: HashMap<String, String>,
}

impl LanguageRegistry {
    /// Create a new language registry with built-in syntax definitions.
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let mut language_map = HashMap::new();

        // Map common language names to syntect syntax names
        let mappings = vec![
            ("rust", "Rust"),
            ("rs", "Rust"),
            ("python", "Python"),
            ("py", "Python"),
            ("javascript", "JavaScript"),
            ("js", "JavaScript"),
            ("typescript", "TypeScript"),
            ("ts", "TypeScript"),
            ("go", "Go"),
            ("java", "Java"),
            ("c", "C"),
            ("cpp", "C++"),
            ("c++", "C++"),
            ("cc", "C++"),
            ("shell", "Shell Script"),
            ("bash", "Shell Script"),
            ("sh", "Shell Script"),
            ("yaml", "YAML"),
            ("yml", "YAML"),
            ("json", "JSON"),
            ("toml", "TOML"),
            ("markdown", "Markdown"),
            ("md", "Markdown"),
            ("html", "HTML"),
            ("css", "CSS"),
            ("sql", "SQL"),
            ("xml", "XML"),
            ("dockerfile", "Dockerfile"),
            ("docker", "Dockerfile"),
        ];

        for (key, value) in mappings {
            language_map.insert(key.to_lowercase(), value.to_string());
        }

        Self {
            syntax_set,
            language_map,
        }
    }

    /// Get the syntax set for syntect operations.
    pub fn syntax_set(&self) -> &SyntaxSet {
        &self.syntax_set
    }

    /// Find syntax definition for a language identifier.
    ///
    /// Returns the syntax definition if found, or None for unknown languages.
    pub fn find_syntax(&self, language: &str) -> Option<&syntect::parsing::SyntaxReference> {
        let normalized = language.to_lowercase();
        
        // First try direct lookup
        if let Some(syntax_name) = self.language_map.get(&normalized) {
            return self.syntax_set.find_syntax_by_name(syntax_name);
        }

        // Try finding by name directly
        self.syntax_set.find_syntax_by_name(language)
            .or_else(|| self.syntax_set.find_syntax_by_extension(&normalized))
    }

    /// Check if a language is supported.
    pub fn is_supported(&self, language: &str) -> bool {
        self.find_syntax(language).is_some()
    }

    /// Get list of supported language names.
    pub fn supported_languages(&self) -> Vec<String> {
        self.language_map.keys().cloned().collect()
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_language_detection() {
        let registry = LanguageRegistry::new();
        assert!(registry.is_supported("rust"));
        assert!(registry.is_supported("rs"));
        assert!(registry.find_syntax("rust").is_some());
    }

    #[test]
    fn test_python_language_detection() {
        let registry = LanguageRegistry::new();
        assert!(registry.is_supported("python"));
        assert!(registry.is_supported("py"));
        assert!(registry.find_syntax("python").is_some());
    }

    #[test]
    fn test_unknown_language() {
        let registry = LanguageRegistry::new();
        assert!(!registry.is_supported("unknown_lang_xyz"));
        assert!(registry.find_syntax("unknown_lang_xyz").is_none());
    }

    #[test]
    fn test_case_insensitive() {
        let registry = LanguageRegistry::new();
        assert!(registry.is_supported("RUST"));
        assert!(registry.is_supported("Python"));
        assert!(registry.is_supported("JAVASCRIPT"));
    }
}

