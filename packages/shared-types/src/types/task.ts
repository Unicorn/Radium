/**
 * Task-related type definitions matching the Rust proto definitions.
 */

export interface Task {
	id: string;
	name: string;
	description: string;
	agentId: string;
	inputJson: string;
	state: string;
	resultJson: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateTaskRequest {
	task: Task;
}

export interface CreateTaskResponse {
	taskId: string;
}

export interface GetTaskRequest {
	taskId: string;
}

export interface GetTaskResponse {
	task: Task;
}

export interface ListTasksRequest {}

export interface ListTasksResponse {
	tasks: Task[];
}

export interface UpdateTaskRequest {
	task: Task;
}

export interface UpdateTaskResponse {
	taskId: string;
}

export interface DeleteTaskRequest {
	taskId: string;
}

export interface DeleteTaskResponse {
	success: boolean;
}

