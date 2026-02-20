//! Conditional component schema
//!
//! The Conditional component evaluates conditions to determine workflow branching.
//! It supports simple comparisons, compound conditions, and raw expressions.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Comparison operators for conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonOperator {
    /// Strict equality (===)
    Equals,
    /// Strict inequality (!==)
    NotEquals,
    /// Greater than (>)
    GreaterThan,
    /// Less than (<)
    LessThan,
    /// Greater or equal (>=)
    GreaterOrEqual,
    /// Less or equal (<=)
    LessOrEqual,
    /// String/array contains
    Contains,
    /// String starts with
    StartsWith,
    /// String ends with
    EndsWith,
    /// Regex match
    Matches,
    /// Is null check
    IsNull,
    /// Is not null check
    IsNotNull,
    /// Is empty (string or array)
    IsEmpty,
    /// Is not empty
    IsNotEmpty,
}

impl ComparisonOperator {
    /// Convert to TypeScript operator or method
    pub fn to_typescript(&self) -> &'static str {
        match self {
            ComparisonOperator::Equals => "===",
            ComparisonOperator::NotEquals => "!==",
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::GreaterOrEqual => ">=",
            ComparisonOperator::LessOrEqual => "<=",
            ComparisonOperator::Contains => ".includes",
            ComparisonOperator::StartsWith => ".startsWith",
            ComparisonOperator::EndsWith => ".endsWith",
            ComparisonOperator::Matches => "regex.test",
            ComparisonOperator::IsNull => "=== null",
            ComparisonOperator::IsNotNull => "!== null",
            ComparisonOperator::IsEmpty => "isEmpty",
            ComparisonOperator::IsNotEmpty => "isNotEmpty",
        }
    }

    /// Check if operator is unary (only needs left operand)
    pub fn is_unary(&self) -> bool {
        matches!(
            self,
            ComparisonOperator::IsNull
                | ComparisonOperator::IsNotNull
                | ComparisonOperator::IsEmpty
                | ComparisonOperator::IsNotEmpty
        )
    }
}

/// A single condition
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    /// Left side of comparison (variable reference)
    #[validate(length(min = 1, message = "Left operand is required"))]
    pub left: String,

    /// Comparison operator
    pub operator: ComparisonOperator,

    /// Right side of comparison (value or variable reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<serde_json::Value>,
}

impl Condition {
    /// Create a new condition
    pub fn new(
        left: impl Into<String>,
        operator: ComparisonOperator,
        right: Option<serde_json::Value>,
    ) -> Self {
        Self {
            left: left.into(),
            operator,
            right,
        }
    }

    /// Create an equals condition
    pub fn equals(left: impl Into<String>, right: serde_json::Value) -> Self {
        Self::new(left, ComparisonOperator::Equals, Some(right))
    }

    /// Create a not equals condition
    pub fn not_equals(left: impl Into<String>, right: serde_json::Value) -> Self {
        Self::new(left, ComparisonOperator::NotEquals, Some(right))
    }

    /// Create a greater than condition
    pub fn greater_than(left: impl Into<String>, right: serde_json::Value) -> Self {
        Self::new(left, ComparisonOperator::GreaterThan, Some(right))
    }

    /// Create a less than condition
    pub fn less_than(left: impl Into<String>, right: serde_json::Value) -> Self {
        Self::new(left, ComparisonOperator::LessThan, Some(right))
    }

    /// Create an is null condition
    pub fn is_null(left: impl Into<String>) -> Self {
        Self::new(left, ComparisonOperator::IsNull, None)
    }

    /// Create an is not null condition
    pub fn is_not_null(left: impl Into<String>) -> Self {
        Self::new(left, ComparisonOperator::IsNotNull, None)
    }

    /// Convert to TypeScript expression
    pub fn to_typescript(&self) -> String {
        let left = format!("state.variables.{}", self.left);

        match self.operator {
            ComparisonOperator::Equals => {
                format!("{} === {}", left, self.right_to_ts())
            }
            ComparisonOperator::NotEquals => {
                format!("{} !== {}", left, self.right_to_ts())
            }
            ComparisonOperator::GreaterThan => {
                format!("{} > {}", left, self.right_to_ts())
            }
            ComparisonOperator::LessThan => {
                format!("{} < {}", left, self.right_to_ts())
            }
            ComparisonOperator::GreaterOrEqual => {
                format!("{} >= {}", left, self.right_to_ts())
            }
            ComparisonOperator::LessOrEqual => {
                format!("{} <= {}", left, self.right_to_ts())
            }
            ComparisonOperator::Contains => {
                format!("{}.includes({})", left, self.right_to_ts())
            }
            ComparisonOperator::StartsWith => {
                format!("{}.startsWith({})", left, self.right_to_ts())
            }
            ComparisonOperator::EndsWith => {
                format!("{}.endsWith({})", left, self.right_to_ts())
            }
            ComparisonOperator::Matches => {
                format!("new RegExp({}).test({})", self.right_to_ts(), left)
            }
            ComparisonOperator::IsNull => {
                format!("{} === null", left)
            }
            ComparisonOperator::IsNotNull => {
                format!("{} !== null", left)
            }
            ComparisonOperator::IsEmpty => {
                format!("({} === '' || {}.length === 0)", left, left)
            }
            ComparisonOperator::IsNotEmpty => {
                format!("({} !== '' && {}.length > 0)", left, left)
            }
        }
    }

    fn right_to_ts(&self) -> String {
        match &self.right {
            Some(v) => match v {
                serde_json::Value::String(s) => format!("'{}'", s),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "null".to_string(),
                _ => v.to_string(),
            },
            None => "undefined".to_string(),
        }
    }
}

/// Logical operators for compound conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogicalOperator {
    #[default]
    And,
    Or,
}

impl LogicalOperator {
    /// Convert to TypeScript operator
    pub fn to_typescript(&self) -> &'static str {
        match self {
            LogicalOperator::And => "&&",
            LogicalOperator::Or => "||",
        }
    }
}

/// A condition group (single, compound, or expression)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionGroup {
    /// A single condition
    Single(Condition),
    /// Compound condition with logical operator
    Compound {
        operator: LogicalOperator,
        conditions: Vec<ConditionGroup>,
    },
    /// Raw expression string
    Expression(String),
}

impl ConditionGroup {
    /// Create a single condition group
    pub fn single(condition: Condition) -> Self {
        ConditionGroup::Single(condition)
    }

    /// Create an AND compound condition
    pub fn and(conditions: Vec<ConditionGroup>) -> Self {
        ConditionGroup::Compound {
            operator: LogicalOperator::And,
            conditions,
        }
    }

    /// Create an OR compound condition
    pub fn or(conditions: Vec<ConditionGroup>) -> Self {
        ConditionGroup::Compound {
            operator: LogicalOperator::Or,
            conditions,
        }
    }

    /// Create from raw expression
    pub fn expression(expr: impl Into<String>) -> Self {
        ConditionGroup::Expression(expr.into())
    }

    /// Convert to TypeScript expression
    pub fn to_typescript(&self) -> String {
        match self {
            ConditionGroup::Single(c) => c.to_typescript(),
            ConditionGroup::Compound {
                operator,
                conditions,
            } => {
                let op = operator.to_typescript();
                let parts: Vec<String> = conditions
                    .iter()
                    .map(|c| format!("({})", c.to_typescript()))
                    .collect();
                parts.join(&format!(" {} ", op))
            }
            ConditionGroup::Expression(expr) => expr.clone(),
        }
    }
}

/// Conditional component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalInput {
    /// The condition(s) to evaluate
    pub condition: ConditionGroup,

    /// Label for the 'true' branch
    #[serde(default = "default_true_label")]
    pub true_label: String,

    /// Label for the 'false' branch
    #[serde(default = "default_false_label")]
    pub false_label: String,
}

fn default_true_label() -> String {
    "Yes".to_string()
}

fn default_false_label() -> String {
    "No".to_string()
}

impl ConditionalInput {
    /// Create a new conditional input
    pub fn new(condition: ConditionGroup) -> Self {
        Self {
            condition,
            true_label: default_true_label(),
            false_label: default_false_label(),
        }
    }

    /// Set custom branch labels
    pub fn with_labels(
        mut self,
        true_label: impl Into<String>,
        false_label: impl Into<String>,
    ) -> Self {
        self.true_label = true_label.into();
        self.false_label = false_label.into();
        self
    }
}

impl Default for ConditionalInput {
    fn default() -> Self {
        Self {
            condition: ConditionGroup::expression("true"),
            true_label: default_true_label(),
            false_label: default_false_label(),
        }
    }
}

/// Conditional component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalOutput {
    /// The evaluation result
    pub result: bool,

    /// Which branch was taken (true_label or false_label)
    pub branch: String,

    /// The evaluated expression (for debugging)
    pub evaluated_expression: String,
}

impl ConditionalOutput {
    /// Create output for true branch
    pub fn true_branch(label: impl Into<String>, expression: impl Into<String>) -> Self {
        Self {
            result: true,
            branch: label.into(),
            evaluated_expression: expression.into(),
        }
    }

    /// Create output for false branch
    pub fn false_branch(label: impl Into<String>, expression: impl Into<String>) -> Self {
        Self {
            result: false,
            branch: label.into(),
            evaluated_expression: expression.into(),
        }
    }
}

impl Default for ConditionalOutput {
    fn default() -> Self {
        Self {
            result: true,
            branch: default_true_label(),
            evaluated_expression: "true".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_operator_typescript() {
        assert_eq!(ComparisonOperator::Equals.to_typescript(), "===");
        assert_eq!(ComparisonOperator::NotEquals.to_typescript(), "!==");
        assert_eq!(ComparisonOperator::GreaterThan.to_typescript(), ">");
        assert_eq!(ComparisonOperator::Contains.to_typescript(), ".includes");
    }

    #[test]
    fn test_comparison_operator_is_unary() {
        assert!(ComparisonOperator::IsNull.is_unary());
        assert!(ComparisonOperator::IsNotNull.is_unary());
        assert!(ComparisonOperator::IsEmpty.is_unary());
        assert!(!ComparisonOperator::Equals.is_unary());
        assert!(!ComparisonOperator::Contains.is_unary());
    }

    #[test]
    fn test_condition_equals() {
        let condition = Condition::equals("status", serde_json::json!("active"));
        assert_eq!(condition.left, "status");
        assert!(matches!(condition.operator, ComparisonOperator::Equals));
    }

    #[test]
    fn test_condition_to_typescript() {
        let condition = Condition::equals("status", serde_json::json!("active"));
        let ts = condition.to_typescript();
        assert_eq!(ts, "state.variables.status === 'active'");

        let condition = Condition::greater_than("count", serde_json::json!(10));
        let ts = condition.to_typescript();
        assert_eq!(ts, "state.variables.count > 10");

        let condition = Condition::is_null("optional");
        let ts = condition.to_typescript();
        assert_eq!(ts, "state.variables.optional === null");
    }

    #[test]
    fn test_condition_group_single() {
        let condition = Condition::equals("x", serde_json::json!(1));
        let group = ConditionGroup::single(condition);
        let ts = group.to_typescript();
        assert_eq!(ts, "state.variables.x === 1");
    }

    #[test]
    fn test_condition_group_compound_and() {
        let c1 = Condition::greater_than("x", serde_json::json!(0));
        let c2 = Condition::less_than("x", serde_json::json!(100));
        let group = ConditionGroup::and(vec![
            ConditionGroup::single(c1),
            ConditionGroup::single(c2),
        ]);
        let ts = group.to_typescript();
        assert!(ts.contains("&&"));
        assert!(ts.contains("state.variables.x > 0"));
        assert!(ts.contains("state.variables.x < 100"));
    }

    #[test]
    fn test_condition_group_compound_or() {
        let c1 = Condition::equals("status", serde_json::json!("active"));
        let c2 = Condition::equals("status", serde_json::json!("pending"));
        let group = ConditionGroup::or(vec![
            ConditionGroup::single(c1),
            ConditionGroup::single(c2),
        ]);
        let ts = group.to_typescript();
        assert!(ts.contains("||"));
    }

    #[test]
    fn test_condition_group_expression() {
        let group = ConditionGroup::expression("customCheck(value)");
        let ts = group.to_typescript();
        assert_eq!(ts, "customCheck(value)");
    }

    #[test]
    fn test_conditional_input() {
        let condition = Condition::equals("approved", serde_json::json!(true));
        let input = ConditionalInput::new(ConditionGroup::single(condition))
            .with_labels("Approved", "Rejected");

        assert_eq!(input.true_label, "Approved");
        assert_eq!(input.false_label, "Rejected");
    }

    #[test]
    fn test_conditional_input_serialization() {
        let condition = Condition::equals("status", serde_json::json!("ready"));
        let input = ConditionalInput::new(ConditionGroup::single(condition));

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("condition"));
        assert!(json.contains("trueLabel"));
        assert!(json.contains("falseLabel"));
    }

    #[test]
    fn test_conditional_output() {
        let output = ConditionalOutput::true_branch("Yes", "state.variables.x > 0");
        assert!(output.result);
        assert_eq!(output.branch, "Yes");

        let output = ConditionalOutput::false_branch("No", "state.variables.x > 0");
        assert!(!output.result);
        assert_eq!(output.branch, "No");
    }
}
