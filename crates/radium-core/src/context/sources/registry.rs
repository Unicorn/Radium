//! Source reader registry for routing URIs to appropriate readers.

use std::collections::HashMap;

use super::traits::SourceReader;

/// Registry that maps URI schemes to their corresponding SourceReader implementations.
pub struct SourceRegistry {
    /// Map of scheme to reader implementation.
    readers: HashMap<String, Box<dyn SourceReader>>,
}

impl SourceRegistry {
    /// Creates a new, empty registry.
    pub fn new() -> Self {
        Self {
            readers: HashMap::new(),
        }
    }

    /// Registers a reader for its scheme.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader implementation to register
    pub fn register(&mut self, reader: Box<dyn SourceReader>) {
        let scheme = reader.scheme().to_string();
        self.readers.insert(scheme, reader);
    }

    /// Gets a reader for the given URI by extracting its scheme.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to get a reader for
    ///
    /// # Returns
    ///
    /// A reference to the appropriate reader, or None if no reader is registered
    /// for the URI's scheme.
    pub fn get_reader(&self, uri: &str) -> Option<&dyn SourceReader> {
        let scheme = Self::extract_scheme(uri);
        self.readers.get(&scheme).map(|r| r.as_ref())
    }

    /// Extracts the scheme from a URI.
    ///
    /// For URIs with explicit schemes (e.g., "http://example.com"), returns the scheme.
    /// For paths without schemes, defaults to "file".
    /// Also handles "https" scheme by routing to "http" reader.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to extract scheme from
    ///
    /// # Returns
    ///
    /// The scheme string (e.g., "file", "http", "jira")
    fn extract_scheme(uri: &str) -> String {
        // Check for explicit scheme
        if let Some(pos) = uri.find("://") {
            let scheme = &uri[..pos];
            // Route https to http reader
            if scheme == "https" {
                return "http".to_string();
            }
            return scheme.to_string();
        }

        // No scheme found, default to file
        "file".to_string()
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::sources::LocalFileReader;

    #[test]
    fn test_extract_scheme_with_explicit_scheme() {
        assert_eq!(SourceRegistry::extract_scheme("file:///path/to/file"), "file");
        assert_eq!(SourceRegistry::extract_scheme("http://example.com"), "http");
        assert_eq!(SourceRegistry::extract_scheme("https://example.com"), "http");
        assert_eq!(SourceRegistry::extract_scheme("jira://PROJ-123"), "jira");
        assert_eq!(
            SourceRegistry::extract_scheme("braingrid://REQ-456"),
            "braingrid"
        );
    }

    #[test]
    fn test_extract_scheme_without_scheme() {
        assert_eq!(SourceRegistry::extract_scheme("path/to/file.txt"), "file");
        assert_eq!(SourceRegistry::extract_scheme("relative/path"), "file");
    }

    #[test]
    fn test_register_and_get_reader() {
        let mut registry = SourceRegistry::new();
        let reader = Box::new(LocalFileReader::new());
        registry.register(reader);

        let retrieved = registry.get_reader("file:///test.txt");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().scheme(), "file");
    }

    #[test]
    fn test_get_reader_for_unknown_scheme() {
        let registry = SourceRegistry::new();
        let retrieved = registry.get_reader("unknown://test");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_https_routes_to_http_reader() {
        let mut registry = SourceRegistry::new();
        // Register http reader (which handles both http and https)
        let reader = Box::new(crate::context::sources::HttpReader::new());
        registry.register(reader);

        // https should route to http reader
        let http_reader = registry.get_reader("http://example.com");
        let https_reader = registry.get_reader("https://example.com");
        assert!(http_reader.is_some());
        assert!(https_reader.is_some());
        assert_eq!(http_reader.unwrap().scheme(), "http");
        assert_eq!(https_reader.unwrap().scheme(), "http");
    }
}
