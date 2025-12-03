import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useAgentStore } from './agent';
import type { Agent } from '@radium/shared-types';

describe('useAgentStore', () => {
	beforeEach(() => {
		// Reset store state
		useAgentStore.setState({
			agents: [],
			selectedAgent: null,
			loading: false,
			error: null,
		});
	});

	it('should initialize with empty state', () => {
		const state = useAgentStore.getState();
		expect(state.agents).toEqual([]);
		expect(state.selectedAgent).toBeNull();
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
	});

	it('should set agents', () => {
		const agents: Agent[] = [
			{
				id: 'agent-1',
				name: 'Test Agent',
				description: 'Test',
				configJson: '{}',
				state: 'idle',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			},
		];
		useAgentStore.getState().setAgents(agents);
		expect(useAgentStore.getState().agents).toEqual(agents);
	});

	it('should set selected agent', () => {
		const agent: Agent = {
			id: 'agent-1',
			name: 'Test Agent',
			description: 'Test',
			configJson: '{}',
			state: 'idle',
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString(),
		};
		useAgentStore.getState().setSelectedAgent(agent);
		expect(useAgentStore.getState().selectedAgent).toEqual(agent);
	});

	it('should set loading state', () => {
		useAgentStore.getState().setLoading(true);
		expect(useAgentStore.getState().loading).toBe(true);
	});

	it('should set error', () => {
		useAgentStore.getState().setError('Test error');
		expect(useAgentStore.getState().error).toBe('Test error');
	});
});

