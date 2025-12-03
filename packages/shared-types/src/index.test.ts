import { describe, it, expect } from 'vitest';
import type {
	Agent,
	Workflow,
	Task,
	RegisteredAgent,
} from './index';

describe('shared-types', () => {
	describe('Type exports', () => {
		it('should export Agent type', () => {
			const agent: Agent = {
				id: 'test-id',
				name: 'Test Agent',
				description: 'Test description',
				configJson: '{}',
				state: 'idle',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};
			expect(agent.id).toBe('test-id');
			expect(agent.name).toBe('Test Agent');
		});

		it('should export Workflow type', () => {
			const workflow: Workflow = {
				id: 'test-id',
				name: 'Test Workflow',
				description: 'Test description',
				steps: [],
				state: 'draft',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};
			expect(workflow.id).toBe('test-id');
			expect(workflow.steps).toEqual([]);
		});

		it('should export Task type', () => {
			const task: Task = {
				id: 'test-id',
				name: 'Test Task',
				description: 'Test description',
				agentId: 'agent-1',
				inputJson: '{}',
				state: 'pending',
				resultJson: '',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};
			expect(task.id).toBe('test-id');
			expect(task.agentId).toBe('agent-1');
		});

		it('should export RegisteredAgent type', () => {
			const registered: RegisteredAgent = {
				id: 'test-id',
				description: 'Test description',
				state: 'idle',
			};
			expect(registered.id).toBe('test-id');
			expect(registered.state).toBe('idle');
		});
	});
});

