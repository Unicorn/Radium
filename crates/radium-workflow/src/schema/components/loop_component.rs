//! Loop component schema
//!
//! The Loop component provides iteration capabilities for workflows.
//! Supports: ForEach, While, DoWhile, Count, and Batch modes.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Types of loops
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum LoopType {
    /// Iterate over an array
    #[default]
    ForEach,
    /// Condition-based loop (check before each iteration)
    While,
    /// Condition-based loop (check after each iteration)
    DoWhile,
    /// Fixed number of iterations
    Count,
    /// Process items in batches
    Batch,
}

impl LoopType {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            LoopType::ForEach => "'forEach'",
            LoopType::While => "'while'",
            LoopType::DoWhile => "'doWhile'",
            LoopType::Count => "'count'",
            LoopType::Batch => "'batch'",
        }
    }
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BatchConfig {
    /// Number of items per batch
    #[validate(range(min = 1, message = "Batch size must be at least 1"))]
    pub batch_size: usize,

    /// Whether to process batches in parallel
    #[serde(default)]
    pub parallel: bool,

    /// Delay between batches in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_between_batches_ms: Option<u64>,
}

impl BatchConfig {
    /// Create a new batch config
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            parallel: false,
            delay_between_batches_ms: None,
        }
    }

    /// Enable parallel processing
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// Set delay between batches
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_between_batches_ms = Some(delay_ms);
        self
    }
}

/// Loop component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoopInput {
    /// Type of loop
    pub loop_type: LoopType,

    /// Variable reference to array to iterate (for ForEach/Batch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<String>,

    /// Condition expression (for While/DoWhile)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    /// Number of iterations (for Count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,

    /// Current item variable name
    #[serde(default = "default_item_var")]
    pub item_variable: String,

    /// Current index variable name
    #[serde(default = "default_index_var")]
    pub index_variable: String,

    /// Batch configuration (for Batch type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_config: Option<BatchConfig>,

    /// Maximum iterations (safety limit)
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u64,

    /// Threshold to trigger continue-as-new
    #[serde(default = "default_continue_threshold")]
    pub continue_as_new_threshold: u64,
}

fn default_item_var() -> String {
    "item".to_string()
}

fn default_index_var() -> String {
    "index".to_string()
}

fn default_max_iterations() -> u64 {
    10_000
}

fn default_continue_threshold() -> u64 {
    1_000
}

impl LoopInput {
    /// Create a ForEach loop
    pub fn for_each(items: impl Into<String>) -> Self {
        Self {
            loop_type: LoopType::ForEach,
            items: Some(items.into()),
            condition: None,
            count: None,
            item_variable: default_item_var(),
            index_variable: default_index_var(),
            batch_config: None,
            max_iterations: default_max_iterations(),
            continue_as_new_threshold: default_continue_threshold(),
        }
    }

    /// Create a While loop
    pub fn while_loop(condition: impl Into<String>) -> Self {
        Self {
            loop_type: LoopType::While,
            items: None,
            condition: Some(condition.into()),
            count: None,
            item_variable: default_item_var(),
            index_variable: default_index_var(),
            batch_config: None,
            max_iterations: default_max_iterations(),
            continue_as_new_threshold: default_continue_threshold(),
        }
    }

    /// Create a DoWhile loop
    pub fn do_while(condition: impl Into<String>) -> Self {
        Self {
            loop_type: LoopType::DoWhile,
            items: None,
            condition: Some(condition.into()),
            count: None,
            item_variable: default_item_var(),
            index_variable: default_index_var(),
            batch_config: None,
            max_iterations: default_max_iterations(),
            continue_as_new_threshold: default_continue_threshold(),
        }
    }

    /// Create a Count loop
    pub fn count(iterations: u64) -> Self {
        Self {
            loop_type: LoopType::Count,
            items: None,
            condition: None,
            count: Some(iterations),
            item_variable: default_item_var(),
            index_variable: default_index_var(),
            batch_config: None,
            max_iterations: default_max_iterations(),
            continue_as_new_threshold: default_continue_threshold(),
        }
    }

    /// Create a Batch loop
    pub fn batch(items: impl Into<String>, batch_config: BatchConfig) -> Self {
        Self {
            loop_type: LoopType::Batch,
            items: Some(items.into()),
            condition: None,
            count: None,
            item_variable: default_item_var(),
            index_variable: default_index_var(),
            batch_config: Some(batch_config),
            max_iterations: default_max_iterations(),
            continue_as_new_threshold: default_continue_threshold(),
        }
    }

    /// Set item variable name
    pub fn with_item_variable(mut self, name: impl Into<String>) -> Self {
        self.item_variable = name.into();
        self
    }

    /// Set index variable name
    pub fn with_index_variable(mut self, name: impl Into<String>) -> Self {
        self.index_variable = name.into();
        self
    }

    /// Set max iterations
    pub fn with_max_iterations(mut self, max: u64) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set continue-as-new threshold
    pub fn with_continue_threshold(mut self, threshold: u64) -> Self {
        self.continue_as_new_threshold = threshold;
        self
    }

    /// Validate loop configuration
    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        match self.loop_type {
            LoopType::ForEach | LoopType::Batch => {
                if self.items.is_none() {
                    errors.push("ForEach/Batch loop requires items array reference".to_string());
                }
            }
            LoopType::While | LoopType::DoWhile => {
                if self.condition.is_none() {
                    errors.push("While/DoWhile loop requires condition expression".to_string());
                }
            }
            LoopType::Count => {
                if self.count.is_none() {
                    errors.push("Count loop requires count value".to_string());
                }
            }
        }

        if self.loop_type == LoopType::Batch && self.batch_config.is_none() {
            errors.push("Batch loop requires batch configuration".to_string());
        }

        if self.max_iterations == 0 {
            errors.push("max_iterations must be greater than 0".to_string());
        }

        if self.continue_as_new_threshold > self.max_iterations {
            errors.push("continue_as_new_threshold cannot exceed max_iterations".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for LoopInput {
    fn default() -> Self {
        Self::for_each("items")
    }
}

/// Loop component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopOutput {
    /// Whether the loop completed normally
    pub completed: bool,

    /// Number of iterations completed
    pub iterations_completed: u64,

    /// Total items (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_items: Option<u64>,

    /// Results collected from iterations
    #[serde(default)]
    pub results: Vec<serde_json::Value>,

    /// Whether continue-as-new was triggered
    #[serde(default)]
    pub continued_as_new: bool,
}

impl LoopOutput {
    /// Create a completed loop output
    pub fn completed(iterations: u64, results: Vec<serde_json::Value>) -> Self {
        Self {
            completed: true,
            iterations_completed: iterations,
            total_items: Some(iterations),
            results,
            continued_as_new: false,
        }
    }

    /// Create an output for continue-as-new
    pub fn continue_as_new(iterations: u64, total: u64) -> Self {
        Self {
            completed: false,
            iterations_completed: iterations,
            total_items: Some(total),
            results: Vec::new(),
            continued_as_new: true,
        }
    }
}

impl Default for LoopOutput {
    fn default() -> Self {
        Self {
            completed: true,
            iterations_completed: 0,
            total_items: None,
            results: Vec::new(),
            continued_as_new: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_type_serialization() {
        assert_eq!(
            serde_json::to_string(&LoopType::ForEach).unwrap(),
            "\"forEach\""
        );
        assert_eq!(
            serde_json::to_string(&LoopType::While).unwrap(),
            "\"while\""
        );
        assert_eq!(
            serde_json::to_string(&LoopType::Batch).unwrap(),
            "\"batch\""
        );
    }

    #[test]
    fn test_batch_config() {
        let config = BatchConfig::new(10).parallel().with_delay(1000);
        assert_eq!(config.batch_size, 10);
        assert!(config.parallel);
        assert_eq!(config.delay_between_batches_ms, Some(1000));
    }

    #[test]
    fn test_loop_input_for_each() {
        let input = LoopInput::for_each("users");
        assert_eq!(input.loop_type, LoopType::ForEach);
        assert_eq!(input.items, Some("users".to_string()));
        assert!(input.validate_config().is_ok());
    }

    #[test]
    fn test_loop_input_while() {
        let input = LoopInput::while_loop("hasMore");
        assert_eq!(input.loop_type, LoopType::While);
        assert_eq!(input.condition, Some("hasMore".to_string()));
        assert!(input.validate_config().is_ok());
    }

    #[test]
    fn test_loop_input_count() {
        let input = LoopInput::count(10);
        assert_eq!(input.loop_type, LoopType::Count);
        assert_eq!(input.count, Some(10));
        assert!(input.validate_config().is_ok());
    }

    #[test]
    fn test_loop_input_batch() {
        let config = BatchConfig::new(5);
        let input = LoopInput::batch("items", config);
        assert_eq!(input.loop_type, LoopType::Batch);
        assert!(input.batch_config.is_some());
        assert!(input.validate_config().is_ok());
    }

    #[test]
    fn test_loop_input_custom_variables() {
        let input = LoopInput::for_each("users")
            .with_item_variable("user")
            .with_index_variable("i");

        assert_eq!(input.item_variable, "user");
        assert_eq!(input.index_variable, "i");
    }

    #[test]
    fn test_loop_input_validation_errors() {
        // ForEach without items
        let input = LoopInput {
            loop_type: LoopType::ForEach,
            items: None,
            ..Default::default()
        };
        assert!(input.validate_config().is_err());

        // While without condition
        let input = LoopInput {
            loop_type: LoopType::While,
            condition: None,
            ..Default::default()
        };
        assert!(input.validate_config().is_err());

        // Count without count
        let input = LoopInput {
            loop_type: LoopType::Count,
            count: None,
            ..Default::default()
        };
        assert!(input.validate_config().is_err());

        // Invalid max_iterations
        let input = LoopInput {
            max_iterations: 0,
            ..LoopInput::for_each("items")
        };
        assert!(input.validate_config().is_err());

        // Invalid threshold
        let input = LoopInput {
            max_iterations: 100,
            continue_as_new_threshold: 200,
            ..LoopInput::for_each("items")
        };
        assert!(input.validate_config().is_err());
    }

    #[test]
    fn test_loop_input_serialization() {
        let input = LoopInput::for_each("items")
            .with_max_iterations(500)
            .with_continue_threshold(100);

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("loopType"));
        assert!(json.contains("items"));
        assert!(json.contains("maxIterations"));
    }

    #[test]
    fn test_loop_output_completed() {
        let output = LoopOutput::completed(10, vec![serde_json::json!(1), serde_json::json!(2)]);
        assert!(output.completed);
        assert_eq!(output.iterations_completed, 10);
        assert!(!output.continued_as_new);
    }

    #[test]
    fn test_loop_output_continue_as_new() {
        let output = LoopOutput::continue_as_new(1000, 5000);
        assert!(!output.completed);
        assert_eq!(output.iterations_completed, 1000);
        assert!(output.continued_as_new);
    }
}
