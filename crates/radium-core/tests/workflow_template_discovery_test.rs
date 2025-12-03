//! Tests for workflow template discovery system.

use radium_core::workflow::template_discovery::TemplateDiscovery;
use radium_core::workflow::templates::{WorkflowStep, WorkflowTemplate};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_template_discovery_new() {
    let discovery = TemplateDiscovery::new();
    let paths = discovery.search_paths();

    // Should have at least project-local templates path
    assert!(!paths.is_empty());
    // Should include current directory templates
    assert!(paths.iter().any(|p| p.ends_with("templates")));
}

#[test]
fn test_template_discovery_with_paths() {
    let custom_paths = vec![PathBuf::from("/custom/path1"), PathBuf::from("/custom/path2")];
    let discovery = TemplateDiscovery::with_paths(custom_paths.clone());
    assert_eq!(discovery.search_paths(), custom_paths.as_slice());
}

#[test]
fn test_template_discovery_with_paths_empty() {
    let discovery = TemplateDiscovery::with_paths(vec![]);
    assert_eq!(discovery.search_paths().len(), 0);
}

#[test]
fn test_discover_all_empty_paths() {
    let discovery = TemplateDiscovery::with_paths(vec![]);
    let templates = discovery.discover_all().unwrap();
    assert!(templates.is_empty());
}

#[test]
fn test_discover_all_nonexistent_directory() {
    let discovery = TemplateDiscovery::with_paths(vec![PathBuf::from("/nonexistent/path")]);
    let templates = discovery.discover_all().unwrap();
    // Should return empty, not error
    assert!(templates.is_empty());
}

#[test]
fn test_discover_all_single_template() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();

    // Create a valid template file
    let template =
        WorkflowTemplate::new("test-template").add_step(WorkflowStep::agent_step("agent-1"));
    let template_path = templates_dir.join("test-template.json");
    template.save_to_file(&template_path).unwrap();

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir]);
    let templates = discovery.discover_all().unwrap();

    assert_eq!(templates.len(), 1);
    assert!(templates.contains_key("test-template"));
    assert_eq!(templates.get("test-template").unwrap().name, "test-template");
}

#[test]
fn test_discover_all_multiple_templates() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();

    // Create multiple template files
    for i in 1..=3 {
        let template = WorkflowTemplate::new(format!("template-{}", i))
            .add_step(WorkflowStep::agent_step(format!("agent-{}", i)));
        let template_path = templates_dir.join(format!("template-{}.json", i));
        template.save_to_file(&template_path).unwrap();
    }

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir]);
    let templates = discovery.discover_all().unwrap();

    assert_eq!(templates.len(), 3);
    assert!(templates.contains_key("template-1"));
    assert!(templates.contains_key("template-2"));
    assert!(templates.contains_key("template-3"));
}

#[test]
fn test_discover_all_ignores_non_json_files() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();

    // Create a JSON template
    let template =
        WorkflowTemplate::new("valid-template").add_step(WorkflowStep::agent_step("agent-1"));
    template.save_to_file(&templates_dir.join("valid-template.json")).unwrap();

    // Create a non-JSON file
    std::fs::write(templates_dir.join("invalid.txt"), "not a template").unwrap();

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir]);
    let templates = discovery.discover_all().unwrap();

    // Should only find the JSON file
    assert_eq!(templates.len(), 1);
    assert!(templates.contains_key("valid-template"));
}

#[test]
fn test_discover_all_handles_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();

    // Create invalid JSON file
    std::fs::write(templates_dir.join("invalid.json"), "{ invalid json }").unwrap();

    // Create valid template
    let template =
        WorkflowTemplate::new("valid-template").add_step(WorkflowStep::agent_step("agent-1"));
    template.save_to_file(&templates_dir.join("valid-template.json")).unwrap();

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir]);
    let templates = discovery.discover_all().unwrap();

    // Should only find the valid template, invalid one is silently skipped
    assert_eq!(templates.len(), 1);
    assert!(templates.contains_key("valid-template"));
}

#[test]
fn test_discover_all_duplicate_names() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir1 = temp_dir.path().join("templates1");
    let templates_dir2 = temp_dir.path().join("templates2");
    std::fs::create_dir_all(&templates_dir1).unwrap();
    std::fs::create_dir_all(&templates_dir2).unwrap();

    // Create templates with same name in different directories
    let template1 = WorkflowTemplate::new("duplicate")
        .with_description("First")
        .add_step(WorkflowStep::agent_step("agent-1"));
    template1.save_to_file(&templates_dir1.join("duplicate.json")).unwrap();

    let template2 = WorkflowTemplate::new("duplicate")
        .with_description("Second")
        .add_step(WorkflowStep::agent_step("agent-2"));
    template2.save_to_file(&templates_dir2.join("duplicate.json")).unwrap();

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir1, templates_dir2]);
    let templates = discovery.discover_all().unwrap();

    // Last one wins (based on search path order)
    assert_eq!(templates.len(), 1);
    assert_eq!(templates.get("duplicate").unwrap().description.as_deref(), Some("Second"));
}

#[test]
fn test_find_by_name_exact_match() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();

    let template =
        WorkflowTemplate::new("test-template").add_step(WorkflowStep::agent_step("agent-1"));
    template.save_to_file(&templates_dir.join("test-template.json")).unwrap();

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir]);
    let found = discovery.find_by_name("test-template").unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "test-template");
}

#[test]
fn test_find_by_name_with_extension() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();

    let template =
        WorkflowTemplate::new("test-template").add_step(WorkflowStep::agent_step("agent-1"));
    template.save_to_file(&templates_dir.join("test-template.json")).unwrap();

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir]);
    // Should find even without .json extension
    let found = discovery.find_by_name("test-template").unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "test-template");
}

#[test]
fn test_find_by_name_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir]);
    let found = discovery.find_by_name("nonexistent").unwrap();

    assert!(found.is_none());
}

#[test]
fn test_find_by_name_multiple_paths() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir1 = temp_dir.path().join("templates1");
    let templates_dir2 = temp_dir.path().join("templates2");
    std::fs::create_dir_all(&templates_dir1).unwrap();
    std::fs::create_dir_all(&templates_dir2).unwrap();

    // Create template in second directory
    let template =
        WorkflowTemplate::new("test-template").add_step(WorkflowStep::agent_step("agent-1"));
    template.save_to_file(&templates_dir2.join("test-template.json")).unwrap();

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir1, templates_dir2]);
    let found = discovery.find_by_name("test-template").unwrap();

    // Should find in second path
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "test-template");
}

#[test]
fn test_find_by_name_priority() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir1 = temp_dir.path().join("templates1");
    let templates_dir2 = temp_dir.path().join("templates2");
    std::fs::create_dir_all(&templates_dir1).unwrap();
    std::fs::create_dir_all(&templates_dir2).unwrap();

    // Create templates with same name in both directories
    let template1 = WorkflowTemplate::new("test-template")
        .with_description("First")
        .add_step(WorkflowStep::agent_step("agent-1"));
    template1.save_to_file(&templates_dir1.join("test-template.json")).unwrap();

    let template2 = WorkflowTemplate::new("test-template")
        .with_description("Second")
        .add_step(WorkflowStep::agent_step("agent-2"));
    template2.save_to_file(&templates_dir2.join("test-template.json")).unwrap();

    let discovery = TemplateDiscovery::with_paths(vec![templates_dir1, templates_dir2]);
    let found = discovery.find_by_name("test-template").unwrap();

    // Should find first one (first path has priority)
    assert!(found.is_some());
    assert_eq!(found.unwrap().description.as_deref(), Some("First"));
}

#[test]
fn test_find_by_name_io_error() {
    // Test with a path that causes IO error (e.g., permission denied)
    // This is hard to test without actually creating permission issues
    // For now, we test that invalid paths are handled gracefully
    let discovery = TemplateDiscovery::with_paths(vec![PathBuf::from("/nonexistent/path")]);
    let found = discovery.find_by_name("test").unwrap();
    assert!(found.is_none());
}

#[test]
fn test_template_discovery_default() {
    let discovery = TemplateDiscovery::default();
    // Default should be same as new()
    assert!(!discovery.search_paths().is_empty());
}
