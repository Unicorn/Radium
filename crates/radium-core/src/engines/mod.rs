//! Engine abstraction layer for AI providers.
//!
//! This module provides a pluggable engine system for supporting multiple
//! AI providers (Claude, OpenAI, Gemini, etc.).
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::engines::{EngineRegistry, ExecutionRequest};
//! use radium_core::engines::providers::MockEngine;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = EngineRegistry::new();
//!
//! // Register an engine
//! let engine = Arc::new(MockEngine::new());
//! registry.register(engine)?;
//! registry.set_default("mock")?;
//!
//! // Execute a request
//! let engine = registry.get_default()?;
//! let request = ExecutionRequest::new("mock-model-1".to_string(), "Hello!".to_string());
//! let response = engine.execute(request).await?;
//!
//! println!("Response: {}", response.content);
//! # Ok(())
//! # }
//! ```

mod config;
mod detection;
mod engine_trait;
mod error;
pub mod providers;
mod registry;

pub use config::{GlobalEngineConfig, PerEngineConfig};
pub use detection::BinaryDetector;
pub use engine_trait::{Engine, EngineMetadata, ExecutionRequest, ExecutionResponse, TokenUsage};
pub use error::{EngineError, Result};
pub use registry::{EngineHealth, EngineRegistry, HealthStatus};
