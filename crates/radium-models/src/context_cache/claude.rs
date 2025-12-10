//! Claude-specific context caching implementation.

use async_trait::async_trait;
use std::time::Duration;

use crate::context_cache::types::{CacheError, CacheHandle, CachedContext};
use crate::context_cache::ContextCache;

/// Claude context cache implementation.
///
/// Claude uses `cache_control` blocks in messages to enable prompt caching.
/// The caching is implicit - when cache_control is present, Claude caches
/// the content automatically. There's no explicit cache creation API.
///
/// # Cache Behavior
///
/// - Cache is created automatically when cache_control blocks are present
/// - Minimum 1024 tokens required for cache creation
/// - Default TTL is 5 minutes
/// - Cache breakpoints mark message indices where caching should start
#[derive(Debug, Clone)]
pub struct ClaudeContextCache;

impl ClaudeContextCache {
    /// Create a new Claude context cache instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Apply cache breakpoints to messages.
    ///
    /// This marks messages at the specified indices as cacheable by adding
    /// cache_control blocks to their content.
    ///
    /// # Arguments
    /// * `messages` - Mutable reference to Claude messages
    /// * `breakpoints` - Vector of message indices where caching should start
    pub fn apply_cache_breakpoints(
        messages: &mut [crate::claude::ClaudeMessage],
        breakpoints: &[usize],
    ) {
        use crate::claude::CacheControl;
        use crate::claude::ClaudeContentBlock;
        use crate::claude::ClaudeMessageContent;

        let cache_control = Some(CacheControl {
            cache_type: "ephemeral".to_string(),
        });

        for &breakpoint in breakpoints {
            if breakpoint < messages.len() {
                match &mut messages[breakpoint].content {
                    ClaudeMessageContent::String(_) => {
                        // For string content, we'd need to convert to blocks
                        // This is a limitation - cache_control only works with blocks
                    }
                    ClaudeMessageContent::Blocks(blocks) => {
                        for block in blocks {
                            match block {
                                ClaudeContentBlock::Text { cache_control: ref mut cc, .. } => {
                                    *cc = cache_control.clone();
                                }
                                ClaudeContentBlock::Image { cache_control: ref mut cc, .. } => {
                                    *cc = cache_control.clone();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for ClaudeContextCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextCache for ClaudeContextCache {
    async fn create_cache(
        &self,
        _content: &str,
        _ttl: Duration,
    ) -> Result<CacheHandle, CacheError> {
        // Claude caching is implicit via cache_control blocks
        // No explicit cache creation needed
        Ok(CacheHandle::Claude {
            cache_control: serde_json::json!({ "type": "ephemeral" }),
        })
    }

    async fn get_cache(
        &self,
        _handle: &CacheHandle,
    ) -> Result<Option<CachedContext>, CacheError> {
        // Claude doesn't expose cache metadata
        // Caching is handled automatically by the API
        Err(CacheError::ProviderNotSupported {
            provider: "Claude".to_string(),
        })
    }

    async fn refresh_cache(&self, _handle: &CacheHandle) -> Result<(), CacheError> {
        // Claude manages cache lifecycle automatically (5-minute TTL)
        // No explicit refresh needed
        Ok(())
    }

    async fn delete_cache(&self, _handle: &CacheHandle) -> Result<(), CacheError> {
        // Claude manages cache lifecycle automatically
        // No explicit deletion needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_context_cache_new() {
        let cache = ClaudeContextCache::new();
        // Just verify it compiles and can be created
        assert!(std::mem::size_of_val(&cache) > 0);
    }

    #[tokio::test]
    async fn test_create_cache() {
        let cache = ClaudeContextCache::new();
        let handle = cache
            .create_cache("test content", Duration::from_secs(300))
            .await
            .unwrap();

        match handle {
            CacheHandle::Claude { cache_control } => {
                assert_eq!(cache_control["type"], "ephemeral");
            }
            _ => panic!("Expected Claude cache handle"),
        }
    }
}

