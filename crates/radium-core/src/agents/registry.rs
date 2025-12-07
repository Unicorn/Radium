//! Agent registry for runtime agent management.
//!
//! Provides a thread-safe registry for managing discovered agents at runtime.

use crate::agents::config::AgentConfig;
use crate::agents::discovery::{AgentDiscovery, DiscoveryOptions};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use strsim::levenshtein;

/// Search mode for text matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Exact match (case-insensitive).
    Exact,
    /// Contains match (substring, case-insensitive).
    Contains,
    /// Fuzzy match using Levenshtein distance.
    Fuzzy,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::Contains
    }
}

/// Logic mode for combining filter criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicMode {
    /// All criteria must match (AND).
    And,
    /// Any criteria can match (OR).
    Or,
}

impl Default for LogicMode {
    fn default() -> Self {
        LogicMode::And
    }
}

/// Filter criteria for agent filtering.
#[derive(Debug, Clone)]
pub struct FilterCriteria {
    /// Filter by category (partial match, case-insensitive).
    pub category: Option<String>,
    /// Filter by engine (exact match, case-insensitive).
    pub engine: Option<String>,
    /// Filter by model (partial match, case-insensitive).
    pub model: Option<String>,
    /// Filter by tags (any tag must match).
    pub tags: Option<Vec<String>>,
    /// Search mode for text matching.
    pub search_mode: SearchMode,
    /// Logic mode for combining criteria.
    pub logic_mode: LogicMode,
    /// Similarity threshold for fuzzy search (0.0 to 1.0, default 0.7).
    pub fuzzy_threshold: f64,
}

impl Default for FilterCriteria {
    fn default() -> Self {
        Self {
            category: None,
            engine: None,
            model: None,
            tags: None,
            search_mode: SearchMode::Contains,
            logic_mode: LogicMode::And,
            fuzzy_threshold: 0.7,
        }
    }
}

/// Sort order for agent sorting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort by agent name (alphabetical).
    Name,
    /// Sort by category (alphabetical).
    Category,
    /// Sort by engine (alphabetical).
    Engine,
    /// Sort by multiple fields (chained sorting).
    Multiple(Vec<SortField>),
}

/// Sort field for multi-field sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    /// Sort by agent name.
    Name,
    /// Sort by category.
    Category,
    /// Sort by engine.
    Engine,
    /// Sort by model.
    Model,
    /// Sort by ID.
    Id,
}

/// Agent registry errors.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Agent not found.
    #[error("agent not found: {0}")]
    NotFound(String),

    /// Discovery error.
    #[error("discovery error: {0}")]
    Discovery(#[from] crate::agents::discovery::DiscoveryError),

    /// Lock poisoned.
    #[error("lock poisoned: {0}")]
    LockPoisoned(String),

    /// Agent already registered.
    #[error("agent already registered: {0}")]
    AlreadyRegistered(String),
}

/// Result type for registry operations.
pub type Result<T> = std::result::Result<T, RegistryError>;

/// Agent registry for runtime management.
///
/// Maintains a thread-safe collection of discovered agents and provides
/// lookup, filtering, and management operations.
pub struct AgentRegistry {
    /// Registered agents indexed by ID.
    agents: Arc<RwLock<HashMap<String, AgentConfig>>>,
}

impl AgentRegistry {
    /// Creates a new empty agent registry.
    pub fn new() -> Self {
        Self { agents: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Creates a new agent registry and automatically discovers agents.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails.
    pub fn with_discovery() -> Result<Self> {
        let registry = Self::new();
        registry.discover_and_register()?;
        Ok(registry)
    }

    /// Creates a new agent registry with custom discovery options.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails.
    pub fn with_discovery_options(options: DiscoveryOptions) -> Result<Self> {
        let registry = Self::new();
        registry.discover_and_register_with_options(options)?;
        Ok(registry)
    }

    /// Discovers and registers all agents using default discovery.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails or lock is poisoned.
    pub fn discover_and_register(&self) -> Result<()> {
        let discovery = AgentDiscovery::new();
        let discovered_agents = discovery.discover_all()?;

        let mut agents =
            self.agents.write().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        agents.extend(discovered_agents);

        Ok(())
    }

    /// Discovers and registers all agents with custom options.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails or lock is poisoned.
    pub fn discover_and_register_with_options(&self, options: DiscoveryOptions) -> Result<()> {
        let discovery = AgentDiscovery::with_options(options);
        let discovered_agents = discovery.discover_all()?;

        let mut agents =
            self.agents.write().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        agents.extend(discovered_agents);

        Ok(())
    }

    /// Registers a single agent.
    ///
    /// # Errors
    ///
    /// Returns error if agent with same ID already exists or lock is poisoned.
    pub fn register(&self, agent: AgentConfig) -> Result<()> {
        let mut agents =
            self.agents.write().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        if agents.contains_key(&agent.id) {
            return Err(RegistryError::AlreadyRegistered(agent.id));
        }

        agents.insert(agent.id.clone(), agent);

        Ok(())
    }

    /// Registers a single agent, replacing if it already exists.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn register_or_replace(&self, agent: AgentConfig) -> Result<()> {
        let mut agents =
            self.agents.write().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        agents.insert(agent.id.clone(), agent);

        Ok(())
    }

    /// Gets an agent by ID.
    ///
    /// # Errors
    ///
    /// Returns error if agent not found or lock is poisoned.
    pub fn get(&self, id: &str) -> Result<AgentConfig> {
        let agents = self.agents.read().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        agents.get(id).cloned().ok_or_else(|| RegistryError::NotFound(id.to_string()))
    }

    /// Checks if an agent exists.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn contains(&self, id: &str) -> Result<bool> {
        let agents = self.agents.read().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        Ok(agents.contains_key(id))
    }

    /// Lists all registered agent IDs.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn list_ids(&self) -> Result<Vec<String>> {
        let agents = self.agents.read().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        Ok(agents.keys().cloned().collect())
    }

    /// Lists all registered agents.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn list_all(&self) -> Result<Vec<AgentConfig>> {
        let agents = self.agents.read().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        Ok(agents.values().cloned().collect())
    }

    /// Filters agents by a predicate function.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn filter<F>(&self, predicate: F) -> Result<Vec<AgentConfig>>
    where
        F: Fn(&AgentConfig) -> bool,
    {
        let agents = self.agents.read().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        Ok(agents.values().filter(|a| predicate(a)).cloned().collect())
    }

    /// Searches agents by name (case-insensitive partial match).
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn search(&self, query: &str) -> Result<Vec<AgentConfig>> {
        self.search_with_mode(query, SearchMode::Contains)
    }

    /// Searches agents with a specific search mode.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn search_with_mode(&self, query: &str, mode: SearchMode) -> Result<Vec<AgentConfig>> {
        let query_lower = query.to_lowercase();
        let threshold = 0.7; // Default fuzzy threshold

        self.filter(|agent| {
            let name_lower = agent.name.to_lowercase();
            let desc_lower = agent.description.to_lowercase();
            let id_lower = agent.id.to_lowercase();

            match mode {
                SearchMode::Exact => {
                    name_lower == query_lower
                        || desc_lower == query_lower
                        || id_lower == query_lower
                }
                SearchMode::Contains => {
                    name_lower.contains(&query_lower)
                        || desc_lower.contains(&query_lower)
                        || id_lower.contains(&query_lower)
                }
                SearchMode::Fuzzy => {
                    Self::fuzzy_match(&name_lower, &query_lower, threshold)
                        || Self::fuzzy_match(&desc_lower, &query_lower, threshold)
                        || Self::fuzzy_match(&id_lower, &query_lower, threshold)
                }
            }
        })
    }

    /// Checks if two strings match using fuzzy matching.
    fn fuzzy_match(text: &str, query: &str, threshold: f64) -> bool {
        if text.is_empty() || query.is_empty() {
            return false;
        }

        let distance = levenshtein(text, query);
        let max_len = text.len().max(query.len());
        if max_len == 0 {
            return false;
        }

        let similarity = 1.0 - (distance as f64 / max_len as f64);
        similarity >= threshold
    }

    /// Returns the number of registered agents.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn count(&self) -> Result<usize> {
        let agents = self.agents.read().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        Ok(agents.len())
    }

    /// Finds agents by category.
    ///
    /// # Arguments
    /// * `category` - The category to match (case-insensitive partial match)
    ///
    /// # Returns
    /// Vector of agents in the category
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn find_by_category(&self, category: &str) -> Result<Vec<AgentConfig>> {
        let category_lower = category.to_lowercase();
        self.filter(|agent| {
            agent.category
                .as_ref()
                .map(|c| c.to_lowercase().contains(&category_lower))
                .unwrap_or(false)
        })
    }

    /// Finds agents similar to a given agent (same category).
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID to find similar agents for
    ///
    /// # Returns
    /// Vector of similar agents (excluding the original)
    ///
    /// # Errors
    /// Returns error if agent not found or lock is poisoned
    pub fn find_similar(&self, agent_id: &str) -> Result<Vec<AgentConfig>> {
        let agent = self.get(agent_id)?;
        let category = agent.category.clone();

        if let Some(cat) = category {
            self.find_by_category(&cat).map(|mut agents| {
                agents.retain(|a| a.id != agent_id);
                agents
            })
        } else {
            Ok(Vec::new())
        }
    }

    /// Clears all registered agents.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn clear(&self) -> Result<()> {
        let mut agents =
            self.agents.write().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        agents.clear();

        Ok(())
    }

    /// Removes an agent by ID.
    ///
    /// # Errors
    ///
    /// Returns error if agent not found or lock is poisoned.
    pub fn remove(&self, id: &str) -> Result<AgentConfig> {
        let mut agents =
            self.agents.write().map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;

        agents.remove(id).ok_or_else(|| RegistryError::NotFound(id.to_string()))
    }

    /// Filters agents by category (partial match, case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn filter_by_category(&self, category: &str) -> Result<Vec<AgentConfig>> {
        let category_lower = category.to_lowercase();
        self.filter(|agent| {
            agent.category.as_ref().map_or(false, |c| {
                c.to_lowercase().contains(&category_lower)
            })
        })
    }

    /// Filters agents by engine (exact match, case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn filter_by_engine(&self, engine: &str) -> Result<Vec<AgentConfig>> {
        let engine_lower = engine.to_lowercase();
        self.filter(|agent| {
            agent.engine.as_ref().map_or(false, |e| e.to_lowercase() == engine_lower)
        })
    }

    /// Filters agents by model (partial match, case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn filter_by_model(&self, model: &str) -> Result<Vec<AgentConfig>> {
        let model_lower = model.to_lowercase();
        self.filter(|agent| {
            agent.model.as_ref().map_or(false, |m| {
                m.to_lowercase().contains(&model_lower)
            })
        })
    }

    /// Filters agents using combined criteria.
    ///
    /// Uses AND or OR logic based on `criteria.logic_mode`.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn filter_combined(&self, criteria: &FilterCriteria) -> Result<Vec<AgentConfig>> {
        self.filter(|agent| {
            let mut matches = Vec::new();

            // Category filter
            if let Some(ref category) = criteria.category {
                let category_lower = category.to_lowercase();
                let agent_category = agent.category.as_ref().map(|c| c.to_lowercase()).unwrap_or_default();
                matches.push(Self::match_field(&agent_category, &category_lower, criteria.search_mode, criteria.fuzzy_threshold));
            }

            // Engine filter
            if let Some(ref engine) = criteria.engine {
                let engine_lower = engine.to_lowercase();
                let agent_engine = agent.engine.as_ref().map(|e| e.to_lowercase()).unwrap_or_default();
                matches.push(Self::match_field(&agent_engine, &engine_lower, SearchMode::Exact, criteria.fuzzy_threshold));
            }

            // Model filter
            if let Some(ref model) = criteria.model {
                let model_lower = model.to_lowercase();
                let agent_model = agent.model.as_ref().map(|m| m.to_lowercase()).unwrap_or_default();
                matches.push(Self::match_field(&agent_model, &model_lower, criteria.search_mode, criteria.fuzzy_threshold));
            }

            // Tags filter
            if let Some(ref tags) = criteria.tags {
                let agent_tags = Self::extract_tags(agent);
                let tag_match = tags.iter().any(|tag| {
                    agent_tags.iter().any(|agent_tag| {
                        Self::match_field(agent_tag, tag, criteria.search_mode, criteria.fuzzy_threshold)
                    })
                });
                matches.push(tag_match);
            }

            // Apply logic mode
            if matches.is_empty() {
                true // No filters specified, include all
            } else {
                match criteria.logic_mode {
                    LogicMode::And => matches.iter().all(|&m| m),
                    LogicMode::Or => matches.iter().any(|&m| m),
                }
            }
        })
    }

    /// Matches a field value against a query using the specified search mode.
    fn match_field(field: &str, query: &str, mode: SearchMode, threshold: f64) -> bool {
        match mode {
            SearchMode::Exact => field == query,
            SearchMode::Contains => field.contains(query),
            SearchMode::Fuzzy => Self::fuzzy_match(field, query, threshold),
        }
    }

    /// Extracts tags from an agent (placeholder - tags not yet in AgentConfig).
    /// This is a placeholder for future tag support.
    fn extract_tags(_agent: &AgentConfig) -> Vec<String> {
        // TODO: Extract tags from agent metadata when tags are added to AgentConfig
        Vec::new()
    }

    /// Sorts agents by the specified order.
    ///
    /// # Errors
    ///
    /// Returns error if lock is poisoned.
    pub fn sort(&self, order: SortOrder) -> Result<Vec<AgentConfig>> {
        let mut agents = self.list_all()?;

        match order {
            SortOrder::Name => {
                agents.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortOrder::Category => {
                agents.sort_by(|a, b| {
                    a.category
                        .as_ref()
                        .unwrap_or(&String::new())
                        .cmp(b.category.as_ref().unwrap_or(&String::new()))
                });
            }
            SortOrder::Engine => {
                agents.sort_by(|a, b| {
                    a.engine
                        .as_ref()
                        .unwrap_or(&String::new())
                        .cmp(b.engine.as_ref().unwrap_or(&String::new()))
                });
            }
            SortOrder::Multiple(fields) => {
                agents.sort_by(|a, b| {
                    for field in &fields {
                        let cmp = match field {
                            SortField::Name => a.name.cmp(&b.name),
                            SortField::Category => {
                                a.category
                                    .as_ref()
                                    .unwrap_or(&String::new())
                                    .cmp(b.category.as_ref().unwrap_or(&String::new()))
                            }
                            SortField::Engine => {
                                a.engine
                                    .as_ref()
                                    .unwrap_or(&String::new())
                                    .cmp(b.engine.as_ref().unwrap_or(&String::new()))
                            }
                            SortField::Model => {
                                a.model
                                    .as_ref()
                                    .unwrap_or(&String::new())
                                    .cmp(b.model.as_ref().unwrap_or(&String::new()))
                            }
                            SortField::Id => a.id.cmp(&b.id),
                        };
                        if cmp != std::cmp::Ordering::Equal {
                            return cmp;
                        }
                    }
                    std::cmp::Ordering::Equal
                });
            }
        }

        Ok(agents)
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent(id: &str, name: &str) -> AgentConfig {
        use std::path::PathBuf;

        AgentConfig {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("Test agent: {}", name),
            prompt_path: PathBuf::from("test.md"),
            mirror_path: None,
            engine: None,
            model: None,
            reasoning_effort: None,
            loop_behavior: None,
            trigger_behavior: None,
            category: None,
            file_path: None,
            capabilities: crate::agents::config::AgentCapabilities::default(),
            sandbox: None,
            persona_config: None,
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = AgentRegistry::new();
        assert_eq!(registry.count().unwrap(), 0);
    }

    #[test]
    fn test_register_agent() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("test-1", "Test Agent");

        registry.register(agent).unwrap();
        assert_eq!(registry.count().unwrap(), 1);

        let retrieved = registry.get("test-1").unwrap();
        assert_eq!(retrieved.id, "test-1");
        assert_eq!(retrieved.name, "Test Agent");
    }

    #[test]
    fn test_register_duplicate_fails() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("test-1", "Test Agent");

        registry.register(agent.clone()).unwrap();
        let result = registry.register(agent);

        assert!(matches!(result, Err(RegistryError::AlreadyRegistered(_))));
    }

    #[test]
    fn test_register_or_replace() {
        let registry = AgentRegistry::new();
        let agent1 = create_test_agent("test-1", "Agent V1");
        let agent2 = create_test_agent("test-1", "Agent V2");

        registry.register_or_replace(agent1).unwrap();
        registry.register_or_replace(agent2).unwrap();

        assert_eq!(registry.count().unwrap(), 1);
        let retrieved = registry.get("test-1").unwrap();
        assert_eq!(retrieved.name, "Agent V2");
    }

    #[test]
    fn test_get_not_found() {
        let registry = AgentRegistry::new();
        let result = registry.get("nonexistent");

        assert!(matches!(result, Err(RegistryError::NotFound(_))));
    }

    #[test]
    fn test_contains() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("test-1", "Test Agent");

        assert!(!registry.contains("test-1").unwrap());

        registry.register(agent).unwrap();

        assert!(registry.contains("test-1").unwrap());
    }

    #[test]
    fn test_list_ids() {
        let registry = AgentRegistry::new();

        registry.register(create_test_agent("agent-1", "Agent 1")).unwrap();
        registry.register(create_test_agent("agent-2", "Agent 2")).unwrap();
        registry.register(create_test_agent("agent-3", "Agent 3")).unwrap();

        let mut ids = registry.list_ids().unwrap();
        ids.sort();

        assert_eq!(ids, vec!["agent-1", "agent-2", "agent-3"]);
    }

    #[test]
    fn test_list_all() {
        let registry = AgentRegistry::new();

        registry.register(create_test_agent("agent-1", "Agent 1")).unwrap();
        registry.register(create_test_agent("agent-2", "Agent 2")).unwrap();

        let agents = registry.list_all().unwrap();
        assert_eq!(agents.len(), 2);
    }

    #[test]
    fn test_filter() {
        let registry = AgentRegistry::new();

        registry.register(create_test_agent("agent-1", "Code Agent")).unwrap();
        registry.register(create_test_agent("agent-2", "Test Agent")).unwrap();
        registry.register(create_test_agent("agent-3", "Code Generator")).unwrap();

        let code_agents = registry.filter(|a| a.name.contains("Code")).unwrap();
        assert_eq!(code_agents.len(), 2);
    }

    #[test]
    fn test_search() {
        let registry = AgentRegistry::new();

        registry.register(create_test_agent("agent-1", "Code Agent")).unwrap();
        registry.register(create_test_agent("agent-2", "Test Agent")).unwrap();
        registry.register(create_test_agent("agent-3", "Documentation Agent")).unwrap();

        let results = registry.search("code").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "agent-1");

        let results = registry.search("agent").unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_remove() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("test-1", "Test Agent");

        registry.register(agent).unwrap();
        assert_eq!(registry.count().unwrap(), 1);

        let removed = registry.remove("test-1").unwrap();
        assert_eq!(removed.id, "test-1");
        assert_eq!(registry.count().unwrap(), 0);
    }

    #[test]
    fn test_clear() {
        let registry = AgentRegistry::new();

        registry.register(create_test_agent("agent-1", "Agent 1")).unwrap();
        registry.register(create_test_agent("agent-2", "Agent 2")).unwrap();

        assert_eq!(registry.count().unwrap(), 2);

        registry.clear().unwrap();

        assert_eq!(registry.count().unwrap(), 0);
    }

    #[test]
    fn test_filter_by_category() {
        let registry = AgentRegistry::new();

        registry
            .register(create_test_agent("agent-1", "Agent 1").with_category("core"))
            .unwrap();
        registry
            .register(create_test_agent("agent-2", "Agent 2").with_category("testing"))
            .unwrap();
        registry
            .register(create_test_agent("agent-3", "Agent 3").with_category("core"))
            .unwrap();

        let core_agents = registry.filter_by_category("core").unwrap();
        assert_eq!(core_agents.len(), 2);
        assert!(core_agents.iter().all(|a| a.category.as_ref().map_or(false, |c| c.contains("core"))));

        let testing_agents = registry.filter_by_category("test").unwrap();
        assert_eq!(testing_agents.len(), 1);
    }

    #[test]
    fn test_filter_by_engine() {
        let registry = AgentRegistry::new();

        registry
            .register(create_test_agent("agent-1", "Agent 1").with_engine("gemini"))
            .unwrap();
        registry
            .register(create_test_agent("agent-2", "Agent 2").with_engine("openai"))
            .unwrap();
        registry
            .register(create_test_agent("agent-3", "Agent 3").with_engine("gemini"))
            .unwrap();

        let gemini_agents = registry.filter_by_engine("gemini").unwrap();
        assert_eq!(gemini_agents.len(), 2);
        assert!(gemini_agents.iter().all(|a| a.engine.as_ref().map_or(false, |e| e == "gemini")));

        let openai_agents = registry.filter_by_engine("openai").unwrap();
        assert_eq!(openai_agents.len(), 1);
    }

    #[test]
    fn test_concurrent_reads() {
        use std::sync::Arc;
        use std::thread;

        let registry = Arc::new(AgentRegistry::new());
        
        // Register some agents
        registry.register(create_test_agent("agent-1", "Agent 1")).unwrap();
        registry.register(create_test_agent("agent-2", "Agent 2")).unwrap();
        registry.register(create_test_agent("agent-3", "Agent 3")).unwrap();

        // Spawn multiple threads that read concurrently
        let mut handles = vec![];
        for _ in 0..10 {
            let reg = registry.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let _ = reg.get("agent-1").unwrap();
                    let _ = reg.count().unwrap();
                    let _ = reg.list_all().unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify registry is still intact
        assert_eq!(registry.count().unwrap(), 3);
    }

    #[test]
    fn test_concurrent_register_and_get() {
        use std::sync::Arc;
        use std::thread;

        let registry = Arc::new(AgentRegistry::new());

        // Spawn threads that register and read concurrently
        let mut handles = vec![];
        for i in 0..5 {
            let reg = registry.clone();
            let handle = thread::spawn(move || {
                let agent_id = format!("agent-{}", i);
                let agent = create_test_agent(&agent_id, &format!("Agent {}", i));
                reg.register(agent).unwrap();
                
                // Try to get immediately after registering
                for _ in 0..10 {
                    let _ = reg.get(&agent_id).unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all agents were registered
        assert_eq!(registry.count().unwrap(), 5);
    }

    #[test]
    fn test_concurrent_filter_operations() {
        use std::sync::Arc;
        use std::thread;

        let registry = Arc::new(AgentRegistry::new());
        
        // Register agents
        for i in 0..10 {
            let agent = create_test_agent(&format!("agent-{}", i), &format!("Agent {}", i))
                .with_category(if i % 2 == 0 { "core" } else { "test" });
            registry.register(agent).unwrap();
        }

        // Spawn threads that filter concurrently
        let mut handles = vec![];
        for _ in 0..5 {
            let reg = registry.clone();
            let handle = thread::spawn(move || {
                for _ in 0..20 {
                    let _ = reg.filter(|a| a.category.as_ref().map_or(false, |c| c == "core")).unwrap();
                    let _ = reg.filter_by_category("core").unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify registry is still intact
        assert_eq!(registry.count().unwrap(), 10);
    }

    #[test]
    fn test_concurrent_write_contention() {
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        let registry = Arc::new(AgentRegistry::new());

        // Spawn threads that write concurrently
        let mut handles = vec![];
        for i in 0..10 {
            let reg = registry.clone();
            let handle = thread::spawn(move || {
                // Small delay to increase contention
                thread::sleep(Duration::from_millis(1));
                let agent = create_test_agent(&format!("agent-{}", i), &format!("Agent {}", i));
                reg.register(agent).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all agents were registered (some may have been replaced)
        assert_eq!(registry.count().unwrap(), 10);
    }

    #[test]
    fn test_filter_by_model() {
        let registry = AgentRegistry::new();

        registry
            .register(create_test_agent("agent-1", "Agent 1").with_model("gemini-2.0-flash-exp"))
            .unwrap();
        registry
            .register(create_test_agent("agent-2", "Agent 2").with_model("gpt-4"))
            .unwrap();
        registry
            .register(create_test_agent("agent-3", "Agent 3").with_model("gemini-2.0-flash-thinking"))
            .unwrap();

        let flash_agents = registry.filter_by_model("flash").unwrap();
        assert_eq!(flash_agents.len(), 2);
        assert!(flash_agents.iter().all(|a| a.model.as_ref().map_or(false, |m| m.contains("flash"))));
    }

    #[test]
    fn test_filter_combined() {
        let registry = AgentRegistry::new();

        registry
            .register(
                create_test_agent("agent-1", "Agent 1")
                    .with_category("core")
                    .with_engine("gemini")
                    .with_model("gemini-2.0-flash-exp"),
            )
            .unwrap();
        registry
            .register(
                create_test_agent("agent-2", "Agent 2")
                    .with_category("core")
                    .with_engine("openai")
                    .with_model("gpt-4"),
            )
            .unwrap();
        registry
            .register(
                create_test_agent("agent-3", "Agent 3")
                    .with_category("testing")
                    .with_engine("gemini")
                    .with_model("gemini-2.0-flash-exp"),
            )
            .unwrap();

        // Filter by category and engine
        let criteria = FilterCriteria {
            category: Some("core".to_string()),
            engine: Some("gemini".to_string()),
            model: None,
            tags: None,
            search_mode: SearchMode::Contains,
            logic_mode: LogicMode::And,
            fuzzy_threshold: 0.7,
        };
        let filtered = registry.filter_combined(&criteria).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "agent-1");

        // Filter by all criteria
        let criteria = FilterCriteria {
            category: Some("core".to_string()),
            engine: Some("gemini".to_string()),
            model: Some("flash".to_string()),
            tags: None,
            search_mode: SearchMode::Contains,
            logic_mode: LogicMode::And,
            fuzzy_threshold: 0.7,
        };
        let filtered = registry.filter_combined(&criteria).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "agent-1");
    }

    #[test]
    fn test_sort() {
        let registry = AgentRegistry::new();

        registry
            .register(create_test_agent("z-agent", "Z Agent").with_category("zebra"))
            .unwrap();
        registry
            .register(create_test_agent("a-agent", "A Agent").with_category("alpha"))
            .unwrap();
        registry
            .register(create_test_agent("m-agent", "M Agent").with_category("middle"))
            .unwrap();

        // Sort by name
        let sorted = registry.sort(SortOrder::Name).unwrap();
        assert_eq!(sorted[0].id, "a-agent");
        assert_eq!(sorted[1].id, "m-agent");
        assert_eq!(sorted[2].id, "z-agent");

        // Sort by category
        let sorted = registry.sort(SortOrder::Category).unwrap();
        assert_eq!(sorted[0].category.as_ref().unwrap(), "alpha");
        assert_eq!(sorted[1].category.as_ref().unwrap(), "middle");
        assert_eq!(sorted[2].category.as_ref().unwrap(), "zebra");

        // Sort by engine
        registry.clear().unwrap();
        registry
            .register(create_test_agent("agent-1", "Agent 1").with_engine("z-engine"))
            .unwrap();
        registry
            .register(create_test_agent("agent-2", "Agent 2").with_engine("a-engine"))
            .unwrap();
        registry
            .register(create_test_agent("agent-3", "Agent 3").with_engine("m-engine"))
            .unwrap();

        let sorted = registry.sort(SortOrder::Engine).unwrap();
        assert_eq!(sorted[0].engine.as_ref().unwrap(), "a-engine");
        assert_eq!(sorted[1].engine.as_ref().unwrap(), "m-engine");
        assert_eq!(sorted[2].engine.as_ref().unwrap(), "z-engine");
    }
}
