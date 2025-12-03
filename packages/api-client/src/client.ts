/**
 * gRPC-Web client configuration and connection management.
 */

// Note: gRPC-Web client implementation would use @improbable-eng/grpc-web
// For now, using fetch-based implementation
import type { PingResponse } from '@radium/shared-types';

export interface ClientConfig {
	baseUrl: string;
	timeout?: number;
}

export class RadiumClient {
	private config: ClientConfig;
	private metadata: Record<string, string>;

	constructor(config: ClientConfig) {
		this.config = {
			timeout: 30000,
			...config,
		};
		this.metadata = {};
	}

	/**
	 * Set metadata for all requests (e.g., authentication tokens).
	 */
	setMetadata(key: string, value: string): void {
		// Metadata handling for future gRPC-Web implementation
		// For now, store in a simple object
		(this.metadata as any)[key] = value;
	}

	/**
	 * Get the base URL for gRPC-Web requests.
	 */
	getBaseUrl(): string {
		return this.config.baseUrl;
	}

	/**
	 * Get metadata for requests.
	 */
	getMetadata(): any {
		return this.metadata;
	}

	/**
	 * Ping the server to check connectivity.
	 */
	async ping(message: string = 'ping'): Promise<PingResponse> {
		return new Promise((resolve, reject) => {
			const url = `${this.config.baseUrl}/radium.Radium/Ping`;
			
			fetch(url, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'X-Grpc-Web': '1',
				},
				body: JSON.stringify({ message }),
			})
				.then((response) => {
					if (!response.ok) {
						throw new Error(`HTTP error! status: ${response.status}`);
					}
					return response.json();
				})
				.then((data) => resolve(data as PingResponse))
				.catch((error) => reject(error));
		});
	}
}

