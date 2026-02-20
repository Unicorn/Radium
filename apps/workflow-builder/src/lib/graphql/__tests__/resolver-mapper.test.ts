/**
 * GraphQL Resolver Mapper Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  getResolverConfig,
  getWorkflowForQuery,
  getWorkflowForMutation,
  mapGraphQLInputToWorkflow,
} from '../resolver-mapper';

describe('GraphQL Resolver Mapper', () => {
  let mockSupabase: SupabaseClient<Database>;

  beforeEach(() => {
    mockSupabase = {
      from: vi.fn(),
    } as any;
  });

  describe('getResolverConfig', () => {
    it('should return resolver config when it exists', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        graphql_schema: {
          queries: [
            {
              queryName: 'getUser',
              workflowId: 'workflow-123',
              inputMapping: {
                id: 'userId',
              },
            },
          ],
          mutations: [
            {
              mutationName: 'createUser',
              workflowId: 'workflow-456',
              inputMapping: {
                name: 'userName',
                email: 'userEmail',
              },
            },
          ],
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

      const result = await getResolverConfig('si-123', mockSupabase);

      expect(result).toEqual({
        queries: [
          {
            queryName: 'getUser',
            workflowId: 'workflow-123',
            inputMapping: {
              id: 'userId',
            },
          },
        ],
        mutations: [
          {
            mutationName: 'createUser',
            workflowId: 'workflow-456',
            inputMapping: {
              name: 'userName',
              email: 'userEmail',
            },
          },
        ],
      });
    });

    it('should return null when config does not exist', async () => {
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

      const result = await getResolverConfig('si-123', mockSupabase);

      expect(result).toBeNull();
    });
  });

  describe('getWorkflowForQuery', () => {
    it('should return workflow ID for existing query', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        graphql_schema: {
          queries: [
            {
              queryName: 'getUser',
              workflowId: 'workflow-123',
            },
          ],
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

      const result = await getWorkflowForQuery('getUser', 'si-123', mockSupabase);

      expect(result).toBe('workflow-123');
    });

    it('should return null for non-existent query', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        graphql_schema: {
          queries: [
            {
              queryName: 'getUser',
              workflowId: 'workflow-123',
            },
          ],
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

      const result = await getWorkflowForQuery('getPost', 'si-123', mockSupabase);

      expect(result).toBeNull();
    });
  });

  describe('getWorkflowForMutation', () => {
    it('should return workflow ID for existing mutation', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        graphql_schema: {
          mutations: [
            {
              mutationName: 'createUser',
              workflowId: 'workflow-456',
            },
          ],
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

      const result = await getWorkflowForMutation('createUser', 'si-123', mockSupabase);

      expect(result).toBe('workflow-456');
    });

    it('should return null for non-existent mutation', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        graphql_schema: {
          mutations: [
            {
              mutationName: 'createUser',
              workflowId: 'workflow-456',
            },
          ],
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

      const result = await getWorkflowForMutation('updateUser', 'si-123', mockSupabase);

      expect(result).toBeNull();
    });
  });

  describe('mapGraphQLInputToWorkflow', () => {
    it('should map GraphQL input to workflow input using mapping', () => {
      const graphQLInput = {
        id: 'user-123',
        name: 'John Doe',
        email: 'john@example.com',
      };

      const inputMapping = {
        id: 'userId',
        name: 'userName',
        email: 'userEmail',
      };

      const result = mapGraphQLInputToWorkflow(graphQLInput, inputMapping);

      expect(result).toEqual({
        userId: 'user-123',
        userName: 'John Doe',
        userEmail: 'john@example.com',
      });
    });

    it('should return original input when no mapping provided', () => {
      const graphQLInput = {
        id: 'user-123',
        name: 'John Doe',
      };

      const result = mapGraphQLInputToWorkflow(graphQLInput);

      expect(result).toEqual(graphQLInput);
    });

    it('should only map fields that exist in input', () => {
      const graphQLInput = {
        id: 'user-123',
        name: 'John Doe',
      };

      const inputMapping = {
        id: 'userId',
        name: 'userName',
        email: 'userEmail', // Not in input
      };

      const result = mapGraphQLInputToWorkflow(graphQLInput, inputMapping);

      expect(result).toEqual({
        userId: 'user-123',
        userName: 'John Doe',
      });
      expect(result).not.toHaveProperty('userEmail');
    });
  });
});

