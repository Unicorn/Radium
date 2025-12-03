/**
 * Orchestrator-related type definitions matching the Rust proto definitions.
 */

export interface ExecuteAgentRequest {
	agentId: string;
	input: string;
	modelType?: string; // "mock", "gemini", "openai"
	modelId?: string;
}

export interface ExecuteAgentResponse {
	success: boolean;
	output: string;
	error?: string;
}

export interface StartAgentRequest {
	agentId: string;
}

export interface StartAgentResponse {
	success: boolean;
	error?: string;
}

export interface StopAgentRequest {
	agentId: string;
}

export interface StopAgentResponse {
	success: boolean;
	error?: string;
}

export interface RegisteredAgent {
	id: string;
	description: string;
	state: string; // "idle", "running", "paused", "stopped", "error"
}

export interface GetRegisteredAgentsRequest {}

export interface GetRegisteredAgentsResponse {
	agents: RegisteredAgent[];
}

export interface RegisterAgentRequest {
	agentId: string;
	agentType: string; // "echo", "simple", "chat"
	description: string;
}

export interface RegisterAgentResponse {
	success: boolean;
	error?: string;
}

