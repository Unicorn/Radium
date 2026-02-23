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
        tags: Vec::new(),
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

// ── Agent tag filtering round-trip ─────────────────────────────────────────

fn create_tagged_agent(id: &str, tags: Vec<&str>) -> AgentConfig {
    use std::path::PathBuf;
    use radium_core::agents::config::AgentCapabilities;

    AgentConfig {
        id: id.to_string(),
        name: format!("{} Agent", id),
        description: String::new(),
        prompt_path: PathBuf::from("test.md"),
        mirror_path: None,
        engine: None,
        model: None,
        reasoning_effort: None,
        loop_behavior: None,
        trigger_behavior: None,
        category: None,
        file_path: None,
        capabilities: AgentCapabilities::default(),
        sandbox: None,
        persona_config: None,
        routing: None,
        safety_behavior: None,
        code_execution_enabled: None,
        tags: tags.into_iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn test_tag_filter_single_match() {
    let registry = AgentRegistry::new();
    registry.register(create_tagged_agent("code-agent", vec!["code", "rust"])).unwrap();
    registry.register(create_tagged_agent("review-agent", vec!["review", "code"])).unwrap();
    registry.register(create_tagged_agent("design-agent", vec!["design", "ui"])).unwrap();

    let mut criteria = FilterCriteria::default();
    criteria.tags = Some(vec!["rust".to_string()]);

    let results = registry.filter_combined(&criteria).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "code-agent");
}

#[test]
fn test_tag_filter_matches_multiple_agents() {
    let registry = AgentRegistry::new();
    registry.register(create_tagged_agent("code-agent", vec!["code", "rust"])).unwrap();
    registry.register(create_tagged_agent("review-agent", vec!["review", "code"])).unwrap();
    registry.register(create_tagged_agent("design-agent", vec!["design", "ui"])).unwrap();

    let mut criteria = FilterCriteria::default();
    criteria.tags = Some(vec!["code".to_string()]);

    let results = registry.filter_combined(&criteria).unwrap();
    assert_eq!(results.len(), 2);
    let ids: Vec<&str> = results.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"code-agent"));
    assert!(ids.contains(&"review-agent"));
}

#[test]
fn test_tag_filter_no_match() {
    let registry = AgentRegistry::new();
    registry.register(create_tagged_agent("code-agent", vec!["code", "rust"])).unwrap();

    let mut criteria = FilterCriteria::default();
    criteria.tags = Some(vec!["python".to_string()]);

    let results = registry.filter_combined(&criteria).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_tag_filter_empty_tags_on_agent() {
    let registry = AgentRegistry::new();
    // Agent with no tags — a tag filter should not match it
    registry.register(create_tagged_agent("plain-agent", vec![])).unwrap();

    let mut criteria = FilterCriteria::default();
    criteria.tags = Some(vec!["code".to_string()]);

    let results = registry.filter_combined(&criteria).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_tag_filter_none_means_no_tag_constraint() {
    let registry = AgentRegistry::new();
    registry.register(create_tagged_agent("code-agent", vec!["code"])).unwrap();
    registry.register(create_tagged_agent("plain-agent", vec![])).unwrap();

    // FilterCriteria with tags = None should return all agents (no tag constraint)
    let criteria = FilterCriteria::default();
    let results = registry.filter_combined(&criteria).unwrap();
    assert_eq!(results.len(), 2);
}
