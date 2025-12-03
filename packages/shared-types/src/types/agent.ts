/**
 * Agent-related type definitions matching the Rust proto definitions.
 */

export interface Agent {
	id: string;
	name: string;
	description: string;
	configJson: string;
	state: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateAgentRequest {
	agent: Agent;
}

export interface CreateAgentResponse {
	agentId: string;
}

export interface GetAgentRequest {
	agentId: string;
}

export interface GetAgentResponse {
	agent: Agent;
}

export interface ListAgentsRequest {}

export interface ListAgentsResponse {
	agents: Agent[];
}

export interface UpdateAgentRequest {
	agent: Agent;
}

export interface UpdateAgentResponse {
	agentId: string;
}

export interface DeleteAgentRequest {
	agentId: string;
}

export interface DeleteAgentResponse {
	success: boolean;
}

