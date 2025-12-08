//! Budget management for tracking and enforcing AI model costs.
//!
//! This module provides budget tracking, pre-execution cost checks, and budget warnings
//! to prevent cost overruns during agent execution.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Budget configuration for cost tracking and enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Maximum budget in USD. None means no budget limit.
    pub max_budget: Option<f64>,
    /// Warning thresholds as percentages (e.g., [80, 90] means warn at 80% and 90%).
    pub warning_at_percent: Vec<u8>,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_budget: None,
            warning_at_percent: vec![80, 90],
        }
    }
}

impl BudgetConfig {
    /// Creates a new budget configuration.
    #[must_use]
    pub fn new(max_budget: Option<f64>) -> Self {
        Self {
            max_budget,
            warning_at_percent: vec![80, 90],
        }
    }

    /// Sets warning thresholds.
    #[must_use]
    pub fn with_warning_thresholds(mut self, thresholds: Vec<u8>) -> Self {
        self.warning_at_percent = thresholds;
        self
    }
}

/// Budget status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    /// Total budget limit in USD (None if unlimited).
    pub total_budget: Option<f64>,
    /// Amount spent so far in USD.
    pub spent_amount: f64,
    /// Remaining budget in USD (None if unlimited).
    pub remaining_budget: Option<f64>,
    /// Percentage of budget used (0-100, or >100 if over budget).
    pub percentage_used: f64,
}

/// Budget errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetError {
    /// Budget limit exceeded.
    BudgetExceeded {
        spent: f64,
        limit: f64,
        requested: f64,
    },
    /// Budget warning threshold reached.
    BudgetWarning {
        spent: f64,
        limit: f64,
        percentage: f64,
    },
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::BudgetExceeded { spent, limit, requested } => {
                write!(
                    f,
                    "Budget exceeded: ${:.2} spent of ${:.2} limit (requested ${:.2})",
                    spent, limit, requested
                )
            }
            BudgetError::BudgetWarning { spent, limit, percentage } => {
                write!(
                    f,
                    "Budget warning: ${:.2} spent of ${:.2} limit ({:.1}% used)",
                    spent, limit, percentage
                )
            }
        }
    }
}

impl std::error::Error for BudgetError {}

/// Budget manager for tracking costs and enforcing limits.
#[derive(Debug, Clone)]
pub struct BudgetManager {
    config: BudgetConfig,
    spent_amount: Arc<Mutex<f64>>,
}

impl BudgetManager {
    /// Creates a new budget manager with the given configuration.
    #[must_use]
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            spent_amount: Arc::new(Mutex::new(0.0)),
        }
    }

    /// Creates a budget manager with a simple budget limit.
    #[must_use]
    pub fn with_limit(max_budget: f64) -> Self {
        Self::new(BudgetConfig::new(Some(max_budget)))
    }

    /// Checks if the estimated cost is within budget.
    ///
    /// # Errors
    /// Returns `BudgetError::BudgetExceeded` if the estimated cost would exceed the budget.
    /// Returns `BudgetError::BudgetWarning` if a warning threshold is reached.
    pub fn check_budget_available(&self, estimated_cost: f64) -> Result<(), BudgetError> {
        let spent = *self.spent_amount.lock().unwrap();

        if let Some(limit) = self.config.max_budget {
            // Check if budget would be exceeded
            if spent + estimated_cost > limit {
                return Err(BudgetError::BudgetExceeded {
                    spent,
                    limit,
                    requested: estimated_cost,
                });
            }

            // Check warning thresholds
            let percentage = (spent / limit) * 100.0;
            for threshold in &self.config.warning_at_percent {
                if percentage >= f64::from(*threshold) && percentage < f64::from(*threshold) + 1.0 {
                    return Err(BudgetError::BudgetWarning {
                        spent,
                        limit,
                        percentage,
                    });
                }
            }
        }

        Ok(())
    }

    /// Records an actual cost after execution.
    pub fn record_cost(&self, actual_cost: f64) {
        let mut spent = self.spent_amount.lock().unwrap();
        *spent += actual_cost;
    }

    /// Gets the current budget status.
    #[must_use]
    pub fn get_budget_status(&self) -> BudgetStatus {
        let spent = *self.spent_amount.lock().unwrap();

        if let Some(limit) = self.config.max_budget {
            let remaining = (limit - spent).max(0.0);
            let percentage = (spent / limit) * 100.0;

            BudgetStatus {
                total_budget: Some(limit),
                spent_amount: spent,
                remaining_budget: Some(remaining),
                percentage_used: percentage,
            }
        } else {
            BudgetStatus {
                total_budget: None,
                spent_amount: spent,
                remaining_budget: None,
                percentage_used: 0.0,
            }
        }
    }

    /// Gets the current spent amount.
    #[must_use]
    pub fn get_spent(&self) -> f64 {
        *self.spent_amount.lock().unwrap()
    }

    /// Resets the spent amount to zero.
    pub fn reset(&self) {
        let mut spent = self.spent_amount.lock().unwrap();
        *spent = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_enforcement_blocks_execution() {
        // Setup: BudgetManager with $1.00 limit, $0.95 already spent
        let manager = BudgetManager::with_limit(1.0);
        manager.record_cost(0.95);

        // Action: check_budget_available($0.10)
        let result = manager.check_budget_available(0.10);

        // Expect: Returns Err(BudgetError::BudgetExceeded)
        assert!(result.is_err());
        if let Err(BudgetError::BudgetExceeded { spent, limit, requested }) = result {
            assert!((spent - 0.95).abs() < 0.01);
            assert!((limit - 1.0).abs() < 0.01);
            assert!((requested - 0.10).abs() < 0.01);
        } else {
            panic!("Expected BudgetExceeded error");
        }
    }

    #[test]
    fn test_budget_warning_at_threshold() {
        // Setup: BudgetManager with $10.00 limit, warning at 80%, $8.50 spent
        let config = BudgetConfig::new(Some(10.0)).with_warning_thresholds(vec![80]);
        let manager = BudgetManager::new(config);
        manager.record_cost(8.5);

        // Action: check_budget_available($0.10)
        let result = manager.check_budget_available(0.10);

        // Expect: Returns Err(BudgetError::BudgetWarning) with remaining budget info
        assert!(result.is_err());
        if let Err(BudgetError::BudgetWarning { spent, limit, percentage }) = result {
            assert!((spent - 8.5).abs() < 0.01);
            assert!((limit - 10.0).abs() < 0.01);
            assert!(percentage >= 80.0 && percentage < 90.0);
        } else {
            panic!("Expected BudgetWarning error");
        }
    }

    #[test]
    fn test_budget_allows_execution_within_limit() {
        let manager = BudgetManager::with_limit(10.0);
        manager.record_cost(5.0);

        let result = manager.check_budget_available(3.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_budget_status_tracking() {
        let manager = BudgetManager::with_limit(10.0);
        manager.record_cost(3.5);

        let status = manager.get_budget_status();
        assert_eq!(status.total_budget, Some(10.0));
        assert!((status.spent_amount - 3.5).abs() < 0.01);
        assert_eq!(status.remaining_budget, Some(6.5));
        assert!((status.percentage_used - 35.0).abs() < 0.01);
    }

    #[test]
    fn test_budget_reset() {
        let manager = BudgetManager::with_limit(10.0);
        manager.record_cost(5.0);
        assert!((manager.get_spent() - 5.0).abs() < 0.01);

        manager.reset();
        assert!((manager.get_spent() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_unlimited_budget() {
        let config = BudgetConfig::new(None);
        let manager = BudgetManager::new(config);
        manager.record_cost(1000.0);

        let result = manager.check_budget_available(5000.0);
        assert!(result.is_ok());

        let status = manager.get_budget_status();
        assert_eq!(status.total_budget, None);
        assert_eq!(status.remaining_budget, None);
        assert_eq!(status.percentage_used, 0.0);
    }
}

