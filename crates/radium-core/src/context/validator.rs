//! Source validation orchestration.

use std::sync::Arc;
use std::time::Instant;

use tracing::{debug, info, warn};

use super::sources::{SourceRegistry, SourceError};

/// Validates multiple sources concurrently and returns structured results.
pub struct SourceValidator {
    /// Registry containing all source readers.
    registry: Arc<SourceRegistry>,
}

impl SourceValidator {
    /// Creates a new source validator with the given registry.
    pub fn new(registry: SourceRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
        }
    }

    /// Validates multiple sources concurrently.
    ///
    /// # Arguments
    ///
    /// * `sources` - List of source URIs to validate
    ///
    /// # Returns
    ///
    /// A vector of validation results, one per source
    pub async fn validate_sources(
        &self,
        sources: Vec<String>,
    ) -> Vec<SourceValidationResult> {
        let start_time = Instant::now();
        let source_count = sources.len();

        info!(
            source_count = source_count,
            "Starting source validation"
        );

        // Spawn concurrent validation tasks
        let handles: Vec<_> = sources
            .into_iter()
            .map(|source| {
                let registry = Arc::clone(&self.registry);
                tokio::spawn(async move {
                    Self::validate_single_source(&registry, &source).await
                })
            })
            .collect();

        // Collect all results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    if result.accessible {
                        debug!(
                            source = %result.source,
                            size_bytes = result.size_bytes,
                            "Source validation successful"
                        );
                    } else {
                        warn!(
                            source = %result.source,
                            error = %result.error_message,
                            "Source validation failed"
                        );
                    }
                    results.push(result);
                }
                Err(_) => {
                    warn!("Validation task panicked");
                    results.push(SourceValidationResult {
                        source: "unknown".to_string(),
                        accessible: false,
                        error_message: "Task panicked".to_string(),
                        size_bytes: 0,
                    });
                }
            }
        }

        let duration = start_time.elapsed();
        let valid_count = results.iter().filter(|r| r.accessible).count();
        let invalid_count = results.len() - valid_count;

        info!(
            source_count = source_count,
            valid_count = valid_count,
            invalid_count = invalid_count,
            duration_ms = duration.as_millis(),
            "Source validation completed"
        );

        results
    }

    /// Validates a single source.
    async fn validate_single_source(
        registry: &SourceRegistry,
        source: &str,
    ) -> SourceValidationResult {
        debug!(source = %source, "Validating source");

        // Get the appropriate reader for this source
        let reader = match registry.get_reader(source) {
            Some(r) => r,
            None => {
                warn!(
                    source = %source,
                    "No reader registered for source scheme"
                );
                return SourceValidationResult {
                    source: source.to_string(),
                    accessible: false,
                    error_message: format!("No reader registered for scheme in: {}", source),
                    size_bytes: 0,
                };
            }
        };

        let scheme = reader.scheme();
        debug!(source = %source, scheme = scheme, "Using reader for source");

        // Verify the source
        match reader.verify(source).await {
            Ok(metadata) => {
                let result = SourceValidationResult {
                    source: source.to_string(),
                    accessible: metadata.accessible,
                    error_message: if metadata.accessible {
                        String::new()
                    } else {
                        "Source verification returned inaccessible".to_string()
                    },
                    size_bytes: metadata.size_bytes.unwrap_or(0) as i64,
                };
                debug!(
                    source = %source,
                    accessible = result.accessible,
                    size_bytes = result.size_bytes,
                    "Source verification completed"
                );
                result
            }
            Err(e) => {
                let error_msg = Self::format_error_message(&e);
                debug!(
                    source = %source,
                    error = %error_msg,
                    "Source verification error"
                );
                SourceValidationResult {
                    source: source.to_string(),
                    accessible: false,
                    error_message: error_msg,
                    size_bytes: 0,
                }
            }
        }
    }

    /// Formats a SourceError into a user-friendly error message.
    fn format_error_message(error: &SourceError) -> String {
        match error {
            SourceError::NotFound(msg) => {
                if msg.contains("File not found") || msg.contains("not found") {
                    format!("File not found: {}", msg)
                } else {
                    format!("Not found: {}", msg)
                }
            }
            SourceError::Unauthorized(msg) => format!("Authentication required: {}", msg),
            SourceError::NetworkError(msg) => format!("Network error: {}", msg),
            SourceError::InvalidUri(msg) => format!("Invalid URI: {}", msg),
            SourceError::IoError(e) => format!("I/O error: {}", e),
            SourceError::Other(msg) => msg.clone(),
        }
    }
}

/// Result of validating a single source.
#[derive(Debug, Clone)]
pub struct SourceValidationResult {
    /// The source URI/path that was validated.
    pub source: String,
    /// Whether the source is accessible.
    pub accessible: bool,
    /// Error message if not accessible (empty if accessible).
    pub error_message: String,
    /// Size in bytes if known, 0 otherwise.
    pub size_bytes: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::sources::{LocalFileReader, SourceRegistry};

    #[tokio::test]
    async fn test_validate_sources_empty_list() {
        let registry = SourceRegistry::new();
        let validator = SourceValidator::new(registry);
        let results = validator.validate_sources(vec![]).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_format_error_message() {
        let not_found = SourceError::not_found("test.txt");
        let msg = SourceValidator::format_error_message(&not_found);
        assert!(msg.to_lowercase().contains("not found"));
    }
}