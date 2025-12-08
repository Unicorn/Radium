//! Complexity estimation for model routing.

use super::types::{ComplexityScore, ComplexityWeights, TaskType};
use tracing::debug;

/// Complexity estimator for routing decisions.
pub struct ComplexityEstimator {
    /// Weights for scoring factors.
    weights: ComplexityWeights,
    /// Maximum token count for normalization (50K tokens = 1.0).
    max_tokens: u64,
}

impl ComplexityEstimator {
    /// Creates a new complexity estimator with default weights.
    #[must_use]
    pub fn new() -> Self {
        Self {
            weights: ComplexityWeights::default(),
            max_tokens: 50_000,
        }
    }

    /// Creates a new complexity estimator with custom weights.
    #[must_use]
    pub fn with_weights(weights: ComplexityWeights) -> Self {
        Self {
            weights,
            max_tokens: 50_000,
        }
    }

    /// Estimates complexity for a task based on input and context.
    ///
    /// # Arguments
    /// * `input` - The input prompt/text
    /// * `agent_id` - Optional agent ID for context
    ///
    /// # Returns
    /// A ComplexityScore with overall score and factor breakdowns.
    pub fn estimate(&self, input: &str, _agent_id: Option<&str>) -> ComplexityScore {
        // 1. Token count normalization
        let estimated_tokens = self.estimate_tokens(input);
        let token_count_factor = (estimated_tokens as f64 / self.max_tokens as f64).min(1.0);

        // 2. Task type classification
        let task_type = TaskType::classify(input);
        let task_type_factor = task_type.complexity_factor();

        // 3. Reasoning requirements (basic heuristics)
        let reasoning_factor = self.detect_reasoning_complexity(input);

        // 4. Context complexity (basic file/cross-reference detection)
        let context_factor = self.detect_context_complexity(input);

        let score = ComplexityScore::new(
            token_count_factor,
            task_type_factor,
            reasoning_factor,
            context_factor,
            &self.weights,
        );

        debug!(
            complexity_score = score.score,
            token_factor = score.token_count_factor,
            task_type_factor = score.task_type_factor,
            reasoning_factor = score.reasoning_factor,
            context_factor = score.context_factor,
            "Estimated task complexity"
        );

        score
    }

    /// Estimates token count for input (rough approximation).
    fn estimate_tokens(&self, input: &str) -> u64 {
        // Rough approximation: ~4 characters per token
        // This is a simplification; real tokenization would be more accurate
        (input.len() as f64 / 4.0).ceil() as u64
    }

    /// Detects reasoning complexity from input (0-1 scale).
    fn detect_reasoning_complexity(&self, input: &str) -> f64 {
        let lower = input.to_lowercase();
        let mut score: f64 = 0.0;

        // Multi-step indicators
        if lower.contains("step") || lower.contains("step-by-step") || lower.contains("multiple") {
            score += 0.3;
        }

        // Dependency indicators
        if lower.contains("depend") || lower.contains("require") || lower.contains("prerequisite") {
            score += 0.3;
        }

        // Conditional logic indicators
        if lower.contains("if") || lower.contains("when") || lower.contains("condition") {
            score += 0.2;
        }

        // Comparison/analysis indicators
        if lower.contains("compare") || lower.contains("analyze") || lower.contains("evaluate") {
            score += 0.3;
        }

        // Trade-off indicators (complex decision making)
        if lower.contains("trade-off") || lower.contains("tradeoff") || lower.contains("trade off") {
            score += 0.4;
        }

        // Pattern/architecture indicators (complex refactoring)
        if lower.contains("pattern") || lower.contains("architecture") || lower.contains("design") {
            score += 0.3;
        }

        // Ensure score is between 0 and 1
        score.min(1.0)
    }

    /// Detects context complexity from input (0-1 scale).
    fn detect_context_complexity(&self, input: &str) -> f64 {
        let lower = input.to_lowercase();
        let mut score: f64 = 0.0;

        // File references
        let file_count = lower.matches(".rs").count()
            + lower.matches(".ts").count()
            + lower.matches(".py").count()
            + lower.matches(".js").count()
            + lower.matches(".md").count();
        score += (file_count as f64 * 0.15).min(0.5);

        // Cross-reference indicators
        if lower.contains("import") || lower.contains("use") || lower.contains("from") {
            score += 0.25;
        }

        // Module/package indicators
        if lower.contains("module") || lower.contains("package") || lower.contains("crate") {
            score += 0.3;
        }

        // Architecture/system indicators
        if lower.contains("architecture") || lower.contains("system") || lower.contains("design") {
            score += 0.3;
        }

        // Service/microservice indicators (complex distributed systems)
        if lower.contains("service") || lower.contains("microservice") {
            score += 0.2;
        }

        // Ensure score is between 0 and 1
        score.min(1.0)
    }
}

impl Default for ComplexityEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_task_complexity() {
        let estimator = ComplexityEstimator::new();
        let score = estimator.estimate("format this JSON", None);
        
        // Simple formatting task should have low complexity
        assert!(score.score < 40.0, "Expected low complexity for simple task");
    }

    #[test]
    fn test_code_refactor_complexity() {
        let estimator = ComplexityEstimator::new();
        let score = estimator.estimate(
            "refactor this module to use dependency injection pattern with multiple services",
            None,
        );
        
        // Code refactoring should have higher complexity than simple tasks
        // Note: actual score depends on input length and keyword detection
        assert!(score.score > 35.0, "Expected moderate-high complexity for code refactoring, got {}", score.score);
        assert!(score.task_type_factor > 0.7, "Expected code task type");
        assert!(score.score > 30.0, "Code refactoring should score higher than simple formatting");
    }

    #[test]
    fn test_reasoning_task_complexity() {
        let estimator = ComplexityEstimator::new();
        let score = estimator.estimate(
            "analyze the trade-offs between microservices and monolithic architecture, considering scalability and deployment complexity",
            None,
        );
        
        // Complex reasoning should have high complexity
        // This input has many complexity indicators: trade-off, analyze, architecture, complexity
        // With improved heuristics and longer input, should score reasonably high
        assert!(score.score >= 50.0, "Expected high complexity for reasoning task, got {}", score.score);
        assert!(score.reasoning_factor > 0.4, "Expected high reasoning factor, got {}", score.reasoning_factor);
    }

    #[test]
    fn test_token_normalization() {
        let estimator = ComplexityEstimator::new();
        
        // Small input
        let small_score = estimator.estimate("hello", None);
        
        // Large input (simulated with repeated text)
        let large_input = "x".repeat(100_000);
        let large_score = estimator.estimate(&large_input, None);
        
        // Large input should have higher token count factor
        assert!(
            large_score.token_count_factor > small_score.token_count_factor,
            "Expected larger input to have higher token count factor"
        );
    }
}

