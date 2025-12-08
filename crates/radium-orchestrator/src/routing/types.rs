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

