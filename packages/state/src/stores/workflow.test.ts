import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useWorkflowStore } from './workflow';
import type { WorkflowService } from '@radium/api-client';
import type { Workflow } from '@radium/shared-types';

describe('WorkflowStore', () => {
	let mockService: WorkflowService;

	beforeEach(() => {
		// Reset store
		useWorkflowStore.setState({
			workflows: [],
			selectedWorkflow: null,
			executions: [],
			loading: false,
			error: null,
		});

		// Create mock service
		mockService = {
			listWorkflows: vi.fn(),
			getWorkflow: vi.fn(),
			createWorkflow: vi.fn(),
			updateWorkflow: vi.fn(),
			deleteWorkflow: vi.fn(),
			executeWorkflow: vi.fn(),
			listWorkflowExecutions: vi.fn(),
		} as any;
	});

	describe('setWorkflows', () => {
		it('should set workflows', () => {
			const workflows: Workflow[] = [
				{
					id: 'workflow-1',
					name: 'Test Workflow',
					description: 'Test',
					steps: [],
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			];

			useWorkflowStore.getState().setWorkflows(workflows);
			expect(useWorkflowStore.getState().workflows).toEqual(workflows);
		});
	});

	describe('setSelectedWorkflow', () => {
		it('should set selected workflow', () => {
			const workflow: Workflow = {
				id: 'workflow-1',
				name: 'Test Workflow',
				description: 'Test',
				steps: [],
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			useWorkflowStore.getState().setSelectedWorkflow(workflow);
			expect(useWorkflowStore.getState().selectedWorkflow).toEqual(workflow);
		});

		it('should clear selected workflow', () => {
			useWorkflowStore.getState().setSelectedWorkflow(null);
			expect(useWorkflowStore.getState().selectedWorkflow).toBeNull();
		});
	});

	describe('fetchWorkflows', () => {
		it('should fetch workflows successfully', async () => {
			const mockWorkflows: Workflow[] = [
				{
					id: 'workflow-1',
					name: 'Workflow 1',
					description: 'First',
					steps: [],
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
				{
					id: 'workflow-2',
					name: 'Workflow 2',
					description: 'Second',
					steps: [],
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				},
			];

			(mockService.listWorkflows as any).mockResolvedValueOnce({
				workflows: mockWorkflows,
			});

			await useWorkflowStore.getState().fetchWorkflows(mockService);

			expect(useWorkflowStore.getState().workflows).toEqual(mockWorkflows);
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBeNull();
		});

		it('should handle fetch error', async () => {
			(mockService.listWorkflows as any).mockRejectedValueOnce(
				new Error('Network error')
			);

			await useWorkflowStore.getState().fetchWorkflows(mockService);

			expect(useWorkflowStore.getState().workflows).toEqual([]);
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBe('Network error');
		});
	});

	describe('fetchWorkflow', () => {
		it('should fetch single workflow successfully', async () => {
			const mockWorkflow: Workflow = {
				id: 'workflow-1',
				name: 'Test Workflow',
				description: 'Test',
				steps: [],
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			(mockService.getWorkflow as any).mockResolvedValueOnce({
				workflow: mockWorkflow,
			});

			await useWorkflowStore
				.getState()
				.fetchWorkflow(mockService, 'workflow-1');

			expect(useWorkflowStore.getState().selectedWorkflow).toEqual(mockWorkflow);
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBeNull();
		});

		it('should handle fetch error', async () => {
			(mockService.getWorkflow as any).mockRejectedValueOnce(
				new Error('Workflow not found')
			);

			await useWorkflowStore
				.getState()
				.fetchWorkflow(mockService, 'nonexistent');

			expect(useWorkflowStore.getState().selectedWorkflow).toBeNull();
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBe('Workflow not found');
		});
	});

	describe('createWorkflow', () => {
		it('should create workflow successfully', async () => {
			const newWorkflow: Workflow = {
				id: '',
				name: 'New Workflow',
				description: 'New',
				steps: [],
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			(mockService.createWorkflow as any).mockResolvedValueOnce({
				workflowId: 'workflow-1',
			});
			(mockService.listWorkflows as any).mockResolvedValueOnce({
				workflows: [{ ...newWorkflow, id: 'workflow-1' }],
			});

			await useWorkflowStore.getState().createWorkflow(mockService, newWorkflow);

			expect(mockService.createWorkflow).toHaveBeenCalledWith({
				workflow: newWorkflow,
			});
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBeNull();
		});

		it('should handle creation error', async () => {
			const newWorkflow: Workflow = {
				id: '',
				name: 'New Workflow',
				description: 'New',
				steps: [],
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			(mockService.createWorkflow as any).mockRejectedValueOnce(
				new Error('Creation failed')
			);

			await useWorkflowStore.getState().createWorkflow(mockService, newWorkflow);

			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBe('Creation failed');
		});
	});

	describe('updateWorkflow', () => {
		it('should update workflow successfully', async () => {
			const updatedWorkflow: Workflow = {
				id: 'workflow-1',
				name: 'Updated Workflow',
				description: 'Updated',
				steps: [],
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			(mockService.updateWorkflow as any).mockResolvedValueOnce({
				success: true,
			});
			(mockService.listWorkflows as any).mockResolvedValueOnce({
				workflows: [updatedWorkflow],
			});

			await useWorkflowStore
				.getState()
				.updateWorkflow(mockService, updatedWorkflow);

			expect(mockService.updateWorkflow).toHaveBeenCalledWith({
				workflow: updatedWorkflow,
			});
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBeNull();
		});

		it('should handle update error', async () => {
			const workflow: Workflow = {
				id: 'workflow-1',
				name: 'Test',
				description: 'Test',
				steps: [],
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			(mockService.updateWorkflow as any).mockRejectedValueOnce(
				new Error('Update failed')
			);

			await useWorkflowStore.getState().updateWorkflow(mockService, workflow);

			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBe('Update failed');
		});
	});

	describe('deleteWorkflow', () => {
		it('should delete workflow successfully', async () => {
			(mockService.deleteWorkflow as any).mockResolvedValueOnce({
				success: true,
			});
			(mockService.listWorkflows as any).mockResolvedValueOnce({
				workflows: [],
			});

			await useWorkflowStore
				.getState()
				.deleteWorkflow(mockService, 'workflow-1');

			expect(mockService.deleteWorkflow).toHaveBeenCalledWith({
				workflowId: 'workflow-1',
			});
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBeNull();
		});

		it('should handle deletion error', async () => {
			(mockService.deleteWorkflow as any).mockRejectedValueOnce(
				new Error('Deletion failed')
			);

			await useWorkflowStore
				.getState()
				.deleteWorkflow(mockService, 'workflow-1');

			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBe('Deletion failed');
		});
	});

	describe('executeWorkflow', () => {
		it('should execute workflow successfully', async () => {
			(mockService.executeWorkflow as any).mockResolvedValueOnce({
				executionId: 'exec-1',
			});
			(mockService.listWorkflowExecutions as any).mockResolvedValueOnce({
				executions: [
					{
						id: 'exec-1',
						workflowId: 'workflow-1',
						status: 'running',
						startedAt: new Date().toISOString(),
					},
				],
			});

			await useWorkflowStore
				.getState()
				.executeWorkflow(mockService, 'workflow-1', false);

			expect(mockService.executeWorkflow).toHaveBeenCalledWith({
				workflowId: 'workflow-1',
				useParallel: false,
			});
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBeNull();
		});

		it('should handle execution error', async () => {
			(mockService.executeWorkflow as any).mockRejectedValueOnce(
				new Error('Execution failed')
			);

			await useWorkflowStore
				.getState()
				.executeWorkflow(mockService, 'workflow-1', false);

			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBe('Execution failed');
		});
	});

	describe('fetchExecutions', () => {
		it('should fetch executions successfully', async () => {
			const mockExecutions = [
				{
					id: 'exec-1',
					workflowId: 'workflow-1',
					status: 'completed',
					startedAt: new Date().toISOString(),
					completedAt: new Date().toISOString(),
				},
			];

			(mockService.listWorkflowExecutions as any).mockResolvedValueOnce({
				executions: mockExecutions,
			});

			await useWorkflowStore.getState().fetchExecutions(mockService);

			expect(useWorkflowStore.getState().executions).toEqual(mockExecutions);
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBeNull();
		});

		it('should handle fetch executions error', async () => {
			(mockService.listWorkflowExecutions as any).mockRejectedValueOnce(
				new Error('Failed to fetch executions')
			);

			await useWorkflowStore.getState().fetchExecutions(mockService);

			expect(useWorkflowStore.getState().executions).toEqual([]);
			expect(useWorkflowStore.getState().loading).toBe(false);
			expect(useWorkflowStore.getState().error).toBe('Failed to fetch executions');
		});
	});
});
