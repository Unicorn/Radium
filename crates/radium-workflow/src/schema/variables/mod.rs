//! Variable system for workflow definitions
//!
//! This module provides a comprehensive type system for workflow variables,
//! including:
//!
//! - **Types** (`types.rs`): All supported variable types (string, number, boolean, etc.)
//! - **Values** (`value.rs`): Runtime values with type information
//! - **Definitions** (`definition.rs`): Variable metadata, scoping, and constraints
//! - **References** (`reference.rs`): JSONPath-style variable access expressions
//!
//! ## Example Usage
//!
//! ```rust
//! use radium_workflow::schema::variables::{
//!     VariableType, VariableValue, VariableDefinition, VariableReference,
//!     VariableScope, VariableConstraints,
//! };
//!
//! // Define a variable
//! let user_id = VariableDefinition::new("userId", VariableType::String)
//!     .required()
//!     .workflow_scope()
//!     .with_description("The user's unique identifier")
//!     .with_constraints(VariableConstraints::new().length(Some(1), Some(100)));
//!
//! // Create a value
//! let value = VariableValue::String("user-123".to_string());
//!
//! // Validate the value
//! user_id.validate_value(&value).expect("Value should be valid");
//!
//! // Parse a reference
//! let reference = VariableReference::parse("$.workflow.input.userId").unwrap();
//! println!("TypeScript: {}", reference.to_typescript());
//! ```

mod definition;
mod reference;
mod types;
mod value;

pub use definition::{
    ConstraintViolation, DefinitionValidationError, VariableConstraints, VariableDefinition,
    VariableScope,
};
pub use reference::{PathSegment, ReferenceParseError, VariableReference};
pub use types::VariableType;
pub use value::VariableValue;

#[cfg(test)]
mod tests {
    use super::*;

    /// Integration test for the variable system
    #[test]
    fn test_variable_system_integration() {
        // 1. Define a variable with constraints
        let counter_def = VariableDefinition::new("counter", VariableType::Integer)
            .required()
            .workflow_scope()
            .with_description("A counter that tracks iterations")
            .with_constraints(
                VariableConstraints::new()
                    .range(Some(0.0), Some(1000.0))
                    .nullable(false),
            );

        // 2. Create valid and invalid values
        let valid_value = VariableValue::Integer(50);
        let invalid_value = VariableValue::Integer(2000);
        let wrong_type = VariableValue::String("not a number".to_string());

        // 3. Validate
        assert!(counter_def.validate_value(&valid_value).is_ok());
        assert!(counter_def.validate_value(&invalid_value).is_err());
        assert!(counter_def.validate_value(&wrong_type).is_err());

        // 4. Create a reference to access this variable
        let reference = VariableReference::parse("$.counter").unwrap();
        assert_eq!(reference.root_variable(), Some("counter"));
        assert_eq!(reference.to_typescript(), "state.counter");
    }

    #[test]
    fn test_nullable_string_variable() {
        let optional_name = VariableDefinition::new("nickname", VariableType::String)
            .with_constraints(VariableConstraints::new().nullable(true));

        // Both string and null should be valid
        assert!(optional_name.validate_value(&VariableValue::String("Bob".to_string())).is_ok());
        assert!(optional_name.validate_value(&VariableValue::Null).is_ok());

        // TypeScript type should include null
        assert_eq!(optional_name.to_typescript_type(), "string | null");
    }

    #[test]
    fn test_enum_constrained_variable() {
        let status = VariableDefinition::new("status", VariableType::String)
            .required()
            .with_constraints(
                VariableConstraints::new().enum_values(vec![
                    "pending".to_string(),
                    "active".to_string(),
                    "completed".to_string(),
                    "cancelled".to_string(),
                ]),
            );

        assert!(status.validate_value(&VariableValue::String("active".to_string())).is_ok());
        assert!(status.validate_value(&VariableValue::String("invalid".to_string())).is_err());
    }

    #[test]
    fn test_nested_reference_access() {
        let reference = VariableReference::parse("$.order.items[0].product.name").unwrap();

        assert_eq!(reference.root_variable(), Some("order"));
        assert!(reference.has_index_access());
        assert!(!reference.is_collection_access());
        assert_eq!(reference.to_typescript(), "state.order.items[0].product.name");
        assert_eq!(reference.to_typescript_safe(), "state?.order?.items?.[0]?.product?.name");
    }

    #[test]
    fn test_collection_mapping() {
        let reference = VariableReference::parse("$.users[*].email").unwrap();

        assert!(reference.is_collection_access());
        assert_eq!(reference.to_typescript(), "state.users.map(item => item.email)");
    }

    #[test]
    fn test_filtered_collection() {
        let reference = VariableReference::parse("$.orders[?(@.total > 100)].id").unwrap();

        assert!(reference.is_collection_access());
        assert_eq!(
            reference.to_typescript(),
            "state.orders.filter(item => item.total > 100).id"
        );
    }

    #[test]
    fn test_computed_variable() {
        let total = VariableDefinition::new("total", VariableType::Number)
            .computed("price * quantity");

        assert!(total.is_computed());
        assert_eq!(total.computed, Some("price * quantity".to_string()));
    }

    #[test]
    fn test_transformed_variable() {
        let upper_name = VariableDefinition::new("upperName", VariableType::String)
            .transformed_from("name", "uppercase");

        assert!(upper_name.is_transformed());
        assert_eq!(upper_name.source, Some("name".to_string()));
        assert_eq!(upper_name.transform, Some("uppercase".to_string()));
    }

    #[test]
    fn test_type_compatibility() {
        // Integer can be assigned to Number
        assert!(VariableType::Integer.is_assignable_to(&VariableType::Number));

        // But not vice versa
        assert!(!VariableType::Number.is_assignable_to(&VariableType::Integer));

        // Any accepts everything
        assert!(VariableType::String.is_assignable_to(&VariableType::Any));
        assert!(VariableType::Object.is_assignable_to(&VariableType::Any));
    }

    #[test]
    fn test_value_coercion() {
        let int_value = VariableValue::Integer(42);

        // Can coerce to number
        assert_eq!(int_value.as_number(), Some(42.0));

        // Can coerce to string
        assert_eq!(int_value.as_string(), Some("42".to_string()));

        // Can coerce to boolean
        assert_eq!(int_value.as_boolean(), Some(true));
    }
}
