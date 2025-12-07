//! Integration tests for Memory Store with Plan Execution.
//!
//! Tests that verify memory store correctly persists agent outputs during plan execution
//! and that subsequent agents can access previous outputs via ContextManager.

use radium_abstraction::{ChatMessage, Model, ModelError, ModelParameters, ModelResponse};
use radium_core::context::ContextManager;
use radium_core::memory::{MemoryEntry, MemoryStore};
use radium_core::models::{Iteration, PlanManifest, PlanTask};
use radium_core::planning::{ExecutionConfig, PlanExecutor, RunMode};
use radium_core::workspace::{RequirementId, Workspace};
use std::path::PathBuf;
use std::pin::Pin;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use tempfile::TempDir;

// Mock model that returns predefined responses
struct MockMemoryModel {
    response: String,
}

impl MockMemoryModel {
    fn new(response: String) -> Self {
        Self { response }
    }
}

#[async_trait::async_trait]
impl Model for MockMemoryModel {
    async fn generate_text(
        &self,
        _prompt: &str,
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        Ok(ModelResponse {
            content: self.response.clone(),
            usage: None,
            model_id: Some("mock".to_string()),
        })
    }

    async fn generate_chat_completion(
        &self,
        _messages: &[ChatMessage],
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        self.generate_text("", _params).await
    }

    fn model_id(&self) -> &str {
        "mock"
    }
}

// Helper to create a temporary workspace
fn create_temp_workspace() -> (TempDir, Workspace) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();
    
    // Create .radium directory structure
    std::fs::create_dir_all(workspace_root.join(".radium")).unwrap();
    
    let workspace = Workspace::create(workspace_root).unwrap();
    (temp_dir, workspace)
}

// Helper to create a manifest with multiple agents
fn create_multi_agent_manifest(req_id: RequirementId) -> PlanManifest {
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

    let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
    
    let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
    task1.agent_id = Some("plan-agent".to_string());
    
    let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
    task2.agent_id = Some("code-agent".to_string());
    
    let mut task3 = PlanTask::new("I1", 3, "Task 3".to_string());
    task3.agent_id = Some("review-agent".to_string());

    iter1.add_task(task1);
    iter1.add_task(task2);
    iter1.add_task(task3);
    manifest.add_iteration(iter1);

    manifest
}

#[tokio::test]
async fn test_memory_persistence_across_plan_execution() {
    let (_temp_dir, workspace) = create_temp_workspace();
    let req_id = RequirementId::from_str("REQ-1").unwrap();
    let mut manifest = create_multi_agent_manifest(req_id);

    // Create MemoryStore and ContextManager
    let memory_store = Arc::new(Mutex::new(
        MemoryStore::new(workspace.root(), req_id).unwrap()
    ));
    
    let context_manager = Arc::new(Mutex::new(
        ContextManager::for_plan(&workspace, req_id).unwrap()
    ));

    // Create executor with memory store and context manager
    let config = ExecutionConfig {
        resume: false,
        skip_completed: false,
        check_dependencies: false,
        state_path: workspace.root().join(".radium/plan/test_manifest.json"),
        context_files: None,
        run_mode: RunMode::Bounded(1),
        context_manager: Some(context_manager),
        memory_store: Some(memory_store.clone()),
        requirement_id: Some(req_id),
    };

    let executor = PlanExecutor::with_config(config);

    // Create mock models with different responses
    let model1 = Arc::new(MockMemoryModel::new("Plan output from plan-agent".to_string()));
    let model2 = Arc::new(MockMemoryModel::new("Code output from code-agent".to_string()));
    let model3 = Arc::new(MockMemoryModel::new("Review output from review-agent".to_string()));

    // Execute tasks sequentially
    let iter = manifest.get_iteration("I1").unwrap();
    let task1 = iter.get_task("I1.T1").unwrap();
    let task2 = iter.get_task("I1.T2").unwrap();
    let task3 = iter.get_task("I1.T3").unwrap();

    // Execute task 1
    let result1 = executor.execute_task(task1, model1).await.unwrap();
    assert!(result1.success);
    assert!(result1.response.is_some());

    // Execute task 2
    let result2 = executor.execute_task(task2, model2).await.unwrap();
    assert!(result2.success);

    // Execute task 3
    let result3 = executor.execute_task(task3, model3).await.unwrap();
    assert!(result3.success);

    // Verify all three memory entries exist
    let store = memory_store.lock().unwrap();
    
    // Check plan-agent memory
    let entry1 = store.get("plan-agent").unwrap();
    assert_eq!(entry1.agent_id, "plan-agent");
    assert!(entry1.output.contains("Plan output"));

    // Check code-agent memory
    let entry2 = store.get("code-agent").unwrap();
    assert_eq!(entry2.agent_id, "code-agent");
    assert!(entry2.output.contains("Code output"));

    // Check review-agent memory
    let entry3 = store.get("review-agent").unwrap();
    assert_eq!(entry3.agent_id, "review-agent");
    assert!(entry3.output.contains("Review output"));

    // Verify all agents are listed
    let agents = store.list_agents();
    assert_eq!(agents.len(), 3);
    assert!(agents.contains(&"plan-agent".to_string()));
    assert!(agents.contains(&"code-agent".to_string()));
    assert!(agents.contains(&"review-agent".to_string()));
}

#[tokio::test]
async fn test_memory_retrieval_by_subsequent_agents() {
    let (_temp_dir, workspace) = create_temp_workspace();
    let req_id = RequirementId::from_str("REQ-2").unwrap();
    let manifest = create_multi_agent_manifest(req_id);

    // Create MemoryStore and ContextManager
    let memory_store = Arc::new(Mutex::new(
        MemoryStore::new(workspace.root(), req_id).unwrap()
    ));
    
    let context_manager = Arc::new(Mutex::new(
        ContextManager::for_plan(&workspace, req_id).unwrap()
    ));

    // First, manually store output from plan-agent
    let output = "This is the plan output from plan-agent. ".repeat(100); // Long output to test truncation
    let entry = MemoryEntry::new("plan-agent".to_string(), output.clone());
    {
        let mut store = memory_store.lock().unwrap();
        store.store(entry).unwrap();
    }

    // Verify the output is truncated to 2000 chars
    {
        let store = memory_store.lock().unwrap();
        let retrieved = store.get("plan-agent").unwrap();
        assert!(retrieved.output.len() <= 2000);
        assert_eq!(&retrieved.output, &output.chars().rev().take(2000).collect::<String>().chars().rev().collect::<String>());
    }

    // Use ContextManager to gather memory context
    {
        let mut manager = context_manager.lock().unwrap();
        let memory_context = manager.gather_memory_context("plan-agent").unwrap();
        assert!(memory_context.is_some());
        let context = memory_context.unwrap();
        assert!(context.contains("plan-agent"));
        // Should contain the truncated output
        assert!(context.contains("plan output"));
    }
}

#[tokio::test]
async fn test_memory_isolation_between_requirement_ids() {
    let (_temp_dir, workspace) = create_temp_workspace();
    let req_id1 = RequirementId::from_str("REQ-3").unwrap();
    let req_id2 = RequirementId::from_str("REQ-4").unwrap();

    // Create separate memory stores for each requirement
    let memory_store1 = MemoryStore::new(workspace.root(), req_id1).unwrap();
    let memory_store2 = MemoryStore::new(workspace.root(), req_id2).unwrap();

    // Store entries in first store
    let entry1 = MemoryEntry::new("agent-1".to_string(), "Output for REQ-003".to_string());
    let mut store1 = memory_store1;
    store1.store(entry1).unwrap();

    // Store entries in second store
    let entry2 = MemoryEntry::new("agent-1".to_string(), "Output for REQ-004".to_string());
    let mut store2 = memory_store2;
    store2.store(entry2).unwrap();

    // Verify isolation - each store should have its own entry
    let retrieved1 = store1.get("agent-1").unwrap();
    assert_eq!(retrieved1.output, "Output for REQ-003");

    let retrieved2 = store2.get("agent-1").unwrap();
    assert_eq!(retrieved2.output, "Output for REQ-004");

    // Verify they are stored in different directories
    let path1 = workspace.root().join(".radium/plan/REQ-3/memory/agent-1.json");
    let path2 = workspace.root().join(".radium/plan/REQ-4/memory/agent-1.json");
    assert!(path1.exists());
    assert!(path2.exists());
    assert_ne!(path1, path2);
}

#[tokio::test]
async fn test_memory_cache_file_synchronization() {
    let (_temp_dir, workspace) = create_temp_workspace();
    let req_id = RequirementId::from_str("REQ-5").unwrap();

    // Create and store entry
    let mut store = MemoryStore::new(workspace.root(), req_id).unwrap();
    let entry = MemoryEntry::new("test-agent".to_string(), "Test output".to_string());
    store.store(entry).unwrap();

    // Verify it's in cache
    let cached = store.get("test-agent").unwrap();
    assert_eq!(cached.output, "Test output");

    // Verify it's on disk
    let file_path = workspace.root().join(".radium/plan/REQ-5/memory/test-agent.json");
    assert!(file_path.exists());

    // Open a new store instance (simulates restart)
    let store2 = MemoryStore::open(workspace.root(), req_id).unwrap();
    
    // Should load from disk and have the entry
    let retrieved = store2.get("test-agent").unwrap();
    assert_eq!(retrieved.output, "Test output");
}

