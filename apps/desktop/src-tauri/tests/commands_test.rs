//! Integration tests for Tauri commands.
//!
//! These tests verify that all Tauri commands work correctly with a running Radium server.
//! Tests are marked as ignored and should be run manually with:
//! cargo test --test commands_test -- --ignored
//!
//! Note: These tests verify the command logic by testing through the ClientManager
//! and gRPC client directly, which is the same path the Tauri commands use.

use radium_core::{config::Config, server};
use radium_desktop_lib::client::ClientManager;
use radium_core::proto::{
    Agent, CreateAgentRequest, CreateTaskRequest, CreateWorkflowRequest, DeleteAgentRequest,
    DeleteWorkflowRequest, ExecuteAgentRequest, ExecuteWorkflowRequest, GetAgentRequest, GetTaskRequest,
    GetWorkflowRequest, GetRegisteredAgentsRequest, ListAgentsRequest, ListTasksRequest, ListWorkflowsRequest,
    RegisterAgentRequest, StartAgentRequest, StopAgentRequest, UpdateAgentRequest, UpdateWorkflowRequest, Workflow,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use tonic::Request;

/// Start a test server on a random available port
async fn start_test_server() -> u16 {
    use std::net::TcpListener;
    
    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    
    let mut config = Config::default();
    config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
    config.server.enable_grpc_web = false;

    tokio::spawn(async move {
        server::run(&config).await.expect("Server failed to run");
    });
    
    // Give the server a moment to start
    time::sleep(Duration::from_millis(500)).await;
    
    port
}

/// Create a ClientManager pointing to the test server
async fn create_client_manager(port: u16) -> Arc<Mutex<ClientManager>> {
    let server_address = format!("http://127.0.0.1:{}", port);
    let client_manager = ClientManager::with_address(server_address);
    Arc::new(Mutex::new(client_manager))
}

// ============================================================================
// Agent Command Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_create_agent_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    let agent = Agent {
        id: "test-agent-1".to_string(),
        name: "Test Agent".to_string(),
        description: "A test agent".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    let request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    let response = client.create_agent(request).await.unwrap();
    assert_eq!(response.into_inner().agent_id, "test-agent-1");
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_list_agents_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    // Create a few agents first
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    for i in 1..=2 {
        let agent = Agent {
            id: format!("test-agent-list-{}", i),
            name: format!("Agent {}", i),
            description: format!("Agent {} description", i),
            config_json: "{}".to_string(),
            state: "\"idle\"".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        let request = Request::new(CreateAgentRequest {
            agent: Some(agent),
        });
        client.create_agent(request).await.unwrap();
    }
    
    // List agents using gRPC client directly (same path as Tauri command)
    let request = Request::new(ListAgentsRequest {});
    let response = client.list_agents(request).await.unwrap();
    let agents = response.into_inner().agents;
    
    assert!(agents.len() >= 2);
    let ids: Vec<String> = agents.iter().map(|a| a.id.clone()).collect();
    assert!(ids.contains(&"test-agent-list-1".to_string()));
    assert!(ids.contains(&"test-agent-list-2".to_string()));
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_get_agent_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create an agent
    let agent = Agent {
        id: "test-agent-get".to_string(),
        name: "Get Test Agent".to_string(),
        description: "Agent for get test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let create_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(create_request).await.unwrap();
    
    // Get the agent using gRPC client directly
    let request = Request::new(GetAgentRequest {
        agent_id: "test-agent-get".to_string(),
    });
    let response = client.get_agent(request).await.unwrap();
    let agent = response.into_inner().agent.unwrap();
    
    assert_eq!(agent.id, "test-agent-get");
    assert_eq!(agent.name, "Get Test Agent");
    assert_eq!(agent.description, "Agent for get test");
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_get_agent_not_found() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    let request = Request::new(GetAgentRequest {
        agent_id: "nonexistent-agent".to_string(),
    });
    let result = client.get_agent(request).await;
    
    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_update_agent_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create an agent
    let agent = Agent {
        id: "test-agent-update".to_string(),
        name: "Original Name".to_string(),
        description: "Original description".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let create_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(create_request).await.unwrap();
    
    // Update the agent using gRPC client directly
    // First get the current agent
    let get_request = Request::new(GetAgentRequest {
        agent_id: "test-agent-update".to_string(),
    });
    let get_response = client.get_agent(get_request).await.unwrap();
    let mut agent = get_response.into_inner().agent.unwrap();
    
    // Update fields
    agent.name = "Updated Name".to_string();
    agent.description = "Updated description".to_string();
    agent.updated_at = chrono::Utc::now().to_rfc3339();
    
    let update_request = Request::new(UpdateAgentRequest {
        agent: Some(agent),
    });
    let update_response = client.update_agent(update_request).await.unwrap();
    assert_eq!(update_response.into_inner().agent_id, "test-agent-update");
    
    // Verify the update
    let get_request = Request::new(GetAgentRequest {
        agent_id: "test-agent-update".to_string(),
    });
    let get_response = client.get_agent(get_request).await.unwrap();
    let agent = get_response.into_inner().agent.unwrap();
    assert_eq!(agent.name, "Updated Name");
    assert_eq!(agent.description, "Updated description");
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_delete_agent_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create an agent
    let agent = Agent {
        id: "test-agent-delete".to_string(),
        name: "Delete Test Agent".to_string(),
        description: "Agent to delete".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let create_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(create_request).await.unwrap();
    
    // Delete the agent using gRPC client directly
    let request = Request::new(DeleteAgentRequest {
        agent_id: "test-agent-delete".to_string(),
    });
    let response = client.delete_agent(request).await.unwrap();
    assert!(response.into_inner().success);
    
    // Verify the agent is deleted
    let get_request = Request::new(GetAgentRequest {
        agent_id: "test-agent-delete".to_string(),
    });
    let get_result = client.get_agent(get_request).await;
    assert!(get_result.is_err());
}

// ============================================================================
// Workflow Command Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_create_workflow_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    let workflow = Workflow {
        id: "test-workflow-1".to_string(),
        name: "Test Workflow".to_string(),
        description: "A test workflow".to_string(),
        steps: vec![],
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    let request = Request::new(CreateWorkflowRequest {
        workflow: Some(workflow),
    });
    let response = client.create_workflow(request).await.unwrap();
    assert_eq!(response.into_inner().workflow_id, "test-workflow-1");
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_list_workflows_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create a few workflows
    for i in 1..=2 {
        let workflow = Workflow {
            id: format!("test-workflow-list-{}", i),
            name: format!("Workflow {}", i),
            description: format!("Workflow {} description", i),
            steps: vec![],
            state: "\"idle\"".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        let request = Request::new(CreateWorkflowRequest {
            workflow: Some(workflow),
        });
        client.create_workflow(request).await.unwrap();
    }
    
    // List workflows
    let request = Request::new(ListWorkflowsRequest {});
    let response = client.list_workflows(request).await.unwrap();
    let workflows = response.into_inner().workflows;
    
    assert!(workflows.len() >= 2);
    let ids: Vec<String> = workflows.iter().map(|w| w.id.clone()).collect();
    assert!(ids.contains(&"test-workflow-list-1".to_string()));
    assert!(ids.contains(&"test-workflow-list-2".to_string()));
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_get_workflow_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create a workflow
    let workflow = Workflow {
        id: "test-workflow-get".to_string(),
        name: "Get Test Workflow".to_string(),
        description: "Workflow for get test".to_string(),
        steps: vec![],
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let create_request = Request::new(CreateWorkflowRequest {
        workflow: Some(workflow),
    });
    client.create_workflow(create_request).await.unwrap();
    
    // Get the workflow
    let get_request = Request::new(GetWorkflowRequest {
        workflow_id: "test-workflow-get".to_string(),
    });
    let response = client.get_workflow(get_request).await.unwrap();
    let workflow = response.into_inner().workflow.unwrap();
    
    assert_eq!(workflow.id, "test-workflow-get");
    assert_eq!(workflow.name, "Get Test Workflow");
    assert_eq!(workflow.description, "Workflow for get test");
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_update_workflow_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create a workflow
    let workflow = Workflow {
        id: "test-workflow-update".to_string(),
        name: "Original Name".to_string(),
        description: "Original description".to_string(),
        steps: vec![],
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let create_request = Request::new(CreateWorkflowRequest {
        workflow: Some(workflow),
    });
    client.create_workflow(create_request).await.unwrap();
    
    // Update the workflow
    let get_request = Request::new(GetWorkflowRequest {
        workflow_id: "test-workflow-update".to_string(),
    });
    let get_response = client.get_workflow(get_request).await.unwrap();
    let mut workflow = get_response.into_inner().workflow.unwrap();
    
    workflow.name = "Updated Name".to_string();
    workflow.description = "Updated description".to_string();
    workflow.updated_at = chrono::Utc::now().to_rfc3339();
    
    let update_request = Request::new(UpdateWorkflowRequest {
        workflow: Some(workflow),
    });
    let update_response = client.update_workflow(update_request).await.unwrap();
    assert_eq!(update_response.into_inner().workflow_id, "test-workflow-update");
    
    // Verify the update
    let get_request = Request::new(GetWorkflowRequest {
        workflow_id: "test-workflow-update".to_string(),
    });
    let get_response = client.get_workflow(get_request).await.unwrap();
    let workflow = get_response.into_inner().workflow.unwrap();
    assert_eq!(workflow.name, "Updated Name");
    assert_eq!(workflow.description, "Updated description");
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_delete_workflow_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create a workflow
    let workflow = Workflow {
        id: "test-workflow-delete".to_string(),
        name: "Delete Test Workflow".to_string(),
        description: "Workflow to delete".to_string(),
        steps: vec![],
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let create_request = Request::new(CreateWorkflowRequest {
        workflow: Some(workflow),
    });
    client.create_workflow(create_request).await.unwrap();
    
    // Delete the workflow
    let delete_request = Request::new(DeleteWorkflowRequest {
        workflow_id: "test-workflow-delete".to_string(),
    });
    let response = client.delete_workflow(delete_request).await.unwrap();
    assert!(response.into_inner().success);
    
    // Verify the workflow is deleted
    let get_request = Request::new(GetWorkflowRequest {
        workflow_id: "test-workflow-delete".to_string(),
    });
    let get_result = client.get_workflow(get_request).await;
    assert!(get_result.is_err());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_execute_workflow_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create a workflow
    let workflow = Workflow {
        id: "test-workflow-execute".to_string(),
        name: "Execute Test Workflow".to_string(),
        description: "Workflow to execute".to_string(),
        steps: vec![],
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let create_request = Request::new(CreateWorkflowRequest {
        workflow: Some(workflow),
    });
    client.create_workflow(create_request).await.unwrap();
    
    // Execute the workflow
    let execute_request = Request::new(ExecuteWorkflowRequest {
        workflow_id: "test-workflow-execute".to_string(),
        use_parallel: false,
    });
    let result = client.execute_workflow(execute_request).await;
    
    // Execution may succeed or fail depending on workflow steps, but should return a result
    assert!(result.is_ok());
    let response = result.unwrap().into_inner();
    assert_eq!(response.workflow_id, "test-workflow-execute");
    assert!(response.execution_id.len() > 0);
}

// ============================================================================
// Task Command Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_create_task_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // First create an agent (tasks need an agent_id)
    let agent = Agent {
        id: "test-agent-for-task".to_string(),
        name: "Agent for Task".to_string(),
        description: "Agent for task creation".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    // Create a task
    let task = radium_core::proto::Task {
        id: "test-task-1".to_string(),
        name: "Test Task".to_string(),
        description: "A test task".to_string(),
        agent_id: "test-agent-for-task".to_string(),
        input_json: r#"{"input": "test"}"#.to_string(),
        state: "\"pending\"".to_string(),
        result_json: "{}".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let request = Request::new(CreateTaskRequest {
        task: Some(task),
    });
    let response = client.create_task(request).await.unwrap();
    assert_eq!(response.into_inner().task_id, "test-task-1");
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_list_tasks_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create an agent first
    let agent = Agent {
        id: "test-agent-tasks".to_string(),
        name: "Agent for Tasks".to_string(),
        description: "Agent for task listing".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    // Create a few tasks
    for i in 1..=2 {
        let task = radium_core::proto::Task {
            id: format!("test-task-list-{}", i),
            name: format!("Task {}", i),
            description: format!("Task {} description", i),
            agent_id: "test-agent-tasks".to_string(),
            input_json: "{}".to_string(),
            state: "\"pending\"".to_string(),
            result_json: "{}".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        let request = Request::new(CreateTaskRequest {
            task: Some(task),
        });
        client.create_task(request).await.unwrap();
    }
    
    // List tasks
    let request = Request::new(ListTasksRequest {});
    let response = client.list_tasks(request).await.unwrap();
    let tasks = response.into_inner().tasks;
    
    assert!(tasks.len() >= 2);
    let ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
    assert!(ids.contains(&"test-task-list-1".to_string()));
    assert!(ids.contains(&"test-task-list-2".to_string()));
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_get_task_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create an agent first
    let agent = Agent {
        id: "test-agent-get-task".to_string(),
        name: "Agent for Get Task".to_string(),
        description: "Agent for task get test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    // Create a task
    let task = radium_core::proto::Task {
        id: "test-task-get".to_string(),
        name: "Get Test Task".to_string(),
        description: "Task for get test".to_string(),
        agent_id: "test-agent-get-task".to_string(),
        input_json: r#"{"input": "test data"}"#.to_string(),
        state: "\"pending\"".to_string(),
        result_json: "{}".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let create_request = Request::new(CreateTaskRequest {
        task: Some(task),
    });
    client.create_task(create_request).await.unwrap();
    
    // Get the task
    let get_request = Request::new(GetTaskRequest {
        task_id: "test-task-get".to_string(),
    });
    let response = client.get_task(get_request).await.unwrap();
    let task = response.into_inner().task.unwrap();
    
    assert_eq!(task.id, "test-task-get");
    assert_eq!(task.name, "Get Test Task");
    assert_eq!(task.description, "Task for get test");
    assert_eq!(task.agent_id, "test-agent-get-task");
    assert!(task.input_json.contains("test data"));
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_get_task_not_found() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    let request = Request::new(GetTaskRequest {
        task_id: "nonexistent-task".to_string(),
    });
    let result = client.get_task(request).await;
    
    assert!(result.is_err());
}

// ============================================================================
// Orchestrator Command Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_register_agent_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // First create an agent in the database
    let agent = Agent {
        id: "test-orchestrator-agent".to_string(),
        name: "Orchestrator Test Agent".to_string(),
        description: "Agent for orchestrator tests".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    // Register the agent with the orchestrator
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-orchestrator-agent".to_string(),
        agent_type: "simple".to_string(),
        description: "Simple agent for testing".to_string(),
    });
    let response = client.register_agent(register_request).await.unwrap();
    let result = response.into_inner();
    
    assert!(result.success);
    assert!(result.error.is_none());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_get_registered_agents_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create and register an agent
    let agent = Agent {
        id: "test-registered-list".to_string(),
        name: "List Test Agent".to_string(),
        description: "Agent for list test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-registered-list".to_string(),
        agent_type: "simple".to_string(),
        description: "Test agent".to_string(),
    });
    client.register_agent(register_request).await.unwrap();
    
    // Get registered agents
    let list_request = Request::new(GetRegisteredAgentsRequest {});
    let response = client.get_registered_agents(list_request).await.unwrap();
    let agents = response.into_inner().agents;
    
    assert!(agents.len() >= 1);
    let ids: Vec<String> = agents.iter().map(|a| a.id.clone()).collect();
    assert!(ids.contains(&"test-registered-list".to_string()));
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_execute_agent_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create and register an agent
    let agent = Agent {
        id: "test-execute-agent".to_string(),
        name: "Execute Test Agent".to_string(),
        description: "Agent for execution test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-execute-agent".to_string(),
        agent_type: "simple".to_string(),
        description: "Test agent".to_string(),
    });
    client.register_agent(register_request).await.unwrap();
    
    // Execute the agent
    let execute_request = Request::new(ExecuteAgentRequest {
        agent_id: "test-execute-agent".to_string(),
        input: "Hello, world!".to_string(),
        model_type: Some("mock".to_string()),
        model_id: None,
    });
    let response = client.execute_agent(execute_request).await.unwrap();
    let result = response.into_inner();
    
    // Execution should return a result (may succeed or fail depending on model availability)
    assert!(result.output.len() > 0 || result.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_start_agent_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create and register an agent
    let agent = Agent {
        id: "test-start-agent".to_string(),
        name: "Start Test Agent".to_string(),
        description: "Agent for start test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-start-agent".to_string(),
        agent_type: "simple".to_string(),
        description: "Test agent".to_string(),
    });
    client.register_agent(register_request).await.unwrap();
    
    // Start the agent
    let start_request = Request::new(StartAgentRequest {
        agent_id: "test-start-agent".to_string(),
    });
    let response = client.start_agent(start_request).await.unwrap();
    let result = response.into_inner();
    
    // Start should succeed (or return an error if already started)
    assert!(result.success || result.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_stop_agent_command() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create and register an agent
    let agent = Agent {
        id: "test-stop-agent".to_string(),
        name: "Stop Test Agent".to_string(),
        description: "Agent for stop test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-stop-agent".to_string(),
        agent_type: "simple".to_string(),
        description: "Test agent".to_string(),
    });
    client.register_agent(register_request).await.unwrap();
    
    // Stop the agent
    let stop_request = Request::new(StopAgentRequest {
        agent_id: "test-stop-agent".to_string(),
    });
    let response = client.stop_agent(stop_request).await.unwrap();
    let result = response.into_inner();
    
    // Stop should succeed (or return an error if not running)
    assert!(result.success || result.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_execute_agent_not_found() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Try to execute a non-existent agent
    let execute_request = Request::new(ExecuteAgentRequest {
        agent_id: "nonexistent-agent".to_string(),
        input: "test".to_string(),
        model_type: None,
        model_id: None,
    });
    let response = client.execute_agent(execute_request).await.unwrap();
    let result = response.into_inner();
    
    // Should return an error
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_register_agent_chat_type() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create an agent
    let agent = Agent {
        id: "test-chat-agent".to_string(),
        name: "Chat Test Agent".to_string(),
        description: "Agent for chat type test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    // Register as chat type
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-chat-agent".to_string(),
        agent_type: "chat".to_string(),
        description: "Chat agent for testing".to_string(),
    });
    let response = client.register_agent(register_request).await.unwrap();
    let result = response.into_inner();
    
    assert!(result.success);
    assert!(result.error.is_none());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_register_agent_echo_type() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create an agent
    let agent = Agent {
        id: "test-echo-agent".to_string(),
        name: "Echo Test Agent".to_string(),
        description: "Agent for echo type test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    // Register as echo type
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-echo-agent".to_string(),
        agent_type: "echo".to_string(),
        description: "Echo agent for testing".to_string(),
    });
    let response = client.register_agent(register_request).await.unwrap();
    let result = response.into_inner();
    
    assert!(result.success);
    assert!(result.error.is_none());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_register_agent_not_found() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Try to register an agent that doesn't exist
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "nonexistent-agent-register".to_string(),
        agent_type: "simple".to_string(),
        description: "Non-existent agent".to_string(),
    });
    let response = client.register_agent(register_request).await.unwrap();
    let result = response.into_inner();
    
    // Should return an error
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_register_agent_twice() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create an agent
    let agent = Agent {
        id: "test-double-register".to_string(),
        name: "Double Register Agent".to_string(),
        description: "Agent for double register test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    // Register first time
    let register_request1 = Request::new(RegisterAgentRequest {
        agent_id: "test-double-register".to_string(),
        agent_type: "simple".to_string(),
        description: "First registration".to_string(),
    });
    let response1 = client.register_agent(register_request1).await.unwrap();
    assert!(response1.into_inner().success);
    
    // Register second time (should handle gracefully - may succeed or fail depending on implementation)
    let register_request2 = Request::new(RegisterAgentRequest {
        agent_id: "test-double-register".to_string(),
        agent_type: "simple".to_string(),
        description: "Second registration".to_string(),
    });
    let response2 = client.register_agent(register_request2).await.unwrap();
    let result2 = response2.into_inner();
    // Implementation may allow re-registration or return an error
    assert!(result2.success || result2.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_agent_lifecycle_transitions() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create and register an agent
    let agent = Agent {
        id: "test-lifecycle-transitions".to_string(),
        name: "Lifecycle Test Agent".to_string(),
        description: "Agent for lifecycle transition test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-lifecycle-transitions".to_string(),
        agent_type: "simple".to_string(),
        description: "Test agent".to_string(),
    });
    client.register_agent(register_request).await.unwrap();
    
    // Start the agent
    let start_request = Request::new(StartAgentRequest {
        agent_id: "test-lifecycle-transitions".to_string(),
    });
    let start_response = client.start_agent(start_request).await.unwrap();
    assert!(start_response.into_inner().success);
    
    // Stop the agent
    let stop_request = Request::new(StopAgentRequest {
        agent_id: "test-lifecycle-transitions".to_string(),
    });
    let stop_response = client.stop_agent(stop_request).await.unwrap();
    assert!(stop_response.into_inner().success);
    
    // Try to stop again (should handle gracefully)
    let stop_request2 = Request::new(StopAgentRequest {
        agent_id: "test-lifecycle-transitions".to_string(),
    });
    let stop_response2 = client.stop_agent(stop_request2).await.unwrap();
    let result2 = stop_response2.into_inner();
    // May succeed or return an error if already stopped
    assert!(result2.success || result2.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_execute_agent_with_model_config() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create and register an agent
    let agent = Agent {
        id: "test-model-config".to_string(),
        name: "Model Config Test Agent".to_string(),
        description: "Agent for model config test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-model-config".to_string(),
        agent_type: "simple".to_string(),
        description: "Test agent".to_string(),
    });
    client.register_agent(register_request).await.unwrap();
    
    // Execute with model type and model ID
    let execute_request = Request::new(ExecuteAgentRequest {
        agent_id: "test-model-config".to_string(),
        input: "Test input with model config".to_string(),
        model_type: Some("mock".to_string()),
        model_id: Some("test-model-id".to_string()),
    });
    let response = client.execute_agent(execute_request).await.unwrap();
    let result = response.into_inner();
    
    // Should return a result (may succeed or fail depending on model availability)
    assert!(result.output.len() > 0 || result.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_multiple_registered_agents() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create and register multiple agents
    for i in 1..=3 {
        let agent = Agent {
            id: format!("test-multi-agent-{}", i),
            name: format!("Multi Agent {}", i),
            description: format!("Agent {} for multi test", i),
            config_json: "{}".to_string(),
            state: "\"idle\"".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        let agent_request = Request::new(CreateAgentRequest {
            agent: Some(agent),
        });
        client.create_agent(agent_request).await.unwrap();
        
        let register_request = Request::new(RegisterAgentRequest {
            agent_id: format!("test-multi-agent-{}", i),
            agent_type: "simple".to_string(),
            description: format!("Test agent {}", i),
        });
        client.register_agent(register_request).await.unwrap();
    }
    
    // Get all registered agents
    let list_request = Request::new(GetRegisteredAgentsRequest {});
    let response = client.get_registered_agents(list_request).await.unwrap();
    let agents = response.into_inner().agents;
    
    assert!(agents.len() >= 3);
    let ids: Vec<String> = agents.iter().map(|a| a.id.clone()).collect();
    assert!(ids.contains(&"test-multi-agent-1".to_string()));
    assert!(ids.contains(&"test-multi-agent-2".to_string()));
    assert!(ids.contains(&"test-multi-agent-3".to_string()));
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_start_agent_not_registered() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Try to start an agent that exists but isn't registered
    let agent = Agent {
        id: "test-not-registered-start".to_string(),
        name: "Not Registered Agent".to_string(),
        description: "Agent not registered".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    // Try to start without registering
    let start_request = Request::new(StartAgentRequest {
        agent_id: "test-not-registered-start".to_string(),
    });
    let response = client.start_agent(start_request).await.unwrap();
    let result = response.into_inner();
    
    // Should return an error
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_stop_agent_not_registered() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Try to stop an agent that exists but isn't registered
    let agent = Agent {
        id: "test-not-registered-stop".to_string(),
        name: "Not Registered Agent".to_string(),
        description: "Agent not registered".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    // Try to stop without registering
    let stop_request = Request::new(StopAgentRequest {
        agent_id: "test-not-registered-stop".to_string(),
    });
    let response = client.stop_agent(stop_request).await.unwrap();
    let result = response.into_inner();
    
    // Should return an error
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_execute_agent_empty_input() {
    let port = start_test_server().await;
    let client_manager = create_client_manager(port).await;
    
    let manager = client_manager.lock().await;
    let mut client = manager.get_client().await.unwrap();
    
    // Create and register an agent
    let agent = Agent {
        id: "test-empty-input".to_string(),
        name: "Empty Input Test Agent".to_string(),
        description: "Agent for empty input test".to_string(),
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    let agent_request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    client.create_agent(agent_request).await.unwrap();
    
    let register_request = Request::new(RegisterAgentRequest {
        agent_id: "test-empty-input".to_string(),
        agent_type: "simple".to_string(),
        description: "Test agent".to_string(),
    });
    client.register_agent(register_request).await.unwrap();
    
    // Execute with empty input
    let execute_request = Request::new(ExecuteAgentRequest {
        agent_id: "test-empty-input".to_string(),
        input: "".to_string(),
        model_type: None,
        model_id: None,
    });
    let response = client.execute_agent(execute_request).await.unwrap();
    let result = response.into_inner();
    
    // Should handle empty input (may succeed with empty output or return an error)
    // Just verify we got a response
    assert!(result.success || result.error.is_some());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test commands_test -- --ignored"]
async fn test_connection_error_handling() {
    // Use an invalid port to test connection error handling
    let invalid_port = 65535;
    let client_manager = create_client_manager(invalid_port).await;
    
    let manager = client_manager.lock().await;
    let result = manager.get_client().await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Failed to connect") || error.contains("connect"));
}
