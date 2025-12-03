/**
 * Task store using Zustand for state management.
 */

import { create } from 'zustand';
import type { Task } from '@radium/shared-types';
import type { TaskService } from '@radium/api-client';

export interface TaskStore {
	// State
	tasks: Task[];
	selectedTask: Task | null;
	loading: boolean;
	error: string | null;

	// Actions
	setTasks: (tasks: Task[]) => void;
	setSelectedTask: (task: Task | null) => void;
	setLoading: (loading: boolean) => void;
	setError: (error: string | null) => void;

	// CRUD Operations
	fetchTasks: (service: TaskService) => Promise<void>;
	fetchTask: (service: TaskService, taskId: string) => Promise<void>;
	createTask: (service: TaskService, task: Task) => Promise<void>;
	updateTask: (service: TaskService, task: Task) => Promise<void>;
	deleteTask: (service: TaskService, taskId: string) => Promise<void>;
}

export const useTaskStore = create<TaskStore>((set, get) => ({
	// Initial state
	tasks: [],
	selectedTask: null,
	loading: false,
	error: null,

	// Basic setters
	setTasks: (tasks) => set({ tasks }),
	setSelectedTask: (task) => set({ selectedTask: task }),
	setLoading: (loading) => set({ loading }),
	setError: (error) => set({ error }),

	// Fetch all tasks
	fetchTasks: async (service) => {
		set({ loading: true, error: null });
		try {
			const response = await service.listTasks();
			set({ tasks: response.tasks, loading: false });
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to fetch tasks',
				loading: false,
			});
		}
	},

	// Fetch single task
	fetchTask: async (service, taskId) => {
		set({ loading: true, error: null });
		try {
			const response = await service.getTask({ taskId });
			set({ selectedTask: response.task, loading: false });
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to fetch task',
				loading: false,
			});
		}
	},

	// Create task
	createTask: async (service, task) => {
		set({ loading: true, error: null });
		try {
			await service.createTask({ task });
			// Refresh the list
			await get().fetchTasks(service);
			set({ loading: false });
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to create task',
				loading: false,
			});
		}
	},

	// Update task
	updateTask: async (service, task) => {
		set({ loading: true, error: null });
		try {
			await service.updateTask({ task });
			// Update local state
			set((state) => ({
				tasks: state.tasks.map((t) => (t.id === task.id ? task : t)),
				selectedTask:
					state.selectedTask?.id === task.id ? task : state.selectedTask,
				loading: false,
			}));
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to update task',
				loading: false,
			});
		}
	},

	// Delete task
	deleteTask: async (service, taskId) => {
		set({ loading: true, error: null });
		try {
			await service.deleteTask({ taskId });
			// Update local state
			set((state) => ({
				tasks: state.tasks.filter((t) => t.id !== taskId),
				selectedTask:
					state.selectedTask?.id === taskId ? null : state.selectedTask,
				loading: false,
			}));
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to delete task',
				loading: false,
			});
		}
	},
}));

