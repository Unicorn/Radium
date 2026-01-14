import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TaskService } from './task';
import { RadiumClient } from '../client';
import type {
	CreateTaskRequest,
	UpdateTaskRequest,
} from '@radium/shared-types';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch as any;

describe('TaskService', () => {
	let service: TaskService;
	let client: RadiumClient;

	beforeEach(() => {
		client = new RadiumClient({ baseUrl: 'http://localhost:50051' });
		service = new TaskService(client);
		vi.clearAllMocks();
	});

	describe('createTask', () => {
		it('should create a new task', async () => {
			const request: CreateTaskRequest = {
				task: {
					id: '',
					name: 'Test Task',
					description: 'Test task description',
					status: 'pending',
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ taskId: 'task-1' }),
			} as Response);

			const result = await service.createTask(request);
			expect(result.taskId).toBe('task-1');
			expect(mockFetch).toHaveBeenCalledWith(
				'http://localhost:50051/radium.Radium/CreateTask',
				expect.objectContaining({
					method: 'POST',
					headers: {
						'Content-Type': 'application/json',
						'X-Grpc-Web': '1',
					},
				})
			);
		});

		it('should handle creation errors', async () => {
			const request: CreateTaskRequest = {
				task: {
					id: '',
					name: 'Test Task',
					description: 'Test',
					status: 'pending',
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			};

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 500,
			} as Response);

			await expect(service.createTask(request)).rejects.toThrow();
		});
	});

	describe('getTask', () => {
		it('should get task by ID', async () => {
			const mockTask = {
				id: 'task-1',
				name: 'Test Task',
				description: 'Test',
				status: 'in_progress',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ task: mockTask }),
			} as Response);

			const result = await service.getTask({ taskId: 'task-1' });
			expect(result.task).toEqual(mockTask);
			expect(mockFetch).toHaveBeenCalledWith(
				'http://localhost:50051/radium.Radium/GetTask',
				expect.any(Object)
			);
		});

		it('should handle not found errors', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 404,
			} as Response);

			await expect(
				service.getTask({ taskId: 'nonexistent' })
			).rejects.toThrow();
		});
	});

	describe('listTasks', () => {
		it('should list all tasks', async () => {
			const mockTasks = [
				{
					id: 'task-1',
					name: 'Task 1',
					description: 'First task',
					status: 'completed',
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
				{
					id: 'task-2',
					name: 'Task 2',
					description: 'Second task',
					status: 'pending',
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			];

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ tasks: mockTasks }),
			} as Response);

			const result = await service.listTasks();
			expect(result.tasks).toEqual(mockTasks);
			expect(result.tasks).toHaveLength(2);
		});

		it('should list tasks with empty result', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ tasks: [] }),
			} as Response);

			const result = await service.listTasks();
			expect(result.tasks).toEqual([]);
		});

		it('should list tasks with filters', async () => {
			const mockTasks = [
				{
					id: 'task-1',
					name: 'Task 1',
					description: 'Completed task',
					status: 'completed',
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			];

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ tasks: mockTasks }),
			} as Response);

			const result = await service.listTasks({ status: 'completed' });
			expect(result.tasks).toEqual(mockTasks);
			expect(result.tasks).toHaveLength(1);
			expect(result.tasks[0].status).toBe('completed');
		});
	});

	describe('updateTask', () => {
		it('should update an existing task', async () => {
			const request: UpdateTaskRequest = {
				task: {
					id: 'task-1',
					name: 'Updated Task',
					description: 'Updated description',
					status: 'completed',
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ success: true }),
			} as Response);

			const result = await service.updateTask(request);
			expect(result.success).toBe(true);
		});

		it('should handle update validation errors', async () => {
			const request: UpdateTaskRequest = {
				task: {
					id: 'task-1',
					name: '',
					description: '',
					status: 'pending',
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			};

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 400,
			} as Response);

			await expect(service.updateTask(request)).rejects.toThrow();
		});
	});

	describe('deleteTask', () => {
		it('should delete a task', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ success: true }),
			} as Response);

			const result = await service.deleteTask({ taskId: 'task-1' });
			expect(result.success).toBe(true);
		});

		it('should handle delete errors', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 404,
			} as Response);

			await expect(
				service.deleteTask({ taskId: 'nonexistent' })
			).rejects.toThrow();
		});
	});
});
