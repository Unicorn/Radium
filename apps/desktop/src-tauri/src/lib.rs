//! Radium Desktop - Tauri Application Library
//!
//! This module contains the Tauri commands and application logic.

pub mod client;

use client::ClientManager;
use radium_core::config::Config;
use radium_core::server::manager::EmbeddedServer;
use radium_core::workflow::{CompletionEvent, CompletionOptions, CompletionService};
use radium_core::Workspace;
use radium_core::proto::{
    Agent, CreateAgentRequest, CreateTaskRequest, CreateWorkflowRequest, DeleteAgentRequest,
    DeleteWorkflowRequest, ExecuteAgentRequest, ExecuteWorkflowRequest, GetAgentRequest, GetTaskRequest,
    GetWorkflowRequest, GetRegisteredAgentsRequest, ListAgentsRequest, ListTasksRequest, ListWorkflowsRequest, PingRequest,
    PingResponse, RegisterAgentRequest, RegisteredAgent, StartAgentRequest, StopAgentRequest, Task, UpdateAgentRequest, UpdateWorkflowRequest, Workflow, WorkflowStep,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use tonic::Request;
use tracing::{error, info};

/// JSON-serializable agent representation
#[derive(Debug, Serialize, Deserialize)]
struct AgentJson {
    id: String,
    name: String,
    description: String,
    config_json: String,
    state: String,
    created_at: String,
    updated_at: String,
}

impl From<Agent> for AgentJson {
    fn from(agent: Agent) -> Self {
        AgentJson {
            id: agent.id,
            name: agent.name,
            description: agent.description,
            config_json: agent.config_json,
            state: agent.state,
            created_at: agent.created_at,
            updated_at: agent.updated_at,
        }
    }
}

/// JSON-serializable workflow step representation
#[derive(Debug, Serialize, Deserialize)]
struct WorkflowStepJson {
    id: String,
    name: String,
    description: String,
    task_id: String,
    config_json: String,
    order: i32,
}

impl From<WorkflowStep> for WorkflowStepJson {
    fn from(step: WorkflowStep) -> Self {
        WorkflowStepJson {
            id: step.id,
            name: step.name,
            description: step.description,
            task_id: step.task_id,
            config_json: step.config_json,
            order: step.order,
        }
    }
}

/// JSON-serializable workflow representation
#[derive(Debug, Serialize, Deserialize)]
struct WorkflowJson {
    id: String,
    name: String,
    description: String,
    steps: Vec<WorkflowStepJson>,
    state: String,
    created_at: String,
    updated_at: String,
}

impl From<Workflow> for WorkflowJson {
    fn from(workflow: Workflow) -> Self {
        WorkflowJson {
            id: workflow.id,
            name: workflow.name,
            description: workflow.description,
            steps: workflow.steps.into_iter().map(WorkflowStepJson::from).collect(),
            state: workflow.state,
            created_at: workflow.created_at,
            updated_at: workflow.updated_at,
        }
    }
}

/// JSON-serializable task representation
#[derive(Debug, Serialize, Deserialize)]
struct TaskJson {
    id: String,
    name: String,
    description: String,
    agent_id: String,
    input_json: String,
    state: String,
    result_json: String,
    created_at: String,
    updated_at: String,
}

impl From<Task> for TaskJson {
    fn from(task: Task) -> Self {
        TaskJson {
            id: task.id,
            name: task.name,
            description: task.description,
            agent_id: task.agent_id,
            input_json: task.input_json,
            state: task.state,
            result_json: task.result_json,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }
}

/// JSON-serializable registered agent representation
#[derive(Debug, Serialize, Deserialize)]
struct RegisteredAgentJson {
    id: String,
    description: String,
    state: String,
}

impl From<RegisteredAgent> for RegisteredAgentJson {
    fn from(agent: RegisteredAgent) -> Self {
        RegisteredAgentJson {
            id: agent.id,
            description: agent.description,
            state: agent.state,
        }
    }
}

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub client_manager: Arc<Mutex<ClientManager>>,
    /// Embedded server instance (if running)
    pub embedded_server: Arc<Mutex<Option<EmbeddedServer>>>,
}

/// Ping the Radium server and return the response.
///
/// This connects to the gRPC server and calls the Ping RPC.
#[tauri::command]
async fn ping_server(message: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    info!(message = %message, "Ping command received");
    
    // Get client from manager
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    // Call Ping RPC
    let request = Request::new(PingRequest {
        message: message.clone(),
    });
    
    let response = client
        .ping(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let ping_response: PingResponse = response.into_inner();
    
    info!(response = %ping_response.message, "Ping response received");
    
    Ok(format!(
        "Server Response: {}\n\nMessage sent: \"{}\"",
        ping_response.message, message
    ))
}

/// Internal helper function for creating an agent
async fn create_agent_internal(
    id: String,
    name: String,
    description: String,
    client_manager: &Arc<Mutex<ClientManager>>,
) -> Result<String, String> {
    info!(agent_id = %id, "Create agent command received");
    
    let client_manager = client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let agent = Agent {
        id: id.clone(),
        name,
        description,
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    let request = Request::new(CreateAgentRequest {
        agent: Some(agent),
    });
    
    let response = client
        .create_agent(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let agent_id = response.into_inner().agent_id;
    info!(agent_id = %agent_id, "Agent created");
    
    Ok(serde_json::json!({ "agent_id": agent_id }).to_string())
}

/// Create a new agent
#[tauri::command]
async fn create_agent(
    id: String,
    name: String,
    description: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    create_agent_internal(id, name, description, &state.client_manager).await
}

/// List all agents
#[tauri::command]
async fn list_agents(state: tauri::State<'_, AppState>) -> Result<String, String> {
    info!("List agents command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(ListAgentsRequest {});
    let response = client
        .list_agents(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let agents = response.into_inner().agents;
    info!(count = agents.len(), "Agents retrieved");
    
    let agents_json: Vec<AgentJson> = agents.into_iter().map(AgentJson::from).collect();
    Ok(serde_json::to_string(&agents_json).map_err(|e| format!("JSON serialization failed: {}", e))?)
}

/// Get agent details
#[tauri::command]
async fn get_agent(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(agent_id = %agent_id, "Get agent command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(GetAgentRequest {
        agent_id: agent_id.clone(),
    });
    
    let response = client
        .get_agent(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let agent = response.into_inner().agent.ok_or_else(|| {
        format!("Agent not found: {}", agent_id)
    })?;
    
    info!(agent_id = %agent.id, "Agent retrieved");
    let agent_json = AgentJson::from(agent);
    Ok(serde_json::to_string(&agent_json).map_err(|e| format!("JSON serialization failed: {}", e))?)
}

/// Update an agent
#[tauri::command]
async fn update_agent(
    id: String,
    name: Option<String>,
    description: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(agent_id = %id, "Update agent command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    // First get the current agent
    let get_request = Request::new(GetAgentRequest {
        agent_id: id.clone(),
    });
    let get_response = client
        .get_agent(get_request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let mut agent = get_response
        .into_inner()
        .agent
        .ok_or_else(|| format!("Agent not found: {}", id))?;
    
    // Update fields if provided
    if let Some(new_name) = name {
        agent.name = new_name;
    }
    if let Some(new_description) = description {
        agent.description = new_description;
    }
    
    agent.updated_at = chrono::Utc::now().to_rfc3339();
    
    let request = Request::new(UpdateAgentRequest {
        agent: Some(agent),
    });
    
    let response = client
        .update_agent(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let agent_id = response.into_inner().agent_id;
    info!(agent_id = %agent_id, "Agent updated");
    
    Ok(serde_json::json!({ "agent_id": agent_id }).to_string())
}

/// Delete an agent
#[tauri::command]
async fn delete_agent(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(agent_id = %agent_id, "Delete agent command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(DeleteAgentRequest {
        agent_id: agent_id.clone(),
    });
    
    let response = client
        .delete_agent(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let success = response.into_inner().success;
    if success {
        info!(agent_id = %agent_id, "Agent deleted");
        Ok(serde_json::json!({ "success": true, "agent_id": agent_id }).to_string())
    } else {
        Err(format!("Failed to delete agent: {}", agent_id))
    }
}

/// Create a new workflow
#[tauri::command]
async fn create_workflow(
    id: String,
    name: String,
    description: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(workflow_id = %id, "Create workflow command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let workflow = Workflow {
        id: id.clone(),
        name,
        description,
        steps: vec![],
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    let request = Request::new(CreateWorkflowRequest {
        workflow: Some(workflow),
    });
    
    let response = client
        .create_workflow(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let workflow_id = response.into_inner().workflow_id;
    info!(workflow_id = %workflow_id, "Workflow created");
    
    Ok(serde_json::json!({ "workflow_id": workflow_id }).to_string())
}

/// List all workflows
#[tauri::command]
async fn list_workflows(state: tauri::State<'_, AppState>) -> Result<String, String> {
    info!("List workflows command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(ListWorkflowsRequest {});
    let response = client
        .list_workflows(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let workflows = response.into_inner().workflows;
    info!(count = workflows.len(), "Workflows retrieved");
    
    let workflows_json: Vec<WorkflowJson> = workflows.into_iter().map(WorkflowJson::from).collect();
    Ok(serde_json::to_string(&workflows_json).map_err(|e| format!("JSON serialization failed: {}", e))?)
}

/// Get workflow details
#[tauri::command]
async fn get_workflow(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(workflow_id = %workflow_id, "Get workflow command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(GetWorkflowRequest {
        workflow_id: workflow_id.clone(),
    });
    
    let response = client
        .get_workflow(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let workflow = response.into_inner().workflow.ok_or_else(|| {
        format!("Workflow not found: {}", workflow_id)
    })?;
    
    info!(workflow_id = %workflow.id, "Workflow retrieved");
    let workflow_json = WorkflowJson::from(workflow);
    Ok(serde_json::to_string(&workflow_json).map_err(|e| format!("JSON serialization failed: {}", e))?)
}

/// Update a workflow
#[tauri::command]
async fn update_workflow(
    id: String,
    name: Option<String>,
    description: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(workflow_id = %id, "Update workflow command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    // First get the current workflow
    let get_request = Request::new(GetWorkflowRequest {
        workflow_id: id.clone(),
    });
    let get_response = client
        .get_workflow(get_request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let mut workflow = get_response
        .into_inner()
        .workflow
        .ok_or_else(|| format!("Workflow not found: {}", id))?;
    
    // Update fields if provided
    if let Some(new_name) = name {
        workflow.name = new_name;
    }
    if let Some(new_description) = description {
        workflow.description = new_description;
    }
    
    workflow.updated_at = chrono::Utc::now().to_rfc3339();
    
    let request = Request::new(UpdateWorkflowRequest {
        workflow: Some(workflow),
    });
    
    let response = client
        .update_workflow(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let workflow_id = response.into_inner().workflow_id;
    info!(workflow_id = %workflow_id, "Workflow updated");
    
    Ok(serde_json::json!({ "workflow_id": workflow_id }).to_string())
}

/// Delete a workflow
#[tauri::command]
async fn delete_workflow(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(workflow_id = %workflow_id, "Delete workflow command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(DeleteWorkflowRequest {
        workflow_id: workflow_id.clone(),
    });
    
    let response = client
        .delete_workflow(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let success = response.into_inner().success;
    if success {
        info!(workflow_id = %workflow_id, "Workflow deleted");
        Ok(serde_json::json!({ "success": true, "workflow_id": workflow_id }).to_string())
    } else {
        Err(format!("Failed to delete workflow: {}", workflow_id))
    }
}

/// Execute a workflow
#[tauri::command]
async fn execute_workflow(
    workflow_id: String,
    use_parallel: bool,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(workflow_id = %workflow_id, parallel = use_parallel, "Execute workflow command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(ExecuteWorkflowRequest {
        workflow_id: workflow_id.clone(),
        use_parallel,
    });
    
    let response = client
        .execute_workflow(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let result = response.into_inner();
    info!(workflow_id = %workflow_id, success = result.success, "Workflow execution completed");
    
    Ok(serde_json::json!({
        "execution_id": result.execution_id,
        "workflow_id": result.workflow_id,
        "success": result.success,
        "error": result.error,
        "final_state": result.final_state
    }).to_string())
}

/// Create a new task
#[tauri::command]
async fn create_task(
    id: String,
    name: String,
    description: String,
    agent_id: String,
    input_json: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(task_id = %id, "Create task command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let task = Task {
        id: id.clone(),
        name,
        description,
        agent_id,
        input_json: input_json.unwrap_or_else(|| "{}".to_string()),
        state: "\"pending\"".to_string(),
        result_json: "{}".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    let request = Request::new(CreateTaskRequest {
        task: Some(task),
    });
    
    let response = client
        .create_task(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let task_id = response.into_inner().task_id;
    info!(task_id = %task_id, "Task created");
    
    Ok(serde_json::json!({ "task_id": task_id }).to_string())
}

/// List all tasks
#[tauri::command]
async fn list_tasks(state: tauri::State<'_, AppState>) -> Result<String, String> {
    info!("List tasks command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(ListTasksRequest {});
    let response = client
        .list_tasks(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let tasks = response.into_inner().tasks;
    info!(count = tasks.len(), "Tasks retrieved");
    
    let tasks_json: Vec<TaskJson> = tasks.into_iter().map(TaskJson::from).collect();
    Ok(serde_json::to_string(&tasks_json).map_err(|e| format!("JSON serialization failed: {}", e))?)
}

/// Get task details
#[tauri::command]
async fn get_task(
    task_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(task_id = %task_id, "Get task command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(GetTaskRequest {
        task_id: task_id.clone(),
    });
    
    let response = client
        .get_task(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let task = response.into_inner().task.ok_or_else(|| {
        format!("Task not found: {}", task_id)
    })?;
    
    info!(task_id = %task.id, "Task retrieved");
    let task_json = TaskJson::from(task);
    Ok(serde_json::to_string(&task_json).map_err(|e| format!("JSON serialization failed: {}", e))?)
}

/// Register an agent with the orchestrator
#[tauri::command]
async fn register_agent(
    agent_id: String,
    agent_type: String,
    description: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(agent_id = %agent_id, agent_type = %agent_type, "Register agent command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(RegisterAgentRequest {
        agent_id: agent_id.clone(),
        agent_type,
        description,
    });
    
    let response = client
        .register_agent(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let result = response.into_inner();
    if result.success {
        info!(agent_id = %agent_id, "Agent registered");
        Ok(serde_json::json!({ "success": true, "agent_id": agent_id }).to_string())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

/// Execute an agent
#[tauri::command]
async fn execute_agent(
    agent_id: String,
    input: String,
    model_type: Option<String>,
    model_id: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(agent_id = %agent_id, "Execute agent command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(ExecuteAgentRequest {
        agent_id: agent_id.clone(),
        input,
        model_type,
        model_id,
    });
    
    let response = client
        .execute_agent(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let result = response.into_inner();
    info!(agent_id = %agent_id, success = result.success, "Agent execution completed");
    
    Ok(serde_json::json!({
        "success": result.success,
        "output": result.output,
        "error": result.error
    }).to_string())
}

/// Start an agent lifecycle
#[tauri::command]
async fn start_agent(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(agent_id = %agent_id, "Start agent command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(StartAgentRequest {
        agent_id: agent_id.clone(),
    });
    
    let response = client
        .start_agent(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let result = response.into_inner();
    if result.success {
        info!(agent_id = %agent_id, "Agent started");
        Ok(serde_json::json!({ "success": true, "agent_id": agent_id }).to_string())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

/// Stop an agent lifecycle
#[tauri::command]
async fn stop_agent(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!(agent_id = %agent_id, "Stop agent command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(StopAgentRequest {
        agent_id: agent_id.clone(),
    });
    
    let response = client
        .stop_agent(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let result = response.into_inner();
    if result.success {
        info!(agent_id = %agent_id, "Agent stopped");
        Ok(serde_json::json!({ "success": true, "agent_id": agent_id }).to_string())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

/// Complete a requirement from source
#[tauri::command]
async fn complete_task(
    source: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    info!(source = %source, "Complete task command received");

    // Discover workspace
    let workspace = Workspace::discover()
        .map_err(|e| format!("Failed to discover workspace: {}", e))?;
    workspace
        .ensure_structure()
        .map_err(|e| format!("Failed to ensure workspace structure: {}", e))?;

    // Create completion service
    let service = CompletionService::new();

    // Create options
    let options = CompletionOptions {
        workspace_path: workspace.root().to_path_buf(),
        engine: std::env::var("RADIUM_ENGINE").unwrap_or_else(|_| "mock".to_string()),
        model_id: std::env::var("RADIUM_MODEL").ok(),
        requirement_id: None,
    };

    // Execute workflow
    let mut event_rx = service
        .execute(source.clone(), options)
        .await
        .map_err(|e| format!("Failed to start completion workflow: {}", e))?;

    // Process events and emit to frontend
    let mut last_event: Option<String> = None;
    while let Some(event) = event_rx.recv().await {
        let event_json = serde_json::to_string(&event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;

        // Emit event to frontend
        app_handle
            .emit("complete-progress", event_json.clone())
            .map_err(|e| format!("Failed to emit event: {}", e))?;

        last_event = Some(event_json);

        // Break on completion or error
        match event {
            CompletionEvent::Completed | CompletionEvent::Error { .. } => break,
            _ => {}
        }
    }

    Ok(last_event.unwrap_or_else(|| "{\"type\":\"unknown\"}".to_string()))
}

/// Get all registered agents
#[tauri::command]
async fn get_registered_agents(state: tauri::State<'_, AppState>) -> Result<String, String> {
    info!("Get registered agents command received");
    
    let client_manager = state.client_manager.lock().await;
    let mut client = client_manager
        .get_client()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;
    
    let request = Request::new(GetRegisteredAgentsRequest {});
    let response = client
        .get_registered_agents(request)
        .await
        .map_err(|e| format!("gRPC call failed: {}", e))?;
    
    let agents = response.into_inner().agents;
    info!(count = agents.len(), "Registered agents retrieved");
    
    let agents_json: Vec<RegisteredAgentJson> = agents.into_iter().map(RegisteredAgentJson::from).collect();
    Ok(serde_json::to_string(&agents_json).map_err(|e| format!("JSON serialization failed: {}", e))?)
}

/// Run the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            client_manager: Arc::new(Mutex::new(ClientManager::new())),
            embedded_server: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            ping_server,
            create_agent,
            list_agents,
            get_agent,
            update_agent,
            delete_agent,
            create_workflow,
            list_workflows,
            get_workflow,
            update_workflow,
            delete_workflow,
            execute_workflow,
            create_task,
            list_tasks,
            get_task,
            register_agent,
            execute_agent,
            start_agent,
            stop_agent,
            get_registered_agents,
            complete_task
        ])
        .setup(|app| {
            info!("Radium Desktop starting up");

            // Start embedded server in background
            let state = app.state::<AppState>();
            let server_state = state.embedded_server.clone();
            
            tokio::spawn(async move {
                let config = Config::default();
                let mut server = EmbeddedServer::new(config);
                
                match server.start().await {
                    Ok(()) => {
                        info!("Embedded server started, waiting for readiness...");
                        match server.wait_for_ready(std::time::Duration::from_secs(10)).await {
                            Ok(()) => {
                                info!("Embedded server is ready");
                                
                                // Store server in app state for later cleanup
                                let mut server_guard = server_state.lock().await;
                                *server_guard = Some(server);
                            }
                            Err(e) => {
                                error!(error = %e, "Failed to wait for server readiness");
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to start embedded server");
                    }
                }
            });
            

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                } else {
                    info!("Main window not found, skipping devtools");
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

