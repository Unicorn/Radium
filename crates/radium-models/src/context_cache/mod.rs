//! Context caching module for provider-agnostic prompt/context caching.
//!
//! This module provides a unified interface for caching large, reusable prompts
//! and contexts across different AI model providers. Context caching reduces
//! token costs by 50%+ for repeated context by caching processed tokens at the
//! provider level.
//!
//! # Overview
//!
//! Context caching is different from model instance caching:
//! - **Model instance caching** (`crates/radium-models/src/cache/`): Caches loaded
//!   model objects in memory to reduce initialization overhead.
//! - **Context caching** (this module): Caches processed prompt tokens at the
//!   API level to reduce token costs and latency.
//!
//! # Provider Support
//!
//! - **Claude**: Uses `cache_control` blocks in messages (5-minute TTL, minimum 1024 tokens)
//! - **OpenAI**: Automatic caching for GPT-4+ models (no explicit control)
//! - **Gemini**: Explicit `cachedContent` API (hours TTL, requires cache identifier)
//!
//! # Usage
//!
//! ```rust,no_run
//! use radium_models::context_cache::{ContextCache, CacheHandle};
//! use std::time::Duration;
//!
//! # async fn example(cache: impl ContextCache) -> Result<(), Box<dyn std::error::Error>> {
//! // Create a cache for a large system prompt
//! let handle = cache.create_cache(
//!     "You are a helpful assistant...",
//!     Duration::from_secs(300)
//! ).await?;
//!
//! // Use the cache in subsequent requests
//! // (integration happens in model implementations)
//!
//! // Clean up when done
//! cache.delete_cache(&handle).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Module Structure
//!
//! - `trait.rs`: Core `ContextCache` trait
//! - `types.rs`: Data types (`CacheHandle`, `CachedContext`, `CacheError`)
//! - Provider-specific implementations (created in separate tasks):
//!   - `claude.rs`: Claude context cache implementation
//!   - `openai.rs`: OpenAI context cache implementation
//!   - `gemini.rs`: Gemini context cache implementation
//! - `registry.rs`: Cache metadata storage and management (created in separate task)

pub mod r#trait;
pub mod types;

pub use r#trait::ContextCache;
pub use types::{CacheError, CacheHandle, CachedContext};

