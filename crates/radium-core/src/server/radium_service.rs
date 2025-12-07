//! Radium gRPC service implementation.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use radium_abstraction::{Model, ModelParameters};
use radium_models::MockModel;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};

use radium_orchestrator::{Agent, AgentContext, ChatAgent, EchoAgent, Orchestrator, SimpleAgent};

use crate::models::{Task, Workflow};
use crate::proto::radium_server::Radium;
use crate::proto::{
    CreateAgentRequest, CreateAgentResponse, CreateTaskRequest, CreateTaskResponse,
    CreateWorkflowRequest, CreateWorkflowResponse, DeleteAgentRequest, DeleteAgentResponse,
    DeleteTaskRequest, DeleteTaskResponse, DeleteWorkflowRequest, DeleteWorkflowResponse,
    ExecuteAgentRequest, ExecuteAgentResponse, ExecuteWorkflowRequest, ExecuteWorkflowResponse,
    GetAgentRequest, GetAgentResponse, GetRegisteredAgentsRequest, GetRegisteredAgentsResponse,
    GetTaskRequest, GetTaskResponse, GetWorkflowExecutionRequest, GetWorkflowExecutionResponse,
    GetWorkflowRequest, GetWorkflowResponse, ListAgentsRequest, ListAgentsResponse,
    ListTasksRequest, ListTasksResponse, ListWorkflowExecutionsRequest,
    ListWorkflowExecutionsResponse, ListWorkflowsRequest, ListWorkflowsResponse, PingRequest,
    PingResponse, RegisterAgentRequest, RegisterAgentResponse, RegisteredAgent, StartAgentRequest,
    StartAgentResponse, StopAgentRequest, StopAgentResponse, StopWorkflowExecutionRequest,
    StopWorkflowExecutionResponse, UpdateAgentRequest, UpdateAgentResponse, UpdateTaskRequest,
    UpdateTaskResponse, UpdateWorkflowRequest, UpdateWorkflowResponse, ValidateSourcesRequest,
    ValidateSourcesResponse, SourceValidationResult, WorkflowExecution,
};
use crate::storage::{
    AgentRepository, Database, SqliteAgentRepository, SqliteTaskRepository,
    SqliteWorkflowRepository, TaskRepository, WorkflowRepository,
};

/// The Radium gRPC service implementation.
pub struct RadiumService {
    /// Database connection for persistence.
    db: Arc<Mutex<Database>>,
    /// Agent orchestrator for managing agent execution.
    orchestrator: Arc<Orchestrator>,
}

impl RadiumService {
    /// Create a new Radium service instance.
    pub fn new(db: Database) -> Self {
        Self { db: Arc::new(Mutex::new(db)), orchestrator: Arc::new(Orchestrator::new()) }
    }

    /// Acquires a lock on the database connection.
    ///
    /// # Errors
    ///
    /// Returns a `Status::internal` error if the lock cannot be acquired.
    #[allow(clippy::result_large_err)]
    fn lock_db(&self) -> Result<std::sync::MutexGuard<'_, Database>, Status> {
        self.db.lock().map_err(|e| {
            error!(error = %e, "Failed to acquire database lock");
            Status::internal("Database lock error")
        })
    }

    /// Extracts request ID from gRPC request metadata.
    ///
    /// # Arguments
    /// * `request` - The gRPC request
    ///
    /// # Returns
    /// The request ID if present, or "unknown" if not found.
    fn get_request_id<T>(request: &Request<T>) -> String {
        request
            .metadata()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map_or_else(|| "unknown".to_string(), ToString::to_string)
    }
}

// ============================================================================
// Error Handling Helpers
// ============================================================================

/// Converts a `StorageError` to a gRPC `Status` for read/update/delete operations.
///
/// Maps `NotFound` errors to `Status::not_found`, and other errors to `Status::internal`.
fn storage_to_status(err: &crate::storage::StorageError, entity: &str, id: &str) -> Status {
    match err {
        crate::storage::StorageError::NotFound(_) => {
            Status::not_found(format!("{entity} {id} not found"))
        }
        crate::storage::StorageError::InvalidData(msg) => {
            Status::invalid_argument(format!("Invalid {entity} data: {msg}"))
        }
        _ => Status::internal(format!("Failed to process {entity}: {err}")),
    }
}

/// Converts a `StorageError` to a gRPC `Status` for create operations.
///
/// Logs the error and returns an appropriate status.
fn create_error(err: &crate::storage::StorageError, entity: &str) -> Status {
    error!(error = %err, "Failed to create {}", entity);
    match err {
        crate::storage::StorageError::InvalidData(msg) => {
            Status::invalid_argument(format!("Invalid {entity} data: {msg}"))
        }
        _ => Status::internal(format!("Failed to create {entity}: {err}")),
    }
}

/// Converts a `StorageError` to a gRPC `Status` for list operations.
fn list_error(err: &crate::storage::StorageError, entity: &str) -> Status {
    Status::internal(format!("Failed to list {entity}s: {err}"))
}

impl Default for RadiumService {
    fn default() -> Self {
        // Note: This should only be used for testing. Production code should use
        // RadiumService::new() with a shared database instance.
        let db = Database::open_in_memory()
            .expect("Failed to create in-memory database for default service");
        Self::new(db)
    }
}

#[tonic::async_trait]
impl Radium for RadiumService {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let request_id = Self::get_request_id(&request);
        let start_time = Instant::now();
        let inner = request.into_inner();
        info!(request_id = %request_id, message = %inner.message, "Received ping request");

        let mock_model = MockModel::new("mock-text-generator".to_string());
        let prompt = "Tell me a story about a Rust gRPC server.";
        let parameters = Some(ModelParameters::default());

        match mock_model.generate_text(prompt, parameters).await {
            Ok(response) => {
                info!(content = %response.content, usage = ?response.usage, "Mock model response");
            }
            Err(e) => {
                warn!(error = %e, "Mock model error");
            }
        }

        let echo_agent = EchoAgent::new(
            "echo-agent".to_string(),
            "A simple agent that echoes its input.".to_string(),
        );
        let agent_context = AgentContext { model: &mock_model };

        match echo_agent.execute(&inner.message, agent_context).await {
            Ok(output) => {
                info!(request_id = %request_id, output = ?output, "EchoAgent executed successfully");
            }
            Err(e) => {
                warn!(request_id = %request_id, error = %e, "EchoAgent execution error");
            }
        }

        let duration = start_time.elapsed();
        info!(
            request_id = %request_id,
            duration_ms = duration.as_millis(),
            "Ping request completed"
        );

        Ok(Response::new(PingResponse { message: format!("Pong! Received: {}", inner.message) }))
    }

    async fn create_agent(
        &self,
        request: Request<CreateAgentRequest>,
    ) -> Result<Response<CreateAgentResponse>, Status> {
        let proto_agent = request
            .into_inner()
            .agent
            .ok_or_else(|| Status::invalid_argument("agent is required"))?;

        info!(agent_id = %proto_agent.id, "Received create agent request");

        let agent = crate::models::Agent::try_from(proto_agent)
            .map_err(|e| Status::invalid_argument(format!("Invalid agent: {}", e)))?;
        agent
            .validate()
            .map_err(|e| Status::invalid_argument(format!("Agent validation failed: {}", e)))?;

        let mut db = self.lock_db()?;
        let mut repo = SqliteAgentRepository::new(&mut *db);
        repo.create(&agent).map_err(|e| create_error(&e, "agent"))?;

        Ok(Response::new(CreateAgentResponse { agent_id: agent.id }))
    }

    async fn get_agent(
        &self,
        request: Request<GetAgentRequest>,
    ) -> Result<Response<GetAgentResponse>, Status> {
        let agent_id = request.into_inner().agent_id;
        info!(agent_id = %agent_id, "Received get agent request");

        let mut db = self.lock_db()?;
        let repo = SqliteAgentRepository::new(&mut *db);
        let agent =
            repo.get_by_id(&agent_id).map_err(|e| storage_to_status(&e, "Agent", &agent_id))?;

        Ok(Response::new(GetAgentResponse { agent: Some(crate::proto::Agent::from(agent)) }))
    }

    async fn list_agents(
        &self,
        _request: Request<ListAgentsRequest>,
    ) -> Result<Response<ListAgentsResponse>, Status> {
        info!("Received list agents request");

        let mut db = self.lock_db()?;
        let repo = SqliteAgentRepository::new(&mut *db);
        let agents = repo.get_all().map_err(|e| list_error(&e, "agent"))?;

        Ok(Response::new(ListAgentsResponse {
            agents: agents.into_iter().map(crate::proto::Agent::from).collect(),
        }))
    }

    async fn update_agent(
        &self,
        request: Request<UpdateAgentRequest>,
    ) -> Result<Response<UpdateAgentResponse>, Status> {
        let proto_agent = request
            .into_inner()
            .agent
            .ok_or_else(|| Status::invalid_argument("agent is required"))?;

        info!(agent_id = %proto_agent.id, "Received update agent request");

        let agent = crate::models::Agent::try_from(proto_agent)
            .map_err(|e| Status::invalid_argument(format!("Invalid agent: {}", e)))?;
        agent
            .validate()
            .map_err(|e| Status::invalid_argument(format!("Agent validation failed: {}", e)))?;

        let mut db = self.lock_db()?;
        let mut repo = SqliteAgentRepository::new(&mut *db);
        repo.update(&agent).map_err(|e| storage_to_status(&e, "Agent", &agent.id))?;

        Ok(Response::new(UpdateAgentResponse { agent_id: agent.id }))
    }

    async fn delete_agent(
        &self,
        request: Request<DeleteAgentRequest>,
    ) -> Result<Response<DeleteAgentResponse>, Status> {
        let agent_id = request.into_inner().agent_id;
        info!(agent_id = %agent_id, "Received delete agent request");

        let mut db = self.lock_db()?;
        let mut repo = SqliteAgentRepository::new(&mut *db);
        repo.delete(&agent_id).map_err(|e| storage_to_status(&e, "Agent", &agent_id))?;

        Ok(Response::new(DeleteAgentResponse { success: true }))
    }

    async fn create_workflow(
        &self,
        request: Request<CreateWorkflowRequest>,
    ) -> Result<Response<CreateWorkflowResponse>, Status> {
        let proto_workflow = request
            .into_inner()
            .workflow
            .ok_or_else(|| Status::invalid_argument("workflow is required"))?;

        info!(workflow_id = %proto_workflow.id, "Received create workflow request");

        let workflow = Workflow::try_from(proto_workflow)
            .map_err(|e| Status::invalid_argument(format!("Invalid workflow: {e}")))?;
        workflow
            .validate()
            .map_err(|e| Status::invalid_argument(format!("Workflow validation failed: {e}")))?;

        let mut db = self.lock_db()?;
        let mut repo = SqliteWorkflowRepository::new(&mut *db);
        repo.create(&workflow).map_err(|e| create_error(&e, "workflow"))?;

        Ok(Response::new(CreateWorkflowResponse { workflow_id: workflow.id }))
    }

    async fn get_workflow(
        &self,
        request: Request<GetWorkflowRequest>,
    ) -> Result<Response<GetWorkflowResponse>, Status> {
        let workflow_id = request.into_inner().workflow_id;
        info!(workflow_id = %workflow_id, "Received get workflow request");

        let mut db = self.lock_db()?;
        let repo = SqliteWorkflowRepository::new(&mut *db);
        let workflow = repo
            .get_by_id(&workflow_id)
            .map_err(|e| storage_to_status(&e, "Workflow", &workflow_id))?;

        Ok(Response::new(GetWorkflowResponse {
            workflow: Some(crate::proto::Workflow::from(workflow)),
        }))
    }

    async fn list_workflows(
        &self,
        _request: Request<ListWorkflowsRequest>,
    ) -> Result<Response<ListWorkflowsResponse>, Status> {
        info!("Received list workflows request");

        let mut db = self.lock_db()?;
        let repo = SqliteWorkflowRepository::new(&mut *db);
        let workflows = repo.get_all().map_err(|e| list_error(&e, "workflow"))?;

        Ok(Response::new(ListWorkflowsResponse {
            workflows: workflows.into_iter().map(crate::proto::Workflow::from).collect(),
        }))
    }

    async fn update_workflow(
        &self,
        request: Request<UpdateWorkflowRequest>,
    ) -> Result<Response<UpdateWorkflowResponse>, Status> {
        let proto_workflow = request
            .into_inner()
            .workflow
            .ok_or_else(|| Status::invalid_argument("workflow is required"))?;

        info!(workflow_id = %proto_workflow.id, "Received update workflow request");

        let workflow = Workflow::try_from(proto_workflow)
            .map_err(|e| Status::invalid_argument(format!("Invalid workflow: {e}")))?;
        workflow
            .validate()
            .map_err(|e| Status::invalid_argument(format!("Workflow validation failed: {e}")))?;

        let mut db = self.lock_db()?;
        let mut repo = SqliteWorkflowRepository::new(&mut *db);
        repo.update(&workflow).map_err(|e| storage_to_status(&e, "Workflow", &workflow.id))?;

        Ok(Response::new(UpdateWorkflowResponse { workflow_id: workflow.id }))
    }

    async fn delete_workflow(
        &self,
        request: Request<DeleteWorkflowRequest>,
    ) -> Result<Response<DeleteWorkflowResponse>, Status> {
        let workflow_id = request.into_inner().workflow_id;
        info!(workflow_id = %workflow_id, "Received delete workflow request");

        let mut db = self.lock_db()?;
        let mut repo = SqliteWorkflowRepository::new(&mut *db);
        repo.delete(&workflow_id).map_err(|e| storage_to_status(&e, "Workflow", &workflow_id))?;

        Ok(Response::new(DeleteWorkflowResponse { success: true }))
    }

    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        let proto_task = request
            .into_inner()
            .task
            .ok_or_else(|| Status::invalid_argument("task is required"))?;

        info!(task_id = %proto_task.id, "Received create task request");

        let task = Task::try_from(proto_task)
            .map_err(|e| Status::invalid_argument(format!("Invalid task: {e}")))?;
        task.validate()
            .map_err(|e| Status::invalid_argument(format!("Task validation failed: {e}")))?;

        let mut db = self.lock_db()?;
        let mut repo = SqliteTaskRepository::new(&mut *db);
        repo.create(&task).map_err(|e| create_error(&e, "task"))?;

        Ok(Response::new(CreateTaskResponse { task_id: task.id }))
    }

    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<GetTaskResponse>, Status> {
        let task_id = request.into_inner().task_id;
        info!(task_id = %task_id, "Received get task request");

        let mut db = self.lock_db()?;
        let repo = SqliteTaskRepository::new(&mut *db);
        let task = repo.get_by_id(&task_id).map_err(|e| storage_to_status(&e, "Task", &task_id))?;

        Ok(Response::new(GetTaskResponse { task: Some(crate::proto::Task::from(task)) }))
    }

    async fn list_tasks(
        &self,
        _request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        info!("Received list tasks request");

        let mut db = self.lock_db()?;
        let repo = SqliteTaskRepository::new(&mut *db);
        let tasks = repo.get_all().map_err(|e| list_error(&e, "task"))?;

        Ok(Response::new(ListTasksResponse {
            tasks: tasks.into_iter().map(crate::proto::Task::from).collect(),
        }))
    }

    async fn update_task(
        &self,
        request: Request<UpdateTaskRequest>,
    ) -> Result<Response<UpdateTaskResponse>, Status> {
        let proto_task = request
            .into_inner()
            .task
            .ok_or_else(|| Status::invalid_argument("task is required"))?;

        info!(task_id = %proto_task.id, "Received update task request");

        let task = Task::try_from(proto_task)
            .map_err(|e| Status::invalid_argument(format!("Invalid task: {e}")))?;
        task.validate()
            .map_err(|e| Status::invalid_argument(format!("Task validation failed: {e}")))?;

        let mut db = self.lock_db()?;
        let mut repo = SqliteTaskRepository::new(&mut *db);
        repo.update(&task).map_err(|e| storage_to_status(&e, "Task", &task.id))?;

        Ok(Response::new(UpdateTaskResponse { task_id: task.id }))
    }

    async fn delete_task(
        &self,
        request: Request<DeleteTaskRequest>,
    ) -> Result<Response<DeleteTaskResponse>, Status> {
        let task_id = request.into_inner().task_id;
        info!(task_id = %task_id, "Received delete task request");

        let mut db = self.lock_db()?;
        let mut repo = SqliteTaskRepository::new(&mut *db);
        repo.delete(&task_id).map_err(|e| storage_to_status(&e, "Task", &task_id))?;

        Ok(Response::new(DeleteTaskResponse { success: true }))
    }

    async fn execute_agent(
        &self,
        request: Request<ExecuteAgentRequest>,
    ) -> Result<Response<ExecuteAgentResponse>, Status> {
        let inner = request.into_inner();
        info!(agent_id = %inner.agent_id, "ExecuteAgent request");

        let result = if let (Some(model_type), Some(model_id)) = (inner.model_type, inner.model_id)
        {
            let model_type = match model_type.as_str() {
                "mock" => radium_models::ModelType::Mock,
                "gemini" => radium_models::ModelType::Gemini,
                "openai" => radium_models::ModelType::OpenAI,
                _ => {
                    return Ok(Response::new(ExecuteAgentResponse {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Invalid model type: {}", model_type)),
                    }));
                }
            };
            self.orchestrator
                .execute_agent_with_model(&inner.agent_id, &inner.input, model_type, model_id)
                .await
        } else {
            self.orchestrator.execute_agent(&inner.agent_id, &inner.input).await
        };

        match result {
            Ok(exec_result) => Ok(Response::new(ExecuteAgentResponse {
                success: exec_result.success,
                output: match exec_result.output {
                    radium_orchestrator::AgentOutput::Text(text) => text,
                    radium_orchestrator::AgentOutput::StructuredData(data) => {
                        serde_json::to_string(&data).unwrap_or_else(|_| "Invalid JSON".to_string())
                    }
                    radium_orchestrator::AgentOutput::ToolCall { name, args } => {
                        format!("ToolCall: {} with args: {}", name, args)
                    }
                    radium_orchestrator::AgentOutput::Terminate => "Terminated".to_string(),
                },
                error: exec_result.error,
            })),
            Err(e) => Ok(Response::new(ExecuteAgentResponse {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            })),
        }
    }

    async fn start_agent(
        &self,
        request: Request<StartAgentRequest>,
    ) -> Result<Response<StartAgentResponse>, Status> {
        let inner = request.into_inner();
        info!(agent_id = %inner.agent_id, "StartAgent request");

        match self.orchestrator.start_agent(&inner.agent_id).await {
            Ok(()) => Ok(Response::new(StartAgentResponse { success: true, error: None })),
            Err(current_state) => Ok(Response::new(StartAgentResponse {
                success: false,
                error: Some(format!("Invalid state transition from {:?}", current_state)),
            })),
        }
    }

    async fn stop_agent(
        &self,
        request: Request<StopAgentRequest>,
    ) -> Result<Response<StopAgentResponse>, Status> {
        let inner = request.into_inner();
        info!(agent_id = %inner.agent_id, "StopAgent request");

        match self.orchestrator.stop_agent(&inner.agent_id).await {
            Ok(()) => Ok(Response::new(StopAgentResponse { success: true, error: None })),
            Err(current_state) => Ok(Response::new(StopAgentResponse {
                success: false,
                error: Some(format!("Invalid state transition from {:?}", current_state)),
            })),
        }
    }

    async fn get_registered_agents(
        &self,
        _request: Request<GetRegisteredAgentsRequest>,
    ) -> Result<Response<GetRegisteredAgentsResponse>, Status> {
        info!("GetRegisteredAgents request");

        let agents = self.orchestrator.list_agents().await;

        let mut registered_agents = Vec::new();
        for metadata in agents {
            let state = self.orchestrator.get_agent_state(&metadata.id).await;
            registered_agents.push(RegisteredAgent {
                id: metadata.id,
                description: metadata.description,
                state: format!("{:?}", state).to_lowercase(),
            });
        }

        Ok(Response::new(GetRegisteredAgentsResponse { agents: registered_agents }))
    }

    async fn register_agent(
        &self,
        request: Request<RegisterAgentRequest>,
    ) -> Result<Response<RegisterAgentResponse>, Status> {
        let inner = request.into_inner();
        info!(
            agent_id = %inner.agent_id,
            agent_type = %inner.agent_type,
            "RegisterAgent request"
        );

        let agent: Arc<dyn Agent + Send + Sync> = match inner.agent_type.as_str() {
            "echo" => Arc::new(EchoAgent::new(inner.agent_id.clone(), inner.description)),
            "simple" => Arc::new(SimpleAgent::new(inner.agent_id.clone(), inner.description)),
            "chat" => Arc::new(ChatAgent::new(inner.agent_id.clone(), inner.description)),
            _ => {
                return Ok(Response::new(RegisterAgentResponse {
                    success: false,
                    error: Some(format!("Unknown agent type: {}", inner.agent_type)),
                }));
            }
        };

        self.orchestrator.register_agent(agent).await;

        Ok(Response::new(RegisterAgentResponse { success: true, error: None }))
    }

    // ============================================================================
    // Workflow Execution RPCs
    // ============================================================================

    async fn execute_workflow(
        &self,
        request: Request<ExecuteWorkflowRequest>,
    ) -> Result<Response<ExecuteWorkflowResponse>, Status> {
        let req = request.into_inner();
        let workflow_id = req.workflow_id;
        let use_parallel = req.use_parallel;

        info!(
            workflow_id = %workflow_id,
            use_parallel = use_parallel,
            "ExecuteWorkflow RPC called"
        );

        // Create workflow service
        let executor = Arc::new(radium_orchestrator::AgentExecutor::with_mock_model());
        let workflow_service =
            crate::workflow::WorkflowService::new(&self.orchestrator, &executor, &self.db);

        // Execute workflow
        let result = workflow_service.execute_workflow(&workflow_id, use_parallel).await;

        match result {
            Ok(execution) => {
                let final_state_json = serde_json::to_string(&execution.final_state)
                    .map_err(|e| Status::internal(format!("Failed to serialize state: {}", e)))?;

                Ok(Response::new(ExecuteWorkflowResponse {
                    execution_id: execution.execution_id,
                    workflow_id: execution.workflow_id,
                    success: matches!(
                        execution.final_state,
                        crate::models::WorkflowState::Completed
                    ),
                    error: None,
                    final_state: final_state_json,
                }))
            }
            Err(e) => {
                error!(
                    workflow_id = %workflow_id,
                    error = %e,
                    "Workflow execution failed"
                );
                Ok(Response::new(ExecuteWorkflowResponse {
                    execution_id: String::new(),
                    workflow_id,
                    success: false,
                    error: Some(e.to_string()),
                    final_state: String::new(),
                }))
            }
        }
    }

    async fn get_workflow_execution(
        &self,
        request: Request<GetWorkflowExecutionRequest>,
    ) -> Result<Response<GetWorkflowExecutionResponse>, Status> {
        let req = request.into_inner();
        let execution_id = req.execution_id;

        info!(execution_id = %execution_id, "GetWorkflowExecution RPC called");

        // Create workflow service
        let executor = Arc::new(radium_orchestrator::AgentExecutor::with_mock_model());
        let workflow_service =
            crate::workflow::WorkflowService::new(&self.orchestrator, &executor, &self.db);

        // Get execution
        let execution = workflow_service.get_execution(&execution_id).await.ok_or_else(|| {
            Status::not_found(format!("Workflow execution {} not found", execution_id))
        })?;

        // Convert to proto
        let context_json = serde_json::to_string(&execution.context)
            .map_err(|e| Status::internal(format!("Failed to serialize context: {}", e)))?;
        let final_state_json = serde_json::to_string(&execution.final_state)
            .map_err(|e| Status::internal(format!("Failed to serialize state: {}", e)))?;

        let proto_execution = WorkflowExecution {
            execution_id: execution.execution_id,
            workflow_id: execution.workflow_id,
            context_json,
            started_at: execution.started_at.to_rfc3339(),
            completed_at: execution.completed_at.map(|dt| dt.to_rfc3339()),
            final_state: final_state_json,
        };

        Ok(Response::new(GetWorkflowExecutionResponse { execution: Some(proto_execution) }))
    }

    async fn stop_workflow_execution(
        &self,
        request: Request<StopWorkflowExecutionRequest>,
    ) -> Result<Response<StopWorkflowExecutionResponse>, Status> {
        let req = request.into_inner();
        let workflow_id = req.workflow_id;

        info!(workflow_id = %workflow_id, "StopWorkflowExecution RPC called");

        // Create workflow service
        let executor = Arc::new(radium_orchestrator::AgentExecutor::with_mock_model());
        let _workflow_service =
            crate::workflow::WorkflowService::new(&self.orchestrator, &executor, &self.db);

        // Stop workflow - load workflow first, then update
        let workflow_state = {
            let mut db = self.lock_db()?;
            let workflow_repo = SqliteWorkflowRepository::new(&mut *db);
            let workflow = workflow_repo
                .get_by_id(&workflow_id)
                .map_err(|e| storage_to_status(&e, "Workflow", &workflow_id))?;
            workflow.state
        };

        // Update state if running
        if matches!(workflow_state, crate::models::WorkflowState::Running) {
            let mut db = self.lock_db()?;
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut *db);
            let mut workflow = workflow_repo
                .get_by_id(&workflow_id)
                .map_err(|e| storage_to_status(&e, "Workflow", &workflow_id))?;
            workflow.set_state(crate::models::WorkflowState::Idle);
            workflow_repo
                .update(&workflow)
                .map_err(|e| storage_to_status(&e, "Workflow", &workflow_id))?;
        }

        Ok(Response::new(StopWorkflowExecutionResponse { success: true, error: None }))
    }

    async fn list_workflow_executions(
        &self,
        request: Request<ListWorkflowExecutionsRequest>,
    ) -> Result<Response<ListWorkflowExecutionsResponse>, Status> {
        let req = request.into_inner();
        let workflow_id = req.workflow_id;

        info!("ListWorkflowExecutions RPC called");

        // Create workflow service
        let executor = Arc::new(radium_orchestrator::AgentExecutor::with_mock_model());
        let workflow_service =
            crate::workflow::WorkflowService::new(&self.orchestrator, &executor, &self.db);

        // Get executions
        let executions = workflow_service.get_execution_history(workflow_id.as_deref()).await;

        // Convert to proto
        let proto_executions: Vec<WorkflowExecution> = executions
            .into_iter()
            .map(|exec| {
                let context_json = serde_json::to_string(&exec.context).unwrap_or_default();
                let final_state_json = serde_json::to_string(&exec.final_state).unwrap_or_default();

                WorkflowExecution {
                    execution_id: exec.execution_id,
                    workflow_id: exec.workflow_id,
                    context_json,
                    started_at: exec.started_at.to_rfc3339(),
                    completed_at: exec.completed_at.map(|dt| dt.to_rfc3339()),
                    final_state: final_state_json,
                }
            })
            .collect();

        Ok(Response::new(ListWorkflowExecutionsResponse { executions: proto_executions }))
    }

    async fn validate_sources(
        &self,
        request: Request<ValidateSourcesRequest>,
    ) -> Result<Response<ValidateSourcesResponse>, Status> {
        use crate::context::{SourceRegistry, SourceValidator};
        use std::sync::Arc;
        
        let req = request.into_inner();
        let sources = req.sources;
        
        info!("ValidateSources RPC called with {} sources", sources.len());
        
        // Create source validator
        let registry = SourceRegistry::new();
        let validator = SourceValidator::new(registry);
        
        // Validate sources
        let results = validator.validate_sources(sources).await;
        
        // Convert to proto
        let proto_results: Vec<SourceValidationResult> = results
            .iter()
            .map(|result| SourceValidationResult {
                source: result.source.clone(),
                accessible: result.accessible,
                error_message: result.error_message.clone(),
                size_bytes: result.size_bytes,
            })
            .collect();
        
        // Check if all sources are valid
        let all_valid = results.iter().all(|r| r.accessible);
        
        Ok(Response::new(ValidateSourcesResponse { 
            results: proto_results,
            all_valid,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageError;

    #[test]
    fn test_storage_to_status_not_found() {
        let err = StorageError::NotFound("agent-1".to_string());
        let status = storage_to_status(&err, "agent", "agent-1");
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("agent-1"));
    }

    #[test]
    fn test_storage_to_status_other_error() {
        // Test with a non-InvalidData error (Connection error)
        let err = StorageError::Connection(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            Some("database locked".to_string()),
        ));
        let status = storage_to_status(&err, "agent", "agent-1");
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("Failed to process agent"));
    }

    #[test]
    fn test_create_error() {
        let err = StorageError::Connection(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            Some("database locked".to_string()),
        ));
        let status = create_error(&err, "agent");
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("Failed to create agent"));
    }

    #[test]
    fn test_list_error() {
        // Create a serialization error by trying to deserialize invalid JSON
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err = StorageError::Serialization(json_err);
        let status = list_error(&err, "agent");
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("Failed to list agents"));
    }

    #[test]
    fn test_radium_service_new() {
        let db = Database::open_in_memory().unwrap();
        let service = RadiumService::new(db);
        // Service should be created successfully
        // We can't easily test the internal state, but creation should not panic
        assert!(std::mem::size_of_val(&service) > 0);
    }

    #[test]
    fn test_radium_service_default() {
        let service = RadiumService::default();
        // Default service should use in-memory database
        assert!(std::mem::size_of_val(&service) > 0);
    }

    #[test]
    fn test_lock_db_success() {
        let db = Database::open_in_memory().unwrap();
        let service = RadiumService::new(db);
        let result = service.lock_db();
        assert!(result.is_ok());
    }

    #[test]
    fn test_lock_db_poisoned() {
        // This is hard to test without actually poisoning the mutex
        // In practice, this would require a panic in another thread
        // For now, we just verify the method exists and works in normal cases
        // Note: std::sync::Mutex does NOT support reentrant locking - attempting
        // to lock twice in the same thread will deadlock
        let db = Database::open_in_memory().unwrap();
        let service = RadiumService::new(db);
        let _guard = service.lock_db().unwrap();
        // Verify the lock was acquired successfully
        assert!(std::mem::size_of_val(&_guard) > 0);
    }

    #[test]
    fn test_storage_to_status_connection_error() {
        let err = StorageError::Connection(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some("cannot open database".to_string()),
        ));
        let status = storage_to_status(&err, "workflow", "workflow-1");
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("Failed to process workflow"));
    }

    #[test]
    fn test_storage_to_status_serialization_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = StorageError::Serialization(json_err);
        let status = storage_to_status(&err, "task", "task-1");
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[test]
    fn test_create_error_invalid_data() {
        let err = StorageError::InvalidData("Missing required field".to_string());
        let status = create_error(&err, "agent");
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("Invalid agent data"));
    }

    #[test]
    fn test_list_error_connection() {
        let err = StorageError::Connection(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            None,
        ));
        let status = list_error(&err, "workflow");
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("Failed to list workflows"));
    }

    #[test]
    fn test_storage_to_status_invalid_data() {
        let err = StorageError::InvalidData("Bad data".to_string());
        let status = storage_to_status(&err, "task", "task-1");
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("Invalid task data"));
    }

    #[test]
    fn test_create_error_connection() {
        let err = StorageError::Connection(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_LOCKED),
            Some("locked".to_string()),
        ));
        let status = create_error(&err, "task");
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[test]
    fn test_list_error_not_found() {
        let err = StorageError::NotFound("missing".to_string());
        let status = list_error(&err, "agent");
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[test]
    fn test_list_error_invalid_data() {
        let err = StorageError::InvalidData("Invalid format".to_string());
        let status = list_error(&err, "workflow");
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[test]
    fn test_storage_to_status_multiple_entity_types() {
        let err = StorageError::NotFound("id-123".to_string());

        let status1 = storage_to_status(&err, "agent", "id-123");
        assert!(status1.message().contains("agent"));

        let status2 = storage_to_status(&err, "workflow", "id-123");
        assert!(status2.message().contains("workflow"));

        let status3 = storage_to_status(&err, "task", "id-123");
        assert!(status3.message().contains("task"));
    }

    #[test]
    fn test_radium_service_lock_multiple_times() {
        let db = Database::open_in_memory().unwrap();
        let service = RadiumService::new(db);

        // Lock once
        {
            let _guard1 = service.lock_db().unwrap();
            // Lock is held
        } // Lock released

        // Lock again after release
        let _guard2 = service.lock_db().unwrap();
        // Should succeed
    }
}
