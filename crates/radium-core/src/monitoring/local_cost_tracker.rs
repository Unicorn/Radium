//! Local model cost tracker for duration-based cost calculation.

use crate::config::engine_costs::{EngineConfig, EngineCostsConfig};
use crate::monitoring::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Cost rate information for an engine.
#[derive(Debug, Clone)]
pub struct EngineCostRate {
    /// Cost per second of execution in USD.
    pub cost_per_second: f64,
    /// Minimum billable duration in seconds.
    pub min_billable_duration: f64,
}

impl From<&EngineConfig> for EngineCostRate {
    fn from(config: &EngineConfig) -> Self {
        Self {
            cost_per_second: config.cost_per_second,
            min_billable_duration: config.min_billable_duration,
        }
    }
}

/// Local model cost tracker with thread-safe configuration access.
pub struct LocalModelCostTracker {
    /// Configuration path.
    config_path: PathBuf,
    /// Engine cost configuration (thread-safe shared state).
    config: Arc<RwLock<EngineCostsConfig>>,
}

impl LocalModelCostTracker {
    /// Create a new local model cost tracker.
    ///
    /// # Arguments
    /// * `config_path` - Path to the engine costs configuration file
    ///
    /// # Errors
    /// Returns error if configuration cannot be loaded or validated.
    pub fn new(config_path: impl AsRef<Path>) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();
        let config = EngineCostsConfig::load(&config_path)?;

        Ok(Self {
            config_path,
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Calculate cost for an engine execution based on duration.
    ///
    /// # Arguments
    /// * `engine_id` - Engine identifier (e.g., "ollama", "lm-studio")
    /// * `duration` - Execution duration
    ///
    /// # Returns
    /// Calculated cost in USD. Returns 0.0 if engine is not configured (backward compatible).
    pub fn calculate_cost(&self, engine_id: &str, duration: Duration) -> f64 {
        let config = self.config.read().unwrap();
        
        let engine_config = match config.get_rate(engine_id) {
            Some(rate) => rate,
            None => return 0.0, // Missing config defaults to $0.00
        };

        // Apply minimum billable duration
        let billable_duration_secs = if duration.as_secs_f64() < engine_config.min_billable_duration {
            engine_config.min_billable_duration
        } else {
            duration.as_secs_f64()
        };

        // Calculate cost: duration * rate
        billable_duration_secs * engine_config.cost_per_second
    }

    /// Reload configuration from file.
    ///
    /// This allows hot reloading of cost rates without restarting the system.
    ///
    /// # Errors
    /// Returns error if configuration cannot be loaded or validated.
    /// Previous configuration is kept on error (atomic update).
    pub fn reload_config(&self) -> Result<()> {
        // Load and validate new config before applying
        let new_config = EngineCostsConfig::load(&self.config_path)?;

        // Atomically replace configuration
        {
            let mut config = self.config.write().unwrap();
            *config = new_config;
        }

        Ok(())
    }

    /// Get cost rate for an engine (for inspection/debugging).
    ///
    /// # Arguments
    /// * `engine_id` - Engine identifier
    ///
    /// # Returns
    /// Cost rate if configured, None otherwise
    pub fn get_rate(&self, engine_id: &str) -> Option<EngineCostRate> {
        let config = self.config.read().unwrap();
        config.get_rate(engine_id).map(EngineCostRate::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_cost_normal_duration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let tracker = LocalModelCostTracker::new(&config_path).unwrap();

        // 5 seconds at $0.0001/second = $0.0005
        let cost = tracker.calculate_cost("ollama", Duration::from_secs(5));
        assert!((cost - 0.0005).abs() < 0.000001);
    }

    #[test]
    fn test_calculate_cost_minimum_billable() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let tracker = LocalModelCostTracker::new(&config_path).unwrap();

        // 0.05 seconds, but min is 0.1, so should charge for 0.1 seconds
        let cost = tracker.calculate_cost("ollama", Duration::from_millis(50));
        let expected = 0.1 * 0.0001; // 0.00001
        assert!((cost - expected).abs() < 0.000001);
    }

    #[test]
    fn test_calculate_cost_zero_duration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.0
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let tracker = LocalModelCostTracker::new(&config_path).unwrap();

        let cost = tracker.calculate_cost("ollama", Duration::from_secs(0));
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_calculate_cost_missing_engine() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let tracker = LocalModelCostTracker::new(&config_path).unwrap();

        // Unknown engine should return 0.0 (backward compatible)
        let cost = tracker.calculate_cost("unknown-engine", Duration::from_secs(5));
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_reload_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        // Initial config
        let initial_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, initial_content).unwrap();

        let tracker = LocalModelCostTracker::new(&config_path).unwrap();

        // Calculate with initial rate
        let cost1 = tracker.calculate_cost("ollama", Duration::from_secs(1));
        assert!((cost1 - 0.0001).abs() < 0.000001);

        // Update config file
        let updated_content = r#"
[engines.ollama]
cost_per_second = 0.0002
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, updated_content).unwrap();

        // Reload config
        tracker.reload_config().unwrap();

        // Calculate with new rate
        let cost2 = tracker.calculate_cost("ollama", Duration::from_secs(1));
        assert!((cost2 - 0.0002).abs() < 0.000001);
    }

    #[test]
    fn test_get_rate() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let tracker = LocalModelCostTracker::new(&config_path).unwrap();

        let rate = tracker.get_rate("ollama").unwrap();
        assert_eq!(rate.cost_per_second, 0.0001);
        assert_eq!(rate.min_billable_duration, 0.1);

        assert!(tracker.get_rate("unknown").is_none());
    }

    #[test]
    fn test_thread_safety() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let tracker = Arc::new(LocalModelCostTracker::new(&config_path).unwrap());

        // Spawn multiple threads calculating costs concurrently
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let tracker_clone = Arc::clone(&tracker);
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        let cost = tracker_clone.calculate_cost("ollama", Duration::from_secs(1));
                        assert!((cost - 0.0001).abs() < 0.000001);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

