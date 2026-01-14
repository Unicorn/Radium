import { describe, it, expect, vi, beforeEach } from 'vitest';
import { WorkflowService } from './workflow';
import { RadiumClient } from '../client';
import type {
	CreateWorkflowRequest,
	UpdateWorkflowRequest,
	ExecuteWorkflowRequest,
} from '@radium/shared-types';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch as any;

describe('WorkflowService', () => {
	let service: WorkflowService;
	let client: RadiumClient;

	beforeEach(() => {
		client = new RadiumClient({ baseUrl: 'http://localhost:50051' });
		service = new WorkflowService(client);
		vi.clearAllMocks();
	});

	describe('createWorkflow', () => {
		it('should create a new workflow', async () => {
			const request: CreateWorkflowRequest = {
				workflow: {
					id: '',
					name: 'Test Workflow',
					description: 'Test workflow description',
					steps: [],
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ workflowId: 'workflow-1' }),
			} as Response);

			const result = await service.createWorkflow(request);
			expect(result.workflowId).toBe('workflow-1');
			expect(mockFetch).toHaveBeenCalledWith(
				'http://localhost:50051/radium.Radium/CreateWorkflow',
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
			const request: CreateWorkflowRequest = {
				workflow: {
					id: '',
					name: 'Test Workflow',
					description: 'Test',
					steps: [],
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			};

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 500,
			} as Response);

			await expect(service.createWorkflow(request)).rejects.toThrow();
		});
	});

	describe('getWorkflow', () => {
		it('should get workflow by ID', async () => {
			const mockWorkflow = {
				id: 'workflow-1',
				name: 'Test Workflow',
				description: 'Test',
				steps: [],
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ workflow: mockWorkflow }),
			} as Response);

			const result = await service.getWorkflow({ workflowId: 'workflow-1' });
			expect(result.workflow).toEqual(mockWorkflow);
			expect(mockFetch).toHaveBeenCalledWith(
				'http://localhost:50051/radium.Radium/GetWorkflow',
				expect.any(Object)
			);
		});

		it('should handle not found errors', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 404,
			} as Response);

			await expect(
				service.getWorkflow({ workflowId: 'nonexistent' })
			).rejects.toThrow();
		});
	});

	describe('listWorkflows', () => {
		it('should list all workflows', async () => {
			const mockWorkflows = [
				{
					id: 'workflow-1',
					name: 'Workflow 1',
					description: 'First workflow',
					steps: [],
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
				{
					id: 'workflow-2',
					name: 'Workflow 2',
					description: 'Second workflow',
					steps: [],
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			];

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ workflows: mockWorkflows }),
			} as Response);

			const result = await service.listWorkflows();
			expect(result.workflows).toEqual(mockWorkflows);
			expect(result.workflows).toHaveLength(2);
		});

		it('should list workflows with empty result', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ workflows: [] }),
			} as Response);

			const result = await service.listWorkflows();
			expect(result.workflows).toEqual([]);
		});
	});

	describe('updateWorkflow', () => {
		it('should update an existing workflow', async () => {
			const request: UpdateWorkflowRequest = {
				workflow: {
					id: 'workflow-1',
					name: 'Updated Workflow',
					description: 'Updated description',
					steps: [],
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ success: true }),
			} as Response);

			const result = await service.updateWorkflow(request);
			expect(result.success).toBe(true);
		});
	});

	describe('deleteWorkflow', () => {
		it('should delete a workflow', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ success: true }),
			} as Response);

			const result = await service.deleteWorkflow({ workflowId: 'workflow-1' });
			expect(result.success).toBe(true);
		});
	});

	describe('executeWorkflow', () => {
		it('should execute a workflow', async () => {
			const request: ExecuteWorkflowRequest = {
				workflowId: 'workflow-1',
				input: { key: 'value' },
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ executionId: 'exec-1' }),
			} as Response);

			const result = await service.executeWorkflow(request);
			expect(result.executionId).toBe('exec-1');
		});

		it('should handle execution errors', async () => {
			const request: ExecuteWorkflowRequest = {
				workflowId: 'workflow-1',
				input: {},
			};

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 400,
			} as Response);

			await expect(service.executeWorkflow(request)).rejects.toThrow();
		});
	});

	describe('getWorkflowExecution', () => {
		it('should get workflow execution details', async () => {
			const mockExecution = {
				id: 'exec-1',
				workflowId: 'workflow-1',
				status: 'running',
				startedAt: new Date().toISOString(),
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ execution: mockExecution }),
			} as Response);

			const result = await service.getWorkflowExecution({ executionId: 'exec-1' });
			expect(result.execution).toEqual(mockExecution);
		});
	});

	describe('stopWorkflowExecution', () => {
		it('should stop a workflow execution', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ success: true }),
			} as Response);

			const result = await service.stopWorkflowExecution({ executionId: 'exec-1' });
			expect(result.success).toBe(true);
		});
	});

	describe('listWorkflowExecutions', () => {
		it('should list workflow executions', async () => {
			const mockExecutions = [
				{
					id: 'exec-1',
					workflowId: 'workflow-1',
					status: 'completed',
					startedAt: new Date().toISOString(),
					completedAt: new Date().toISOString(),
				},
				{
					id: 'exec-2',
					workflowId: 'workflow-1',
					status: 'running',
					startedAt: new Date().toISOString(),
				},
			];

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ executions: mockExecutions }),
			} as Response);

			const result = await service.listWorkflowExecutions();
			expect(result.executions).toEqual(mockExecutions);
			expect(result.executions).toHaveLength(2);
		});
	});
});
