//! Adapter for BudgetManager to implement orchestrator traits.
//!
//! This module provides trait implementations to avoid circular dependencies
//! between radium-core and radium-orchestrator.

use super::budget::{BudgetError, BudgetManager};
use radium_abstraction::budget::{BudgetCheckResult, BudgetManagerTrait};

impl BudgetManagerTrait for BudgetManager {
    fn check_budget_available(&self, estimated_cost: f64) -> Result<(), BudgetCheckResult> {
        // Call the BudgetManager's native method which returns Result<(), BudgetError>
        // and convert the error to BudgetCheckResult
        match BudgetManager::check_budget_available(self, estimated_cost) {
            Ok(()) => Ok(()),
            Err(BudgetError::BudgetExceeded { spent, limit, requested }) => {
                Err(BudgetCheckResult::BudgetExceeded { spent, limit, requested })
            }
            Err(BudgetError::BudgetWarning { spent, limit, percentage }) => {
                Err(BudgetCheckResult::BudgetWarning { spent, limit, percentage })
            }
            // For other error types, convert to BudgetWarning with default values
            Err(err) => {
                // Log the error and return a generic warning
                tracing::warn!("Budget check returned non-standard error: {}", err);
                Ok(())
            }
        }
    }

    fn record_cost(&self, actual_cost: f64) {
        // Call the BudgetManager's native record_cost method
        BudgetManager::record_cost(self, actual_cost);
    }

    fn get_budget_status_string(&self) -> Option<String> {
        let status = BudgetManager::get_budget_status(self);
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

