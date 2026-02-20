//! Advanced tests for agent registry filtering and sorting.

use radium_core::agents::config::AgentConfig;
use radium_core::agents::registry::{
    AgentRegistry, FilterCriteria, LogicMode, SearchMode, SortField, SortOrder,
};
use std::path::PathBuf;

fn create_test_agent(id: &str, name: &str, category: Option<&str>, engine: Option<&str>) -> AgentConfig {
    use radium_core::agents::config::AgentCapabilities;

    let mut agent = AgentConfig {
        id: id.to_string(),
        name: name.to_string(),
        description: format!("Test agent: {}", name),
        prompt_path: PathBuf::from("test.md"),
        mirror_path: None,
        engine: engine.map(|s| s.to_string()),
        model: None,
        reasoning_effort: None,
        loop_behavior: None,
        trigger_behavior: None,
        category: category.map(|s| s.to_string()),
        file_path: None,
        capabilities: AgentCapabilities::default(),
        sandbox: None,
        persona_config: None,
        routing: None,
        safety_behavior: None,
        code_execution_enabled: None,
    };
    agent
}

#[test]
fn test_fuzzy_search() {
    let registry = AgentRegistry::new();
    registry
        .register(create_test_agent("arch-agent", "Architecture Agent", Some("core"), None))
        .unwrap();
    registry
        .register(create_test_agent("code-agent", "Code Agent", Some("core"), None))
        .unwrap();

    // Fuzzy search should find "arch" even with typo
    let results = registry.search_with_mode("archtecture", SearchMode::Fuzzy).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].id, "arch-agent");
}

#[test]
fn test_or_logic_filtering() {
    let registry = AgentRegistry::new();
    registry
        .register(create_test_agent("agent-1", "Agent 1", Some("core"), Some("gemini")))
        .unwrap();
    registry
        .register(create_test_agent("agent-2", "Agent 2", Some("design"), Some("openai")))
        .unwrap();
    registry
        .register(create_test_agent("agent-3", "Agent 3", Some("testing"), Some("gemini")))
        .unwrap();

    let mut criteria = FilterCriteria::default();
    criteria.category = Some("core".to_string());
    criteria.engine = Some("openai".to_string());
    criteria.logic_mode = LogicMode::Or;

    let results = registry.filter_combined(&criteria).unwrap();
    // Should match agent-1 (core) OR agent-2 (openai)
    assert_eq!(results.len(), 2);
}

#[test]
fn test_multi_field_sorting() {
    let registry = AgentRegistry::new();
    registry
        .register(create_test_agent("z-agent", "Z Agent", Some("zebra"), None))
        .unwrap();
    registry
        .register(create_test_agent("a-agent", "A Agent", Some("alpha"), None))
        .unwrap();
    registry
        .register(create_test_agent("m-agent", "M Agent", Some("alpha"), None))
        .unwrap();

    let sort_order = SortOrder::Multiple(vec![SortField::Category, SortField::Name]);
    let sorted = registry.sort(sort_order).unwrap();

    // Should be sorted by category first (alpha), then by name (A, M)
    assert_eq!(sorted[0].category.as_ref().unwrap(), "alpha");
    assert_eq!(sorted[0].id, "a-agent");
    assert_eq!(sorted[1].id, "m-agent");
    assert_eq!(sorted[2].category.as_ref().unwrap(), "zebra");
}

