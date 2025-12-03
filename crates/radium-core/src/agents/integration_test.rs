//! Integration tests for the complete agent configuration system.
//!
//! Tests agent discovery, configuration loading, prompt templates,
//! caching, and file injection working together.

#[cfg(test)]
mod tests {
    use crate::agents::config::{AgentConfig, AgentConfigFile, ReasoningEffort};
    use crate::agents::discovery::{AgentDiscovery, DiscoveryOptions};
    use crate::prompts::processing::FileInjector;
    use crate::prompts::{PromptCache, PromptContext, PromptTemplate};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_agent(
        dir: &std::path::Path,
        category: &str,
        id: &str,
        prompt_content: &str,
    ) -> PathBuf {
        // Create category directory
        let category_dir = dir.join(category);
        fs::create_dir_all(&category_dir).unwrap();

        // Create prompts directory
        let prompts_dir = dir.join("prompts").join(category);
        fs::create_dir_all(&prompts_dir).unwrap();

        // Create prompt file
        let prompt_path = prompts_dir.join(format!("{}.md", id));
        fs::write(&prompt_path, prompt_content).unwrap();

        // Create agent config
        let config = AgentConfigFile {
            agent: AgentConfig::new(
                id,
                format!("{} Agent", id),
                prompt_path.strip_prefix(dir).unwrap().to_path_buf(),
            )
            .with_description(format!("Test agent {}", id))
            .with_engine("gemini")
            .with_model("gemini-2.0-flash-exp")
            .with_reasoning_effort(ReasoningEffort::Medium),
        };

        let config_path = category_dir.join(format!("{}.toml", id));
        config.save(&config_path).unwrap();

        config_path
    }

    #[test]
    fn test_complete_agent_workflow() {
        let temp = TempDir::new().unwrap();

        // Create agent with prompt template
        let prompt_content = r#"
# Test Agent

Hello {{name}}!

Your task is to {{task}}.
"#;
        create_test_agent(temp.path(), "test-agents", "test-agent", prompt_content);

        // Discover agent
        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };
        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();

        assert_eq!(agents.len(), 1);
        let agent = agents.get("test-agent").unwrap();
        assert_eq!(agent.id, "test-agent");

        // Load prompt template
        let prompt_path = temp.path().join(&agent.prompt_path);
        let template = PromptTemplate::load(&prompt_path).unwrap();

        // Render with context
        let mut context = PromptContext::new();
        context.set("name", "World");
        context.set("task", "test the system");

        let rendered = template.render(&context).unwrap();
        assert!(rendered.contains("Hello World!"));
        assert!(rendered.contains("test the system"));
    }

    #[test]
    fn test_agent_with_prompt_caching() {
        let temp = TempDir::new().unwrap();

        let prompt_content = "Cached prompt {{var}}";
        create_test_agent(temp.path(), "test-agents", "cached-agent", prompt_content);

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };
        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();
        let agent = agents.get("cached-agent").unwrap();

        let prompt_path = temp.path().join(&agent.prompt_path);
        let cache = PromptCache::new();

        // First load
        let template1 = cache.load(&prompt_path).unwrap();
        assert_eq!(cache.len(), 1);

        // Second load should use cache
        let template2 = cache.load(&prompt_path).unwrap();
        assert_eq!(template1.content(), template2.content());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_agent_with_file_injection() {
        let temp = TempDir::new().unwrap();

        // Create files to inject
        let file1 = temp.path().join("context1.md");
        let file2 = temp.path().join("context2.md");
        fs::write(&file1, "Context from file 1").unwrap();
        fs::write(&file2, "Context from file 2").unwrap();

        // Create agent with file injection syntax
        let prompt_content = r#"
# Agent with File Injection

Please review:
agent[input:context1.md,context2.md]

Then complete: {{task}}
"#;
        create_test_agent(temp.path(), "test-agents", "inject-agent", prompt_content);

        // Discover and load agent
        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };
        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();
        let agent = agents.get("inject-agent").unwrap();

        let prompt_path = temp.path().join(&agent.prompt_path);
        let template = PromptTemplate::load(&prompt_path).unwrap();

        // Process file injection
        let injector = FileInjector::with_base_path(temp.path());
        let processed = injector.process(template.content()).unwrap();

        assert!(processed.contains("Context from file 1"));
        assert!(processed.contains("Context from file 2"));
        assert!(processed.contains("File: context1.md"));
        assert!(processed.contains("File: context2.md"));

        // Render with context
        let mut context = PromptContext::new();
        context.set("task", "analyze the files");
        let rendered = template.render(&context).unwrap();
        let final_prompt = injector.process(&rendered).unwrap();

        assert!(final_prompt.contains("analyze the files"));
        assert!(final_prompt.contains("Context from file 1"));
    }

    #[test]
    fn test_multiple_agents_with_different_configs() {
        let temp = TempDir::new().unwrap();

        // Create multiple agents
        create_test_agent(temp.path(), "test-agents", "agent-1", "Prompt 1");
        create_test_agent(temp.path(), "test-agents", "agent-2", "Prompt 2");
        create_test_agent(temp.path(), "rad-agents/design", "design-agent", "Design prompt");

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };
        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();

        assert_eq!(agents.len(), 3);

        // Verify all agents can load their prompts
        let cache = PromptCache::new();
        for (id, agent) in &agents {
            let prompt_path = temp.path().join(&agent.prompt_path);
            let template = cache.load(&prompt_path).unwrap();
            assert!(!template.content().is_empty(), "Agent {} should have prompt content", id);
        }

        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_agent_prompt_with_placeholders_and_injection() {
        let temp = TempDir::new().unwrap();

        // Create context file
        let context_file = temp.path().join("requirements.md");
        fs::write(&context_file, "Requirement: Build a test system").unwrap();

        // Create agent with both placeholders and file injection
        let prompt_content = r#"
# Complex Agent

Project: {{project_name}}
Version: {{version}}

Requirements:
agent[input:requirements.md]

Please implement: {{task}}
"#;
        create_test_agent(temp.path(), "test-agents", "complex-agent", prompt_content);

        // Discover and load
        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };
        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();
        let agent = agents.get("complex-agent").unwrap();

        let prompt_path = temp.path().join(&agent.prompt_path);
        let template = PromptTemplate::load(&prompt_path).unwrap();

        // Render placeholders
        let mut context = PromptContext::new();
        context.set("project_name", "Radium");
        context.set("version", "1.0.0");
        context.set("task", "implement tests");
        let rendered = template.render(&context).unwrap();

        // Process file injection
        let injector = FileInjector::with_base_path(temp.path());
        let final_prompt = injector.process(&rendered).unwrap();

        assert!(final_prompt.contains("Project: Radium"));
        assert!(final_prompt.contains("Version: 1.0.0"));
        assert!(final_prompt.contains("implement tests"));
        assert!(final_prompt.contains("Requirement: Build a test system"));
    }
}
