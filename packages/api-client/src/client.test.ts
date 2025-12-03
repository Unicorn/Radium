import { describe, it, expect, vi, beforeEach } from 'vitest';
import { RadiumClient } from './client';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch as any;

describe('RadiumClient', () => {
	let client: RadiumClient;

	beforeEach(() => {
		client = new RadiumClient({ baseUrl: 'http://localhost:50051' });
		vi.clearAllMocks();
	});

	describe('constructor', () => {
		it('should create client with base URL', () => {
			expect(client.getBaseUrl()).toBe('http://localhost:50051');
		});

		it('should use default timeout', () => {
			const defaultClient = new RadiumClient({ baseUrl: 'http://localhost:50051' });
			expect(defaultClient.getBaseUrl()).toBe('http://localhost:50051');
		});
	});

	describe('setMetadata', () => {
		it('should set metadata', () => {
			client.setMetadata('authorization', 'Bearer token');
			const metadata = client.getMetadata();
			expect(metadata['authorization']).toBe('Bearer token');
		});
	});

	describe('ping', () => {
		it('should ping server successfully', async () => {
			const mockResponse = { message: 'pong' };
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse,
			} as Response);

			const result = await client.ping('test');
			expect(result.message).toBe('pong');
			expect(mockFetch).toHaveBeenCalledWith(
				'http://localhost:50051/radium.Radium/Ping',
				expect.objectContaining({
					method: 'POST',
					headers: expect.objectContaining({
						'Content-Type': 'application/json',
						'X-Grpc-Web': '1',
					}),
				})
			);
		});

		it('should handle ping errors', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 500,
			} as Response);

			await expect(client.ping()).rejects.toThrow();
		});
	});
});

