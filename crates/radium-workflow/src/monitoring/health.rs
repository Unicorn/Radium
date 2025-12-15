//! Health Check Module
//!
//! Provides health check infrastructure for monitoring service health.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is healthy
    Healthy,
    /// Component is degraded but functional
    Degraded,
    /// Component is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

impl HealthStatus {
    /// Check if status indicates the service can accept traffic
    pub fn is_ready(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Check if status indicates the service is alive
    pub fn is_alive(&self) -> bool {
        !matches!(self, HealthStatus::Unhealthy)
    }

    /// Get HTTP status code for this health status
    pub fn http_status_code(&self) -> u16 {
        match self {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200,
            HealthStatus::Unhealthy => 503,
            HealthStatus::Unknown => 503,
        }
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Result of a single health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Name of the component checked
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Optional message with details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// How long the check took
    pub duration_ms: u64,
    /// When the check was performed
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy(name: impl Into<String>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            duration_ms: duration.as_millis() as u64,
            checked_at: chrono::Utc::now(),
        }
    }

    /// Create a degraded result
    pub fn degraded(name: impl Into<String>, message: impl Into<String>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            duration_ms: duration.as_millis() as u64,
            checked_at: chrono::Utc::now(),
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            duration_ms: duration.as_millis() as u64,
            checked_at: chrono::Utc::now(),
        }
    }

    /// Create an unknown result
    pub fn unknown(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unknown,
            message: Some(message.into()),
            duration_ms: 0,
            checked_at: chrono::Utc::now(),
        }
    }
}

/// Aggregated health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall health status
    pub status: HealthStatus,
    /// Individual component checks
    pub checks: Vec<HealthCheckResult>,
    /// Service version
    pub version: String,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
    /// When this report was generated
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthReport {
    /// Check if the service is ready to accept traffic
    pub fn is_ready(&self) -> bool {
        self.status.is_ready()
    }

    /// Check if the service is alive
    pub fn is_alive(&self) -> bool {
        self.status.is_alive()
    }

    /// Get HTTP status code
    pub fn http_status_code(&self) -> u16 {
        self.status.http_status_code()
    }
}

/// A health check function
pub type HealthCheckFn = Box<dyn Fn() -> HealthCheckResult + Send + Sync>;

/// Health check registry
pub struct HealthChecker {
    /// Registered health checks
    checks: RwLock<HashMap<String, HealthCheckFn>>,
    /// When the service started
    start_time: Instant,
    /// Service version
    version: String,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            checks: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
            version: version.into(),
        }
    }

    /// Register a health check
    pub fn register<F>(&self, name: impl Into<String>, check: F)
    where
        F: Fn() -> HealthCheckResult + Send + Sync + 'static,
    {
        let mut checks = self.checks.write().unwrap();
        checks.insert(name.into(), Box::new(check));
    }

    /// Run all health checks
    pub fn check_all(&self) -> HealthReport {
        let checks = self.checks.read().unwrap();
        let mut results = Vec::with_capacity(checks.len());
        let mut overall_status = HealthStatus::Healthy;

        for (name, check_fn) in checks.iter() {
            let start = Instant::now();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check_fn()));

            let check_result = match result {
                Ok(r) => r,
                Err(_) => HealthCheckResult::unhealthy(
                    name.clone(),
                    "Health check panicked",
                    start.elapsed(),
                ),
            };

            // Update overall status (worst wins)
            overall_status = match (&overall_status, &check_result.status) {
                (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
                (HealthStatus::Unhealthy, _) => HealthStatus::Unhealthy,
                (_, HealthStatus::Unknown) => HealthStatus::Degraded,
                (HealthStatus::Unknown, _) => HealthStatus::Degraded,
                (_, HealthStatus::Degraded) => HealthStatus::Degraded,
                (HealthStatus::Degraded, _) => HealthStatus::Degraded,
                _ => HealthStatus::Healthy,
            };

            results.push(check_result);
        }

        HealthReport {
            status: overall_status,
            checks: results,
            version: self.version.clone(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Run a specific health check
    pub fn check(&self, name: &str) -> Option<HealthCheckResult> {
        let checks = self.checks.read().unwrap();
        checks.get(name).map(|check_fn| {
            let start = Instant::now();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check_fn()));
            match result {
                Ok(r) => r,
                Err(_) => HealthCheckResult::unhealthy(
                    name,
                    "Health check panicked",
                    start.elapsed(),
                ),
            }
        })
    }

    /// Get service uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Liveness probe (is the service alive?)
    pub fn liveness(&self) -> HealthCheckResult {
        HealthCheckResult::healthy("liveness", Duration::ZERO)
    }

    /// Readiness probe (is the service ready to accept traffic?)
    pub fn readiness(&self) -> HealthReport {
        self.check_all()
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(env!("CARGO_PKG_VERSION"))
    }
}

/// Pre-built health checks for common components
pub mod checks {
    use super::*;

    /// Create a cache health check
    pub fn cache_health<F>(get_stats: F) -> impl Fn() -> HealthCheckResult + Send + Sync
    where
        F: Fn() -> (usize, usize) + Send + Sync + 'static, // (size, capacity)
    {
        move || {
            let start = Instant::now();
            let (size, capacity) = get_stats();
            let usage = size as f64 / capacity as f64;

            if usage > 0.95 {
                HealthCheckResult::degraded(
                    "cache",
                    format!("Cache is {}% full ({}/{})", (usage * 100.0) as u32, size, capacity),
                    start.elapsed(),
                )
            } else {
                HealthCheckResult::healthy("cache", start.elapsed())
            }
        }
    }

    /// Create a memory health check
    pub fn memory_health(_threshold_mb: u64) -> impl Fn() -> HealthCheckResult + Send + Sync {
        move || {
            let start = Instant::now();
            // Note: In a real implementation, you'd use system APIs to get memory usage
            // This is a placeholder that always returns healthy
            HealthCheckResult::healthy("memory", start.elapsed())
        }
    }

    /// Create a simple ping health check
    pub fn ping() -> impl Fn() -> HealthCheckResult + Send + Sync {
        || HealthCheckResult::healthy("ping", Duration::ZERO)
    }

    /// Create a check that validates a condition
    pub fn condition<F>(name: &'static str, check: F) -> impl Fn() -> HealthCheckResult + Send + Sync
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        move || {
            let start = Instant::now();
            match check() {
                Ok(()) => HealthCheckResult::healthy(name, start.elapsed()),
                Err(msg) => HealthCheckResult::unhealthy(name, msg, start.elapsed()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.is_ready());
        assert!(HealthStatus::Degraded.is_ready());
        assert!(!HealthStatus::Unhealthy.is_ready());
        assert!(!HealthStatus::Unknown.is_ready());

        assert!(HealthStatus::Healthy.is_alive());
        assert!(HealthStatus::Degraded.is_alive());
        assert!(!HealthStatus::Unhealthy.is_alive());
        assert!(HealthStatus::Unknown.is_alive());

        assert_eq!(HealthStatus::Healthy.http_status_code(), 200);
        assert_eq!(HealthStatus::Unhealthy.http_status_code(), 503);
    }

    #[test]
    fn test_health_check_result() {
        let healthy = HealthCheckResult::healthy("test", Duration::from_millis(5));
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert!(healthy.message.is_none());

        let degraded = HealthCheckResult::degraded("test", "slow response", Duration::from_millis(10));
        assert_eq!(degraded.status, HealthStatus::Degraded);
        assert!(degraded.message.is_some());

        let unhealthy = HealthCheckResult::unhealthy("test", "connection failed", Duration::from_millis(1000));
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_checker() {
        let checker = HealthChecker::new("1.0.0");

        checker.register("always_healthy", || {
            HealthCheckResult::healthy("always_healthy", Duration::from_millis(1))
        });

        checker.register("always_degraded", || {
            HealthCheckResult::degraded("always_degraded", "test", Duration::from_millis(1))
        });

        let report = checker.check_all();
        assert_eq!(report.checks.len(), 2);
        assert_eq!(report.status, HealthStatus::Degraded);
        assert_eq!(report.version, "1.0.0");
    }

    #[test]
    fn test_health_checker_unhealthy() {
        let checker = HealthChecker::new("1.0.0");

        checker.register("unhealthy", || {
            HealthCheckResult::unhealthy("unhealthy", "broken", Duration::ZERO)
        });

        let report = checker.check_all();
        assert_eq!(report.status, HealthStatus::Unhealthy);
        assert!(!report.is_ready());
        assert!(!report.is_alive());
    }

    #[test]
    fn test_liveness_readiness() {
        let checker = HealthChecker::new("1.0.0");

        let liveness = checker.liveness();
        assert_eq!(liveness.status, HealthStatus::Healthy);

        let readiness = checker.readiness();
        assert!(readiness.is_ready());
    }

    #[test]
    fn test_cache_health_check() {
        let check = checks::cache_health(|| (50, 100));
        let result = check();
        assert_eq!(result.status, HealthStatus::Healthy);

        let check = checks::cache_health(|| (98, 100));
        let result = check();
        assert_eq!(result.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_condition_check() {
        let check = checks::condition("test", || Ok(()));
        let result = check();
        assert_eq!(result.status, HealthStatus::Healthy);

        let check = checks::condition("test", || Err("failed".to_string()));
        let result = check();
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }
}
