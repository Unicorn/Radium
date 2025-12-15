//! Rate Limiting Module
//!
//! Provides rate limiting infrastructure for API protection
//! using token bucket and sliding window algorithms.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u64,
    /// Window duration
    pub window: Duration,
    /// Burst allowance (extra requests allowed in short bursts)
    pub burst: u64,
    /// Whether to enable rate limiting
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            burst: 10,
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    /// Create a config for compilations (lower rate)
    pub fn for_compilations() -> Self {
        Self {
            max_requests: 10,
            window: Duration::from_secs(60),
            burst: 5,
            enabled: true,
        }
    }

    /// Create a config for API requests (higher rate)
    pub fn for_api() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            burst: 20,
            enabled: true,
        }
    }

    /// Create an unlimited config (for testing)
    pub fn unlimited() -> Self {
        Self {
            max_requests: u64::MAX,
            window: Duration::from_secs(1),
            burst: u64::MAX,
            enabled: false,
        }
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Remaining requests in current window
    pub remaining: u64,
    /// When the rate limit resets
    pub reset_at: Instant,
    /// Total limit
    pub limit: u64,
    /// Retry after duration (if not allowed)
    pub retry_after: Option<Duration>,
}

impl RateLimitResult {
    /// Create an allowed result
    pub fn allowed(remaining: u64, reset_at: Instant, limit: u64) -> Self {
        Self {
            allowed: true,
            remaining,
            reset_at,
            limit,
            retry_after: None,
        }
    }

    /// Create a denied result
    pub fn denied(reset_at: Instant, limit: u64) -> Self {
        let retry_after = reset_at.saturating_duration_since(Instant::now());
        Self {
            allowed: false,
            remaining: 0,
            reset_at,
            limit,
            retry_after: Some(retry_after),
        }
    }

    /// Get seconds until reset
    pub fn reset_in_seconds(&self) -> u64 {
        self.reset_at.saturating_duration_since(Instant::now()).as_secs()
    }
}

/// Token bucket state for a single client
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Current token count
    tokens: f64,
    /// Last refill time
    last_refill: Instant,
    /// Maximum tokens (capacity)
    capacity: f64,
    /// Refill rate (tokens per second)
    refill_rate: f64,
}

impl TokenBucket {
    fn new(capacity: u64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity as f64,
            last_refill: Instant::now(),
            capacity: capacity as f64,
            refill_rate,
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn time_until_available(&self, tokens: f64) -> Duration {
        if self.tokens >= tokens {
            Duration::ZERO
        } else {
            let needed = tokens - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }
}

/// Sliding window state for a single client
#[derive(Debug, Clone)]
struct SlidingWindow {
    /// Request timestamps in the current window
    requests: Vec<Instant>,
    /// Window duration
    window: Duration,
    /// Maximum requests per window
    max_requests: u64,
}

impl SlidingWindow {
    fn new(max_requests: u64, window: Duration) -> Self {
        Self {
            requests: Vec::new(),
            window,
            max_requests,
        }
    }

    fn cleanup(&mut self) {
        let cutoff = Instant::now() - self.window;
        self.requests.retain(|&t| t > cutoff);
    }

    fn try_add(&mut self) -> bool {
        self.cleanup();
        if (self.requests.len() as u64) < self.max_requests {
            self.requests.push(Instant::now());
            true
        } else {
            false
        }
    }

    fn remaining(&mut self) -> u64 {
        self.cleanup();
        self.max_requests.saturating_sub(self.requests.len() as u64)
    }

    fn reset_at(&self) -> Instant {
        if let Some(&oldest) = self.requests.first() {
            oldest + self.window
        } else {
            Instant::now() + self.window
        }
    }
}

/// Rate limiter using token bucket algorithm
pub struct TokenBucketLimiter {
    config: RateLimitConfig,
    buckets: RwLock<HashMap<String, TokenBucket>>,
}

impl TokenBucketLimiter {
    /// Create a new token bucket rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a request is allowed for a client
    pub fn check(&self, client_id: &str) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult::allowed(
                self.config.max_requests,
                Instant::now() + self.config.window,
                self.config.max_requests,
            );
        }

        let mut buckets = self.buckets.write().unwrap();
        let refill_rate = self.config.max_requests as f64 / self.config.window.as_secs_f64();
        let capacity = self.config.max_requests + self.config.burst;

        let bucket = buckets
            .entry(client_id.to_string())
            .or_insert_with(|| TokenBucket::new(capacity, refill_rate));

        if bucket.try_consume(1.0) {
            let remaining = bucket.tokens as u64;
            let reset_at = Instant::now() + Duration::from_secs_f64(
                (bucket.capacity - bucket.tokens) / bucket.refill_rate
            );
            RateLimitResult::allowed(remaining, reset_at, self.config.max_requests)
        } else {
            let reset_at = Instant::now() + bucket.time_until_available(1.0);
            RateLimitResult::denied(reset_at, self.config.max_requests)
        }
    }

    /// Reset rate limit for a client
    pub fn reset(&self, client_id: &str) {
        let mut buckets = self.buckets.write().unwrap();
        buckets.remove(client_id);
    }

    /// Clear all rate limit state
    pub fn clear(&self) {
        let mut buckets = self.buckets.write().unwrap();
        buckets.clear();
    }

    /// Get current state for a client
    pub fn get_state(&self, client_id: &str) -> Option<(u64, Instant)> {
        let buckets = self.buckets.read().unwrap();
        buckets.get(client_id).map(|b| {
            let reset_at = Instant::now() + Duration::from_secs_f64(
                (b.capacity - b.tokens) / b.refill_rate
            );
            (b.tokens as u64, reset_at)
        })
    }
}

/// Rate limiter using sliding window algorithm
pub struct SlidingWindowLimiter {
    config: RateLimitConfig,
    windows: RwLock<HashMap<String, SlidingWindow>>,
}

impl SlidingWindowLimiter {
    /// Create a new sliding window rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            windows: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a request is allowed for a client
    pub fn check(&self, client_id: &str) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult::allowed(
                self.config.max_requests,
                Instant::now() + self.config.window,
                self.config.max_requests,
            );
        }

        let mut windows = self.windows.write().unwrap();
        let window = windows
            .entry(client_id.to_string())
            .or_insert_with(|| SlidingWindow::new(self.config.max_requests, self.config.window));

        if window.try_add() {
            RateLimitResult::allowed(
                window.remaining(),
                window.reset_at(),
                self.config.max_requests,
            )
        } else {
            RateLimitResult::denied(window.reset_at(), self.config.max_requests)
        }
    }

    /// Reset rate limit for a client
    pub fn reset(&self, client_id: &str) {
        let mut windows = self.windows.write().unwrap();
        windows.remove(client_id);
    }

    /// Clear all rate limit state
    pub fn clear(&self) {
        let mut windows = self.windows.write().unwrap();
        windows.clear();
    }

    /// Get remaining requests for a client
    pub fn get_remaining(&self, client_id: &str) -> u64 {
        let mut windows = self.windows.write().unwrap();
        if let Some(window) = windows.get_mut(client_id) {
            window.remaining()
        } else {
            self.config.max_requests
        }
    }
}

/// Composite rate limiter that combines multiple strategies
pub struct CompositeRateLimiter {
    /// Per-client rate limiter
    per_client: SlidingWindowLimiter,
    /// Global rate limiter
    global: TokenBucketLimiter,
    /// Per-endpoint rate limiters
    per_endpoint: RwLock<HashMap<String, SlidingWindowLimiter>>,
}

impl CompositeRateLimiter {
    /// Create a new composite rate limiter
    pub fn new(per_client_config: RateLimitConfig, global_config: RateLimitConfig) -> Self {
        Self {
            per_client: SlidingWindowLimiter::new(per_client_config),
            global: TokenBucketLimiter::new(global_config),
            per_endpoint: RwLock::new(HashMap::new()),
        }
    }

    /// Add an endpoint-specific rate limiter
    pub fn add_endpoint(&self, endpoint: impl Into<String>, config: RateLimitConfig) {
        let mut endpoints = self.per_endpoint.write().unwrap();
        endpoints.insert(endpoint.into(), SlidingWindowLimiter::new(config));
    }

    /// Check rate limits for a request
    pub fn check(&self, client_id: &str, endpoint: Option<&str>) -> RateLimitResult {
        // Check global limit first
        let global_result = self.global.check("global");
        if !global_result.allowed {
            return global_result;
        }

        // Check endpoint-specific limit if applicable
        if let Some(ep) = endpoint {
            let endpoints = self.per_endpoint.read().unwrap();
            if let Some(limiter) = endpoints.get(ep) {
                let endpoint_result = limiter.check(client_id);
                if !endpoint_result.allowed {
                    return endpoint_result;
                }
            }
        }

        // Check per-client limit
        self.per_client.check(client_id)
    }

    /// Reset all rate limits for a client
    pub fn reset_client(&self, client_id: &str) {
        self.per_client.reset(client_id);
        let endpoints = self.per_endpoint.read().unwrap();
        for limiter in endpoints.values() {
            limiter.reset(client_id);
        }
    }
}

/// Rate limit key generator
pub struct RateLimitKey;

impl RateLimitKey {
    /// Generate key from IP address
    pub fn from_ip(ip: &str) -> String {
        format!("ip:{}", ip)
    }

    /// Generate key from user ID
    pub fn from_user(user_id: &str) -> String {
        format!("user:{}", user_id)
    }

    /// Generate key from API key
    pub fn from_api_key(api_key: &str) -> String {
        // Hash the API key to avoid storing sensitive data
        format!("api:{:x}", Self::simple_hash(api_key))
    }

    /// Generate composite key
    pub fn composite(user_id: &str, endpoint: &str) -> String {
        format!("{}:{}", user_id, endpoint)
    }

    /// Simple hash function for rate limit keys
    fn simple_hash(s: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(60));
        assert!(config.enabled);

        let unlimited = RateLimitConfig::unlimited();
        assert!(!unlimited.enabled);
    }

    #[test]
    fn test_token_bucket_limiter() {
        let config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(1),
            burst: 2,
            enabled: true,
        };
        let limiter = TokenBucketLimiter::new(config);

        // First 7 requests should succeed (5 + 2 burst)
        for i in 0..7 {
            let result = limiter.check("client1");
            assert!(result.allowed, "Request {} should be allowed", i);
        }

        // 8th request should be denied
        let result = limiter.check("client1");
        assert!(!result.allowed);
        assert!(result.retry_after.is_some());
    }

    #[test]
    fn test_sliding_window_limiter() {
        let config = RateLimitConfig {
            max_requests: 3,
            window: Duration::from_millis(100),
            burst: 0,
            enabled: true,
        };
        let limiter = SlidingWindowLimiter::new(config);

        // First 3 requests should succeed
        for _ in 0..3 {
            let result = limiter.check("client1");
            assert!(result.allowed);
        }

        // 4th request should be denied
        let result = limiter.check("client1");
        assert!(!result.allowed);

        // Wait for window to expire
        thread::sleep(Duration::from_millis(150));

        // Should be allowed again
        let result = limiter.check("client1");
        assert!(result.allowed);
    }

    #[test]
    fn test_rate_limiter_reset() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(60),
            burst: 0,
            enabled: true,
        };
        let limiter = SlidingWindowLimiter::new(config);

        // Use up the limit
        limiter.check("client1");
        limiter.check("client1");
        assert!(!limiter.check("client1").allowed);

        // Reset the client
        limiter.reset("client1");

        // Should be allowed again
        assert!(limiter.check("client1").allowed);
    }

    #[test]
    fn test_disabled_rate_limiter() {
        let config = RateLimitConfig::unlimited();
        let limiter = SlidingWindowLimiter::new(config);

        // All requests should be allowed
        for _ in 0..1000 {
            assert!(limiter.check("client1").allowed);
        }
    }

    #[test]
    fn test_composite_rate_limiter() {
        let client_config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
            burst: 0,
            enabled: true,
        };
        let global_config = RateLimitConfig {
            max_requests: 100,
            window: Duration::from_secs(60),
            burst: 10,
            enabled: true,
        };

        let limiter = CompositeRateLimiter::new(client_config, global_config);

        // Add endpoint-specific limiter
        limiter.add_endpoint("/compile", RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(60),
            burst: 0,
            enabled: true,
        });

        // First 2 compile requests should succeed
        assert!(limiter.check("client1", Some("/compile")).allowed);
        assert!(limiter.check("client1", Some("/compile")).allowed);

        // 3rd compile request should be denied
        assert!(!limiter.check("client1", Some("/compile")).allowed);

        // But other endpoints should still work
        assert!(limiter.check("client1", Some("/other")).allowed);
    }

    #[test]
    fn test_rate_limit_key() {
        assert_eq!(RateLimitKey::from_ip("192.168.1.1"), "ip:192.168.1.1");
        assert_eq!(RateLimitKey::from_user("user123"), "user:user123");
        assert!(RateLimitKey::from_api_key("secret").starts_with("api:"));
        assert_eq!(
            RateLimitKey::composite("user123", "/api"),
            "user123:/api"
        );
    }

    #[test]
    fn test_rate_limit_result() {
        let allowed = RateLimitResult::allowed(5, Instant::now() + Duration::from_secs(60), 10);
        assert!(allowed.allowed);
        assert_eq!(allowed.remaining, 5);
        assert!(allowed.retry_after.is_none());

        let denied = RateLimitResult::denied(Instant::now() + Duration::from_secs(30), 10);
        assert!(!denied.allowed);
        assert_eq!(denied.remaining, 0);
        assert!(denied.retry_after.is_some());
    }

    #[test]
    fn test_multiple_clients() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(60),
            burst: 0,
            enabled: true,
        };
        let limiter = SlidingWindowLimiter::new(config);

        // Client 1 uses up their limit
        limiter.check("client1");
        limiter.check("client1");
        assert!(!limiter.check("client1").allowed);

        // Client 2 should still have their limit
        assert!(limiter.check("client2").allowed);
        assert!(limiter.check("client2").allowed);
        assert!(!limiter.check("client2").allowed);
    }
}
