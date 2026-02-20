import { describe, it, expect, vi, beforeEach } from 'vitest';
import { OrchestratorService } from './orchestrator';
import { RadiumClient } from '../client';
import type {
	ExecuteAgentRequest,
	RegisterAgentRequest,
} from '@radium/shared-types';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch as any;

describe('OrchestratorService', () => {
	let service: OrchestratorService;
	let client: RadiumClient;

	beforeEach(() => {
		client = new RadiumClient({ baseUrl: 'http://localhost:50051' });
		service = new OrchestratorService(client);
		vi.clearAllMocks();
	});

	describe('executeAgent', () => {
		it('should execute an agent with input', async () => {
			const request: ExecuteAgentRequest = {
				agentId: 'agent-1',
				input: 'Test input',
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({
					output: 'Agent response',
					executionId: 'exec-1'
				}),
			} as Response);

			const result = await service.executeAgent(request);
			expect(result.output).toBe('Agent response');
			expect(result.executionId).toBe('exec-1');
			expect(mockFetch).toHaveBeenCalledWith(
				'http://localhost:50051/radium.Radium/ExecuteAgent',
				expect.objectContaining({
					method: 'POST',
					headers: {
						'Content-Type': 'application/json',
						'X-Grpc-Web': '1',
					},
				})
			);
		});

		it('should handle execution errors', async () => {
			const request: ExecuteAgentRequest = {
				agentId: 'agent-1',
				input: 'Test input',
			};

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 500,
			} as Response);

			await expect(service.executeAgent(request)).rejects.toThrow();
		});
	});

	describe('startAgent', () => {
		it('should start an agent', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ success: true }),
			} as Response);

			const result = await service.startAgent({ agentId: 'agent-1' });
			expect(result.success).toBe(true);
			expect(mockFetch).toHaveBeenCalledWith(
				'http://localhost:50051/radium.Radium/StartAgent',
				expect.any(Object)
			);
		});

		it('should handle start errors', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 404,
			} as Response);

			await expect(
				service.startAgent({ agentId: 'nonexistent' })
			).rejects.toThrow();
		});
	});

	describe('stopAgent', () => {
		it('should stop an agent', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ success: true }),
			} as Response);

			const result = await service.stopAgent({ agentId: 'agent-1' });
			expect(result.success).toBe(true);
			expect(mockFetch).toHaveBeenCalledWith(
				'http://localhost:50051/radium.Radium/StopAgent',
				expect.any(Object)
			);
		});

		it('should handle stop errors', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 400,
			} as Response);

			await expect(
				service.stopAgent({ agentId: 'agent-1' })
			).rejects.toThrow();
		});
	});

	describe('getRegisteredAgents', () => {
		it('should get all registered agents', async () => {
			const mockAgents = [
				{
					id: 'agent-1',
					name: 'Agent 1',
					type: 'workflow',
					status: 'active',
				},
				{
					id: 'agent-2',
					name: 'Agent 2',
					type: 'orchestrator',
					status: 'idle',
				},
			];

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ agents: mockAgents }),
			} as Response);

			const result = await service.getRegisteredAgents();
			expect(result.agents).toEqual(mockAgents);
			expect(result.agents).toHaveLength(2);
		});

		it('should handle empty agent list', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ agents: [] }),
			} as Response);

			const result = await service.getRegisteredAgents();
			expect(result.agents).toEqual([]);
		});
	});

	describe('registerAgent', () => {
		it('should register a new agent', async () => {
			const request: RegisterAgentRequest = {
				agentId: 'agent-1',
				name: 'Test Agent',
				type: 'workflow',
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => ({ success: true, agentId: 'agent-1' }),
			} as Response);

			const result = await service.registerAgent(request);
			expect(result.success).toBe(true);
			expect(result.agentId).toBe('agent-1');
		});

		it('should handle registration errors', async () => {
			const request: RegisterAgentRequest = {
				agentId: 'agent-1',
				name: 'Test Agent',
				type: 'workflow',
			};

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 409, // Conflict - agent already exists
			} as Response);

			await expect(service.registerAgent(request)).rejects.toThrow();
		});
	});
});
