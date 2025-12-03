//! Agent configuration and discovery system.
//!
//! This module provides agent configuration management using TOML configuration files.
//!
//! # Example
//!
//! ```toml
//! # agents/my-agents/arch-agent.toml
//! [agent]
//! id = "arch-agent"
//! name = "Architecture Agent"
//! description = "Defines system architecture and technical design decisions"
//! prompt_path = "prompts/agents/my-agents/arch-agent.md"
//! engine = "gemini"  # optional default
//! model = "gemini-2.0-flash-exp"  # optional default
//! reasoning_effort = "medium"  # optional default
//! ```

pub mod config;
pub mod discovery;
pub mod metadata;
pub mod prompt_loader;

pub use config::{AgentConfigError, AgentConfigFile, ReasoningEffort};
pub use discovery::{AgentDiscovery, DiscoveryError};
pub use metadata::{
    AgentMetadata, ContextRequirements, CostTier, IterationSpeed, MetadataError, ModelPriority,
    ModelRecommendation, OutputVolume, PerformanceProfile, RecommendedModels, ThinkingDepth,
};
pub use prompt_loader::{AgentPromptLoader, PromptLoaderError};
