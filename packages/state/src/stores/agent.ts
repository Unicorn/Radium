/**
 * Agent store using Zustand for state management.
 */

import { create } from 'zustand';
import type { Agent } from '@radium/shared-types';
import type { AgentService } from '@radium/api-client';

export interface AgentStore {
	// State
	agents: Agent[];
	selectedAgent: Agent | null;
	loading: boolean;
	error: string | null;

	// Actions
	setAgents: (agents: Agent[]) => void;
	setSelectedAgent: (agent: Agent | null) => void;
	setLoading: (loading: boolean) => void;
	setError: (error: string | null) => void;

	// CRUD Operations
	fetchAgents: (service: AgentService) => Promise<void>;
	fetchAgent: (service: AgentService, agentId: string) => Promise<void>;
	createAgent: (service: AgentService, agent: Agent) => Promise<void>;
	updateAgent: (service: AgentService, agent: Agent) => Promise<void>;
	deleteAgent: (service: AgentService, agentId: string) => Promise<void>;
}

export const useAgentStore = create<AgentStore>((set, get) => ({
	// Initial state
	agents: [],
	selectedAgent: null,
	loading: false,
	error: null,

	// Basic setters
	setAgents: (agents) => set({ agents }),
	setSelectedAgent: (agent) => set({ selectedAgent: agent }),
	setLoading: (loading) => set({ loading }),
	setError: (error) => set({ error }),

	// Fetch all agents
	fetchAgents: async (service) => {
		set({ loading: true, error: null });
		try {
			const response = await service.listAgents();
			set({ agents: response.agents, loading: false });
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to fetch agents',
				loading: false,
			});
		}
	},

	// Fetch single agent
	fetchAgent: async (service, agentId) => {
		set({ loading: true, error: null });
		try {
			const response = await service.getAgent({ agentId });
			set({ selectedAgent: response.agent, loading: false });
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to fetch agent',
				loading: false,
			});
		}
	},

	// Create agent
	createAgent: async (service, agent) => {
		set({ loading: true, error: null });
		try {
			await service.createAgent({ agent });
			// Refresh the list
			await get().fetchAgents(service);
			set({ loading: false });
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to create agent',
				loading: false,
			});
		}
	},

	// Update agent
	updateAgent: async (service, agent) => {
		set({ loading: true, error: null });
		try {
			await service.updateAgent({ agent });
			// Update local state
			set((state) => ({
				agents: state.agents.map((a) => (a.id === agent.id ? agent : a)),
				selectedAgent:
					state.selectedAgent?.id === agent.id ? agent : state.selectedAgent,
				loading: false,
			}));
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to update agent',
				loading: false,
			});
		}
	},

	// Delete agent
	deleteAgent: async (service, agentId) => {
		set({ loading: true, error: null });
		try {
			await service.deleteAgent({ agentId });
			// Update local state
			set((state) => ({
				agents: state.agents.filter((a) => a.id !== agentId),
				selectedAgent:
					state.selectedAgent?.id === agentId ? null : state.selectedAgent,
				loading: false,
			}));
		} catch (error) {
			set({
				error: error instanceof Error ? error.message : 'Failed to delete agent',
				loading: false,
			});
		}
	},
}));

