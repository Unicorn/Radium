/**
 * Orchestrator store using Zustand for state management.
 */

import { create } from 'zustand';
import type {
	RegisteredAgent,
	ExecuteAgentRequest,
	ExecuteAgentResponse,
} from '@radium/shared-types';
import type { OrchestratorService } from '@radium/api-client';

export interface OrchestratorStore {
	// State
	registeredAgents: RegisteredAgent[];
	loading: boolean;
	error: string | null;

	// Actions
	setRegisteredAgents: (agents: RegisteredAgent[]) => void;
	setLoading: (loading: boolean) => void;
	setError: (error: string | null) => void;

	// Operations
	fetchRegisteredAgents: (
		service: OrchestratorService
	) => Promise<void>;
	registerAgent: (
		service: OrchestratorService,
		agentId: string,
		agentType: string,
		description: string
	) => Promise<void>;
	executeAgent: (
		service: OrchestratorService,
		request: ExecuteAgentRequest
	) => Promise<ExecuteAgentResponse>;
	startAgent: (service: OrchestratorService, agentId: string) => Promise<void>;
	stopAgent: (service: OrchestratorService, agentId: string) => Promise<void>;
}

export const useOrchestratorStore = create<OrchestratorStore>((set, get) => ({
	// Initial state
	registeredAgents: [],
	loading: false,
	error: null,

	// Basic setters
	setRegisteredAgents: (agents) => set({ registeredAgents: agents }),
	setLoading: (loading) => set({ loading }),
	setError: (error) => set({ error }),

	// Fetch registered agents
	fetchRegisteredAgents: async (service) => {
		set({ loading: true, error: null });
		try {
			const response = await service.getRegisteredAgents();
			set({ registeredAgents: response.agents, loading: false });
		} catch (error) {
			set({
				error:
					error instanceof Error
						? error.message
						: 'Failed to fetch registered agents',
				loading: false,
			});
		}
	},

	// Register agent
	registerAgent: async (service, agentId, agentType, description) => {
		set({ loading: true, error: null });
		try {
			await service.registerAgent({ agentId, agentType, description });
			// Refresh the list
			await get().fetchRegisteredAgents(service);
			set({ loading: false });
		} catch (error) {
			set({
				error:
					error instanceof Error
						? error.message
						: 'Failed to register agent',
				loading: false,
			});
		}
	},

	// Execute agent
	executeAgent: async (service, request) => {
		set({ loading: true, error: null });
		try {
			const response = await service.executeAgent(request);
			set({ loading: false });
			return response;
		} catch (error) {
			const errorMessage =
				error instanceof Error ? error.message : 'Failed to execute agent';
			set({ error: errorMessage, loading: false });
			throw error;
		}
	},

	// Start agent
	startAgent: async (service, agentId) => {
		set({ loading: true, error: null });
		try {
			await service.startAgent({ agentId });
			// Refresh the list
			await get().fetchRegisteredAgents(service);
			set({ loading: false });
		} catch (error) {
			set({
				error:
					error instanceof Error ? error.message : 'Failed to start agent',
				loading: false,
			});
		}
	},

	// Stop agent
	stopAgent: async (service, agentId) => {
		set({ loading: true, error: null });
		try {
			await service.stopAgent({ agentId });
			// Refresh the list
			await get().fetchRegisteredAgents(service);
			set({ loading: false });
		} catch (error) {
			set({
				error:
					error instanceof Error ? error.message : 'Failed to stop agent',
				loading: false,
			});
		}
	},
}));

