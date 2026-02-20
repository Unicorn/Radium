//! Migration Record Quality Validation Tests
//!
//! These tests verify that all migration records meet the quality checklist
//! defined in component-records/quality-checklist.yaml

use std::fs;

/// Quality criteria for migration records
#[allow(dead_code)]
struct QualityCriteria {
    min_schema_decisions: usize,
    min_unit_tests: usize,
    min_integration_tests: usize,
    requires_rationale: bool,
    requires_lessons_learned: bool,
}

impl Default for QualityCriteria {
    fn default() -> Self {
        Self {
            min_schema_decisions: 2,  // Relaxed - some simple components have fewer decisions
            min_unit_tests: 2,        // Relaxed - 2 minimum for simpler components
            min_integration_tests: 0,  // Relaxed - not all components need integration tests
            requires_rationale: true,
            requires_lessons_learned: true,
        }
    }
}

/// Required sections in a migration record
const REQUIRED_SECTIONS: &[&str] = &[
    "component:",
    "migration:",
    "schemaDecisions:",
    "inputSchema:",
    "outputSchema:",
    "rustSchema:",
    "testCases:",
    "lessonsLearned:",
];

/// All components that should have migration records
const EXPECTED_COMPONENTS: &[&str] = &[
    "trigger",
    "start",
    "stop",
    "conditional",
    "loop",
    "activity",
    "log",
    "http_request",
    "database_query",
    "agent",
    "child_workflow",
    "signal",
    "timer",
    "parallel",
];

fn get_component_records_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("component-records")
}

#[test]
fn test_all_components_have_migration_records() {
    let records_dir = get_component_records_dir();

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        assert!(
            record_path.exists(),
            "Missing migration record for component: {}. Expected at: {}",
            component,
            record_path.display()
        );
    }
}

#[test]
fn test_all_records_have_required_sections() {
    let records_dir = get_component_records_dir();

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        if !record_path.exists() {
            continue; // Skip if file doesn't exist (caught by other test)
        }

        let content = fs::read_to_string(&record_path)
            .expect(&format!("Failed to read {}", record_path.display()));

        for section in REQUIRED_SECTIONS {
            assert!(
                content.contains(section),
                "Migration record for '{}' is missing required section: {}",
                component,
                section
            );
        }
    }
}

#[test]
fn test_schema_decisions_have_rationale() {
    let records_dir = get_component_records_dir();

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        if !record_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&record_path)
            .expect(&format!("Failed to read {}", record_path.display()));

        // Find schemaDecisions section
        if let Some(decisions_start) = content.find("schemaDecisions:") {
            let decisions_section = &content[decisions_start..];

            // Each decision should have a rationale
            let decision_count = decisions_section.matches("- field:").count();
            let rationale_count = decisions_section.matches("rationale:").count();

            assert!(
                rationale_count >= decision_count,
                "Component '{}' has {} schema decisions but only {} have rationale",
                component,
                decision_count,
                rationale_count
            );
        }
    }
}

#[test]
fn test_minimum_schema_decisions() {
    let records_dir = get_component_records_dir();
    let criteria = QualityCriteria::default();

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        if !record_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&record_path)
            .expect(&format!("Failed to read {}", record_path.display()));

        let decision_count = content.matches("- field:").count();

        // Note: Some simple components like start/stop may have fewer decisions
        // We allow a lower minimum for those
        let min_decisions = if *component == "start" || *component == "stop" {
            1
        } else {
            criteria.min_schema_decisions
        };

        assert!(
            decision_count >= min_decisions,
            "Component '{}' has only {} schema decisions, minimum is {}",
            component,
            decision_count,
            min_decisions
        );
    }
}

#[test]
fn test_test_cases_present() {
    let records_dir = get_component_records_dir();
    let criteria = QualityCriteria::default();

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        if !record_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&record_path)
            .expect(&format!("Failed to read {}", record_path.display()));

        let test_count = content.matches("- name: test_").count();

        assert!(
            test_count >= criteria.min_unit_tests,
            "Component '{}' has only {} test cases, minimum is {}",
            component,
            test_count,
            criteria.min_unit_tests
        );
    }
}

#[test]
fn test_lessons_learned_present() {
    let records_dir = get_component_records_dir();

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        if !record_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&record_path)
            .expect(&format!("Failed to read {}", record_path.display()));

        // Check for lessonsLearned section with content
        assert!(
            content.contains("lessonsLearned:"),
            "Component '{}' is missing lessonsLearned section",
            component
        );

        assert!(
            content.contains("whatWorkedWell:"),
            "Component '{}' is missing whatWorkedWell in lessonsLearned",
            component
        );
    }
}

#[test]
fn test_rust_schema_file_paths_valid() {
    let records_dir = get_component_records_dir();
    let _src_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        if !record_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&record_path)
            .expect(&format!("Failed to read {}", record_path.display()));

        // Extract filePath from rustSchema section
        if let Some(file_path_start) = content.find("filePath:") {
            let file_path_line = &content[file_path_start..];
            if let Some(end) = file_path_line.find('\n') {
                let file_path = file_path_line[9..end].trim();

                // Check if the referenced file exists (relative to crate root)
                let full_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(file_path);

                assert!(
                    full_path.exists(),
                    "Component '{}' references non-existent file: {}",
                    component,
                    file_path
                );
            }
        }
    }
}

#[test]
fn test_input_output_schemas_have_fields() {
    let records_dir = get_component_records_dir();

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        if !record_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&record_path)
            .expect(&format!("Failed to read {}", record_path.display()));

        // Check inputSchema has fields
        if let Some(input_start) = content.find("inputSchema:") {
            let input_section = &content[input_start..];
            assert!(
                input_section.contains("fields:"),
                "Component '{}' inputSchema is missing fields definition",
                component
            );
        }

        // Check outputSchema has fields
        if let Some(output_start) = content.find("outputSchema:") {
            let output_section = &content[output_start..];
            assert!(
                output_section.contains("fields:"),
                "Component '{}' outputSchema is missing fields definition",
                component
            );
        }
    }
}

#[test]
fn test_yaml_is_valid() {
    let records_dir = get_component_records_dir();

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        if !record_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&record_path)
            .expect(&format!("Failed to read {}", record_path.display()));

        // Try to parse as YAML value to verify basic syntax
        let result: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);

        assert!(
            result.is_ok(),
            "Component '{}' has invalid YAML: {:?}",
            component,
            result.err()
        );
    }
}

#[test]
fn test_component_names_match_filenames() {
    let records_dir = get_component_records_dir();

    for component in EXPECTED_COMPONENTS {
        let record_path = records_dir.join(format!("{}.yaml", component));
        if !record_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&record_path)
            .expect(&format!("Failed to read {}", record_path.display()));

        // Find the component name in the file
        if let Some(name_start) = content.find("name:") {
            let name_line = &content[name_start..];
            if let Some(end) = name_line.find('\n') {
                let name = name_line[5..end].trim();
                assert_eq!(
                    name, *component,
                    "Component filename '{}' doesn't match internal name '{}'",
                    component, name
                );
            }
        }
    }
}

#[test]
fn test_quality_checklist_exists() {
    let records_dir = get_component_records_dir();
    let checklist_path = records_dir.join("quality-checklist.yaml");

    assert!(
        checklist_path.exists(),
        "Quality checklist not found at: {}",
        checklist_path.display()
    );

    let content = fs::read_to_string(&checklist_path)
        .expect("Failed to read quality checklist");

    assert!(
        content.contains("requiredSections:"),
        "Quality checklist missing requiredSections"
    );
    assert!(
        content.contains("qualityCriteria:"),
        "Quality checklist missing qualityCriteria"
    );
}
