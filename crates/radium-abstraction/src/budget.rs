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
