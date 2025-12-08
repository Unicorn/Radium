//! Adapter for BudgetManager to implement orchestrator traits.
//!
//! This module provides trait implementations to avoid circular dependencies
//! between radium-core and radium-orchestrator.

use super::budget::{BudgetError, BudgetManager, BudgetStatus};
use std::fmt;

/// Trait for budget management (defined in radium-orchestrator to avoid circular dependency).
/// This is a copy of the trait definition - in a real implementation, this would be in a shared crate.
pub trait BudgetManagerTrait: Send + Sync {
    /// Check if estimated cost is within budget.
    fn check_budget_available(&self, estimated_cost: f64) -> Result<(), BudgetCheckResult>;
    
    /// Record an actual cost after execution.
    fn record_cost(&self, actual_cost: f64);
    
    /// Get budget status as a formatted string.
    fn get_budget_status_string(&self) -> Option<String>;
}

/// Result of budget check (copy from orchestrator to avoid dependency).
#[derive(Debug, Clone)]
pub enum BudgetCheckResult {
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

impl BudgetManagerTrait for BudgetManager {
    fn check_budget_available(&self, estimated_cost: f64) -> Result<(), BudgetCheckResult> {
        match self.check_budget_available(estimated_cost) {
            Ok(()) => Ok(()),
            Err(BudgetError::BudgetExceeded { spent, limit, requested }) => {
                Err(BudgetCheckResult::BudgetExceeded { spent, limit, requested })
            }
            Err(BudgetError::BudgetWarning { spent, limit, percentage }) => {
                Err(BudgetCheckResult::BudgetWarning { spent, limit, percentage })
            }
        }
    }

    fn record_cost(&self, actual_cost: f64) {
        self.record_cost(actual_cost);
    }

    fn get_budget_status_string(&self) -> Option<String> {
        let status = self.get_budget_status();
        if let Some(total_budget) = status.total_budget {
            if let Some(remaining) = status.remaining_budget {
                Some(format!(
                    "${:.2} spent of ${:.2} limit ({:.1}% used), ${:.2} remaining",
                    status.spent_amount, total_budget, status.percentage_used, remaining
                ))
            } else {
                Some(format!(
                    "${:.2} spent of ${:.2} limit ({:.1}% used)",
                    status.spent_amount, total_budget, status.percentage_used
                ))
            }
        } else {
            Some(format!("${:.2} spent (no limit)", status.spent_amount))
        }
    }
}

