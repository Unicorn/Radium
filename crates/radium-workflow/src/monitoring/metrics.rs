//! Metrics Collection
//!
//! Provides counters, gauges, histograms, and timers for monitoring
//! workflow compilation and execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Monotonically increasing counter
    Counter,
    /// Value that can go up or down
    Gauge,
    /// Distribution of values
    Histogram,
    /// Timing measurements
    Timer,
}

/// A single metric value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// The metric name
    pub name: String,
    /// The metric type
    pub metric_type: MetricType,
    /// Current value
    pub value: f64,
    /// Labels/tags for the metric
    pub labels: HashMap<String, String>,
    /// When this value was recorded
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Thread-safe counter metric
#[derive(Debug, Default)]
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    /// Create a new counter
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment by a specific amount
    pub fn inc_by(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    /// Get current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Reset to zero
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

/// Thread-safe gauge metric
#[derive(Debug, Default)]
pub struct Gauge {
    value: AtomicI64,
}

impl Gauge {
    /// Create a new gauge
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the value
    pub fn set(&self, val: i64) {
        self.value.store(val, Ordering::Relaxed);
    }

    /// Increment by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement by 1
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Add to current value
    pub fn add(&self, n: i64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    /// Get current value
    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Histogram for tracking value distributions
#[derive(Debug)]
pub struct Histogram {
    /// Bucket boundaries
    buckets: Vec<f64>,
    /// Count per bucket
    bucket_counts: Vec<AtomicU64>,
    /// Sum of all values
    sum: RwLock<f64>,
    /// Total count
    count: AtomicU64,
}

impl Histogram {
    /// Create a histogram with custom buckets
    pub fn new(buckets: Vec<f64>) -> Self {
        let bucket_counts = buckets.iter().map(|_| AtomicU64::new(0)).collect();
        Self {
            buckets,
            bucket_counts,
            sum: RwLock::new(0.0),
            count: AtomicU64::new(0),
        }
    }

    /// Create a histogram with default buckets for latency (in ms)
    pub fn latency_buckets() -> Self {
        Self::new(vec![
            1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
        ])
    }

    /// Create a histogram with default buckets for sizes (in bytes)
    pub fn size_buckets() -> Self {
        Self::new(vec![
            100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0,
        ])
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        // Update sum
        {
            let mut sum = self.sum.write().unwrap();
            *sum += value;
        }

        // Update count
        self.count.fetch_add(1, Ordering::Relaxed);

        // Update buckets
        for (i, bucket) in self.buckets.iter().enumerate() {
            if value <= *bucket {
                self.bucket_counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Get the total count
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the sum of all values
    pub fn sum(&self) -> f64 {
        *self.sum.read().unwrap()
    }

    /// Get the mean
    pub fn mean(&self) -> f64 {
        let count = self.count();
        if count == 0 {
            0.0
        } else {
            self.sum() / count as f64
        }
    }

    /// Get bucket counts
    pub fn bucket_counts(&self) -> Vec<(f64, u64)> {
        self.buckets
            .iter()
            .zip(self.bucket_counts.iter())
            .map(|(b, c)| (*b, c.load(Ordering::Relaxed)))
            .collect()
    }
}

/// Timer for measuring durations
#[derive(Debug)]
pub struct Timer {
    histogram: Histogram,
}

impl Timer {
    /// Create a new timer with latency buckets
    pub fn new() -> Self {
        Self {
            histogram: Histogram::latency_buckets(),
        }
    }

    /// Create a timer with custom buckets (in milliseconds)
    pub fn with_buckets(buckets: Vec<f64>) -> Self {
        Self {
            histogram: Histogram::new(buckets),
        }
    }

    /// Start timing an operation
    pub fn start(&self) -> TimerGuard<'_> {
        TimerGuard {
            timer: self,
            start: Instant::now(),
        }
    }

    /// Record a duration directly
    pub fn record(&self, duration: Duration) {
        self.histogram.observe(duration.as_secs_f64() * 1000.0);
    }

    /// Get statistics
    pub fn stats(&self) -> TimerStats {
        TimerStats {
            count: self.histogram.count(),
            sum_ms: self.histogram.sum(),
            mean_ms: self.histogram.mean(),
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for timing
pub struct TimerGuard<'a> {
    timer: &'a Timer,
    start: Instant,
}

impl Drop for TimerGuard<'_> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.timer.record(duration);
    }
}

/// Timer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStats {
    /// Number of observations
    pub count: u64,
    /// Sum of all durations in milliseconds
    pub sum_ms: f64,
    /// Mean duration in milliseconds
    pub mean_ms: f64,
}

/// Metrics registry for managing all metrics
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    counters: RwLock<HashMap<String, Counter>>,
    gauges: RwLock<HashMap<String, Gauge>>,
    histograms: RwLock<HashMap<String, Histogram>>,
    timers: RwLock<HashMap<String, Timer>>,
}

impl MetricsRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or get a counter
    pub fn counter(&self, name: &str) -> CounterRef<'_> {
        let mut counters = self.counters.write().unwrap();
        if !counters.contains_key(name) {
            counters.insert(name.to_string(), Counter::new());
        }
        CounterRef {
            registry: self,
            name: name.to_string(),
        }
    }

    /// Register or get a gauge
    pub fn gauge(&self, name: &str) -> GaugeRef<'_> {
        let mut gauges = self.gauges.write().unwrap();
        if !gauges.contains_key(name) {
            gauges.insert(name.to_string(), Gauge::new());
        }
        GaugeRef {
            registry: self,
            name: name.to_string(),
        }
    }

    /// Register or get a histogram
    pub fn histogram(&self, name: &str) -> HistogramRef<'_> {
        let mut histograms = self.histograms.write().unwrap();
        if !histograms.contains_key(name) {
            histograms.insert(name.to_string(), Histogram::latency_buckets());
        }
        HistogramRef {
            registry: self,
            name: name.to_string(),
        }
    }

    /// Register or get a timer
    pub fn timer(&self, name: &str) -> TimerRef<'_> {
        let mut timers = self.timers.write().unwrap();
        if !timers.contains_key(name) {
            timers.insert(name.to_string(), Timer::new());
        }
        TimerRef {
            registry: self,
            name: name.to_string(),
        }
    }

    /// Get all metrics as a snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let now = chrono::Utc::now();

        let counters: HashMap<String, u64> = self
            .counters
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.get()))
            .collect();

        let gauges: HashMap<String, i64> = self
            .gauges
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.get()))
            .collect();

        let timers: HashMap<String, TimerStats> = self
            .timers
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.stats()))
            .collect();

        MetricsSnapshot {
            timestamp: now,
            counters,
            gauges,
            timers,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        for counter in self.counters.write().unwrap().values() {
            counter.reset();
        }
        for gauge in self.gauges.write().unwrap().values() {
            gauge.set(0);
        }
        // Note: histograms and timers don't have reset for simplicity
    }
}

/// Reference to a counter in the registry
pub struct CounterRef<'a> {
    registry: &'a MetricsRegistry,
    name: String,
}

impl CounterRef<'_> {
    /// Increment by 1
    pub fn inc(&self) {
        if let Some(counter) = self.registry.counters.read().unwrap().get(&self.name) {
            counter.inc();
        }
    }

    /// Increment by n
    pub fn inc_by(&self, n: u64) {
        if let Some(counter) = self.registry.counters.read().unwrap().get(&self.name) {
            counter.inc_by(n);
        }
    }

    /// Get current value
    pub fn get(&self) -> u64 {
        self.registry
            .counters
            .read()
            .unwrap()
            .get(&self.name)
            .map(|c| c.get())
            .unwrap_or(0)
    }
}

/// Reference to a gauge in the registry
pub struct GaugeRef<'a> {
    registry: &'a MetricsRegistry,
    name: String,
}

impl GaugeRef<'_> {
    /// Set the value
    pub fn set(&self, val: i64) {
        if let Some(gauge) = self.registry.gauges.read().unwrap().get(&self.name) {
            gauge.set(val);
        }
    }

    /// Increment
    pub fn inc(&self) {
        if let Some(gauge) = self.registry.gauges.read().unwrap().get(&self.name) {
            gauge.inc();
        }
    }

    /// Decrement
    pub fn dec(&self) {
        if let Some(gauge) = self.registry.gauges.read().unwrap().get(&self.name) {
            gauge.dec();
        }
    }

    /// Get current value
    pub fn get(&self) -> i64 {
        self.registry
            .gauges
            .read()
            .unwrap()
            .get(&self.name)
            .map(|g| g.get())
            .unwrap_or(0)
    }
}

/// Reference to a histogram in the registry
pub struct HistogramRef<'a> {
    registry: &'a MetricsRegistry,
    name: String,
}

impl HistogramRef<'_> {
    /// Observe a value
    pub fn observe(&self, value: f64) {
        if let Some(histogram) = self.registry.histograms.read().unwrap().get(&self.name) {
            histogram.observe(value);
        }
    }
}

/// Reference to a timer in the registry
pub struct TimerRef<'a> {
    registry: &'a MetricsRegistry,
    name: String,
}

impl TimerRef<'_> {
    /// Record a duration
    pub fn record(&self, duration: Duration) {
        if let Some(timer) = self.registry.timers.read().unwrap().get(&self.name) {
            timer.record(duration);
        }
    }

    /// Get timer statistics
    pub fn stats(&self) -> Option<TimerStats> {
        self.registry
            .timers
            .read()
            .unwrap()
            .get(&self.name)
            .map(|t| t.stats())
    }
}

/// Snapshot of all metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// When this snapshot was taken
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Counter values
    pub counters: HashMap<String, u64>,
    /// Gauge values
    pub gauges: HashMap<String, i64>,
    /// Timer statistics
    pub timers: HashMap<String, TimerStats>,
}

impl MetricsSnapshot {
    /// Format as Prometheus exposition format
    pub fn to_prometheus(&self) -> String {
        let mut output = String::new();

        for (name, value) in &self.counters {
            output.push_str(&format!(
                "# TYPE {} counter\n{} {}\n",
                name, name, value
            ));
        }

        for (name, value) in &self.gauges {
            output.push_str(&format!(
                "# TYPE {} gauge\n{} {}\n",
                name, name, value
            ));
        }

        for (name, stats) in &self.timers {
            output.push_str(&format!(
                "# TYPE {}_duration_ms summary\n{}_duration_ms_count {}\n{}_duration_ms_sum {}\n",
                name, name, stats.count, name, stats.sum_ms
            ));
        }

        output
    }
}

/// Pre-defined workflow metrics
pub struct WorkflowMetrics {
    registry: MetricsRegistry,
}

impl WorkflowMetrics {
    /// Create new workflow metrics
    pub fn new() -> Self {
        Self {
            registry: MetricsRegistry::new(),
        }
    }

    /// Record a compilation
    pub fn record_compilation(&self, duration: Duration, success: bool) {
        self.registry.counter("compilations_total").inc();
        self.registry.timer("compilation_duration").record(duration);

        if success {
            self.registry.counter("compilations_success").inc();
        } else {
            self.registry.counter("compilations_failed").inc();
        }
    }

    /// Record cache hit/miss
    pub fn record_cache_access(&self, hit: bool) {
        self.registry.counter("cache_accesses_total").inc();
        if hit {
            self.registry.counter("cache_hits").inc();
        } else {
            self.registry.counter("cache_misses").inc();
        }
    }

    /// Record validation
    pub fn record_validation(&self, duration: Duration, errors: usize, warnings: usize) {
        self.registry.counter("validations_total").inc();
        self.registry.timer("validation_duration").record(duration);
        self.registry
            .counter("validation_errors_total")
            .inc_by(errors as u64);
        self.registry
            .counter("validation_warnings_total")
            .inc_by(warnings as u64);
    }

    /// Set active compilations gauge
    pub fn set_active_compilations(&self, count: i64) {
        self.registry.gauge("active_compilations").set(count);
    }

    /// Get metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        self.registry.snapshot()
    }

    /// Get underlying registry
    pub fn registry(&self) -> &MetricsRegistry {
        &self.registry
    }
}

impl Default for WorkflowMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new();
        assert_eq!(counter.get(), 0);

        counter.inc();
        assert_eq!(counter.get(), 1);

        counter.inc_by(5);
        assert_eq!(counter.get(), 6);

        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new();
        assert_eq!(gauge.get(), 0);

        gauge.set(10);
        assert_eq!(gauge.get(), 10);

        gauge.inc();
        assert_eq!(gauge.get(), 11);

        gauge.dec();
        assert_eq!(gauge.get(), 10);

        gauge.add(-5);
        assert_eq!(gauge.get(), 5);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new(vec![10.0, 50.0, 100.0]);

        histogram.observe(5.0);
        histogram.observe(25.0);
        histogram.observe(75.0);

        assert_eq!(histogram.count(), 3);
        assert!((histogram.sum() - 105.0).abs() < 0.001);
        assert!((histogram.mean() - 35.0).abs() < 0.001);

        let buckets = histogram.bucket_counts();
        assert_eq!(buckets[0], (10.0, 1));  // 5.0 <= 10
        assert_eq!(buckets[1], (50.0, 2));  // 5.0, 25.0 <= 50
        assert_eq!(buckets[2], (100.0, 3)); // all <= 100
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new();

        timer.record(Duration::from_millis(10));
        timer.record(Duration::from_millis(20));
        timer.record(Duration::from_millis(30));

        let stats = timer.stats();
        assert_eq!(stats.count, 3);
        assert!((stats.sum_ms - 60.0).abs() < 1.0);
        assert!((stats.mean_ms - 20.0).abs() < 1.0);
    }

    #[test]
    fn test_metrics_registry() {
        let registry = MetricsRegistry::new();

        registry.counter("requests").inc();
        registry.counter("requests").inc();
        registry.gauge("active").set(5);

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.counters.get("requests"), Some(&2));
        assert_eq!(snapshot.gauges.get("active"), Some(&5));
    }

    #[test]
    fn test_workflow_metrics() {
        let metrics = WorkflowMetrics::new();

        metrics.record_compilation(Duration::from_millis(100), true);
        metrics.record_compilation(Duration::from_millis(50), false);
        metrics.record_cache_access(true);
        metrics.record_cache_access(false);
        metrics.set_active_compilations(3);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.counters.get("compilations_total"), Some(&2));
        assert_eq!(snapshot.counters.get("compilations_success"), Some(&1));
        assert_eq!(snapshot.counters.get("compilations_failed"), Some(&1));
        assert_eq!(snapshot.counters.get("cache_hits"), Some(&1));
        assert_eq!(snapshot.counters.get("cache_misses"), Some(&1));
        assert_eq!(snapshot.gauges.get("active_compilations"), Some(&3));
    }

    #[test]
    fn test_prometheus_export() {
        let registry = MetricsRegistry::new();
        registry.counter("test_counter").inc_by(42);
        registry.gauge("test_gauge").set(100);

        let snapshot = registry.snapshot();
        let prometheus = snapshot.to_prometheus();

        assert!(prometheus.contains("# TYPE test_counter counter"));
        assert!(prometheus.contains("test_counter 42"));
        assert!(prometheus.contains("# TYPE test_gauge gauge"));
        assert!(prometheus.contains("test_gauge 100"));
    }
}
