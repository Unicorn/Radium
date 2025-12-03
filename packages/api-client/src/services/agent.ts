/**
 * Agent service client for gRPC-Web communication.
 */

// gRPC-Web implementation placeholder
import type {
	Agent,
	CreateAgentRequest,
	CreateAgentResponse,
	GetAgentRequest,
	GetAgentResponse,
	ListAgentsRequest,
	ListAgentsResponse,
	UpdateAgentRequest,
	UpdateAgentResponse,
	DeleteAgentRequest,
	DeleteAgentResponse,
} from '@radium/shared-types';
import type { RadiumClient } from '../client';

export class AgentService {
	constructor(private client: RadiumClient) {}

	/**
	 * Create a new agent.
	 */
	async createAgent(request: CreateAgentRequest): Promise<CreateAgentResponse> {
		return this.callRpc<CreateAgentRequest, CreateAgentResponse>(
			'CreateAgent',
			request
		);
	}

	/**
	 * Get an agent by ID.
	 */
	async getAgent(request: GetAgentRequest): Promise<GetAgentResponse> {
		return this.callRpc<GetAgentRequest, GetAgentResponse>(
			'GetAgent',
			request
		);
	}

	/**
	 * List all agents.
	 */
	async listAgents(request: ListAgentsRequest = {}): Promise<ListAgentsResponse> {
		return this.callRpc<ListAgentsRequest, ListAgentsResponse>(
			'ListAgents',
			request
		);
	}

	/**
	 * Update an existing agent.
	 */
	async updateAgent(request: UpdateAgentRequest): Promise<UpdateAgentResponse> {
		return this.callRpc<UpdateAgentRequest, UpdateAgentResponse>(
			'UpdateAgent',
			request
		);
	}

	/**
	 * Delete an agent by ID.
	 */
	async deleteAgent(request: DeleteAgentRequest): Promise<DeleteAgentResponse> {
		return this.callRpc<DeleteAgentRequest, DeleteAgentResponse>(
			'DeleteAgent',
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
			// This is a simplified implementation
			// In a real implementation, we would use proto-generated code
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

