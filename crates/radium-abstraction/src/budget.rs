//! Budget management traits and types.
//!
//! This module defines the budget management interface to avoid circular dependencies
//! between radium-core and radium-orchestrator.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Trait for budget management.
///
/// This trait provides the interface for checking budget availability, recording costs,
/// and retrieving budget status. It's implemented by budget managers to enable cost
/// tracking and enforcement during agent execution.
pub trait BudgetManagerTrait: Send + Sync {
    /// Check if estimated cost is within budget.
    ///
    /// # Arguments
    /// * `estimated_cost` - The estimated cost in USD for the operation
    ///
    /// # Returns
    /// * `Ok(())` if the operation is within budget
    /// * `Err(BudgetCheckResult)` if budget would be exceeded or warning threshold reached
    fn check_budget_available(&self, estimated_cost: f64) -> Result<(), BudgetCheckResult>;

    /// Record an actual cost after execution.
    ///
    /// # Arguments
    /// * `actual_cost` - The actual cost in USD that was incurred
    fn record_cost(&self, actual_cost: f64);

    /// Get budget status as a formatted string.
    ///
    /// # Returns
    /// * `Some(String)` with formatted budget status
    /// * `None` if budget tracking is disabled
    fn get_budget_status_string(&self) -> Option<String>;
}

/// Result of budget check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BudgetCheckResult {
    /// Budget limit exceeded.
    BudgetExceeded {
        /// Amount already spent in USD.
        spent: f64,
        /// Budget limit in USD.
        limit: f64,
        /// Requested amount in USD.
        requested: f64,
    },
    /// Budget warning threshold reached.
    BudgetWarning {
        /// Amount already spent in USD.
        spent: f64,
        /// Budget limit in USD.
        limit: f64,
        /// Percentage of budget used.
        percentage: f64,
    },
}

impl fmt::Display for BudgetCheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BudgetCheckResult::BudgetExceeded { spent, limit, requested } => {
                write!(
                    f,
                    "Budget exceeded: ${:.2} spent of ${:.2} limit (requested ${:.2})",
                    spent, limit, requested
                )
            }
            BudgetCheckResult::BudgetWarning { spent, limit, percentage } => {
                write!(
                    f,
                    "Budget warning: ${:.2} spent of ${:.2} limit ({:.1}% used)",
                    spent, limit, percentage
                )
            }
        }
    }
}

impl std::error::Error for BudgetCheckResult {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_exceeded_display() {
        let result = BudgetCheckResult::BudgetExceeded {
            spent: 10.50,
            limit: 15.00,
            requested: 5.00,
        };
        let display = format!("{}", result);
        assert!(display.contains("Budget exceeded"));
        assert!(display.contains("$10.50"));
        assert!(display.contains("$15.00"));
        assert!(display.contains("$5.00"));
    }

    #[test]
    fn test_budget_warning_display() {
        let result = BudgetCheckResult::BudgetWarning {
            spent: 7.50,
            limit: 10.00,
            percentage: 75.0,
        };
        let display = format!("{}", result);
        assert!(display.contains("Budget warning"));
        assert!(display.contains("$7.50"));
        assert!(display.contains("$10.00"));
        assert!(display.contains("75.0%"));
    }

    #[test]
    fn test_budget_exceeded_serialization() {
        let result = BudgetCheckResult::BudgetExceeded {
            spent: 10.50,
            limit: 15.00,
            requested: 5.00,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: BudgetCheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_budget_warning_serialization() {
        let result = BudgetCheckResult::BudgetWarning {
            spent: 7.50,
            limit: 10.00,
            percentage: 75.0,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: BudgetCheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_budget_exceeded_is_error() {
        let result = BudgetCheckResult::BudgetExceeded {
            spent: 10.50,
            limit: 15.00,
            requested: 5.00,
        };
        // Verify it implements Error trait
        let _error: &dyn std::error::Error = &result;
    }

    #[test]
    fn test_budget_check_result_clone() {
        let result = BudgetCheckResult::BudgetExceeded {
            spent: 10.50,
            limit: 15.00,
            requested: 5.00,
        };
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    // Mock implementation for testing trait
    struct MockBudgetManager {
        spent: f64,
        limit: f64,
        warning_threshold: f64,
    }

    impl BudgetManagerTrait for MockBudgetManager {
        fn check_budget_available(&self, estimated_cost: f64) -> Result<(), BudgetCheckResult> {
            let total_cost = self.spent + estimated_cost;

            if total_cost > self.limit {
                return Err(BudgetCheckResult::BudgetExceeded {
                    spent: self.spent,
                    limit: self.limit,
                    requested: estimated_cost,
                });
            }

            let percentage = (total_cost / self.limit) * 100.0;
            if percentage >= self.warning_threshold {
                return Err(BudgetCheckResult::BudgetWarning {
                    spent: self.spent,
                    limit: self.limit,
                    percentage,
                });
            }

            Ok(())
        }

        fn record_cost(&self, _actual_cost: f64) {
            // Mock implementation - would update spent in real impl
        }

        fn get_budget_status_string(&self) -> Option<String> {
            Some(format!("${:.2} / ${:.2}", self.spent, self.limit))
        }
    }

    #[test]
    fn test_budget_manager_within_budget() {
        let manager = MockBudgetManager {
            spent: 5.0,
            limit: 20.0,
            warning_threshold: 80.0,
        };

        // Should succeed - well within budget
        assert!(manager.check_budget_available(5.0).is_ok());
    }

    #[test]
    fn test_budget_manager_exceeds_budget() {
        let manager = MockBudgetManager {
            spent: 15.0,
            limit: 20.0,
            warning_threshold: 80.0,
        };

        // Should fail - would exceed budget
        let result = manager.check_budget_available(10.0);
        assert!(result.is_err());

        match result.unwrap_err() {
            BudgetCheckResult::BudgetExceeded { spent, limit, requested } => {
                assert_eq!(spent, 15.0);
                assert_eq!(limit, 20.0);
                assert_eq!(requested, 10.0);
            }
            _ => panic!("Expected BudgetExceeded"),
        }
    }

    #[test]
    fn test_budget_manager_warning_threshold() {
        let manager = MockBudgetManager {
            spent: 14.0,
            limit: 20.0,
            warning_threshold: 80.0,
        };

        // Should trigger warning - 17/20 = 85%
        let result = manager.check_budget_available(3.0);
        assert!(result.is_err());

        match result.unwrap_err() {
            BudgetCheckResult::BudgetWarning { spent, limit, percentage } => {
                assert_eq!(spent, 14.0);
                assert_eq!(limit, 20.0);
                assert_eq!(percentage, 85.0);
            }
            _ => panic!("Expected BudgetWarning"),
        }
    }

    #[test]
    fn test_budget_manager_get_status() {
        let manager = MockBudgetManager {
            spent: 10.5,
            limit: 20.0,
            warning_threshold: 80.0,
        };

        let status = manager.get_budget_status_string();
        assert!(status.is_some());
        let status_str = status.unwrap();
        assert!(status_str.contains("$10.50"));
        assert!(status_str.contains("$20.00"));
    }

    #[test]
    fn test_budget_manager_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockBudgetManager>();
    }
}
