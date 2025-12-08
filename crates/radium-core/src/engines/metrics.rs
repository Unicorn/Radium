//! Engine performance metrics and monitoring.

use super::engine_trait::TokenUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Metrics for a single engine execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Engine ID.
    pub engine_id: String,
    /// Model used.
    pub model: String,
    /// Execution latency in milliseconds.
    pub latency_ms: u64,
    /// Token usage.
    pub token_usage: Option<TokenUsage>,
    /// Estimated cost in USD.
    pub estimated_cost: Option<f64>,
    /// Whether execution was successful.
    pub success: bool,
    /// Error message if execution failed.
    pub error: Option<String>,
    /// Timestamp of execution.
    pub timestamp: SystemTime,
}

/// Aggregated metrics for an engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetrics {
    /// Engine ID.
    pub engine_id: String,
    /// Total number of executions.
    pub total_executions: u64,
    /// Number of successful executions.
    pub successful_executions: u64,
    /// Number of failed executions.
    pub failed_executions: u64,
    /// Average latency in milliseconds.
    pub average_latency_ms: f64,
    /// Total tokens used.
    pub total_tokens: u64,
    /// Total estimated cost in USD.
    pub total_cost: f64,
    /// Per-model metrics.
    pub model_metrics: HashMap<String, ModelMetrics>,
}

/// Aggregated metrics for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Model ID.
    pub model_id: String,
    /// Total number of executions.
    pub total_executions: u64,
    /// Average latency in milliseconds.
    pub average_latency_ms: f64,
    /// Total tokens used.
    pub total_tokens: u64,
    /// Total estimated cost in USD.
    pub total_cost: f64,
}

impl ExecutionMetrics {
    /// Creates new execution metrics.
    pub fn new(engine_id: String, model: String, latency: Duration, success: bool) -> Self {
        Self {
            engine_id,
            model,
            latency_ms: latency.as_millis() as u64,
            token_usage: None,
            estimated_cost: None,
            success,
            error: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Sets token usage and calculates estimated cost.
    pub fn with_token_usage(mut self, usage: TokenUsage) -> Self {
        self.token_usage = Some(usage.clone());
        self.estimated_cost = Some(Self::estimate_cost(&self.engine_id, &self.model, &usage));
        self
    }

    /// Looks up pricing for a specific model.
    /// Returns (input_price_per_1M, output_price_per_1M) in USD.
    /// Pricing source: Provider pricing pages as of 2024.
    /// Last updated: December 2024
    fn lookup_model_pricing(engine_id: &str, model_id: &str) -> (f64, f64) {
        // Model-specific pricing (per 1M tokens)
        match (engine_id, model_id) {
            // OpenAI models
            ("openai", model) if model.starts_with("gpt-4") || model == "gpt-4-turbo" || model == "gpt-4o" => {
                (10.0, 30.0) // GPT-4 Turbo pricing
            }
            ("openai", model) if model.contains("gpt-3.5") || model == "gpt-3.5-turbo" => {
                (0.50, 1.50) // GPT-3.5 Turbo pricing
            }
            // Anthropic Claude models
            ("claude", model) | ("anthropic", model) if model.contains("opus") => {
                (15.0, 75.0) // Claude 3 Opus pricing
            }
            ("claude", model) | ("anthropic", model) if model.contains("sonnet") => {
                (3.0, 15.0) // Claude 3 Sonnet pricing (includes claude-3-5-sonnet)
            }
            ("claude", model) | ("anthropic", model) if model.contains("haiku") => {
                (0.25, 1.25) // Claude 3 Haiku pricing
            }
            // Google Gemini models
            ("gemini", model) if model.contains("ultra") => {
                (10.0, 30.0) // Gemini Ultra pricing
            }
            ("gemini", model) if model.contains("pro") => {
                (0.50, 1.50) // Gemini Pro pricing (paid tier)
            }
            // Fallback to provider-level defaults
            ("openai", _) => (10.0, 30.0), // Default OpenAI (assumes GPT-4 tier)
            ("claude", _) | ("anthropic", _) => (3.0, 15.0), // Default Anthropic (assumes Sonnet tier)
            ("gemini", _) => (0.50, 1.50), // Default Gemini (assumes Pro tier)
            _ => (5.0, 15.0), // Unknown provider default
        }
    }

    /// Estimates cost based on engine, model, and token usage.
    fn estimate_cost(engine_id: &str, model_id: &str, usage: &TokenUsage) -> f64 {
        let (input_price, output_price) = Self::lookup_model_pricing(engine_id, model_id);

        let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * input_price;
        let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * output_price;
        input_cost + output_cost
    }
}

impl EngineMetrics {
    /// Creates new engine metrics.
    pub fn new(engine_id: String) -> Self {
        Self {
            engine_id,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_latency_ms: 0.0,
            total_tokens: 0,
            total_cost: 0.0,
            model_metrics: HashMap::new(),
        }
    }

    /// Adds execution metrics to aggregation.
    pub fn add_execution(&mut self, metrics: &ExecutionMetrics) {
        self.total_executions += 1;
        if metrics.success {
            self.successful_executions += 1;
        } else {
            self.failed_executions += 1;
        }

        // Update average latency
        let total_latency = self.average_latency_ms * (self.total_executions - 1) as f64;
        self.average_latency_ms = (total_latency + metrics.latency_ms as f64) / self.total_executions as f64;

        // Update token usage and cost
        if let Some(ref usage) = metrics.token_usage {
            self.total_tokens += usage.total_tokens;
        }
        if let Some(cost) = metrics.estimated_cost {
            self.total_cost += cost;
        }

        // Update model metrics
        let model_metrics = self.model_metrics
            .entry(metrics.model.clone())
            .or_insert_with(|| ModelMetrics {
                model_id: metrics.model.clone(),
                total_executions: 0,
                average_latency_ms: 0.0,
                total_tokens: 0,
                total_cost: 0.0,
            });

        model_metrics.total_executions += 1;
        let model_total_latency = model_metrics.average_latency_ms * (model_metrics.total_executions - 1) as f64;
        model_metrics.average_latency_ms = (model_total_latency + metrics.latency_ms as f64) / model_metrics.total_executions as f64;

        if let Some(ref usage) = metrics.token_usage {
            model_metrics.total_tokens += usage.total_tokens;
        }
        if let Some(cost) = metrics.estimated_cost {
            model_metrics.total_cost += cost;
        }
    }

    /// Gets success rate as percentage.
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            return 0.0;
        }
        (self.successful_executions as f64 / self.total_executions as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_metrics() {
        let metrics = ExecutionMetrics::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            Duration::from_millis(1500),
            true,
        );

        assert_eq!(metrics.engine_id, "openai");
        assert_eq!(metrics.model, "gpt-4");
        assert_eq!(metrics.latency_ms, 1500);
        assert!(metrics.success);
    }

    #[test]
    fn test_engine_metrics_aggregation() {
        let mut engine_metrics = EngineMetrics::new("openai".to_string());

        let exec1 = ExecutionMetrics::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            Duration::from_millis(1000),
            true,
        );

        let exec2 = ExecutionMetrics::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            Duration::from_millis(2000),
            false,
        );

        engine_metrics.add_execution(&exec1);
        engine_metrics.add_execution(&exec2);

        assert_eq!(engine_metrics.total_executions, 2);
        assert_eq!(engine_metrics.successful_executions, 1);
        assert_eq!(engine_metrics.failed_executions, 1);
        assert_eq!(engine_metrics.average_latency_ms, 1500.0);
        assert_eq!(engine_metrics.success_rate(), 50.0);
    }

    #[test]
    fn test_cost_estimation_gpt_3_5() {
        // Test: Accurate cost calculation for GPT-3.5
        // Setup: 1M input tokens, 1M output tokens
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            total_tokens: 2_000_000,
        };

        let metrics = ExecutionMetrics::new(
            "openai".to_string(),
            "gpt-3.5-turbo".to_string(),
            Duration::from_millis(1000),
            true,
        )
        .with_token_usage(usage);

        // Expected: $0.50 (input) + $1.50 (output) = $2.00
        assert!((metrics.estimated_cost.unwrap() - 2.00).abs() < 0.01);
    }

    #[test]
    fn test_cost_estimation_claude_sonnet() {
        // Test: Accurate cost calculation for Claude Sonnet
        // Setup: 1M input tokens, 1M output tokens
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            total_tokens: 2_000_000,
        };

        let metrics = ExecutionMetrics::new(
            "claude".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            Duration::from_millis(1000),
            true,
        )
        .with_token_usage(usage);

        // Expected: $3.00 (input) + $15.00 (output) = $18.00
        assert!((metrics.estimated_cost.unwrap() - 18.00).abs() < 0.01);
    }

    #[test]
    fn test_cost_estimation_gpt_4() {
        // Test: Accurate cost calculation for GPT-4
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            total_tokens: 2_000_000,
        };

        let metrics = ExecutionMetrics::new(
            "openai".to_string(),
            "gpt-4-turbo".to_string(),
            Duration::from_millis(1000),
            true,
        )
        .with_token_usage(usage);

        // Expected: $10.00 (input) + $30.00 (output) = $40.00
        assert!((metrics.estimated_cost.unwrap() - 40.00).abs() < 0.01);
    }

    #[test]
    fn test_cost_estimation_claude_opus() {
        // Test: Accurate cost calculation for Claude Opus
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            total_tokens: 2_000_000,
        };

        let metrics = ExecutionMetrics::new(
            "claude".to_string(),
            "claude-3-opus-20240229".to_string(),
            Duration::from_millis(1000),
            true,
        )
        .with_token_usage(usage);

        // Expected: $15.00 (input) + $75.00 (output) = $90.00
        assert!((metrics.estimated_cost.unwrap() - 90.00).abs() < 0.01);
    }

    #[test]
    fn test_cost_estimation_claude_haiku() {
        // Test: Accurate cost calculation for Claude Haiku
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            total_tokens: 2_000_000,
        };

        let metrics = ExecutionMetrics::new(
            "claude".to_string(),
            "claude-3-haiku-20240307".to_string(),
            Duration::from_millis(1000),
            true,
        )
        .with_token_usage(usage);

        // Expected: $0.25 (input) + $1.25 (output) = $1.50
        assert!((metrics.estimated_cost.unwrap() - 1.50).abs() < 0.01);
    }

    #[test]
    fn test_cost_estimation_gemini_pro() {
        // Test: Accurate cost calculation for Gemini Pro
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            total_tokens: 2_000_000,
        };

        let metrics = ExecutionMetrics::new(
            "gemini".to_string(),
            "gemini-pro".to_string(),
            Duration::from_millis(1000),
            true,
        )
        .with_token_usage(usage);

        // Expected: $0.50 (input) + $1.50 (output) = $2.00
        assert!((metrics.estimated_cost.unwrap() - 2.00).abs() < 0.01);
    }
}

