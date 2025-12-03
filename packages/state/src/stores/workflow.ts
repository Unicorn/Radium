/**
 * Workflow store using Zustand for state management.
 */

import { create } from 'zustand';
import type { Workflow, WorkflowExecution } from '@radium/shared-types';
import type { WorkflowService } from '@radium/api-client';

export interface WorkflowStore {
	// State
	workflows: Workflow[];
	selectedWorkflow: Workflow | null;
	executions: WorkflowExecution[];
	loading: boolean;
	error: string | null;

	// Actions
	setWorkflows: (workflows: Workflow[]) => void;
	setSelectedWorkflow: (workflow: Workflow | null) => void;
	setExecutions: (executions: WorkflowExecution[]) => void;
	setLoading: (loading: boolean) => void;
	setError: (error: string | null) => void;

	// CRUD Operations
	fetchWorkflows: (service: WorkflowService) => Promise<void>;
	fetchWorkflow: (service: WorkflowService, workflowId: string) => Promise<void>;
	createWorkflow: (service: WorkflowService, workflow: Workflow) => Promise<void>;
	updateWorkflow: (
		service: WorkflowService,
		workflow: Workflow
	) => Promise<void>;
	deleteWorkflow: (service: WorkflowService, workflowId: string) => Promise<void>;
	executeWorkflow: (
		service: WorkflowService,
		workflowId: string,
		useParallel: boolean
	) => Promise<void>;
	fetchExecutions: (
		service: WorkflowService,
		workflowId?: string
	) => Promise<void>;
}

export const useWorkflowStore = create<WorkflowStore>((set, get) => ({
	// Initial state
	workflows: [],
	selectedWorkflow: null,
	executions: [],
	loading: false,
	error: null,

	// Basic setters
	setWorkflows: (workflows) => set({ workflows }),
	setSelectedWorkflow: (workflow) => set({ selectedWorkflow: workflow }),
	setExecutions: (executions) => set({ executions }),
	setLoading: (loading) => set({ loading }),
	setError: (error) => set({ error }),

	// Fetch all workflows
	fetchWorkflows: async (service) => {
		set({ loading: true, error: null });
		try {
			const response = await service.listWorkflows();
			set({ workflows: response.workflows, loading: false });
		} catch (error) {
			set({
				error:
					error instanceof Error ? error.message : 'Failed to fetch workflows',
				loading: false,
			});
		}
	},

	// Fetch single workflow
	fetchWorkflow: async (service, workflowId) => {
		set({ loading: true, error: null });
		try {
			const response = await service.getWorkflow({ workflowId });
			set({ selectedWorkflow: response.workflow, loading: false });
		} catch (error) {
			set({
				error:
					error instanceof Error ? error.message : 'Failed to fetch workflow',
				loading: false,
			});
		}
	},

	// Create workflow
	createWorkflow: async (service, workflow) => {
		set({ loading: true, error: null });
		try {
			await service.createWorkflow({ workflow });
			// Refresh the list
			await get().fetchWorkflows(service);
			set({ loading: false });
		} catch (error) {
			set({
				error:
					error instanceof Error ? error.message : 'Failed to create workflow',
				loading: false,
			});
		}
	},

	// Update workflow
	updateWorkflow: async (service, workflow) => {
		set({ loading: true, error: null });
		try {
			await service.updateWorkflow({ workflow });
			// Update local state
			set((state) => ({
				workflows: state.workflows.map((w) =>
					w.id === workflow.id ? workflow : w
				),
				selectedWorkflow:
					state.selectedWorkflow?.id === workflow.id
						? workflow
						: state.selectedWorkflow,
				loading: false,
			}));
		} catch (error) {
			set({
				error:
					error instanceof Error ? error.message : 'Failed to update workflow',
				loading: false,
			});
		}
	},

	// Delete workflow
	deleteWorkflow: async (service, workflowId) => {
		set({ loading: true, error: null });
		try {
			await service.deleteWorkflow({ workflowId });
			// Update local state
			set((state) => ({
				workflows: state.workflows.filter((w) => w.id !== workflowId),
				selectedWorkflow:
					state.selectedWorkflow?.id === workflowId
						? null
						: state.selectedWorkflow,
				loading: false,
			}));
		} catch (error) {
			set({
				error:
					error instanceof Error ? error.message : 'Failed to delete workflow',
				loading: false,
			});
		}
	},

	// Execute workflow
	executeWorkflow: async (service, workflowId, useParallel) => {
		set({ loading: true, error: null });
		try {
			await service.executeWorkflow({ workflowId, useParallel });
			// Refresh executions
			await get().fetchExecutions(service, workflowId);
			set({ loading: false });
		} catch (error) {
			set({
				error:
					error instanceof Error
						? error.message
						: 'Failed to execute workflow',
				loading: false,
			});
		}
	},

	// Fetch workflow executions
	fetchExecutions: async (service, workflowId) => {
		set({ loading: true, error: null });
		try {
			const response = await service.listWorkflowExecutions({ workflowId });
			set({ executions: response.executions, loading: false });
		} catch (error) {
			set({
				error:
					error instanceof Error
						? error.message
						: 'Failed to fetch workflow executions',
				loading: false,
			});
		}
	},
}));

