//! A/B testing framework for model routing validation.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

/// A/B testing configuration.
#[derive(Debug, Clone)]
pub struct ABTestConfig {
    /// Whether A/B testing is enabled.
    pub enabled: bool,
    /// Sample rate for test group (0.0 to 1.0).
    pub sample_rate: f64,
}

impl Default for ABTestConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sample_rate: 0.1, // 10% by default
        }
    }
}

/// A/B test group assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ABTestGroup {
    /// Control group (normal routing).
    Control,
    /// Test group (inverted routing).
    Test,
}

impl ABTestGroup {
    /// Converts to string for telemetry.
    #[must_use]
    pub fn to_string(&self) -> String {
        match self {
            ABTestGroup::Control => "control".to_string(),
            ABTestGroup::Test => "test".to_string(),
        }
    }
}

/// A/B test sampler for random group assignment.
pub struct ABTestSampler {
    /// Configuration.
    config: ABTestConfig,
    /// Counter for pseudo-random sampling (thread-safe).
    counter: AtomicU64,
}

impl ABTestSampler {
    /// Creates a new A/B test sampler.
    ///
    /// # Arguments
    /// * `config` - A/B testing configuration
    #[must_use]
    pub fn new(config: ABTestConfig) -> Self {
        Self {
            config,
            counter: AtomicU64::new(0),
        }
    }
    
    /// Assigns a group for the next test.
    ///
    /// Uses pseudo-random sampling based on sample_rate to determine
    /// if the request should be in the Test group (inverted routing)
    /// or Control group (normal routing). Uses a counter-based hash
    /// approach that is thread-safe and Send/Sync compatible.
    ///
    /// # Returns
    /// ABTestGroup assignment
    pub fn assign_group(&self) -> ABTestGroup {
        if !self.config.enabled {
            return ABTestGroup::Control;
        }
        
        // Use counter-based hashing for thread-safe pseudo-random sampling
        let count = self.counter.fetch_add(1, Ordering::Relaxed);
        let mut hasher = DefaultHasher::new();
        count.hash(&mut hasher);
        std::thread::current().id().hash(&mut hasher);
        let hash = hasher.finish();
        
        // Convert hash to 0-1 range
        let random_value = (hash % 10_000) as f64 / 10_000.0;
        
        if random_value < self.config.sample_rate {
            ABTestGroup::Test
        } else {
            ABTestGroup::Control
        }
    }
    
    /// Gets the current configuration.
    #[must_use]
    pub fn config(&self) -> &ABTestConfig {
        &self.config
    }
}

/// A/B test comparison report.
#[derive(Debug, Clone)]
pub struct ABComparisonReport {
    /// Control group metrics.
    pub control: ABGroupMetrics,
    /// Test group metrics.
    pub test: ABGroupMetrics,
    /// Cost difference (test - control).
    pub cost_difference: f64,
    /// Success rate difference (test - control).
    pub success_rate_difference: f64,
}

/// Metrics for an A/B test group.
#[derive(Debug, Clone, Default)]
pub struct ABGroupMetrics {
    /// Number of requests.
    pub request_count: u64,
    /// Total cost in USD.
    pub total_cost: f64,
    /// Successful requests count.
    pub successful_requests: u64,
    /// Failed requests count.
    pub failed_requests: u64,
    /// Total tokens used.
    pub total_tokens: u64,
}

impl ABGroupMetrics {
    /// Calculates success rate (0.0 to 1.0).
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.request_count == 0 {
            return 0.0;
        }
        self.successful_requests as f64 / self.request_count as f64
    }
    
    /// Calculates average cost per request.
    #[must_use]
    pub fn avg_cost_per_request(&self) -> f64 {
        if self.request_count == 0 {
            return 0.0;
        }
        self.total_cost / self.request_count as f64
    }
}

/// Generates A/B comparison report from telemetry records.
///
/// This function analyzes telemetry records that have been tagged with
/// A/B test group assignments and generates comparison metrics.
///
/// # Arguments
/// * `control_records` - Telemetry records from control group
/// * `test_records` - Telemetry records from test group
///
/// # Returns
/// ABComparisonReport with aggregated metrics
pub fn generate_ab_comparison(
    control_records: &[crate::routing::ab_testing::ABGroupMetrics],
    test_records: &[crate::routing::ab_testing::ABGroupMetrics],
) -> ABComparisonReport {
    // Aggregate control group metrics
    let control = control_records.iter().fold(ABGroupMetrics::default(), |acc, m| {
        ABGroupMetrics {
            request_count: acc.request_count + m.request_count,
            total_cost: acc.total_cost + m.total_cost,
            successful_requests: acc.successful_requests + m.successful_requests,
            failed_requests: acc.failed_requests + m.failed_requests,
            total_tokens: acc.total_tokens + m.total_tokens,
        }
    });
    
    // Aggregate test group metrics
    let test = test_records.iter().fold(ABGroupMetrics::default(), |acc, m| {
        ABGroupMetrics {
            request_count: acc.request_count + m.request_count,
            total_cost: acc.total_cost + m.total_cost,
            successful_requests: acc.successful_requests + m.successful_requests,
            failed_requests: acc.failed_requests + m.failed_requests,
            total_tokens: acc.total_tokens + m.total_tokens,
        }
    });
    
    let cost_difference = test.avg_cost_per_request() - control.avg_cost_per_request();
    let success_rate_difference = test.success_rate() - control.success_rate();
    
    ABComparisonReport {
        control,
        test,
        cost_difference,
        success_rate_difference,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ab_test_sampler_disabled() {
        let config = ABTestConfig {
            enabled: false,
            sample_rate: 0.5,
        };
        let sampler = ABTestSampler::new(config);
        
        // Should always return Control when disabled
        for _ in 0..100 {
            assert_eq!(sampler.assign_group(), ABTestGroup::Control);
        }
    }
    
    #[test]
    fn test_ab_test_sampler_distribution() {
        let config = ABTestConfig {
            enabled: true,
            sample_rate: 0.1,
        };
        let sampler = ABTestSampler::new(config);
        
        // Sample 1000 assignments
        let mut test_count = 0;
        for _ in 0..1000 {
            if sampler.assign_group() == ABTestGroup::Test {
                test_count += 1;
            }
        }
        
        // Should be approximately 10% (90-110 range is acceptable)
        assert!(test_count >= 90 && test_count <= 110, "Expected ~100 test assignments, got {}", test_count);
    }
    
    #[test]
    fn test_ab_group_metrics() {
        let mut metrics = ABGroupMetrics::default();
        metrics.request_count = 10;
        metrics.successful_requests = 8;
        metrics.failed_requests = 2;
        metrics.total_cost = 1.0;
        
        assert_eq!(metrics.success_rate(), 0.8);
        assert_eq!(metrics.avg_cost_per_request(), 0.1);
    }
    
    #[test]
    fn test_generate_ab_comparison() {
        let control_metrics = vec![
            ABGroupMetrics {
                request_count: 5,
                total_cost: 0.5,
                successful_requests: 4,
                failed_requests: 1,
                total_tokens: 1000,
            },
        ];
        
        let test_metrics = vec![
            ABGroupMetrics {
                request_count: 5,
                total_cost: 0.3,
                successful_requests: 3,
                failed_requests: 2,
                total_tokens: 800,
            },
        ];
        
        let report = generate_ab_comparison(&control_metrics, &test_metrics);
        
        assert_eq!(report.control.request_count, 5);
        assert_eq!(report.test.request_count, 5);
        assert!(report.cost_difference < 0.0); // Test group cheaper
    }
}
