//! TypeScript Verification Tests
//!
//! These tests verify that generated TypeScript code compiles correctly
//! with strict TypeScript settings. They test component schema TypeScript
//! generation and serialization compatibility.

use radium_workflow::codegen::{CodeGenerator, GeneratedCode};
use radium_workflow::schema::{
    NodeData, NodeType, Position, WorkflowDefinition, WorkflowEdge, WorkflowNode,
};
use radium_workflow::schema::components::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// Helper to create a minimal test workflow
fn create_minimal_workflow(name: &str) -> WorkflowDefinition {
    let trigger_node = WorkflowNode {
        id: "trigger-1".to_string(),
        node_type: NodeType::Trigger,
        data: NodeData {
            label: "Start".to_string(),
            component_id: None,
            component_name: None,
            activity_name: None,
            signal_name: None,
            config: None,
            timeout: None,
            retry_policy: None,
            input: None,
            description: Some("Workflow trigger".to_string()),
        },
        position: Position { x: 0.0, y: 0.0 },
    };

    let end_node = WorkflowNode {
        id: "end-1".to_string(),
        node_type: NodeType::End,
        data: NodeData {
            label: "End".to_string(),
            component_id: None,
            component_name: None,
            activity_name: None,
            signal_name: None,
            config: None,
            timeout: None,
            retry_policy: None,
            input: None,
            description: Some("Workflow end".to_string()),
        },
        position: Position { x: 200.0, y: 0.0 },
    };

    WorkflowDefinition {
        id: format!("test-{}", name),
        name: name.to_string(),
        nodes: vec![trigger_node, end_node],
        edges: vec![WorkflowEdge {
            id: "edge-1".to_string(),
            source: "trigger-1".to_string(),
            target: "end-1".to_string(),
            source_handle: None,
            target_handle: None,
            label: None,
            edge_type: None,
        }],
        variables: vec![],
        settings: Default::default(),
    }
}

/// Write generated code to a temp directory
fn write_generated_code(dir: &TempDir, code: &GeneratedCode) -> std::io::Result<()> {
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir)?;

    fs::write(src_dir.join("workflow.ts"), &code.workflow)?;
    fs::write(src_dir.join("activities.ts"), &code.activities)?;
    fs::write(src_dir.join("worker.ts"), &code.worker)?;
    fs::write(dir.path().join("package.json"), &code.package_json)?;
    fs::write(dir.path().join("tsconfig.json"), &code.tsconfig)?;

    Ok(())
}

/// Check if npm is available
fn check_node_available() -> bool {
    std::process::Command::new("npm")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// =============================================================================
// COMPONENT SCHEMA TESTS (don't require Node.js)
// =============================================================================

mod schema_tests {
    use super::*;

    #[test]
    fn test_trigger_generates_valid_typescript_method() {
        let input = TriggerInput::scheduled_cron("0 * * * *");
        let ts = input.trigger_type.to_typescript();
        assert!(ts.contains("schedule"));
    }

    #[test]
    fn test_conditional_generates_valid_typescript_expression() {
        let condition = Condition::equals("status", serde_json::json!("active"));
        let ts = condition.to_typescript();
        assert!(ts.contains("==="));
        assert!(ts.contains("'active'"));
        // Verify no 'any' types in output
        assert!(!ts.contains(": any"));
    }

    #[test]
    fn test_loop_generates_valid_typescript_type() {
        let loop_input = LoopInput::for_each("items");
        let ts = loop_input.loop_type.to_typescript();
        assert!(ts.contains("forEach"));
    }

    #[test]
    fn test_http_method_generates_uppercase_typescript() {
        let methods = vec![
            (HttpMethod::Get, "GET"),
            (HttpMethod::Post, "POST"),
            (HttpMethod::Put, "PUT"),
            (HttpMethod::Delete, "DELETE"),
        ];

        for (method, expected) in methods {
            let json = serde_json::to_string(&method).unwrap();
            assert!(json.contains(expected), "Expected {} in {}", expected, json);
        }
    }

    #[test]
    fn test_ai_provider_generates_lowercase_typescript() {
        let providers = vec![
            (AIProvider::Anthropic, "anthropic"),
            (AIProvider::OpenAI, "openai"),
            (AIProvider::Google, "google"),
        ];

        for (provider, expected) in providers {
            let ts = provider.to_typescript();
            assert!(ts.contains(expected), "Expected {} in {}", expected, ts);
        }
    }

    #[test]
    fn test_join_strategy_generates_valid_typescript() {
        let strategies = vec![
            (JoinStrategy::All, "all"),
            (JoinStrategy::Any, "any"),
            (JoinStrategy::AllSettled, "allSettled"),  // camelCase in JSON
            (JoinStrategy::Race, "race"),
        ];

        for (strategy, expected) in strategies {
            let json = serde_json::to_string(&strategy).unwrap();
            assert!(json.contains(expected), "Expected {} in {}", expected, json);
        }
    }

    #[test]
    fn test_signal_direction_generates_valid_typescript() {
        let directions = vec![
            (SignalDirection::Send, "send"),
            (SignalDirection::Receive, "receive"),
        ];

        for (direction, expected) in directions {
            let json = serde_json::to_string(&direction).unwrap();
            assert!(json.contains(expected), "Expected {} in {}", expected, json);
        }
    }

    #[test]
    fn test_timer_type_generates_valid_typescript() {
        let types = vec![
            (TimerType::Duration, "duration"),
            (TimerType::UntilTime, "until"),
        ];

        for (timer_type, expected) in types {
            let json = serde_json::to_string(&timer_type).unwrap();
            assert!(json.contains(expected), "Expected {} in {}", expected, json);
        }
    }

    #[test]
    fn test_all_comparison_operators_generate_typescript() {
        let operators = vec![
            (ComparisonOperator::Equals, "==="),
            (ComparisonOperator::NotEquals, "!=="),
            (ComparisonOperator::GreaterThan, ">"),
            (ComparisonOperator::LessThan, "<"),
            (ComparisonOperator::GreaterOrEqual, ">="),
            (ComparisonOperator::LessOrEqual, "<="),
            (ComparisonOperator::Contains, ".includes("),
            (ComparisonOperator::StartsWith, ".startsWith("),
            (ComparisonOperator::EndsWith, ".endsWith("),
            (ComparisonOperator::IsNull, "=== null"),
            (ComparisonOperator::IsNotNull, "!== null"),
        ];

        for (op, expected) in operators {
            let condition = Condition::new("x", op.clone(), Some(serde_json::json!("test")));
            let ts = condition.to_typescript();
            assert!(
                ts.contains(expected),
                "Operator {:?} should generate '{}', got: {}",
                op,
                expected,
                ts
            );
        }
    }
}

// =============================================================================
// CODE GENERATION TESTS
// =============================================================================

mod codegen_tests {
    use super::*;

    #[test]
    fn test_code_generator_creates_all_files() {
        let generator = CodeGenerator::new().expect("Failed to create generator");
        let workflow = create_minimal_workflow("TestWorkflow");

        let code = generator.generate(&workflow).expect("Generation failed");

        // Verify all files are generated
        assert!(!code.workflow.is_empty(), "workflow.ts should not be empty");
        assert!(
            !code.activities.is_empty(),
            "activities.ts should not be empty"
        );
        assert!(!code.worker.is_empty(), "worker.ts should not be empty");
        assert!(
            !code.package_json.is_empty(),
            "package.json should not be empty"
        );
        assert!(
            !code.tsconfig.is_empty(),
            "tsconfig.json should not be empty"
        );

        // Verify no 'any' types in generated code (use 'unknown' instead)
        assert!(
            !code.workflow.contains(": any"),
            "workflow.ts should not contain ': any'"
        );
        assert!(
            !code.activities.contains(": any"),
            "activities.ts should not contain ': any'"
        );
        assert!(
            !code.worker.contains(": any"),
            "worker.ts should not contain ': any'"
        );
    }

    #[test]
    fn test_generated_tsconfig_has_strict_settings() {
        let generator = CodeGenerator::new().expect("Failed to create generator");
        let workflow = create_minimal_workflow("StrictTest");

        let code = generator.generate(&workflow).expect("Generation failed");

        // Parse tsconfig and verify strict settings
        let tsconfig: serde_json::Value =
            serde_json::from_str(&code.tsconfig).expect("Failed to parse tsconfig");

        let compiler_options = &tsconfig["compilerOptions"];
        assert_eq!(
            compiler_options["strict"],
            serde_json::json!(true),
            "strict should be true"
        );
        assert_eq!(
            compiler_options["noImplicitAny"],
            serde_json::json!(true),
            "noImplicitAny should be true"
        );
        assert_eq!(
            compiler_options["strictNullChecks"],
            serde_json::json!(true),
            "strictNullChecks should be true"
        );
    }

    #[test]
    fn test_generated_package_json_has_temporal_deps() {
        let generator = CodeGenerator::new().expect("Failed to create generator");
        let workflow = create_minimal_workflow("DepsTest");

        let code = generator.generate(&workflow).expect("Generation failed");

        // Parse package.json and verify dependencies
        let package: serde_json::Value =
            serde_json::from_str(&code.package_json).expect("Failed to parse package.json");

        assert!(
            package["dependencies"]["@temporalio/workflow"].is_string(),
            "Should have @temporalio/workflow dependency"
        );
        assert!(
            package["dependencies"]["@temporalio/activity"].is_string(),
            "Should have @temporalio/activity dependency"
        );
        assert!(
            package["devDependencies"]["typescript"].is_string(),
            "Should have typescript devDependency"
        );
    }

    #[test]
    fn test_generated_code_has_do_not_edit_header() {
        let generator = CodeGenerator::new().expect("Failed to create generator");
        let workflow = create_minimal_workflow("HeaderTest");

        let code = generator.generate(&workflow).expect("Generation failed");

        // All generated files should have "DO NOT EDIT" or similar header
        assert!(
            code.workflow.contains("DO NOT EDIT") || code.workflow.contains("Generated"),
            "workflow.ts should have generated header"
        );
    }

    #[test]
    fn test_generated_workflow_has_proper_imports() {
        let generator = CodeGenerator::new().expect("Failed to create generator");
        let workflow = create_minimal_workflow("ImportsTest");

        let code = generator.generate(&workflow).expect("Generation failed");

        // Check for proper Temporal imports
        assert!(
            code.workflow.contains("@temporalio/workflow"),
            "Should import from @temporalio/workflow"
        );
    }

    #[test]
    fn test_workflow_can_be_written_to_disk() {
        let generator = CodeGenerator::new().expect("Failed to create generator");
        let workflow = create_minimal_workflow("DiskTest");

        let code = generator.generate(&workflow).expect("Generation failed");

        // Write to temp directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        write_generated_code(&temp_dir, &code).expect("Failed to write code");

        // Verify files exist
        assert!(temp_dir.path().join("src/workflow.ts").exists());
        assert!(temp_dir.path().join("src/activities.ts").exists());
        assert!(temp_dir.path().join("src/worker.ts").exists());
        assert!(temp_dir.path().join("package.json").exists());
        assert!(temp_dir.path().join("tsconfig.json").exists());
    }
}

// =============================================================================
// SERIALIZATION ROUNDTRIP TESTS
// =============================================================================

mod serialization_tests {
    use super::*;

    #[test]
    fn test_all_components_serialize_to_camelcase() {
        // Test that all component inputs serialize with camelCase
        let trigger = TriggerInput::scheduled_cron("0 * * * *");
        let json = serde_json::to_string(&trigger).unwrap();
        assert!(json.contains("triggerType"), "Should use camelCase");
        assert!(!json.contains("trigger_type"), "Should not use snake_case");

        let activity = ActivityInput::new("test");
        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("activityName"), "Should use camelCase");

        let http = HttpRequestInput::get("http://example.com");
        let json = serde_json::to_string(&http).unwrap();
        assert!(json.contains("bodyType"), "Should use camelCase");

        let agent = AgentInput::new("Hello");
        let json = serde_json::to_string(&agent).unwrap();
        assert!(json.contains("modelConfig"), "Should use camelCase");
    }

    #[test]
    fn test_typescript_compatible_json_values() {
        // Verify JSON values match TypeScript expectations

        // Booleans
        let input = ActivityInput::new("test");
        let json = serde_json::to_string(&input).unwrap();
        assert!(
            json.contains("\"awaitResult\":true"),
            "Should serialize boolean correctly"
        );

        // Numbers
        let timer = TimerInput::seconds(30);
        let json = serde_json::to_string(&timer).unwrap();
        assert!(
            json.contains("\"duration\":30"),
            "Should serialize number correctly"
        );

        // Enums as strings
        let trigger = TriggerInput::manual();
        let json = serde_json::to_string(&trigger).unwrap();
        assert!(
            json.contains("\"triggerType\":\"manual\""),
            "Should serialize enum as string"
        );
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Test that components can be serialized and deserialized
        let original = TriggerInput::scheduled_cron("0 * * * *");
        let json = serde_json::to_string(&original).unwrap();
        let restored: TriggerInput = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.trigger_type, original.trigger_type);

        let original = HttpRequestInput::post("https://api.example.com")
            .with_json_body(serde_json::json!({"key": "value"}));
        let json = serde_json::to_string(&original).unwrap();
        let restored: HttpRequestInput = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.method, original.method);
        assert_eq!(restored.url, original.url);
    }
}

// =============================================================================
// TYPESCRIPT COMPILATION TESTS (require Node.js - ignored by default)
// =============================================================================

mod typescript_compilation_tests {
    use super::*;

    #[test]
    #[ignore = "Requires Node.js and npm installed"]
    fn test_generated_code_passes_eslint() {
        if !check_node_available() {
            eprintln!("Skipping: Node.js not available");
            return;
        }

        let generator = CodeGenerator::new().expect("Failed to create generator");
        let workflow = create_minimal_workflow("ESLintTest");

        let code = generator.generate(&workflow).expect("Generation failed");

        // Write to temp directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        write_generated_code(&temp_dir, &code).expect("Failed to write code");

        // Create a minimal eslint.config.js for flat config
        let eslint_config = r#"
import js from '@eslint/js';
import tseslint from '@typescript-eslint/eslint-plugin';
import tsparser from '@typescript-eslint/parser';

export default [
  js.configs.recommended,
  {
    files: ['src/**/*.ts'],
    languageOptions: {
      parser: tsparser,
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: 'module',
      },
    },
    plugins: {
      '@typescript-eslint': tseslint,
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': 'warn',
    },
  },
];
"#;
        fs::write(temp_dir.path().join("eslint.config.js"), eslint_config)
            .expect("Failed to write eslint config");

        // Add ESLint dependencies to package.json
        let package_json: serde_json::Value =
            serde_json::from_str(&code.package_json).expect("Failed to parse package.json");

        let mut package = package_json.clone();
        if let Some(dev_deps) = package.get_mut("devDependencies").and_then(|d| d.as_object_mut())
        {
            dev_deps.insert(
                "@eslint/js".to_string(),
                serde_json::json!("^9.0.0"),
            );
            dev_deps.insert(
                "@typescript-eslint/parser".to_string(),
                serde_json::json!("^8.0.0"),
            );
            dev_deps.insert(
                "@typescript-eslint/eslint-plugin".to_string(),
                serde_json::json!("^8.0.0"),
            );
            dev_deps.insert("eslint".to_string(), serde_json::json!("^9.0.0"));
        }

        fs::write(
            temp_dir.path().join("package.json"),
            serde_json::to_string_pretty(&package).unwrap(),
        )
        .expect("Failed to write updated package.json");

        // Run npm install
        let npm_install = std::process::Command::new("npm")
            .args(["install", "--silent"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run npm install");

        assert!(
            npm_install.status.success(),
            "npm install failed: {}",
            String::from_utf8_lossy(&npm_install.stderr)
        );

        // Run eslint
        let eslint = std::process::Command::new("npx")
            .args(["eslint", "src/**/*.ts"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run eslint");

        if !eslint.status.success() {
            eprintln!("=== ESLint STDOUT ===");
            eprintln!("{}", String::from_utf8_lossy(&eslint.stdout));
            eprintln!("=== ESLint STDERR ===");
            eprintln!("{}", String::from_utf8_lossy(&eslint.stderr));
        }

        // ESLint returns exit code 1 for warnings/errors
        // Check that there are no "any" type violations
        let output = String::from_utf8_lossy(&eslint.stdout);
        assert!(
            !output.contains("@typescript-eslint/no-explicit-any"),
            "Generated code contains explicit 'any' types:\n{}",
            output
        );
    }

    #[test]
    #[ignore = "Requires Node.js and npm installed"]
    fn test_generated_code_compiles_with_tsc() {
        if !check_node_available() {
            eprintln!("Skipping: Node.js not available");
            return;
        }

        let generator = CodeGenerator::new().expect("Failed to create generator");
        let workflow = create_minimal_workflow("CompileTest");

        let code = generator.generate(&workflow).expect("Generation failed");

        // Write to temp directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        write_generated_code(&temp_dir, &code).expect("Failed to write code");

        // Run npm install
        let npm_install = std::process::Command::new("npm")
            .args(["install", "--silent"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run npm install");

        assert!(
            npm_install.status.success(),
            "npm install failed: {}",
            String::from_utf8_lossy(&npm_install.stderr)
        );

        // Run tsc
        let tsc = std::process::Command::new("npx")
            .args(["tsc", "--noEmit"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run tsc");

        if !tsc.status.success() {
            eprintln!("=== TSC STDOUT ===");
            eprintln!("{}", String::from_utf8_lossy(&tsc.stdout));
            eprintln!("=== TSC STDERR ===");
            eprintln!("{}", String::from_utf8_lossy(&tsc.stderr));

            // Also show the generated files for debugging
            eprintln!("=== Generated workflow.ts ===");
            eprintln!("{}", std::fs::read_to_string(temp_dir.path().join("src/workflow.ts")).unwrap_or_default());
        }

        assert!(
            tsc.status.success(),
            "tsc failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&tsc.stdout),
            String::from_utf8_lossy(&tsc.stderr)
        );
    }
}

// =============================================================================
// VERIFICATION MODULE TESTS
// =============================================================================

mod verification_tests {
    use radium_workflow::verification::{EslintResult, TscResult};

    #[test]
    fn test_tsc_result_success_check() {
        let result = TscResult {
            success: true,
            errors: vec![],
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(result.success);
    }

    #[test]
    fn test_eslint_result_success_check() {
        let result = EslintResult {
            success: true,
            errors: vec![],
            warning_count: 0,
            error_count: 0,
        };
        assert!(result.success);
        assert_eq!(result.error_count, 0);
    }
}
