//! Example metrics hook implementation.
//!
//! This hook aggregates telemetry data including token usage and cost tracking.
//! It demonstrates how to implement hooks for TelemetryCollection hook points.

use async_trait::async_trait;
use radium_core::hooks::registry::{Hook, HookType};
use radium_core::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Aggregated metrics data.
#[derive(Debug, Clone, Default)]
struct MetricsData {
    total_input_tokens: u64,
    total_output_tokens: u64,
    total_tokens: u64,
    total_cost: f64,
    call_count: u64,
    models: std::collections::HashMap<String, ModelMetrics>,
}

#[derive(Debug, Clone, Default)]
struct ModelMetrics {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    cost: f64,
    call_count: u64,
}

/// Metrics hook that aggregates telemetry data.
pub struct MetricsHook {
    name: String,
    priority: HookPriority,
    metrics: Arc<RwLock<MetricsData>>,
}

impl MetricsHook {
    /// Create a new metrics hook.
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
            metrics: Arc::new(RwLock::new(MetricsData::default())),
        }
    }

    /// Get current metrics summary.
    pub async fn get_summary(&self) -> Value {
        let metrics = self.metrics.read().await;
        json!({
            "total_input_tokens": metrics.total_input_tokens,
            "total_output_tokens": metrics.total_output_tokens,
            "total_tokens": metrics.total_tokens,
            "total_cost": metrics.total_cost,
            "call_count": metrics.call_count,
            "models": metrics.models.iter().map(|(k, v)| {
                json!({
                    "model": k,
                    "input_tokens": v.input_tokens,
                    "output_tokens": v.output_tokens,
                    "total_tokens": v.total_tokens,
                    "cost": v.cost,
                    "call_count": v.call_count,
                })
            }).collect::<Vec<_>>(),
        })
    }

    /// Reset metrics.
    pub async fn reset(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = MetricsData::default();
    }
}

#[async_trait]
impl Hook for MetricsHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    fn hook_type(&self) -> HookType {
        HookType::TelemetryCollection
    }

    async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
        // Extract telemetry data from context
        let data = &context.data;
        
        let input_tokens = data.get("input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let output_tokens = data.get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total_tokens = data.get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(input_tokens + output_tokens);
        let estimated_cost = data.get("estimated_cost")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let model = data.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let provider = data.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let agent_id = data.get("agent_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Update aggregated metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_input_tokens += input_tokens;
            metrics.total_output_tokens += output_tokens;
            metrics.total_tokens += total_tokens;
            metrics.total_cost += estimated_cost;
            metrics.call_count += 1;

            // Update per-model metrics
            let model_key = format!("{}:{}", provider, model);
            let model_metrics = metrics.models.entry(model_key.clone()).or_insert_with(|| ModelMetrics::default());
            model_metrics.input_tokens += input_tokens;
            model_metrics.output_tokens += output_tokens;
            model_metrics.total_tokens += total_tokens;
            model_metrics.cost += estimated_cost;
            model_metrics.call_count += 1;
        }

        // Log summary periodically (every 10 calls)
        {
            let metrics = self.metrics.read().await;
            if metrics.call_count % 10 == 0 {
                info!(
                    hook = %self.name,
                    total_tokens = metrics.total_tokens,
                    total_cost = metrics.total_cost,
                    call_count = metrics.call_count,
                    "Metrics summary: {} total tokens, ${:.4} total cost, {} calls",
                    metrics.total_tokens,
                    metrics.total_cost,
                    metrics.call_count
                );
            }
        }

        // Return success with optional modified data (could add custom metrics)
        Ok(HookExecutionResult::success())
    }
}

/// Create a metrics hook.
pub fn create_metrics_hook() -> Arc<dyn Hook> {
    Arc::new(MetricsHook::new("metrics-hook", 100))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_hook() {
        let hook = MetricsHook::new("test-metrics", 100);
        
        let context = HookContext::new(
            "telemetry_collection",
            json!({
                "agent_id": "test-agent",
                "input_tokens": 100,
                "output_tokens": 50,
                "total_tokens": 150,
                "estimated_cost": 0.001,
                "model": "test-model",
                "provider": "test-provider",
            }),
        );
        
        let result = hook.execute(&context).await.unwrap();
        assert!(result.success);
        assert!(result.should_continue);

        // Check that metrics were updated
        let summary = hook.get_summary().await;
        assert_eq!(summary["total_tokens"], 150);
        assert_eq!(summary["total_cost"], 0.001);
        assert_eq!(summary["call_count"], 1);
    }

    #[tokio::test]
    async fn test_metrics_aggregation() {
        let hook = MetricsHook::new("test-metrics", 100);
        
        // Add multiple telemetry records
        for i in 0..5 {
            let context = HookContext::new(
                "telemetry_collection",
                json!({
                    "agent_id": "test-agent",
                    "input_tokens": 100 + i,
                    "output_tokens": 50 + i,
                    "total_tokens": 150 + (i * 2),
                    "estimated_cost": 0.001 + (i as f64 * 0.0001),
                    "model": "test-model",
                    "provider": "test-provider",
                }),
            );
            
            hook.execute(&context).await.unwrap();
        }

        let summary = hook.get_summary().await;
        assert_eq!(summary["call_count"], 5);
        assert!(summary["total_tokens"].as_u64().unwrap() > 150);
    }
}

