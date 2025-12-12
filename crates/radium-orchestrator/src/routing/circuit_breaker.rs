//! Circuit breaker pattern for model failure detection.

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tracing::{debug, warn};

/// Circuit breaker state for a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed - normal operation.
    Closed,
    /// Circuit is open - skipping model until cooldown expires.
    Open(SystemTime),
    /// Circuit is half-open - testing recovery with one request.
    HalfOpen,
}

impl CircuitState {
    /// Checks if the circuit should skip this model.
    pub fn should_skip(&self, cooldown_duration: Duration) -> bool {
        match self {
            CircuitState::Closed => false,
            CircuitState::Open(opened_at) => {
                // Check if cooldown has expired
                if let Ok(elapsed) = opened_at.elapsed() {
                    elapsed < cooldown_duration
                } else {
                    // SystemTime went backwards, treat as expired
                    false
                }
            }
            CircuitState::HalfOpen => false, // Allow one test request
        }
    }
    
    /// Transitions to half-open if cooldown expired.
    pub fn transition_if_cooldown_expired(&self, cooldown_duration: Duration) -> Self {
        match self {
            CircuitState::Open(opened_at) => {
                if let Ok(elapsed) = opened_at.elapsed() {
                    if elapsed >= cooldown_duration {
                        CircuitState::HalfOpen
                    } else {
                        *self
                    }
                } else {
                    // SystemTime went backwards, transition to half-open
                    CircuitState::HalfOpen
                }
            }
            _ => *self,
        }
    }
}

/// Health tracking for a single model.
#[derive(Debug, Clone)]
struct ModelHealth {
    /// Timestamps of successful requests (sliding window).
    successes: VecDeque<SystemTime>,
    /// Timestamps of failed requests (sliding window).
    failures: VecDeque<SystemTime>,
    /// Window duration (default: 5 minutes).
    window_duration: Duration,
}

impl ModelHealth {
    /// Creates new model health tracker.
    fn new(window_duration: Duration) -> Self {
        Self {
            successes: VecDeque::new(),
            failures: VecDeque::new(),
            window_duration,
        }
    }
    
    /// Records a successful request.
    fn record_success(&mut self) {
        let now = SystemTime::now();
        self.successes.push_back(now);
        Self::cleanup_old_entries_static(&mut self.successes, now, self.window_duration);
    }

    /// Records a failed request.
    fn record_failure(&mut self) {
        let now = SystemTime::now();
        self.failures.push_back(now);
        Self::cleanup_old_entries_static(&mut self.failures, now, self.window_duration);
    }
    
    /// Calculates failure rate in the current window.
    fn calculate_failure_rate(&self) -> f64 {
        let total = self.successes.len() + self.failures.len();
        if total == 0 {
            return 0.0;
        }
        self.failures.len() as f64 / total as f64
    }
    
    /// Removes entries outside the time window (static version to avoid borrow conflicts).
    fn cleanup_old_entries_static(entries: &mut VecDeque<SystemTime>, now: SystemTime, window_duration: Duration) {
        while let Some(&oldest) = entries.front() {
            if let Ok(elapsed) = now.duration_since(oldest) {
                if elapsed > window_duration {
                    entries.pop_front();
                } else {
                    break;
                }
            } else {
                // SystemTime went backwards, remove this entry
                entries.pop_front();
            }
        }
    }

    /// Cleans up old entries for both success and failure queues.
    fn cleanup(&mut self) {
        let now = SystemTime::now();
        Self::cleanup_old_entries_static(&mut self.successes, now, self.window_duration);
        Self::cleanup_old_entries_static(&mut self.failures, now, self.window_duration);
    }
}

/// Circuit breaker for tracking model health and skipping failing models.
pub struct CircuitBreaker {
    /// Per-model circuit states (thread-safe).
    states: Arc<RwLock<std::collections::HashMap<String, CircuitState>>>,
    /// Per-model health tracking (thread-safe).
    health: Arc<RwLock<std::collections::HashMap<String, ModelHealth>>>,
    /// Failure rate threshold (default: 0.5 = 50%).
    failure_threshold: f64,
    /// Time window for failure rate calculation (default: 5 minutes).
    window_duration: Duration,
    /// Cooldown duration before transitioning from Open to HalfOpen (default: 60 seconds).
    cooldown_duration: Duration,
    /// Minimum number of total samples (success+failure) before opening the circuit.
    ///
    /// This prevents the circuit from opening on the very first failures and makes
    /// failure-rate decisions more stable.
    min_samples: usize,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker with default settings.
    ///
    /// Defaults:
    /// - Failure threshold: 50%
    /// - Window duration: 5 minutes
    /// - Cooldown duration: 60 seconds
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(std::collections::HashMap::new())),
            health: Arc::new(RwLock::new(std::collections::HashMap::new())),
            failure_threshold: 0.5,
            window_duration: Duration::from_secs(300), // 5 minutes
            cooldown_duration: Duration::from_secs(60), // 60 seconds
            min_samples: 8,
        }
    }
    
    /// Creates a new circuit breaker with custom settings.
    ///
    /// # Arguments
    /// * `failure_threshold` - Failure rate threshold (0.0-1.0)
    /// * `window_duration` - Time window for failure rate calculation
    /// * `cooldown_duration` - Cooldown before transitioning from Open to HalfOpen
    #[must_use]
    pub fn with_settings(
        failure_threshold: f64,
        window_duration: Duration,
        cooldown_duration: Duration,
    ) -> Self {
        Self {
            states: Arc::new(RwLock::new(std::collections::HashMap::new())),
            health: Arc::new(RwLock::new(std::collections::HashMap::new())),
            failure_threshold,
            window_duration,
            cooldown_duration,
            min_samples: 8,
        }
    }

    fn maybe_open_circuit(&self, model_id: &str) {
        // Only consider opening when we have enough data.
        let (failure_rate, total_samples) = {
            let health_map = self.health.read().unwrap();
            match health_map.get(model_id) {
                Some(h) => (h.calculate_failure_rate(), h.successes.len() + h.failures.len()),
                None => (0.0, 0),
            }
        };

        if total_samples < self.min_samples {
            return;
        }

        if failure_rate <= self.failure_threshold {
            return;
        }

        let mut states = self.states.write().unwrap();
        let state = states.entry(model_id.to_string()).or_insert(CircuitState::Closed);
        if matches!(*state, CircuitState::Closed) {
            *state = CircuitState::Open(SystemTime::now());
            warn!(
                model_id = model_id,
                failure_rate = failure_rate,
                threshold = self.failure_threshold,
                total_samples = total_samples,
                "Circuit breaker: Closed -> Open (failure rate exceeded threshold)"
            );
        }
    }
    
    /// Records a successful request for a model.
    ///
    /// # Arguments
    /// * `model_id` - Model identifier
    pub fn record_success(&self, model_id: &str) {
        // Update health tracking
        {
            let mut health_map = self.health.write().unwrap();
            let health = health_map
                .entry(model_id.to_string())
                .or_insert_with(|| ModelHealth::new(self.window_duration));
            health.record_success();
            health.cleanup();
        }
        
        // Update circuit state
        let mut recovered_from_half_open = false;
        {
            let mut states = self.states.write().unwrap();
            let state = states.entry(model_id.to_string()).or_insert(CircuitState::Closed);

            match *state {
                CircuitState::HalfOpen => {
                    // Success in half-open state - transition to closed
                    *state = CircuitState::Closed;
                    debug!(model_id = model_id, "Circuit breaker: HalfOpen -> Closed (recovery successful)");
                    recovered_from_half_open = true;
                }
                CircuitState::Open(_) => {
                    // Still in cooldown, don't change state
                }
                CircuitState::Closed => {
                    // Keep closed; we may open once enough samples accumulate and failure rate is high.
                }
            }
        }

        if recovered_from_half_open {
            // After a successful half-open probe, reset health so we don't immediately reopen
            // due to historical failures from the previously-open period.
            let mut health_map = self.health.write().unwrap();
            health_map.insert(model_id.to_string(), ModelHealth::new(self.window_duration));
            return;
        }

        // Evaluate whether to open circuit based on the updated failure rate and sample size.
        self.maybe_open_circuit(model_id);
    }
    
    /// Records a failed request for a model.
    ///
    /// # Arguments
    /// * `model_id` - Model identifier
    pub fn record_failure(&self, model_id: &str) {
        // Update health tracking
        {
            let mut health_map = self.health.write().unwrap();
            let health = health_map
                .entry(model_id.to_string())
                .or_insert_with(|| ModelHealth::new(self.window_duration));
            health.record_failure();
            health.cleanup();
        }
        
        // Check if we should open the circuit
        let failure_rate = {
            let health_map = self.health.read().unwrap();
            health_map
                .get(model_id)
                .map(|h| h.calculate_failure_rate())
                .unwrap_or(0.0)
        };
        
        // Update circuit state
        {
            let mut states = self.states.write().unwrap();
            let state = states.entry(model_id.to_string()).or_insert(CircuitState::Closed);

            match *state {
                CircuitState::HalfOpen => {
                    // Failure in half-open state - transition back to open
                    *state = CircuitState::Open(SystemTime::now());
                    warn!(model_id = model_id, "Circuit breaker: HalfOpen -> Open (recovery failed)");
                }
                CircuitState::Open(_) => {
                    // Already open, no change needed
                }
                CircuitState::Closed => {
                    // Opening is handled by maybe_open_circuit to enforce min_samples.
                    let _ = failure_rate;
                }
            }
        }

        self.maybe_open_circuit(model_id);
    }
    
    /// Checks if a model should be skipped due to circuit breaker.
    ///
    /// # Arguments
    /// * `model_id` - Model identifier
    ///
    /// # Returns
    /// `true` if the model should be skipped, `false` otherwise
    pub fn should_skip(&self, model_id: &str) -> bool {
        let mut states = self.states.write().unwrap();
        let state = states.entry(model_id.to_string()).or_insert(CircuitState::Closed);
        
        // Check if cooldown expired and transition to half-open
        *state = state.transition_if_cooldown_expired(self.cooldown_duration);
        
        state.should_skip(self.cooldown_duration)
    }
    
    /// Calculates the current failure rate for a model.
    ///
    /// # Arguments
    /// * `model_id` - Model identifier
    ///
    /// # Returns
    /// Failure rate (0.0-1.0) in the current time window
    pub fn calculate_failure_rate(&self, model_id: &str) -> f64 {
        let health_map = self.health.read().unwrap();
        health_map
            .get(model_id)
            .map(|h| h.calculate_failure_rate())
            .unwrap_or(0.0)
    }
    
    /// Gets the current circuit state for a model.
    ///
    /// # Arguments
    /// * `model_id` - Model identifier
    ///
    /// # Returns
    /// Current circuit state
    pub fn get_state(&self, model_id: &str) -> CircuitState {
        let states = self.states.read().unwrap();
        states
            .get(model_id)
            .copied()
            .unwrap_or(CircuitState::Closed)
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_circuit_breaker_creation() {
        let breaker = CircuitBreaker::new();
        assert_eq!(breaker.failure_threshold, 0.5);
        assert_eq!(breaker.window_duration, Duration::from_secs(300));
    }

    #[test]
    fn test_circuit_opens_after_high_failure_rate() {
        let breaker = CircuitBreaker::with_settings(
            0.5, // 50% threshold
            Duration::from_secs(300),
            Duration::from_secs(60),
        );
        
        let model_id = "test-model";
        
        // Record 6 failures and 2 successes (75% failure rate)
        for _ in 0..6 {
            breaker.record_failure(model_id);
        }
        for _ in 0..2 {
            breaker.record_success(model_id);
        }
        
        // Circuit should be open
        assert!(breaker.should_skip(model_id));
        assert!(matches!(breaker.get_state(model_id), CircuitState::Open(_)));
    }

    #[test]
    fn test_circuit_remains_closed_with_acceptable_failure_rate() {
        let breaker = CircuitBreaker::with_settings(
            0.5, // 50% threshold
            Duration::from_secs(300),
            Duration::from_secs(60),
        );
        
        let model_id = "test-model";
        
        // Record 3 failures and 7 successes (30% failure rate)
        for _ in 0..3 {
            breaker.record_failure(model_id);
        }
        for _ in 0..7 {
            breaker.record_success(model_id);
        }
        
        // Circuit should remain closed
        assert!(!breaker.should_skip(model_id));
        assert_eq!(breaker.get_state(model_id), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_transitions_to_half_open_after_cooldown() {
        let breaker = CircuitBreaker::with_settings(
            0.5,
            Duration::from_secs(300),
            Duration::from_millis(100), // Short cooldown for testing
        );
        
        let model_id = "test-model";
        
        // Open the circuit
        for _ in 0..8 {
            breaker.record_failure(model_id);
        }
        assert!(breaker.should_skip(model_id));
        
        // Wait for cooldown
        thread::sleep(Duration::from_millis(150));
        
        // Should transition to half-open
        assert!(!breaker.should_skip(model_id)); // Half-open allows one request
        assert_eq!(breaker.get_state(model_id), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_closes_after_success_in_half_open() {
        let breaker = CircuitBreaker::with_settings(
            0.5,
            Duration::from_secs(300),
            Duration::from_millis(100),
        );
        
        let model_id = "test-model";
        
        // Open the circuit
        for _ in 0..8 {
            breaker.record_failure(model_id);
        }
        
        // Wait for cooldown and transition to half-open
        thread::sleep(Duration::from_millis(150));
        breaker.should_skip(model_id); // Trigger transition
        
        // Record success in half-open state
        breaker.record_success(model_id);
        
        // Should transition to closed
        assert_eq!(breaker.get_state(model_id), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_reopens_after_failure_in_half_open() {
        let breaker = CircuitBreaker::with_settings(
            0.5,
            Duration::from_secs(300),
            Duration::from_millis(100),
        );
        
        let model_id = "test-model";
        
        // Open the circuit
        for _ in 0..8 {
            breaker.record_failure(model_id);
        }
        
        // Wait for cooldown and transition to half-open
        thread::sleep(Duration::from_millis(150));
        breaker.should_skip(model_id); // Trigger transition
        
        // Record failure in half-open state
        breaker.record_failure(model_id);
        
        // Should transition back to open
        assert!(matches!(breaker.get_state(model_id), CircuitState::Open(_)));
    }

    #[test]
    fn test_failure_rate_calculation() {
        let breaker = CircuitBreaker::new();
        let model_id = "test-model";
        
        // Record 2 failures and 3 successes
        for _ in 0..2 {
            breaker.record_failure(model_id);
        }
        for _ in 0..3 {
            breaker.record_success(model_id);
        }
        
        // Failure rate should be 2/5 = 0.4
        let rate = breaker.calculate_failure_rate(model_id);
        assert!((rate - 0.4).abs() < 0.01);
    }
}

