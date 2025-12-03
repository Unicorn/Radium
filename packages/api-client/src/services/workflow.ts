/**
 * Workflow service client for gRPC-Web communication.
 */

import type {
	CreateWorkflowRequest,
	CreateWorkflowResponse,
	GetWorkflowRequest,
	GetWorkflowResponse,
	ListWorkflowsRequest,
	ListWorkflowsResponse,
	UpdateWorkflowRequest,
	UpdateWorkflowResponse,
	DeleteWorkflowRequest,
	DeleteWorkflowResponse,
	ExecuteWorkflowRequest,
	ExecuteWorkflowResponse,
	GetWorkflowExecutionRequest,
	GetWorkflowExecutionResponse,
	StopWorkflowExecutionRequest,
	StopWorkflowExecutionResponse,
	ListWorkflowExecutionsRequest,
	ListWorkflowExecutionsResponse,
} from '@radium/shared-types';
import type { RadiumClient } from '../client';

export class WorkflowService {
	constructor(private client: RadiumClient) {}

	/**
	 * Create a new workflow.
	 */
	async createWorkflow(
		request: CreateWorkflowRequest
	): Promise<CreateWorkflowResponse> {
		return this.callRpc<CreateWorkflowRequest, CreateWorkflowResponse>(
			'CreateWorkflow',
			request
		);
	}

	/**
	 * Get a workflow by ID.
	 */
	async getWorkflow(
		request: GetWorkflowRequest
	): Promise<GetWorkflowResponse> {
		return this.callRpc<GetWorkflowRequest, GetWorkflowResponse>(
			'GetWorkflow',
			request
		);
	}

	/**
	 * List all workflows.
	 */
	async listWorkflows(
		request: ListWorkflowsRequest = {}
	): Promise<ListWorkflowsResponse> {
		return this.callRpc<ListWorkflowsRequest, ListWorkflowsResponse>(
			'ListWorkflows',
			request
		);
	}

	/**
	 * Update an existing workflow.
	 */
	async updateWorkflow(
		request: UpdateWorkflowRequest
	): Promise<UpdateWorkflowResponse> {
		return this.callRpc<UpdateWorkflowRequest, UpdateWorkflowResponse>(
			'UpdateWorkflow',
			request
		);
	}

	/**
	 * Delete a workflow by ID.
	 */
	async deleteWorkflow(
		request: DeleteWorkflowRequest
	): Promise<DeleteWorkflowResponse> {
		return this.callRpc<DeleteWorkflowRequest, DeleteWorkflowResponse>(
			'DeleteWorkflow',
			request
		);
	}

	/**
	 * Execute a workflow.
	 */
	async executeWorkflow(
		request: ExecuteWorkflowRequest
	): Promise<ExecuteWorkflowResponse> {
		return this.callRpc<ExecuteWorkflowRequest, ExecuteWorkflowResponse>(
			'ExecuteWorkflow',
			request
		);
	}

	/**
	 * Get workflow execution details.
	 */
	async getWorkflowExecution(
		request: GetWorkflowExecutionRequest
	): Promise<GetWorkflowExecutionResponse> {
		return this.callRpc<
			GetWorkflowExecutionRequest,
			GetWorkflowExecutionResponse
		>('GetWorkflowExecution', request);
	}

	/**
	 * Stop a workflow execution.
	 */
	async stopWorkflowExecution(
		request: StopWorkflowExecutionRequest
	): Promise<StopWorkflowExecutionResponse> {
		return this.callRpc<
			StopWorkflowExecutionRequest,
			StopWorkflowExecutionResponse
		>('StopWorkflowExecution', request);
	}

	/**
	 * List workflow executions.
	 */
	async listWorkflowExecutions(
		request: ListWorkflowExecutionsRequest = {}
	): Promise<ListWorkflowExecutionsResponse> {
		return this.callRpc<
			ListWorkflowExecutionsRequest,
			ListWorkflowExecutionsResponse
		>('ListWorkflowExecutions', request);
	}

	/**
	 * Generic RPC call helper.
	 */
	private async callRpc<TRequest, TResponse>(
		method: string,
		request: TRequest
	): Promise<TResponse> {
		return new Promise((resolve, reject) => {
			const url = `${this.client.getBaseUrl()}/radium.Radium/${method}`;
			
			fetch(url, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'X-Grpc-Web': '1',
				},
				body: JSON.stringify(request),
			})
				.then((response) => {
					if (!response.ok) {
						throw new Error(`HTTP error! status: ${response.status}`);
					}
					return response.json();
				})
				.then((data) => resolve(data as TResponse))
				.catch((error) => reject(error));
		});
	}
}

