//! Configuration for model caching.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Configuration for the model cache.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheConfig {
    /// Whether model caching is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Seconds before unused models are evicted (default: 1800 = 30 minutes).
    #[serde(default = "default_inactivity_timeout_secs")]
    pub inactivity_timeout_secs: u64,

    /// Maximum number of models to keep in memory (default: 10).
    #[serde(default = "default_max_cache_size")]
    pub max_cache_size: usize,

    /// How often the cleanup task runs in seconds (default: 300 = 5 minutes).
    #[serde(default = "default_cleanup_interval_secs")]
    pub cleanup_interval_secs: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_inactivity_timeout_secs() -> u64 {
    1800 // 30 minutes
}

fn default_max_cache_size() -> usize {
    10
}

fn default_cleanup_interval_secs() -> u64 {
    300 // 5 minutes
}

/// Errors that can occur during cache configuration validation.
#[derive(Debug, Error)]
pub enum CacheConfigError {
    /// Invalid inactivity timeout (must be > 0).
    #[error("Invalid inactivity timeout: must be greater than 0")]
    InvalidInactivityTimeout,

    /// Invalid max cache size (must be > 0).
    #[error("Invalid max cache size: must be greater than 0")]
    InvalidMaxCacheSize,

    /// Invalid cleanup interval (must be > 0).
    #[error("Invalid cleanup interval: must be greater than 0")]
    InvalidCleanupInterval,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            inactivity_timeout_secs: default_inactivity_timeout_secs(),
            max_cache_size: default_max_cache_size(),
            cleanup_interval_secs: default_cleanup_interval_secs(),
        }
    }
}

impl CacheConfig {
    /// Validate the cache configuration.
    ///
    /// # Errors
    /// Returns `CacheConfigError` if any configuration value is invalid.
    pub fn validate(&self) -> Result<(), CacheConfigError> {
        if self.inactivity_timeout_secs == 0 {
            return Err(CacheConfigError::InvalidInactivityTimeout);
        }

        if self.max_cache_size == 0 {
            return Err(CacheConfigError::InvalidMaxCacheSize);
        }

        if self.cleanup_interval_secs == 0 {
            return Err(CacheConfigError::InvalidCleanupInterval);
        }

        Ok(())
    }

    /// Get the inactivity timeout as a Duration.
    #[must_use]
    pub fn inactivity_timeout(&self) -> Duration {
        Duration::from_secs(self.inactivity_timeout_secs)
    }

    /// Get the cleanup interval as a Duration.
    #[must_use]
    pub fn cleanup_interval(&self) -> Duration {
        Duration::from_secs(self.cleanup_interval_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.inactivity_timeout_secs, 1800);
        assert_eq!(config.max_cache_size, 10);
        assert_eq!(config.cleanup_interval_secs, 300);
    }

    #[test]
    fn test_cache_config_validation_valid() {
        let config = CacheConfig {
            enabled: true,
            inactivity_timeout_secs: 1800,
            max_cache_size: 10,
            cleanup_interval_secs: 300,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cache_config_validation_invalid_timeout() {
        let config = CacheConfig {
            enabled: true,
            inactivity_timeout_secs: 0,
            max_cache_size: 10,
            cleanup_interval_secs: 300,
        };

        assert!(matches!(
            config.validate(),
            Err(CacheConfigError::InvalidInactivityTimeout)
        ));
    }

    #[test]
    fn test_cache_config_validation_invalid_max_size() {
        let config = CacheConfig {
            enabled: true,
            inactivity_timeout_secs: 1800,
            max_cache_size: 0,
            cleanup_interval_secs: 300,
        };

        assert!(matches!(
            config.validate(),
            Err(CacheConfigError::InvalidMaxCacheSize)
        ));
    }

    #[test]
    fn test_cache_config_validation_invalid_cleanup_interval() {
        let config = CacheConfig {
            enabled: true,
            inactivity_timeout_secs: 1800,
            max_cache_size: 10,
            cleanup_interval_secs: 0,
        };

        assert!(matches!(
            config.validate(),
            Err(CacheConfigError::InvalidCleanupInterval)
        ));
    }

    #[test]
    fn test_cache_config_durations() {
        let config = CacheConfig {
            enabled: true,
            inactivity_timeout_secs: 1800,
            max_cache_size: 10,
            cleanup_interval_secs: 300,
        };

        assert_eq!(config.inactivity_timeout(), Duration::from_secs(1800));
        assert_eq!(config.cleanup_interval(), Duration::from_secs(300));
    }
}

