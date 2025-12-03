/**
 * Orchestrator service client for gRPC-Web communication.
 */

import type {
	ExecuteAgentRequest,
	ExecuteAgentResponse,
	StartAgentRequest,
	StartAgentResponse,
	StopAgentRequest,
	StopAgentResponse,
	GetRegisteredAgentsRequest,
	GetRegisteredAgentsResponse,
	RegisterAgentRequest,
	RegisterAgentResponse,
} from '@radium/shared-types';
import type { RadiumClient } from '../client';

export class OrchestratorService {
	constructor(private client: RadiumClient) {}

	/**
	 * Execute an agent with input.
	 */
	async executeAgent(
		request: ExecuteAgentRequest
	): Promise<ExecuteAgentResponse> {
		return this.callRpc<ExecuteAgentRequest, ExecuteAgentResponse>(
			'ExecuteAgent',
			request
		);
	}

	/**
	 * Start an agent.
	 */
	async startAgent(request: StartAgentRequest): Promise<StartAgentResponse> {
		return this.callRpc<StartAgentRequest, StartAgentResponse>(
			'StartAgent',
			request
		);
	}

	/**
	 * Stop an agent.
	 */
	async stopAgent(request: StopAgentRequest): Promise<StopAgentResponse> {
		return this.callRpc<StopAgentRequest, StopAgentResponse>(
			'StopAgent',
			request
		);
	}

	/**
	 * Get all registered agents.
	 */
	async getRegisteredAgents(
		request: GetRegisteredAgentsRequest = {}
	): Promise<GetRegisteredAgentsResponse> {
		return this.callRpc<
			GetRegisteredAgentsRequest,
			GetRegisteredAgentsResponse
		>('GetRegisteredAgents', request);
	}

	/**
	 * Register a new agent.
	 */
	async registerAgent(
		request: RegisterAgentRequest
	): Promise<RegisterAgentResponse> {
		return this.callRpc<RegisterAgentRequest, RegisterAgentResponse>(
			'RegisterAgent',
			request
		);
	}

	/**
	 * Generic RPC call helper.
	 */
	private async callRpc<TRequest, TResponse>(
		method: string,
		request: TRequest
	): Promise<TResponse> {
		return new Promise((resolve, reject) => {
			const url = `${this.client.getBaseUrl()}/radium.Radium/${method}`;
			
			fetch(url, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'X-Grpc-Web': '1',
				},
				body: JSON.stringify(request),
			})
				.then((response) => {
					if (!response.ok) {
						throw new Error(`HTTP error! status: ${response.status}`);
					}
					return response.json();
				})
				.then((data) => resolve(data as TResponse))
				.catch((error) => reject(error));
		});
	}
}

