//! Integration tests for Memory Store with Plan Execution.
//!
//! Tests that verify memory store correctly persists agent outputs during plan execution
//! and that subsequent agents can access previous outputs via ContextManager.

use radium_core::context::ContextManager;
use radium_core::memory::{MemoryEntry, MemoryStore};
use radium_core::workspace::{RequirementId, Workspace};
use std::str::FromStr;
use tempfile::TempDir;


// Helper to create a temporary workspace
fn create_temp_workspace() -> (TempDir, Workspace) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();
    
    // Create .radium directory structure
    std::fs::create_dir_all(workspace_root.join(".radium")).unwrap();
    
    let workspace = Workspace::create(workspace_root).unwrap();
    (temp_dir, workspace)
}


#[test]
fn test_memory_persistence_across_plan_execution() {
    let (_temp_dir, workspace) = create_temp_workspace();
    let req_id = RequirementId::from_str("REQ-1").unwrap();

    // Create MemoryStore
    let mut memory_store = MemoryStore::new(workspace.root(), req_id).unwrap();

    // Simulate storing outputs from multiple agents (as executor would do)
    let entry1 = MemoryEntry::new("plan-agent".to_string(), "Plan output from plan-agent".to_string());
    memory_store.store(entry1).unwrap();

    let entry2 = MemoryEntry::new("code-agent".to_string(), "Code output from code-agent".to_string());
    memory_store.store(entry2).unwrap();

    let entry3 = MemoryEntry::new("review-agent".to_string(), "Review output from review-agent".to_string());
    memory_store.store(entry3).unwrap();

    // Verify all three memory entries exist
    // Check plan-agent memory
    let retrieved1 = memory_store.get("plan-agent").unwrap();
    assert_eq!(retrieved1.agent_id, "plan-agent");
    assert!(retrieved1.output.contains("Plan output"));

    // Check code-agent memory
    let retrieved2 = memory_store.get("code-agent").unwrap();
    assert_eq!(retrieved2.agent_id, "code-agent");
    assert!(retrieved2.output.contains("Code output"));

    // Check review-agent memory
    let retrieved3 = memory_store.get("review-agent").unwrap();
    assert_eq!(retrieved3.agent_id, "review-agent");
    assert!(retrieved3.output.contains("Review output"));

    // Verify all agents are listed
    let agents = memory_store.list_agents();
    assert_eq!(agents.len(), 3);
    assert!(agents.contains(&"plan-agent".to_string()));
    assert!(agents.contains(&"code-agent".to_string()));
    assert!(agents.contains(&"review-agent".to_string()));
}

#[test]
fn test_memory_retrieval_by_subsequent_agents() {
    let (_temp_dir, workspace) = create_temp_workspace();
    let req_id = RequirementId::from_str("REQ-2").unwrap();

    // Create MemoryStore and ContextManager
    let mut memory_store = MemoryStore::new(workspace.root(), req_id).unwrap();
    let mut context_manager = ContextManager::for_plan(&workspace, req_id).unwrap();

    // First, manually store output from plan-agent (simulating executor storing output)
    let output = "This is the plan output from plan-agent. ".repeat(100); // Long output to test truncation
    let entry = MemoryEntry::new("plan-agent".to_string(), output.clone());
    memory_store.store(entry).unwrap();

    // Verify the output is truncated to 2000 chars
    let retrieved = memory_store.get("plan-agent").unwrap();
    assert!(retrieved.output.len() <= 2000);
    let expected_truncated: String = output.chars().rev().take(2000).collect::<String>().chars().rev().collect();
    assert_eq!(retrieved.output, expected_truncated);

    // Use ContextManager to gather memory context
    let memory_context = context_manager.gather_memory_context("plan-agent").unwrap();
    assert!(memory_context.is_some());
    let context = memory_context.unwrap();
    assert!(context.contains("plan-agent"));
    // Should contain the truncated output
    assert!(context.contains("plan output"));
}

#[test]
fn test_memory_isolation_between_requirement_ids() {
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

#[test]
fn test_memory_cache_file_synchronization() {
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

