/**
 * Workflow-related type definitions matching the Rust proto definitions.
 */

export interface WorkflowStep {
	id: string;
	name: string;
	description: string;
	taskId: string;
	configJson: string;
	order: number;
}

export interface Workflow {
	id: string;
	name: string;
	description: string;
	steps: WorkflowStep[];
	state: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateWorkflowRequest {
	workflow: Workflow;
}

export interface CreateWorkflowResponse {
	workflowId: string;
}

export interface GetWorkflowRequest {
	workflowId: string;
}

export interface GetWorkflowResponse {
	workflow: Workflow;
}

export interface ListWorkflowsRequest {}

export interface ListWorkflowsResponse {
	workflows: Workflow[];
}

export interface UpdateWorkflowRequest {
	workflow: Workflow;
}

export interface UpdateWorkflowResponse {
	workflowId: string;
}

export interface DeleteWorkflowRequest {
	workflowId: string;
}

export interface DeleteWorkflowResponse {
	success: boolean;
}

export interface ExecuteWorkflowRequest {
	workflowId: string;
	useParallel: boolean;
}

export interface ExecuteWorkflowResponse {
	executionId: string;
	workflowId: string;
	success: boolean;
	error?: string;
	finalState: string; // JSON string of WorkflowState
}

export interface GetWorkflowExecutionRequest {
	executionId: string;
}

export interface WorkflowExecution {
	executionId: string;
	workflowId: string;
	contextJson: string; // JSON string of ExecutionContext
	startedAt: string;
	completedAt?: string;
	finalState: string; // JSON string of WorkflowState
}

export interface GetWorkflowExecutionResponse {
	execution: WorkflowExecution;
}

export interface StopWorkflowExecutionRequest {
	workflowId: string;
}

export interface StopWorkflowExecutionResponse {
	success: boolean;
	error?: string;
}

export interface ListWorkflowExecutionsRequest {
	workflowId?: string;
}

export interface ListWorkflowExecutionsResponse {
	executions: WorkflowExecution[];
}

