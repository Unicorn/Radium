//! End-to-End tests for Complete Memory & Context Workflow.
//!
//! Tests that verify the complete memory and context system works end-to-end
//! in realistic multi-agent plan execution scenarios.

use radium_core::context::ContextManager;
use radium_core::memory::{MemoryEntry, MemoryStore};
use radium_core::workspace::{RequirementId, Workspace};
use std::fs;
use std::str::FromStr;
use tempfile::TempDir;

#[test]
fn test_complete_memory_context_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let req_id = RequirementId::from_str("REQ-E2E-1").unwrap();

    // Step 1: Create MemoryStore and ContextManager for a plan
    let mut memory_store = MemoryStore::new(workspace.root(), req_id).unwrap();
    let mut context_manager = ContextManager::for_plan(&workspace, req_id).unwrap();

    // Step 2: Simulate first agent execution (plan-agent)
    let plan_output = "Plan: Build feature X with 3 tasks".to_string();
    let entry1 = MemoryEntry::new("plan-agent".to_string(), plan_output.clone());
    memory_store.store(entry1).unwrap();

    // Step 3: Verify first agent's output is stored
    let retrieved1 = memory_store.get("plan-agent").unwrap();
    assert_eq!(retrieved1.agent_id, "plan-agent");
    assert!(retrieved1.output.contains("Plan:"));

    // Step 4: Simulate second agent execution (code-agent) that needs context from first
    // The code-agent should be able to access plan-agent's output via ContextManager
    let memory_context = context_manager.gather_memory_context("plan-agent").unwrap();
    assert!(memory_context.is_some());
    let mem_ctx = memory_context.unwrap();
    assert!(mem_ctx.contains("plan-agent"));
    assert!(mem_ctx.contains("Plan:"));

    // Step 5: Store second agent's output
    let code_output = "Code implementation complete for task 1".to_string();
    let entry2 = MemoryEntry::new("code-agent".to_string(), code_output.clone());
    memory_store.store(entry2).unwrap();

    // Step 6: Verify both agents' outputs are stored
    let agents = memory_store.list_agents();
    assert!(agents.contains(&"plan-agent".to_string()));
    assert!(agents.contains(&"code-agent".to_string()));

    // Step 7: Simulate third agent (review-agent) accessing both previous outputs
    let plan_mem = context_manager.gather_memory_context("plan-agent").unwrap();
    let code_mem = context_manager.gather_memory_context("code-agent").unwrap();

    assert!(plan_mem.is_some());
    assert!(code_mem.is_some());
    assert!(plan_mem.unwrap().contains("Plan:"));
    assert!(code_mem.unwrap().contains("Code implementation"));
}

#[test]
fn test_memory_with_context_files_integration() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let req_id = RequirementId::from_str("REQ-E2E-2").unwrap();

    // Create context file
    let context_file = temp_dir.path().join("GEMINI.md");
    fs::write(&context_file, "# Project Guidelines\n\nAlways write tests.").unwrap();

    // Create MemoryStore and ContextManager
    let mut memory_store = MemoryStore::new(workspace.root(), req_id).unwrap();
    let mut context_manager = ContextManager::for_plan(&workspace, req_id).unwrap();

    // Store agent output
    let entry = MemoryEntry::new("test-agent".to_string(), "Agent output".to_string());
    memory_store.store(entry).unwrap();

    // Build context that should include both context files and memory
    let invocation = "test-agent";
    let context = context_manager.build_context(invocation, Some(req_id)).unwrap();

    // Should contain context file content
    assert!(context.contains("Project Guidelines"));
    assert!(context.contains("Always write tests"));

    // Should contain memory context
    assert!(context.contains("test-agent") || context.contains("Agent output"));
}

#[test]
fn test_memory_isolation_across_plans() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let req_id1 = RequirementId::from_str("REQ-E2E-3").unwrap();
    let req_id2 = RequirementId::from_str("REQ-E2E-4").unwrap();

    // Create separate memory stores for different plans
    let mut store1 = MemoryStore::new(workspace.root(), req_id1).unwrap();
    let mut store2 = MemoryStore::new(workspace.root(), req_id2).unwrap();

    // Store same agent ID in both stores with different content
    let entry1 = MemoryEntry::new("agent-1".to_string(), "Output for Plan 1".to_string());
    let entry2 = MemoryEntry::new("agent-1".to_string(), "Output for Plan 2".to_string());

    store1.store(entry1).unwrap();
    store2.store(entry2).unwrap();

    // Verify isolation
    let retrieved1 = store1.get("agent-1").unwrap();
    let retrieved2 = store2.get("agent-1").unwrap();

    assert_eq!(retrieved1.output, "Output for Plan 1");
    assert_eq!(retrieved2.output, "Output for Plan 2");
    assert_ne!(retrieved1.output, retrieved2.output);

    // Verify they're stored in different directories
    let path1 = workspace.root().join(".radium/plan/REQ-E2E-3/memory/agent-1.json");
    let path2 = workspace.root().join(".radium/plan/REQ-E2E-4/memory/agent-1.json");
    assert!(path1.exists());
    assert!(path2.exists());
}

#[test]
fn test_context_manager_builds_comprehensive_context() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let req_id = RequirementId::from_str("REQ-E2E-5").unwrap();

    // Create context file
    let context_file = temp_dir.path().join("GEMINI.md");
    fs::write(&context_file, "# Context File\n\nProject instructions.").unwrap();

    // Create memory store and add entries
    let mut memory_store = MemoryStore::new(workspace.root(), req_id).unwrap();
    let entry = MemoryEntry::new("test-agent".to_string(), "Previous output".to_string());
    memory_store.store(entry).unwrap();

    // Create ContextManager
    let mut context_manager = ContextManager::for_plan(&workspace, req_id).unwrap();

    // Build comprehensive context
    let invocation = "test-agent";
    let context = context_manager.build_context(invocation, Some(req_id)).unwrap();

    // Should contain multiple context sources
    assert!(!context.is_empty());
    
    // Should contain context file content
    assert!(context.contains("Context File") || context.contains("Project instructions"));
    
    // May contain plan context, memory context, etc.
    // The exact structure depends on what's available
}

