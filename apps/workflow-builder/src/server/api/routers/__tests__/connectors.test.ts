/**
 * Connectors Router Tests
 * Tests for connector classification endpoints
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

/**
 * Test Helper: Create a mock tRPC context
 */
function createMockContext() {
  const mockUser = {
    id: 'user-123',
    email: 'test@example.com',
    display_name: 'Test User',
    auth_user_id: 'auth-123',
    role_id: 'role-123',
    organization_id: 'org-123',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    last_seen_at: new Date().toISOString(),
    default_visibility_id: 'vis-123',
    archived_at: null,
  };

  const mockProject = {
    id: 'project-123',
    name: 'Test Project',
    created_by: 'user-123',
  };

  const mockConnector = {
    id: 'connector-123',
    project_id: 'project-123',
    connector_type: 'database',
    name: 'upstash-redis',
    display_name: 'Upstash Redis',
    description: 'Redis connector',
    is_active: true,
    classifications: ['redis'],
  };

  const mockSupabase = {
    from: vi.fn(),
  };

  return {
    user: mockUser,
    supabase: mockSupabase as any,
    project: mockProject,
    connector: mockConnector,
  };
}

describe('Connectors Router - Classification Endpoints', () => {
  describe('getByClassification', () => {
    it('should return connectors with the specified classification', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      const mockConnectors = [
        {
          id: 'connector-123',
          name: 'upstash-redis',
          display_name: 'Upstash Redis',
          description: 'Redis connector',
          connector_type: 'database',
          is_active: true,
          classifications: ['redis'],
        },
      ];

      // Mock project verification
      const mockProjectQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: ctx.project,
          error: null,
        }),
      };

      // Mock connectors query
      const mockConnectorsQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        contains: vi.fn().mockReturnThis(),
        order: vi.fn().mockResolvedValue({
          data: mockConnectors,
          error: null,
        }),
      };

      (ctx.supabase.from as any).mockImplementation((table: string) => {
        if (table === 'projects') return mockProjectQuery;
        if (table === 'connectors') return mockConnectorsQuery;
        return {};
      });

      const caller = connectorsRouter.createCaller(ctx as any);

      const result = await caller.getByClassification({
        projectId: 'project-123',
        classification: 'redis',
      });

      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('connector-123');
      expect(result[0].classifications).toEqual(['redis']);
    });

    it('should throw NOT_FOUND if project does not exist', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      const mockProjectQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: null,
          error: { message: 'Not found' },
        }),
      };

      (ctx.supabase.from as any).mockReturnValue(mockProjectQuery);

      const caller = connectorsRouter.createCaller(ctx as any);

      await expect(
        caller.getByClassification({
          projectId: 'project-123',
          classification: 'redis',
        })
      ).rejects.toThrow('Project not found');
    });

    it('should throw FORBIDDEN if user does not own project', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      const mockProject = {
        id: 'project-123',
        name: 'Test Project',
        created_by: 'other-user',
      };

      const mockProjectQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: mockProject,
          error: null,
        }),
      };

      (ctx.supabase.from as any).mockReturnValue(mockProjectQuery);

      const caller = connectorsRouter.createCaller(ctx as any);

      await expect(
        caller.getByClassification({
          projectId: 'project-123',
          classification: 'redis',
        })
      ).rejects.toThrow('You do not have access to this project');
    });
  });

  describe('addClassification', () => {
    it('should add classification to connector', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      // Mock connector lookup
      const mockConnectorQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: {
            id: 'connector-123',
            project: ctx.project,
          },
          error: null,
        }),
      };

      // Mock classification insert
      const mockClassificationQuery = {
        insert: vi.fn().mockResolvedValue({ error: null }),
      };

      (ctx.supabase.from as any).mockImplementation((table: string) => {
        if (table === 'connectors') return mockConnectorQuery;
        if (table === 'connector_classifications') return mockClassificationQuery;
        return {};
      });

      const caller = connectorsRouter.createCaller(ctx as any);

      const result = await caller.addClassification({
        connectorId: 'connector-123',
        classification: 'redis',
      });

      expect(result.success).toBe(true);
      expect(mockClassificationQuery.insert).toHaveBeenCalledWith({
        connector_id: 'connector-123',
        classification: 'redis',
      });
    });

    it('should not throw if classification already exists', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      const mockConnectorQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: {
            id: 'connector-123',
            project: ctx.project,
          },
          error: null,
        }),
      };

      const mockClassificationQuery = {
        insert: vi.fn().mockResolvedValue({
          error: { code: '23505', message: 'Unique constraint violation' },
        }),
      };

      (ctx.supabase.from as any).mockImplementation((table: string) => {
        if (table === 'connectors') return mockConnectorQuery;
        if (table === 'connector_classifications') return mockClassificationQuery;
        return {};
      });

      const caller = connectorsRouter.createCaller(ctx as any);

      // Should not throw for unique constraint violation
      const result = await caller.addClassification({
        connectorId: 'connector-123',
        classification: 'redis',
      });

      expect(result.success).toBe(true);
    });

    it('should throw FORBIDDEN if user does not own connector', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      const mockConnectorQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: {
            id: 'connector-123',
            project: { id: 'project-123', created_by: 'other-user' },
          },
          error: null,
        }),
      };

      (ctx.supabase.from as any).mockReturnValue(mockConnectorQuery);

      const caller = connectorsRouter.createCaller(ctx as any);

      await expect(
        caller.addClassification({
          connectorId: 'connector-123',
          classification: 'redis',
        })
      ).rejects.toThrow('You do not have access to this connector');
    });
  });

  describe('removeClassification', () => {
    it('should remove classification from connector', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      const mockConnectorQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: {
            id: 'connector-123',
            project: ctx.project,
          },
          error: null,
        }),
      };

      const mockDeleteQuery = {
        delete: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
      };

      (mockDeleteQuery.eq as any).mockImplementation((field: string, value: any) => {
        if (field === 'connector_id') {
          return {
            eq: (field2: string, value2: any) => ({
              eq: vi.fn().mockResolvedValue({ error: null }),
            }),
          };
        }
        return mockDeleteQuery;
      });

      (ctx.supabase.from as any).mockImplementation((table: string) => {
        if (table === 'connectors') return mockConnectorQuery;
        if (table === 'connector_classifications') return mockDeleteQuery;
        return {};
      });

      const caller = connectorsRouter.createCaller(ctx as any);

      const result = await caller.removeClassification({
        connectorId: 'connector-123',
        classification: 'redis',
      });

      expect(result.success).toBe(true);
      expect(mockDeleteQuery.delete).toHaveBeenCalled();
    });

    it('should throw NOT_FOUND if connector does not exist', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      const mockConnectorQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: null,
          error: { message: 'Not found' },
        }),
      };

      (ctx.supabase.from as any).mockReturnValue(mockConnectorQuery);

      const caller = connectorsRouter.createCaller(ctx as any);

      await expect(
        caller.removeClassification({
          connectorId: 'connector-123',
          classification: 'redis',
        })
      ).rejects.toThrow('Connector not found');
    });
  });
});

