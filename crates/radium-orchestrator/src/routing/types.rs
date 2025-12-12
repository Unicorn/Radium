//! Types for model routing system.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Model tier for routing decisions (distinct from budget ModelTier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoutingTier {
    /// Smart tier: High-capability models for complex tasks.
    Smart,
    /// Eco tier: Fast, cost-effective models for simple tasks.
    Eco,
    /// Auto: Let the router decide based on complexity.
    Auto,
}

impl fmt::Display for RoutingTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoutingTier::Smart => write!(f, "smart"),
            RoutingTier::Eco => write!(f, "eco"),
            RoutingTier::Auto => write!(f, "auto"),
        }
    }
}

impl RoutingTier {
    /// Converts a string to RoutingTier.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "smart" => Some(RoutingTier::Smart),
            "eco" => Some(RoutingTier::Eco),
            "auto" => Some(RoutingTier::Auto),
            _ => None,
        }
    }

    /// Converts RoutingTier to string for telemetry.
    pub fn to_telemetry_string(&self) -> String {
        match self {
            RoutingTier::Smart => "smart".to_string(),
            RoutingTier::Eco => "eco".to_string(),
            RoutingTier::Auto => "auto".to_string(),
        }
    }
}

/// Complexity score with breakdown of scoring factors.
#[derive(Debug, Clone)]
pub struct ComplexityScore {
    /// Overall complexity score (0-100).
    pub score: f64,
    /// Token count factor (0-1 normalized).
    pub token_count_factor: f64,
    /// Task type factor (0-1 normalized).
    pub task_type_factor: f64,
    /// Reasoning factor (0-1 normalized).
    pub reasoning_factor: f64,
    /// Context complexity factor (0-1 normalized).
    pub context_factor: f64,
}

impl ComplexityScore {
    /// Creates a new complexity score.
    ///
    /// The score is on a 0-100 scale for comparison with routing thresholds.
    #[must_use]
    pub fn new(
        token_count_factor: f64,
        task_type_factor: f64,
        reasoning_factor: f64,
        context_factor: f64,
        weights: &ComplexityWeights,
    ) -> Self {
        // Calculate weighted sum (0-1 scale)
        let normalized_score = (weights.token_count * token_count_factor)
            + (weights.task_type * task_type_factor)
            + (weights.reasoning * reasoning_factor)
            + (weights.context * context_factor);

        // Scale to 0-100 for threshold comparison
        let score = normalized_score * 100.0;

        Self {
            score,
            token_count_factor,
            task_type_factor,
            reasoning_factor,
            context_factor,
        }
    }
}

/// Weights for complexity scoring factors.
#[derive(Debug, Clone)]
pub struct ComplexityWeights {
    /// Weight for token count factor.
    pub token_count: f64,
    /// Weight for task type factor.
    pub task_type: f64,
    /// Weight for reasoning factor.
    pub reasoning: f64,
    /// Weight for context complexity factor.
    pub context: f64,
}

impl Default for ComplexityWeights {
    fn default() -> Self {
        Self {
            token_count: 0.3,
            task_type: 0.4,
            reasoning: 0.2,
            context: 0.1,
        }
    }
}

/// Task type classification for complexity estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    /// Code generation and refactoring.
    Code,
    /// Complex reasoning and analysis.
    Reasoning,
    /// Text formatting and summarization.
    Formatting,
    /// Simple queries and responses.
    Simple,
}

/// Routing strategy for model selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Complexity-based routing (current default).
    ComplexityBased,
    /// Cost-optimized: Select cheapest model meeting requirements.
    CostOptimized,
    /// Latency-optimized: Select fastest model meeting requirements.
    LatencyOptimized,
    /// Quality-optimized: Select highest tier model meeting requirements.
    QualityOptimized,
}

impl RoutingStrategy {
    /// Converts a string to RoutingStrategy.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "complexity_based" | "complexity-based" | "complexity" => Some(RoutingStrategy::ComplexityBased),
            "cost_optimized" | "cost-optimized" | "cost" => Some(RoutingStrategy::CostOptimized),
            "latency_optimized" | "latency-optimized" | "latency" => Some(RoutingStrategy::LatencyOptimized),
            "quality_optimized" | "quality-optimized" | "quality" => Some(RoutingStrategy::QualityOptimized),
            _ => None,
        }
    }
    
    /// Converts RoutingStrategy to string.
    pub fn to_string(&self) -> String {
        match self {
            RoutingStrategy::ComplexityBased => "complexity_based".to_string(),
            RoutingStrategy::CostOptimized => "cost_optimized".to_string(),
            RoutingStrategy::LatencyOptimized => "latency_optimized".to_string(),
            RoutingStrategy::QualityOptimized => "quality_optimized".to_string(),
        }
    }
}

/// Metadata about a model for routing decisions.
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    /// Model identifier.
    pub model_id: String,
    /// Provider name (e.g., "claude", "openai", "gemini").
    pub provider: String,
    /// Cost per 1M input tokens in USD.
    pub cost_per_1m_input: f64,
    /// Cost per 1M output tokens in USD.
    pub cost_per_1m_output: f64,
    /// Cost per 1M reasoning tokens in USD (for o1/o3 models).
    pub cost_per_1m_reasoning: Option<f64>,
    /// Average latency in milliseconds.
    pub avg_latency_ms: u64,
    /// Quality tier (1-5 scale, where 5 is highest quality).
    pub quality_tier: u8,
}

impl ModelMetadata {
    /// Creates new model metadata.
    pub fn new(
        model_id: String,
        provider: String,
        cost_per_1m_input: f64,
        cost_per_1m_output: f64,
        avg_latency_ms: u64,
        quality_tier: u8,
    ) -> Self {
        Self {
            model_id,
            provider,
            cost_per_1m_input,
            cost_per_1m_output,
            cost_per_1m_reasoning: None,
            avg_latency_ms,
            quality_tier,
        }
    }

    /// Creates new model metadata with reasoning token pricing (for o1/o3 models).
    pub fn with_reasoning(
        model_id: String,
        provider: String,
        cost_per_1m_input: f64,
        cost_per_1m_output: f64,
        cost_per_1m_reasoning: f64,
        avg_latency_ms: u64,
        quality_tier: u8,
    ) -> Self {
        Self {
            model_id,
            provider,
            cost_per_1m_input,
            cost_per_1m_output,
            cost_per_1m_reasoning: Some(cost_per_1m_reasoning),
            avg_latency_ms,
            quality_tier,
        }
    }
}

/// Error types for routing operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RoutingError {
    /// All models in the fallback chain failed.
    #[error("All models in fallback chain failed. Failures: {0:?}")]
    AllModelsFailed(Vec<FailureRecord>),
    
    /// No suitable model found that meets the requirements.
    #[error("No suitable model found: {0}")]
    NoSuitableModel(String),
    
    /// Configuration error.
    #[error("Routing configuration error: {0}")]
    ConfigurationError(String),
}

/// Record of a model failure for tracking and debugging.
#[derive(Debug, Clone)]
pub struct FailureRecord {
    /// Model identifier that failed.
    pub model_id: String,
    /// Timestamp when the failure occurred.
    pub timestamp: std::time::SystemTime,
    /// Error message describing the failure.
    pub error: String,
}

impl FailureRecord {
    /// Creates a new failure record.
    pub fn new(model_id: String, error: String) -> Self {
        Self {
            model_id,
            timestamp: std::time::SystemTime::now(),
            error,
        }
    }
}

/// Fallback chain for model retry logic.
#[derive(Debug, Clone)]
pub struct FallbackChain {
    /// Ordered list of model configurations to try.
    pub models: Vec<radium_models::ModelConfig>,
    /// Maximum number of retries per model before moving to next.
    pub max_retries_per_model: u32,
}

impl FallbackChain {
    /// Creates a new fallback chain.
    pub fn new(models: Vec<radium_models::ModelConfig>) -> Self {
        Self {
            models,
            max_retries_per_model: 1,
        }
    }
    
    /// Creates a new fallback chain with custom retry count.
    pub fn with_retries(models: Vec<radium_models::ModelConfig>, max_retries: u32) -> Self {
        Self {
            models,
            max_retries_per_model: max_retries,
        }
    }
    
    /// Returns the number of models in the chain.
    pub fn len(&self) -> usize {
        self.models.len()
    }
    
    /// Returns true if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}

impl TaskType {
    /// Estimates complexity factor for this task type (0-1 scale).
    #[must_use]
    pub fn complexity_factor(&self) -> f64 {
        match self {
            TaskType::Code => 0.8,
            TaskType::Reasoning => 0.7,
            TaskType::Formatting => 0.2,
            TaskType::Simple => 0.3,
        }
    }

    /// Classifies task type from input text.
    pub fn classify(input: &str) -> Self {
        let lower = input.to_lowercase();
        
        // Check for code-related keywords
        if lower.contains("refactor") || lower.contains("implement") || lower.contains("code")
            || lower.contains("function") || lower.contains("class") || lower.contains("module")
            || lower.contains("architecture") || lower.contains("design pattern")
        {
            return TaskType::Code;
        }
        
        // Check for reasoning-related keywords
        if lower.contains("analyze") || lower.contains("reason") || lower.contains("decide")
            || lower.contains("plan") || lower.contains("strategy") || lower.contains("compare")
            || lower.contains("evaluate") || lower.contains("explain")
        {
            return TaskType::Reasoning;
        }
        
        // Check for formatting-related keywords
        if lower.contains("format") || lower.contains("formatting") || lower.contains("pretty")
            || lower.contains("indent") || lower.contains("style")
        {
            return TaskType::Formatting;
        }
        
        // Default to simple
        TaskType::Simple
    }
}

