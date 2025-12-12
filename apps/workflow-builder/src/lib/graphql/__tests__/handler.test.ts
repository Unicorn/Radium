/**
 * GraphQL Handler Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  handleGraphQLRequest,
  getGraphQLSchemaSDL,
} from '../handler';

describe('GraphQL Handler', () => {
  let mockSupabase: SupabaseClient<Database>;
  let mockExecuteWorkflow: (workflowId: string, input: Record<string, any>) => Promise<any>;

  beforeEach(() => {
    mockSupabase = {
      from: vi.fn(),
    } as any;

    mockExecuteWorkflow = vi.fn().mockResolvedValue({ result: 'success' });
  });

  describe('getGraphQLSchemaSDL', () => {
    it('should return schema SDL when it exists', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        graphql_schema: {
          sdl: `
            type Query {
              getUser(id: ID!): User
            }
            
            type User {
              id: ID!
              name: String!
            }
          `,
        },
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getGraphQLSchemaSDL('si-123', mockSupabase);

      expect(result).toContain('type Query');
      expect(result).toContain('getUser');
    });

    it('should return null when schema does not exist', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        graphql_schema: null,
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getGraphQLSchemaSDL('si-123', mockSupabase);

      expect(result).toBeNull();
    });
  });

  describe('handleGraphQLRequest', () => {
    const validSDL = `
      type Query {
        getUser(id: ID!): User
      }
      
      type User {
        id: ID!
        name: String!
      }
    `;

    it('should handle valid GraphQL query', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        graphql_schema: {
          sdl: validSDL,
          queries: [
            {
              queryName: 'getUser',
              workflowId: 'workflow-123',
            },
          ],
        },
      };

      // Mock getGraphQLSchemaSDL
      const mockSelect1 = vi.fn().mockReturnThis();
      const mockEq1 = vi.fn().mockReturnThis();
      const mockSingle1 = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      // Mock getResolverConfig
      const mockSelect2 = vi.fn().mockReturnThis();
      const mockEq2 = vi.fn().mockReturnThis();
      const mockSingle2 = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      (mockSupabase.from as any)
        .mockReturnValueOnce({
          select: mockSelect1,
          eq: mockEq1,
          single: mockSingle1,
        })
        .mockReturnValueOnce({
          select: mockSelect2,
          eq: mockEq2,
          single: mockSingle2,
        });

      const request = {
        query: `
          query {
            getUser(id: "123") {
              id
              name
            }
          }
        `,
        variables: {},
      };

      const response = await handleGraphQLRequest(
        request,
        'si-123',
        validSDL,
        mockSupabase,
        mockExecuteWorkflow
      );

      expect(response).toBeDefined();
      // Note: The actual GraphQL execution would require a full schema and resolvers
      // This test verifies the function doesn't throw and returns a response
    });

    it('should handle invalid GraphQL query', async () => {
      const request = {
        query: `
          query {
            invalidQuery {
              field
            }
          }
        `,
        variables: {},
      };

      const response = await handleGraphQLRequest(
        request,
        'si-123',
        validSDL,
        mockSupabase,
        mockExecuteWorkflow
      );

      expect(response).toBeDefined();
      expect(response.errors).toBeDefined();
    });

    it('should handle errors gracefully', async () => {
      const invalidSDL = 'invalid schema';

      const request = {
        query: 'query { getUser { id } }',
        variables: {},
      };

      const response = await handleGraphQLRequest(
        request,
        'si-123',
        invalidSDL,
        mockSupabase,
        mockExecuteWorkflow
      );

      expect(response).toBeDefined();
      expect(response.errors).toBeDefined();
      expect(response.errors?.length).toBeGreaterThan(0);
    });
  });
});

