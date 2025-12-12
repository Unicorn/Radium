/**
 * GraphQL API Route Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { POST, GET } from '../graphql/route';
import { NextRequest } from 'next/server';
import { createClient } from '@/lib/supabase/server';
import { getTemporalClient } from '@/lib/temporal/connection';
import { getGraphQLSchemaSDL, handleGraphQLRequest } from '@/lib/graphql/handler';

vi.mock('@/lib/supabase/server', () => ({
  createClient: vi.fn(),
}));

vi.mock('@/lib/temporal/connection', () => ({
  getTemporalClient: vi.fn(),
}));

vi.mock('@/lib/graphql/handler', () => ({
  getGraphQLSchemaSDL: vi.fn(),
  handleGraphQLRequest: vi.fn(),
}));

describe('GraphQL API Route', () => {
  let mockSupabase: any;
  let mockTemporalClient: any;

  beforeEach(() => {
    vi.clearAllMocks();

    mockSupabase = {};
    vi.mocked(createClient).mockResolvedValue(mockSupabase as any);

    mockTemporalClient = {
      getHandle: vi.fn().mockReturnValue({
        query: vi.fn().mockResolvedValue({ result: 'test' }),
      }),
      workflow: vi.fn().mockReturnValue({
        execute: vi.fn().mockResolvedValue({ result: 'test' }),
      }),
    };
    vi.mocked(getTemporalClient).mockResolvedValue(mockTemporalClient as any);
  });

  describe('POST', () => {
    it('should handle valid GraphQL request', async () => {
      const serviceInterfaceId = 'si-123';
      const mockSchemaSDL = 'type Query { getUser: User }';
      const mockResponse = {
        data: { getUser: { id: '123', name: 'Test' } },
        errors: undefined,
      };

      vi.mocked(getGraphQLSchemaSDL).mockResolvedValue(mockSchemaSDL);
      vi.mocked(handleGraphQLRequest).mockResolvedValue(mockResponse);

      const request = new NextRequest('http://localhost/api/graphql?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: 'query { getUser { id name } }',
          variables: {},
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.data).toBeDefined();
    });

    it('should return 400 when service interface ID is missing', async () => {
      const request = new NextRequest('http://localhost/api/graphql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: 'query { getUser { id } }',
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.errors).toBeDefined();
      expect(data.errors[0].message).toContain('Service interface ID is required');
    });

    it('should return 400 when query is missing', async () => {
      const request = new NextRequest('http://localhost/api/graphql?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          variables: {},
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.errors).toBeDefined();
      expect(data.errors[0].message).toContain('GraphQL query is required');
    });

    it('should return 404 when schema not found', async () => {
      vi.mocked(getGraphQLSchemaSDL).mockResolvedValue(null);

      const request = new NextRequest('http://localhost/api/graphql?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: 'query { getUser { id } }',
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(404);
      expect(data.errors).toBeDefined();
      expect(data.errors[0].message).toContain('GraphQL schema not found');
    });

    it('should handle errors gracefully', async () => {
      vi.mocked(getGraphQLSchemaSDL).mockRejectedValue(new Error('Database error'));

      const request = new NextRequest('http://localhost/api/graphql?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: 'query { getUser { id } }',
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(500);
      expect(data.errors).toBeDefined();
    });

    it('should accept service interface ID from header', async () => {
      const mockSchemaSDL = 'type Query { getUser: User }';
      const mockResponse = { data: {}, errors: undefined };

      vi.mocked(getGraphQLSchemaSDL).mockResolvedValue(mockSchemaSDL);
      vi.mocked(handleGraphQLRequest).mockResolvedValue(mockResponse);

      const request = new NextRequest('http://localhost/api/graphql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Service-Interface-Id': 'si-123',
        },
        body: JSON.stringify({
          query: 'query { getUser { id } }',
        }),
      });

      const response = await POST(request);

      expect(response.status).toBe(200);
      expect(getGraphQLSchemaSDL).toHaveBeenCalledWith('si-123', mockSupabase);
    });
  });

  describe('GET', () => {
    it('should return info message', async () => {
      const request = new NextRequest('http://localhost/api/graphql', {
        method: 'GET',
      });

      const response = await GET(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.message).toContain('GraphQL endpoint');
    });
  });
});

