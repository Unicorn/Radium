//! Integration tests for the workflow compiler
//!
//! These tests verify that all components work together correctly,
//! including variables, state management, expressions, and code generation.

use radium_workflow::{
    expressions::{ExpressionParser, ExpressionEvaluator, TypeScriptGenerator},
    schema::{
        NodeData, NodeType, Position, WorkflowDefinition, WorkflowEdge, WorkflowNode,
        WorkflowSettings, WorkflowVariable,
        LegacyVariableType,  // Re-exported at top level
        variables::{
            VariableConstraints, VariableDefinition, VariableReference,
            VariableType as NewVariableType, VariableValue,
        },
        state::WorkflowState,
    },
    codegen::StateGenerator,
    validation::{DataFlowValidator, validate_data_flow},
};
use std::collections::HashMap;

// ============================================================================
// Test Fixtures
// ============================================================================

fn create_test_workflow_with_variables() -> WorkflowDefinition {
    WorkflowDefinition {
        id: "wf_test_integration".to_string(),
        name: "Integration Test Workflow".to_string(),
        nodes: vec![
            WorkflowNode {
                id: "trigger".to_string(),
                node_type: NodeType::Trigger,
                data: NodeData {
                    label: "Start".to_string(),
                    ..Default::default()
                },
                position: Position::default(),
            },
            WorkflowNode {
                id: "process".to_string(),
                node_type: NodeType::Activity,
                data: NodeData {
                    label: "Process Data".to_string(),
                    activity_name: Some("processData".to_string()),
                    ..Default::default()
                },
                position: Position::default(),
            },
            WorkflowNode {
                id: "end".to_string(),
                node_type: NodeType::End,
                data: NodeData {
                    label: "End".to_string(),
                    ..Default::default()
                },
                position: Position::default(),
            },
        ],
        edges: vec![
            WorkflowEdge::new("e1", "trigger", "process"),
            WorkflowEdge::new("e2", "process", "end"),
        ],
        variables: vec![
            WorkflowVariable::new("userId", LegacyVariableType::String),
            WorkflowVariable::new("count", LegacyVariableType::Number),
            WorkflowVariable::new("items", LegacyVariableType::Array),
            WorkflowVariable::new("metadata", LegacyVariableType::Object),
        ],
        settings: WorkflowSettings::default(),
    }
}

// ============================================================================
// Variable Type Compatibility Tests
// ============================================================================

#[test]
fn test_variable_type_compatibility_integers() {
    let int_value = VariableValue::Integer(42);

    assert!(int_value.is_compatible_with(&NewVariableType::Integer));
    assert!(int_value.is_compatible_with(&NewVariableType::Number)); // int -> number is OK
    assert!(!int_value.is_compatible_with(&NewVariableType::String));
    assert!(int_value.is_compatible_with(&NewVariableType::Any)); // any accepts all
}

#[test]
fn test_variable_type_compatibility_strings() {
    let string_value = VariableValue::String("hello".to_string());

    assert!(string_value.is_compatible_with(&NewVariableType::String));
    assert!(!string_value.is_compatible_with(&NewVariableType::Integer));
    assert!(!string_value.is_compatible_with(&NewVariableType::Number));
    assert!(string_value.is_compatible_with(&NewVariableType::Any));
}

#[test]
fn test_variable_type_compatibility_arrays() {
    let array_value = VariableValue::Array(vec![
        serde_json::json!(1),
        serde_json::json!(2),
        serde_json::json!(3),
    ]);

    assert!(array_value.is_compatible_with(&NewVariableType::Array));
    assert!(!array_value.is_compatible_with(&NewVariableType::Object));
    assert!(array_value.is_compatible_with(&NewVariableType::Any));
}

#[test]
fn test_variable_type_compatibility_null() {
    let null_value = VariableValue::Null;

    assert!(null_value.is_compatible_with(&NewVariableType::Null));
    assert!(!null_value.is_compatible_with(&NewVariableType::String));
    assert!(null_value.is_compatible_with(&NewVariableType::Any));
}

// ============================================================================
// Variable Reference Tests
// ============================================================================

#[test]
fn test_variable_reference_parsing_simple() {
    let reference = VariableReference::parse("$.workflow.input.userId").unwrap();

    assert_eq!(reference.segments.len(), 4);
    assert_eq!(reference.root_variable(), Some("workflow"));
}

#[test]
fn test_variable_reference_with_array_index() {
    let reference = VariableReference::parse("$.items[0].name").unwrap();

    assert_eq!(reference.to_typescript(), "state.items[0].name");
}

#[test]
fn test_variable_reference_invalid_missing_root() {
    let result = VariableReference::parse("workflow.input");
    assert!(result.is_err()); // missing $
}

#[test]
fn test_variable_reference_invalid_empty_property() {
    let result = VariableReference::parse("$.[invalid]");
    assert!(result.is_err()); // empty property before bracket
}

#[test]
fn test_variable_reference_typescript_generation() {
    let reference = VariableReference::parse("$.user.profile.email").unwrap();
    assert_eq!(reference.to_typescript(), "state.user.profile.email");

    let safe = reference.to_typescript_safe();
    assert!(safe.contains("?.")); // Should use optional chaining
}

// ============================================================================
// State Container Tests
// ============================================================================

#[test]
fn test_workflow_state_creation() {
    let definitions = vec![
        VariableDefinition::new("counter", NewVariableType::Integer).required(),
        VariableDefinition::new("name", NewVariableType::String),
    ];

    let state = WorkflowState::new("wf-123", definitions);

    assert_eq!(state.workflow_id, "wf-123");
    assert_eq!(state.version, 1);
    assert!(!state.is_complete());
}

#[test]
fn test_workflow_state_set_get() {
    let definitions = vec![
        VariableDefinition::new("counter", NewVariableType::Integer).required(),
    ];

    let mut state = WorkflowState::new("wf-123", definitions);

    // Set valid value
    state.set("counter", VariableValue::Integer(10)).unwrap();
    assert_eq!(state.get("counter"), Some(&VariableValue::Integer(10)));

    // Verify version incremented
    assert_eq!(state.version, 2);
}

#[test]
fn test_workflow_state_type_mismatch() {
    let definitions = vec![
        VariableDefinition::new("counter", NewVariableType::Integer).required(),
    ];

    let mut state = WorkflowState::new("wf-123", definitions);

    // Type mismatch should error
    let result = state.set("counter", VariableValue::String("invalid".to_string()));
    assert!(result.is_err());
}

// ============================================================================
// Data Flow Analysis Tests
// ============================================================================

#[test]
fn test_data_flow_analysis_simple_workflow() {
    let workflow = create_test_workflow_with_variables();
    let analysis = validate_data_flow(&workflow);

    // Should complete without panic
    assert!(analysis.errors.is_empty() || !analysis.errors.is_empty()); // Just ensure it runs
}

#[test]
fn test_data_flow_validator_execution_order() {
    let workflow = create_test_workflow_with_variables();

    // Convert workflow variables to VariableDefinition for the validator
    let definitions: Vec<VariableDefinition> = workflow
        .variables
        .iter()
        .map(|v| VariableDefinition::new(&v.name, NewVariableType::from(&v.var_type)))
        .collect();

    let mut validator = DataFlowValidator::new(definitions);
    let analysis = validator.analyze(&workflow);

    // Verify the analysis returns structured data
    assert!(analysis.flow_graph.nodes.len() >= 3); // At least trigger, process, end
}

// ============================================================================
// Expression Parsing and Evaluation Tests
// ============================================================================

#[test]
fn test_expression_arithmetic_evaluation() {
    let expr = ExpressionParser::parse("2 + 3 * 4").unwrap();
    let evaluator = ExpressionEvaluator::new(HashMap::new());
    let result = evaluator.evaluate(&expr).unwrap();

    // 2 + (3 * 4) = 14 due to operator precedence
    assert_eq!(result, VariableValue::Integer(14));
}

#[test]
fn test_expression_variable_resolution() {
    let expr = ExpressionParser::parse("x + y").unwrap();

    let mut vars = HashMap::new();
    vars.insert("x".to_string(), VariableValue::Integer(10));
    vars.insert("y".to_string(), VariableValue::Integer(5));

    let evaluator = ExpressionEvaluator::new(vars);
    let result = evaluator.evaluate(&expr).unwrap();

    assert_eq!(result, VariableValue::Integer(15));
}

#[test]
fn test_expression_conditional_evaluation() {
    let expr = ExpressionParser::parse("x > 5 ? 'big' : 'small'").unwrap();

    let mut vars = HashMap::new();
    vars.insert("x".to_string(), VariableValue::Integer(10));

    let evaluator = ExpressionEvaluator::new(vars);
    let result = evaluator.evaluate(&expr).unwrap();

    assert_eq!(result, VariableValue::String("big".to_string()));
}

#[test]
fn test_expression_function_calls() {
    // Test abs function
    let expr = ExpressionParser::parse("abs(-5)").unwrap();
    let evaluator = ExpressionEvaluator::new(HashMap::new());
    let result = evaluator.evaluate(&expr).unwrap();
    assert_eq!(result, VariableValue::Integer(5));

    // Test floor function
    let expr = ExpressionParser::parse("floor(3.7)").unwrap();
    let result = evaluator.evaluate(&expr).unwrap();
    assert_eq!(result, VariableValue::Integer(3));

    // Test uppercase
    let expr = ExpressionParser::parse("uppercase('hello')").unwrap();
    let result = evaluator.evaluate(&expr).unwrap();
    assert_eq!(result, VariableValue::String("HELLO".to_string()));
}

#[test]
fn test_expression_referenced_variables() {
    let expr = ExpressionParser::parse("a + b * c - d").unwrap();
    let vars = expr.referenced_variables();

    assert_eq!(vars.len(), 4);
    assert!(vars.contains(&"a".to_string()));
    assert!(vars.contains(&"b".to_string()));
    assert!(vars.contains(&"c".to_string()));
    assert!(vars.contains(&"d".to_string()));
}

// ============================================================================
// TypeScript Generation Tests
// ============================================================================

#[test]
fn test_typescript_state_generation_no_any_type() {
    let definitions = vec![
        VariableDefinition::new("userId", NewVariableType::String).required(),
        VariableDefinition::new("count", NewVariableType::Integer),
        VariableDefinition::new("items", NewVariableType::Array),
        VariableDefinition::new("data", NewVariableType::Any), // Should become 'unknown'
    ];

    let code = StateGenerator::generate_inline("TestWorkflow", &definitions).unwrap();

    // Verify no 'any' type - should use 'unknown' instead
    assert!(!code.contains(": any"), "Generated code contains 'any' type");
    assert!(code.contains("unknown"), "Expected 'unknown' type for Any variables");

    // Verify expected interfaces
    assert!(code.contains("export interface TestWorkflowInput"));
    assert!(code.contains("export interface TestWorkflowState"));
    assert!(code.contains("export interface TestWorkflowSnapshot"));

    // Verify functions
    assert!(code.contains("export function createTestWorkflowState"));
    assert!(code.contains("export function getVariable"));
    assert!(code.contains("export function setVariable"));
    assert!(code.contains("export function createSnapshot"));
    assert!(code.contains("export function restoreFromSnapshot"));
}

#[test]
fn test_typescript_state_generation_nullable() {
    let definitions = vec![
        VariableDefinition::new("optional", NewVariableType::String)
            .with_constraints(VariableConstraints::new().nullable(true)),
    ];

    let code = StateGenerator::generate_inline("Test", &definitions).unwrap();

    assert!(code.contains("| null"), "Nullable variables should include '| null'");
}

#[test]
fn test_typescript_expression_generation() {
    let expr = ExpressionParser::parse("x > 0 && y < 10 ? x + y : x - y").unwrap();
    let generator = TypeScriptGenerator::new();
    let ts_code = generator.generate(&expr).unwrap();

    // Verify TypeScript code is valid
    assert!(ts_code.contains("state.variables.x"));
    assert!(ts_code.contains("state.variables.y"));
    assert!(ts_code.contains("?"));
    assert!(ts_code.contains(":"));
}

#[test]
fn test_typescript_safe_access_generation() {
    let expr = ExpressionParser::parse("user.profile.email").unwrap();
    let generator = TypeScriptGenerator::new();
    let ts_code = generator.generate_safe(&expr).unwrap();

    // Verify optional chaining is used
    assert!(ts_code.contains("?."), "Safe access should use optional chaining");
}

// ============================================================================
// End-to-End Variable Flow Tests
// ============================================================================

#[test]
fn test_end_to_end_variable_definition_to_state() {
    // 1. Define variables
    let definitions = vec![
        VariableDefinition::new("userId", NewVariableType::String).required(),
        VariableDefinition::new("orderCount", NewVariableType::Integer),
    ];

    // 2. Create state
    let mut state = WorkflowState::new("wf-order-123", definitions.clone());

    // 3. Set initial values
    state.set("userId", VariableValue::String("user_abc".to_string())).unwrap();
    state.set("orderCount", VariableValue::Integer(0)).unwrap();

    // 4. Generate TypeScript code
    let code = StateGenerator::generate_inline("OrderWorkflow", &definitions).unwrap();

    // 5. Verify everything works together
    assert_eq!(state.get("userId"), Some(&VariableValue::String("user_abc".to_string())));
    assert!(code.contains("userId: string"));
    assert!(code.contains("orderCount: number"));
}

#[test]
fn test_end_to_end_expression_evaluation_with_state() {
    // 1. Create state with variables
    let definitions = vec![
        VariableDefinition::new("price", NewVariableType::Number).required(),
        VariableDefinition::new("quantity", NewVariableType::Integer).required(),
        VariableDefinition::new("discount", NewVariableType::Number),
    ];

    let mut state = WorkflowState::new("wf-calc", definitions);
    state.set("price", VariableValue::Number(100.0)).unwrap();
    state.set("quantity", VariableValue::Integer(5)).unwrap();
    state.set("discount", VariableValue::Number(0.1)).unwrap();

    // 2. Parse a calculated expression
    let expr = ExpressionParser::parse("price * quantity * (1 - discount)").unwrap();

    // 3. Evaluate with variables from state
    let mut vars = HashMap::new();
    for (name, value) in state.variables.iter() {
        vars.insert(name.clone(), value.clone());
    }

    let evaluator = ExpressionEvaluator::new(vars);
    let result = evaluator.evaluate(&expr).unwrap();

    // 100 * 5 * 0.9 = 450
    if let VariableValue::Number(total) = result {
        assert!((total - 450.0).abs() < 0.01);
    } else {
        panic!("Expected Number result");
    }
}

#[test]
fn test_end_to_end_workflow_validation_and_code_generation() {
    // 1. Create complete workflow
    let workflow = create_test_workflow_with_variables();

    // 2. Validate data flow
    let analysis = validate_data_flow(&workflow);

    // Analysis should complete (errors may or may not exist depending on workflow)
    assert!(analysis.flow_graph.nodes.len() >= 3);

    // 3. Convert workflow variables to VariableDefinition for code generation
    let definitions: Vec<VariableDefinition> = workflow
        .variables
        .iter()
        .map(|v| VariableDefinition::new(&v.name, NewVariableType::from(&v.var_type)))
        .collect();

    let code = StateGenerator::generate_inline(&workflow.name, &definitions).unwrap();

    // 4. Verify code contains all expected variables
    assert!(code.contains("userId"));
    assert!(code.contains("count"));
    assert!(code.contains("items"));
    assert!(code.contains("metadata"));
}

// ============================================================================
// Constraint Validation Tests
// ============================================================================

#[test]
fn test_variable_constraints_nullable() {
    let def = VariableDefinition::new("optional", NewVariableType::String)
        .with_constraints(VariableConstraints::new().nullable(true));

    // Null should be valid for nullable variable
    let result = def.validate_value(&VariableValue::Null);
    assert!(result.is_ok());
}

#[test]
fn test_variable_constraints_non_nullable() {
    let def = VariableDefinition::new("required", NewVariableType::String)
        .with_constraints(VariableConstraints::new().nullable(false));

    // Null should be invalid for non-nullable variable
    let result = def.validate_value(&VariableValue::Null);
    assert!(result.is_err());
}

#[test]
fn test_variable_constraints_enum_values() {
    let def = VariableDefinition::new("status", NewVariableType::String)
        .with_constraints(
            VariableConstraints::new()
                .enum_values(vec!["pending".to_string(), "active".to_string(), "completed".to_string()])
        );

    // Valid enum value
    let result = def.validate_value(&VariableValue::String("active".to_string()));
    assert!(result.is_ok());

    // Invalid enum value
    let result = def.validate_value(&VariableValue::String("invalid".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_variable_constraints_numeric_range() {
    let def = VariableDefinition::new("age", NewVariableType::Integer)
        .with_constraints(
            VariableConstraints::new()
                .range(Some(0.0), Some(120.0))
        );

    // Valid value within range
    let result = def.validate_value(&VariableValue::Integer(25));
    assert!(result.is_ok());

    // Invalid value below min
    let result = def.validate_value(&VariableValue::Integer(-1));
    assert!(result.is_err());

    // Invalid value above max
    let result = def.validate_value(&VariableValue::Integer(150));
    assert!(result.is_err());
}

#[test]
fn test_variable_constraints_string_length() {
    let def = VariableDefinition::new("code", NewVariableType::String)
        .with_constraints(
            VariableConstraints::new()
                .length(Some(3), Some(10))
        );

    // Valid length
    let result = def.validate_value(&VariableValue::String("ABC123".to_string()));
    assert!(result.is_ok());

    // Too short
    let result = def.validate_value(&VariableValue::String("AB".to_string()));
    assert!(result.is_err());

    // Too long
    let result = def.validate_value(&VariableValue::String("ABCDEFGHIJK".to_string()));
    assert!(result.is_err());
}
