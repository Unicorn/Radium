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
pub mod linter;
pub mod metadata;
pub mod model_selector;
pub mod persona;
pub mod registry;
pub mod validation;

pub use config::{AgentConfigError, AgentConfigFile, ReasoningEffort};
pub use discovery::{AgentDiscovery, DiscoveryError};
pub use linter::{AgentLinter, LintError, LintResult, PromptLinter};
pub use metadata::{
    AgentMetadata, ContextRequirements, CostTier, IterationSpeed, MetadataError, ModelPriority,
    ModelRecommendation, OutputVolume, PerformanceProfile, RecommendedModels, ThinkingDepth,
};
pub use model_selector::{
    DefaultModelSelector, FallbackChainSelector, ModelSelector, SelectionError, SelectionResult,
};
pub use persona::{
    ModelPricing, ModelPricingDB, PersonaConfig, PerformanceProfile as PersonaPerformanceProfile,
    RecommendedModels as PersonaRecommendedModels,
};
pub use registry::{AgentRegistry, RegistryError};
pub use validation::{AgentValidator, AgentValidatorImpl, ConfigValidator, PromptValidator, ValidationError};
