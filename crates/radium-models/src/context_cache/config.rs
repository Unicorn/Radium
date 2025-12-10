//! Cache configuration for model-level context caching.

use std::time::Duration;

/// Cache configuration for context caching.
///
/// This configuration is passed to model instances to control
/// context caching behavior.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Whether context caching is enabled.
    pub enabled: bool,
    /// Time-to-live for cached contexts (provider-specific defaults if None).
    pub ttl: Option<Duration>,
    /// Cache breakpoints for Claude (message indices where caching should start).
    pub breakpoints: Option<Vec<usize>>,
    /// Cache identifier for Gemini (reuse existing cache).
    pub identifier: Option<String>,
}

impl CacheConfig {
    /// Create a new cache configuration.
    ///
    /// # Arguments
    /// * `enabled` - Whether caching is enabled
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ttl: None,
            breakpoints: None,
            identifier: None,
        }
    }

    /// Set the TTL for cached contexts.
    ///
    /// # Arguments
    /// * `ttl` - Time-to-live duration
    #[must_use]
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Set cache breakpoints for Claude.
    ///
    /// # Arguments
    /// * `breakpoints` - Vector of message indices where caching should start
    #[must_use]
    pub fn with_breakpoints(mut self, breakpoints: Vec<usize>) -> Self {
        self.breakpoints = Some(breakpoints);
        self
    }

    /// Set cache identifier for Gemini.
    ///
    /// # Arguments
    /// * `identifier` - Cache identifier (e.g., "cachedContents/abc123")
    #[must_use]
    pub fn with_identifier(mut self, identifier: String) -> Self {
        self.identifier = Some(identifier);
        self
    }
}

