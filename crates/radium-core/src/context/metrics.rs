//! Performance metrics for context system operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Metrics for context gathering operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetrics {
    /// Total context gathering time in milliseconds.
    pub total_time_ms: u64,
    
    /// Time per source type (source_type -> milliseconds).
    pub time_per_source_type: HashMap<String, u64>,
    
    /// Cache hit rate for context files (0.0 to 1.0).
    pub cache_hit_rate: f64,
    
    /// Number of cache hits.
    pub cache_hits: u64,
    
    /// Number of cache misses.
    pub cache_misses: u64,
    
    /// Memory store read latency in milliseconds.
    pub memory_read_latency_ms: Option<u64>,
    
    /// Memory store write latency in milliseconds.
    pub memory_write_latency_ms: Option<u64>,
    
    /// Source validation duration in milliseconds.
    pub validation_duration_ms: Option<u64>,
    
    /// Import resolution time in milliseconds.
    pub import_resolution_time_ms: Option<u64>,
    
    /// Template rendering time in milliseconds.
    pub template_rendering_time_ms: Option<u64>,
    
    /// Number of redactions performed during context building.
    pub redaction_count: usize,
    
    /// Timestamp when metrics were collected.
    pub timestamp: SystemTime,
}

impl ContextMetrics {
    /// Creates new empty metrics.
    pub fn new() -> Self {
        Self {
            total_time_ms: 0,
            time_per_source_type: HashMap::new(),
            cache_hit_rate: 0.0,
            cache_hits: 0,
            cache_misses: 0,
            memory_read_latency_ms: None,
            memory_write_latency_ms: None,
            validation_duration_ms: None,
            import_resolution_time_ms: None,
            template_rendering_time_ms: None,
            redaction_count: 0,
            timestamp: SystemTime::now(),
        }
    }

    /// Calculates cache hit rate from hits and misses.
    pub fn calculate_cache_hit_rate(&mut self) {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hit_rate = self.cache_hits as f64 / total as f64;
        }
    }

    /// Records time for a source type.
    pub fn record_source_time(&mut self, source_type: String, time_ms: u64) {
        self.time_per_source_type
            .entry(source_type)
            .and_modify(|e| *e += time_ms)
            .or_insert(time_ms);
    }
}

impl Default for ContextMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated metrics over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedContextMetrics {
    /// Total number of operations.
    pub total_operations: u64,
    
    /// Percentiles for total time (p50, p95, p99) in milliseconds.
    pub total_time_percentiles: Percentiles,
    
    /// Average cache hit rate.
    pub average_cache_hit_rate: f64,
    
    /// Average memory read latency in milliseconds.
    pub average_memory_read_latency_ms: Option<f64>,
    
    /// Average memory write latency in milliseconds.
    pub average_memory_write_latency_ms: Option<f64>,
    
    /// Average validation duration in milliseconds.
    pub average_validation_duration_ms: Option<f64>,
}

/// Percentile values (p50, p95, p99).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Percentiles {
    /// 50th percentile (median).
    pub p50: u64,
    /// 95th percentile.
    pub p95: u64,
    /// 99th percentile.
    pub p99: u64,
}

impl AggregatedContextMetrics {
    /// Aggregates metrics from a collection of individual metrics.
    pub fn from_metrics(metrics: &[ContextMetrics]) -> Self {
        if metrics.is_empty() {
            return Self {
                total_operations: 0,
                total_time_percentiles: Percentiles { p50: 0, p95: 0, p99: 0 },
                average_cache_hit_rate: 0.0,
                average_memory_read_latency_ms: None,
                average_memory_write_latency_ms: None,
                average_validation_duration_ms: None,
            };
        }

        let total_operations = metrics.len() as u64;
        
        // Calculate percentiles for total time
        let mut times: Vec<u64> = metrics.iter().map(|m| m.total_time_ms).collect();
        times.sort();
        let p50 = Self::percentile(&times, 0.50);
        let p95 = Self::percentile(&times, 0.95);
        let p99 = Self::percentile(&times, 0.99);
        
        // Calculate average cache hit rate
        let total_cache_hits: u64 = metrics.iter().map(|m| m.cache_hits).sum();
        let total_cache_misses: u64 = metrics.iter().map(|m| m.cache_misses).sum();
        let average_cache_hit_rate = if total_cache_hits + total_cache_misses > 0 {
            total_cache_hits as f64 / (total_cache_hits + total_cache_misses) as f64
        } else {
            0.0
        };
        
        // Calculate average memory latencies
        let memory_reads: Vec<u64> = metrics
            .iter()
            .filter_map(|m| m.memory_read_latency_ms)
            .collect();
        let average_memory_read_latency_ms = if !memory_reads.is_empty() {
            Some(memory_reads.iter().sum::<u64>() as f64 / memory_reads.len() as f64)
        } else {
            None
        };
        
        let memory_writes: Vec<u64> = metrics
            .iter()
            .filter_map(|m| m.memory_write_latency_ms)
            .collect();
        let average_memory_write_latency_ms = if !memory_writes.is_empty() {
            Some(memory_writes.iter().sum::<u64>() as f64 / memory_writes.len() as f64)
        } else {
            None
        };
        
        // Calculate average validation duration
        let validations: Vec<u64> = metrics
            .iter()
            .filter_map(|m| m.validation_duration_ms)
            .collect();
        let average_validation_duration_ms = if !validations.is_empty() {
            Some(validations.iter().sum::<u64>() as f64 / validations.len() as f64)
        } else {
            None
        };
        
        Self {
            total_operations,
            total_time_percentiles: Percentiles { p50, p95, p99 },
            average_cache_hit_rate,
            average_memory_read_latency_ms,
            average_memory_write_latency_ms,
            average_validation_duration_ms,
        }
    }
    
    /// Calculates a percentile value from a sorted vector.
    fn percentile(sorted_values: &[u64], percentile: f64) -> u64 {
        if sorted_values.is_empty() {
            return 0;
        }
        let index = ((sorted_values.len() - 1) as f64 * percentile).ceil() as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }
}

