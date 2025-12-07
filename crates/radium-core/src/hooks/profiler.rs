//! Hook performance profiling and monitoring.

use crate::hooks::registry::HookType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Statistics for a single hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookStats {
    /// Hook name.
    pub name: String,
    /// Hook type.
    pub hook_type: HookType,
    /// Total number of executions.
    pub execution_count: u64,
    /// Minimum execution time in microseconds.
    pub min_time_us: u64,
    /// Maximum execution time in microseconds.
    pub max_time_us: u64,
    /// Average execution time in microseconds.
    pub avg_time_us: f64,
    /// Total execution time in microseconds.
    pub total_time_us: u64,
}

impl Default for HookStats {
    fn default() -> Self {
        Self {
            name: String::new(),
            hook_type: HookType::BeforeModel,
            execution_count: 0,
            min_time_us: 0,
            max_time_us: 0,
            avg_time_us: 0.0,
            total_time_us: 0,
        }
    }
}

impl HookStats {
    /// Create a new hook stats instance.
    pub fn new(name: String, hook_type: HookType) -> Self {
        Self {
            name,
            hook_type,
            ..Default::default()
        }
    }

    /// Record an execution time.
    pub fn record_execution(&mut self, duration: Duration) {
        let time_us = duration.as_micros() as u64;
        self.execution_count += 1;
        self.total_time_us += time_us;

        if self.execution_count == 1 {
            self.min_time_us = time_us;
            self.max_time_us = time_us;
        } else {
            self.min_time_us = self.min_time_us.min(time_us);
            self.max_time_us = self.max_time_us.max(time_us);
        }

        self.avg_time_us = self.total_time_us as f64 / self.execution_count as f64;
    }
}

/// Aggregate statistics for a hook type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStats {
    /// Hook type.
    pub hook_type: HookType,
    /// Total number of hook executions across all hooks of this type.
    pub total_executions: u64,
    /// Total execution time in microseconds.
    pub total_time_us: u64,
    /// Average execution time per hook in microseconds.
    pub avg_time_us: f64,
    /// Number of unique hooks of this type.
    pub hook_count: usize,
}

/// Hook profiler for collecting performance metrics.
pub struct HookProfiler {
    /// Whether profiling is enabled.
    enabled: bool,
    /// Per-hook statistics.
    hook_stats: Arc<RwLock<HashMap<String, HookStats>>>,
    /// Per-type aggregate statistics.
    type_stats: Arc<RwLock<HashMap<HookType, TypeStats>>>,
}

impl HookProfiler {
    /// Create a new hook profiler.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            hook_stats: Arc::new(RwLock::new(HashMap::new())),
            type_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a disabled profiler (default for production).
    pub fn disabled() -> Self {
        Self::new(false)
    }

    /// Create an enabled profiler.
    pub fn enabled() -> Self {
        Self::new(true)
    }

    /// Enable or disable profiling.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if profiling is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record a hook execution.
    ///
    /// This is a no-op if profiling is disabled, ensuring minimal overhead.
    pub async fn record_execution(
        &self,
        hook_name: &str,
        hook_type: HookType,
        duration: Duration,
    ) {
        if !self.enabled {
            return;
        }

        // Update per-hook stats
        let mut stats = self.hook_stats.write().await;
        let hook_stat = stats
            .entry(hook_name.to_string())
            .or_insert_with(|| HookStats::new(hook_name.to_string(), hook_type));
        hook_stat.record_execution(duration);

        // Update per-type stats
        let mut type_stats = self.type_stats.write().await;
        let type_stat = type_stats
            .entry(hook_type)
            .or_insert_with(|| TypeStats {
                hook_type,
                total_executions: 0,
                total_time_us: 0,
                avg_time_us: 0.0,
                hook_count: 0,
            });

        type_stat.total_executions += 1;
        type_stat.total_time_us += duration.as_micros() as u64;
        type_stat.avg_time_us = type_stat.total_time_us as f64 / type_stat.total_executions as f64;

        // Update hook count (count unique hooks for this type)
        let hook_names_for_type: Vec<String> = stats
            .values()
            .filter(|s| s.hook_type == hook_type)
            .map(|s| s.name.clone())
            .collect();
        type_stat.hook_count = hook_names_for_type.len();
    }

    /// Get statistics for a specific hook.
    pub async fn get_hook_stats(&self, name: &str) -> Option<HookStats> {
        let stats = self.hook_stats.read().await;
        stats.get(name).cloned()
    }

    /// Get aggregate statistics for a hook type.
    pub async fn get_type_stats(&self, hook_type: HookType) -> Option<TypeStats> {
        let type_stats = self.type_stats.read().await;
        type_stats.get(&hook_type).cloned()
    }

    /// Get all hook statistics.
    pub async fn get_all_hook_stats(&self) -> Vec<HookStats> {
        let stats = self.hook_stats.read().await;
        stats.values().cloned().collect()
    }

    /// Get all type statistics.
    pub async fn get_all_type_stats(&self) -> Vec<TypeStats> {
        let type_stats = self.type_stats.read().await;
        type_stats.values().cloned().collect()
    }

    /// Get complete profiling report.
    pub async fn get_all_stats(&self) -> ProfilingReport {
        let hook_stats = self.get_all_hook_stats().await;
        let type_stats = self.get_all_type_stats().await;

        ProfilingReport {
            enabled: self.enabled,
            hook_stats,
            type_stats,
        }
    }

    /// Reset all profiling data.
    pub async fn reset(&self) {
        let mut stats = self.hook_stats.write().await;
        stats.clear();
        let mut type_stats = self.type_stats.write().await;
        type_stats.clear();
    }
}

/// Complete profiling report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingReport {
    /// Whether profiling was enabled when this report was generated.
    pub enabled: bool,
    /// Statistics for all hooks.
    pub hook_stats: Vec<HookStats>,
    /// Aggregate statistics by hook type.
    pub type_stats: Vec<TypeStats>,
}

impl ProfilingReport {
    /// Format as human-readable text.
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        if !self.enabled {
            output.push_str("Profiling is disabled.\n");
            return output;
        }

        if self.hook_stats.is_empty() {
            output.push_str("No profiling data collected yet.\n");
            return output;
        }

        output.push_str("Hook Performance Profiling Report\n");
        output.push_str("==================================\n\n");

        // Type statistics
        if !self.type_stats.is_empty() {
            output.push_str("Aggregate Statistics by Hook Type:\n");
            output.push_str("----------------------------------\n");
            for type_stat in &self.type_stats {
                output.push_str(&format!(
                    "  {}: {} executions, avg {:.2}μs, {} hooks\n",
                    type_stat.hook_type.as_str(),
                    type_stat.total_executions,
                    type_stat.avg_time_us,
                    type_stat.hook_count
                ));
            }
            output.push_str("\n");
        }

        // Hook statistics
        output.push_str("Per-Hook Statistics:\n");
        output.push_str("--------------------\n");
        for stat in &self.hook_stats {
            output.push_str(&format!(
                "  {} ({}):\n",
                stat.name,
                stat.hook_type.as_str()
            ));
            output.push_str(&format!("    Executions: {}\n", stat.execution_count));
            output.push_str(&format!(
                "    Time: min {}μs, max {}μs, avg {:.2}μs, total {}μs\n",
                stat.min_time_us, stat.max_time_us, stat.avg_time_us, stat.total_time_us
            ));
        }

        // Recommendations
        output.push_str("\nRecommendations:\n");
        output.push_str("----------------\n");
        let slow_hooks: Vec<&HookStats> = self
            .hook_stats
            .iter()
            .filter(|s| s.avg_time_us > 10000.0) // > 10ms average
            .collect();

        if slow_hooks.is_empty() {
            output.push_str("  All hooks are performing well (< 10ms average).\n");
        } else {
            output.push_str("  Consider optimizing these hooks:\n");
            for hook in slow_hooks {
                output.push_str(&format!(
                    "    - {} (avg {:.2}μs)\n",
                    hook.name, hook.avg_time_us
                ));
            }
        }

        output
    }

    /// Format as JSON.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for HookProfiler {
    fn default() -> Self {
        Self::disabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_profiler_disabled_no_overhead() {
        let profiler = HookProfiler::disabled();
        assert!(!profiler.is_enabled());

        // Should be a no-op
        profiler
            .record_execution("test-hook", HookType::BeforeModel, Duration::from_millis(10))
            .await;

        let stats = profiler.get_hook_stats("test-hook").await;
        assert!(stats.is_none());
    }

    #[tokio::test]
    async fn test_profiler_records_executions() {
        let profiler = HookProfiler::enabled();
        assert!(profiler.is_enabled());

        profiler
            .record_execution("test-hook", HookType::BeforeModel, Duration::from_millis(10))
            .await;
        profiler
            .record_execution("test-hook", HookType::BeforeModel, Duration::from_millis(20))
            .await;
        profiler
            .record_execution("test-hook", HookType::BeforeModel, Duration::from_millis(30))
            .await;

        let stats = profiler.get_hook_stats("test-hook").await.unwrap();
        assert_eq!(stats.execution_count, 3);
        assert_eq!(stats.min_time_us, 10000);
        assert_eq!(stats.max_time_us, 30000);
        assert!((stats.avg_time_us - 20000.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_profiler_type_stats() {
        let profiler = HookProfiler::enabled();

        profiler
            .record_execution("hook1", HookType::BeforeModel, Duration::from_millis(10))
            .await;
        profiler
            .record_execution("hook2", HookType::BeforeModel, Duration::from_millis(20))
            .await;

        let type_stats = profiler.get_type_stats(HookType::BeforeModel).await.unwrap();
        assert_eq!(type_stats.total_executions, 2);
        assert_eq!(type_stats.hook_count, 2);
    }

    #[tokio::test]
    async fn test_profiler_reset() {
        let profiler = HookProfiler::enabled();

        profiler
            .record_execution("test-hook", HookType::BeforeModel, Duration::from_millis(10))
            .await;

        let stats = profiler.get_hook_stats("test-hook").await;
        assert!(stats.is_some());

        profiler.reset().await;

        let stats = profiler.get_hook_stats("test-hook").await;
        assert!(stats.is_none());
    }

    #[tokio::test]
    async fn test_profiling_report_format() {
        let profiler = HookProfiler::enabled();

        profiler
            .record_execution("test-hook", HookType::BeforeModel, Duration::from_millis(10))
            .await;

        let report = profiler.get_all_stats().await;
        let text = report.to_text();
        assert!(text.contains("Hook Performance Profiling Report"));
        assert!(text.contains("test-hook"));
    }
}

