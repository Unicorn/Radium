//! Validates that all example YAML workflow files parse, transform, validate, and compile.
//!
//! These tests ensure our example files stay correct as the compiler evolves.

use std::path::PathBuf;

use radium_workflow::{transform, YamlWorkflow};
use radium_workflow::codegen::generate;
use radium_workflow::validation::validate;

/// Returns the path to the examples directory in radium-cli.
fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("radium-cli")
        .join("examples")
}

/// Helper: load, parse, transform, validate, and compile a single YAML example.
fn validate_example(filename: &str) {
    let path = examples_dir().join(filename);
    let yaml_str = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

    // Step 1: Parse as YamlWorkflow
    let yaml: YamlWorkflow = serde_yaml::from_str(&yaml_str)
        .unwrap_or_else(|e| panic!("Failed to parse {} as YamlWorkflow: {}", filename, e));

    // Step 2: Transform into WorkflowDefinition
    let definition = transform(&yaml)
        .unwrap_or_else(|e| panic!("Failed to transform {}: {}", filename, e));

    // Step 3: Validate the workflow graph
    let validation_result = validate(&definition);
    assert!(
        validation_result.is_valid(),
        "Validation failed for {}: {:?}",
        filename,
        validation_result.errors
    );

    // Step 4: Compile (generate TypeScript)
    let gen_result = generate(&definition);
    assert!(
        gen_result.is_ok(),
        "Code generation failed for {}: {:?}",
        filename,
        gen_result.err()
    );

    let code = gen_result.unwrap();
    assert!(
        !code.workflow.is_empty(),
        "Generated workflow code is empty for {}",
        filename
    );
    assert!(
        !code.activities.is_empty(),
        "Generated activities code is empty for {}",
        filename
    );
}

#[test]
fn test_minimal_example_compiles() {
    validate_example("minimal.yaml");
}

#[test]
fn test_order_processing_example_compiles() {
    validate_example("order-processing.yaml");
}

#[test]
fn test_data_pipeline_example_compiles() {
    validate_example("data-pipeline.yaml");
}

#[test]
fn test_all_example_files_are_present() {
    let dir = examples_dir();
    assert!(dir.exists(), "Examples directory does not exist: {}", dir.display());

    let expected_files = ["minimal.yaml", "order-processing.yaml", "data-pipeline.yaml"];
    for filename in &expected_files {
        let path = dir.join(filename);
        assert!(path.exists(), "Missing example file: {}", path.display());
    }
}
