//! Core data types for context caching.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, SystemTime};

/// Provider-specific cache handle for identifying cached contexts.
///
/// Each provider has a different way of identifying cached content:
/// - Claude: Uses cache_control metadata (implicit, no explicit handle)
/// - OpenAI: Automatic caching (no explicit handle)
/// - Gemini: Uses cachedContent API with explicit cache names
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheHandle {
    /// Claude cache handle with cache_control metadata.
    ///
    /// Claude uses cache_control blocks in messages, so the handle
    /// contains the cache_control configuration.
    Claude {
        /// Cache control metadata as JSON value.
        cache_control: serde_json::Value,
    },
    /// OpenAI cache handle (automatic, no explicit identifier).
    ///
    /// OpenAI handles caching automatically, so there's no explicit
    /// cache identifier. This variant exists for type consistency.
    OpenAI,
    /// Gemini cache handle with explicit cache name.
    ///
    /// Gemini uses the cachedContent API which returns a cache name
    /// in the format "cachedContents/{cache-id}".
    Gemini {
        /// The cache name from Gemini API (e.g., "cachedContents/abc123").
        cache_name: String,
    },
}

impl fmt::Display for CacheHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheHandle::Claude { .. } => write!(f, "claude:cache_control"),
            CacheHandle::OpenAI => write!(f, "openai:auto"),
            CacheHandle::Gemini { cache_name } => write!(f, "gemini:{}", cache_name),
        }
    }
}

/// Metadata for a cached context.
///
/// This structure stores information about a cached context including
/// when it was created, when it expires, and how many tokens it contains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedContext {
    /// The cache handle identifying this cached context.
    pub handle: CacheHandle,
    /// When the cache was created.
    pub created_at: SystemTime,
    /// When the cache expires.
    pub expires_at: SystemTime,
    /// Number of tokens in the cached content.
    pub token_count: u32,
}

impl CachedContext {
    /// Create a new cached context.
    ///
    /// # Arguments
    /// * `handle` - The cache handle
    /// * `ttl` - Time to live duration
    /// * `token_count` - Number of tokens in the cached content
    ///
    /// # Returns
    /// A new CachedContext with created_at set to now and expires_at
    /// calculated from the TTL.
    pub fn new(handle: CacheHandle, ttl: Duration, token_count: u32) -> Self {
        let now = SystemTime::now();
        Self {
            handle,
            created_at: now,
            expires_at: now
                .checked_add(ttl)
                .unwrap_or_else(|| now + Duration::from_secs(0)),
            token_count,
        }
    }

    /// Check if the cache has expired.
    ///
    /// # Returns
    /// `true` if the current time is past the expiration time.
    pub fn is_expired(&self) -> bool {
        SystemTime::now()
            .duration_since(self.expires_at)
            .is_ok()
    }

    /// Get the remaining TTL for this cache.
    ///
    /// # Returns
    /// `Some(Duration)` if the cache is not expired, `None` if expired.
    pub fn remaining_ttl(&self) -> Option<Duration> {
        let now = SystemTime::now();
        self.expires_at.duration_since(now).ok()
    }
}

/// Errors that can occur during cache operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    /// Cache creation failed.
    CacheCreationFailed {
        /// The provider name.
        provider: String,
        /// Reason for failure.
        reason: String,
    },
    /// The cache has expired.
    CacheExpired {
        /// The expired cache handle.
        handle: CacheHandle,
    },
    /// The cache was not found.
    CacheNotFound {
        /// The missing cache handle.
        handle: CacheHandle,
    },
    /// Invalid cache configuration.
    InvalidCacheConfiguration {
        /// Reason why the configuration is invalid.
        reason: String,
    },
    /// The provider does not support this operation.
    ProviderNotSupported {
        /// The provider name.
        provider: String,
    },
    /// Network or I/O error.
    NetworkError {
        /// Error message.
        message: String,
    },
    /// Parse error when deserializing response.
    ParseError {
        /// Error message.
        message: String,
    },
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::CacheCreationFailed { provider, reason } => {
                write!(f, "Cache creation failed for {}: {}", provider, reason)
            }
            CacheError::CacheExpired { handle } => {
                write!(f, "Cache expired: {}", handle)
            }
            CacheError::CacheNotFound { handle } => {
                write!(f, "Cache not found: {}", handle)
            }
            CacheError::InvalidCacheConfiguration { reason } => {
                write!(f, "Invalid cache configuration: {}", reason)
            }
            CacheError::ProviderNotSupported { provider } => {
                write!(f, "Provider not supported: {}", provider)
            }
            CacheError::NetworkError { message } => {
                write!(f, "Network error: {}", message)
            }
            CacheError::ParseError { message } => {
                write!(f, "Parse error: {}", message)
            }
        }
    }
}

impl std::error::Error for CacheError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_handle_display() {
        let claude_handle = CacheHandle::Claude {
            cache_control: serde_json::json!({"type": "ephemeral"}),
        };
        assert_eq!(claude_handle.to_string(), "claude:cache_control");

        let openai_handle = CacheHandle::OpenAI;
        assert_eq!(openai_handle.to_string(), "openai:auto");

        let gemini_handle = CacheHandle::Gemini {
            cache_name: "cachedContents/abc123".to_string(),
        };
        assert_eq!(gemini_handle.to_string(), "gemini:cachedContents/abc123");
    }

    #[test]
    fn test_cached_context_new() {
        let handle = CacheHandle::OpenAI;
        let ttl = Duration::from_secs(300);
        let context = CachedContext::new(handle.clone(), ttl, 1000);

        assert_eq!(context.handle, handle);
        assert_eq!(context.token_count, 1000);
        assert!(!context.is_expired());
        assert!(context.remaining_ttl().is_some());
    }

    #[test]
    fn test_cached_context_expiration() {
        let handle = CacheHandle::OpenAI;
        let ttl = Duration::from_secs(0); // Expired immediately
        let context = CachedContext::new(handle, ttl, 1000);

        // Wait a tiny bit to ensure expiration
        std::thread::sleep(Duration::from_millis(10));
        assert!(context.is_expired());
        assert!(context.remaining_ttl().is_none());
    }

    #[test]
    fn test_cache_error_display() {
        let error = CacheError::CacheCreationFailed {
            provider: "claude".to_string(),
            reason: "Invalid API key".to_string(),
        };
        assert!(error.to_string().contains("claude"));
        assert!(error.to_string().contains("Invalid API key"));
    }
}

