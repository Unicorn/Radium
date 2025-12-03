/**
 * Radium API Client Package
 * 
 * Provides a type-safe client for communicating with the Radium backend
 * via gRPC-Web.
 */

export { RadiumClient } from './client';
export type { ClientConfig } from './client';
export { AgentService } from './services/agent';
export { WorkflowService } from './services/workflow';
export { TaskService } from './services/task';
export { OrchestratorService } from './services/orchestrator';

/**
 * Create a configured Radium API client with all services.
 */
export function createRadiumClient(config: { baseUrl: string; timeout?: number }) {
	const client = new RadiumClient(config);
	
	return {
		client,
		agents: new AgentService(client),
		workflows: new WorkflowService(client),
		tasks: new TaskService(client),
		orchestrator: new OrchestratorService(client),
	};
}

