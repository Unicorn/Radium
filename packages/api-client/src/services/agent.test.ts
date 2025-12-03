import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentService } from './agent';
import { RadiumClient } from '../client';
import type { Agent } from '@radium/shared-types';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch as any;

describe('AgentService', () => {
	let service: AgentService;
	let client: RadiumClient;

	beforeEach(() => {
		client = new RadiumClient({ baseUrl: 'http://localhost:50051' });
		service = new AgentService(client);
		vi.clearAllMocks();
	});

	describe('listAgents', () => {
		it('should list agents', async () => {
			const mockAgents: Agent[] = [
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

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ agents: mockAgents }),
			} as Response);

			const result = await service.listAgents();
			expect(result.agents).toEqual(mockAgents);
			expect(mockFetch).toHaveBeenCalledWith(
				'http://localhost:50051/radium.Radium/ListAgents',
				expect.any(Object)
			);
		});
	});

	describe('getAgent', () => {
		it('should get agent by ID', async () => {
			const mockAgent: Agent = {
				id: 'agent-1',
				name: 'Test Agent',
				description: 'Test',
				configJson: '{}',
				state: 'idle',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ agent: mockAgent }),
			} as Response);

			const result = await service.getAgent({ agentId: 'agent-1' });
			expect(result.agent).toEqual(mockAgent);
		});
	});

	describe('createAgent', () => {
		it('should create agent', async () => {
			const newAgent: Agent = {
				id: '',
				name: 'New Agent',
				description: 'New',
				configJson: '{}',
				state: 'idle',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ agentId: 'agent-1' }),
			} as Response);

			const result = await service.createAgent({ agent: newAgent });
			expect(result.agentId).toBe('agent-1');
		});
	});
});

