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
        self.estimated_cost = Some(Self::estimate_cost(&self.engine_id, &usage));
        self
    }

    /// Estimates cost based on engine and token usage.
    fn estimate_cost(engine_id: &str, usage: &TokenUsage) -> f64 {
        // Basic pricing estimates (per 1M tokens)
        let (input_price, output_price) = match engine_id {
            "openai" => (10.0, 30.0), // GPT-4 approximate
            "claude" => (15.0, 75.0), // Claude 3 Sonnet approximate
            "gemini" => (0.0, 0.0),   // Free tier
            _ => (5.0, 15.0),          // Default estimate
        };

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
}

