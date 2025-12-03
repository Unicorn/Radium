/**
 * Task service client for gRPC-Web communication.
 */

import type {
	CreateTaskRequest,
	CreateTaskResponse,
	GetTaskRequest,
	GetTaskResponse,
	ListTasksRequest,
	ListTasksResponse,
	UpdateTaskRequest,
	UpdateTaskResponse,
	DeleteTaskRequest,
	DeleteTaskResponse,
} from '@radium/shared-types';
import type { RadiumClient } from '../client';

export class TaskService {
	constructor(private client: RadiumClient) {}

	/**
	 * Create a new task.
	 */
	async createTask(request: CreateTaskRequest): Promise<CreateTaskResponse> {
		return this.callRpc<CreateTaskRequest, CreateTaskResponse>(
			'CreateTask',
			request
		);
	}

	/**
	 * Get a task by ID.
	 */
	async getTask(request: GetTaskRequest): Promise<GetTaskResponse> {
		return this.callRpc<GetTaskRequest, GetTaskResponse>(
			'GetTask',
			request
		);
	}

	/**
	 * List all tasks.
	 */
	async listTasks(request: ListTasksRequest = {}): Promise<ListTasksResponse> {
		return this.callRpc<ListTasksRequest, ListTasksResponse>(
			'ListTasks',
			request
		);
	}

	/**
	 * Update an existing task.
	 */
	async updateTask(request: UpdateTaskRequest): Promise<UpdateTaskResponse> {
		return this.callRpc<UpdateTaskRequest, UpdateTaskResponse>(
			'UpdateTask',
			request
		);
	}

	/**
	 * Delete a task by ID.
	 */
	async deleteTask(request: DeleteTaskRequest): Promise<DeleteTaskResponse> {
		return this.callRpc<DeleteTaskRequest, DeleteTaskResponse>(
			'DeleteTask',
			request
		);
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

