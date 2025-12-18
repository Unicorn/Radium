/**
 * Connectors Router Tests
 * Tests for connector classification endpoints
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

/**
 * Test Helper: Create a mock tRPC context
 */
// Valid UUIDs for testing
const TEST_IDS = {
  authUser: '11111111-1111-1111-1111-111111111111',
  user: '22222222-2222-2222-2222-222222222222',
  project: '33333333-3333-3333-3333-333333333333',
  connector: '44444444-4444-4444-4444-444444444444',
  role: '55555555-5555-5555-5555-555555555555',
  org: '66666666-6666-6666-6666-666666666666',
  visibility: '77777777-7777-7777-7777-777777777777',
  otherUser: '88888888-8888-8888-8888-888888888888',
};

function createMockContext() {
  // Mock auth user (from Supabase auth.getUser())
  const mockAuthUser = {
    id: TEST_IDS.authUser,
    email: 'test@example.com',
    app_metadata: {},
    user_metadata: {},
    aud: 'authenticated',
    created_at: new Date().toISOString(),
  };

  // Mock user record (from users table)
  const mockUser = {
    id: TEST_IDS.user,
    email: 'test@example.com',
    display_name: 'Test User',
    auth_user_id: TEST_IDS.authUser,
    role_id: TEST_IDS.role,
    organization_id: TEST_IDS.org,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    last_seen_at: new Date().toISOString(),
    default_visibility_id: TEST_IDS.visibility,
    archived_at: null,
  };

  const mockProject = {
    id: TEST_IDS.project,
    name: 'Test Project',
    created_by: TEST_IDS.user,
  };

  const mockConnector = {
    id: TEST_IDS.connector,
    project_id: TEST_IDS.project,
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

  // getUserRecord function required by protectedProcedure
  const getUserRecord = vi.fn().mockResolvedValue(mockUser);

  return {
    authUser: mockAuthUser,  // tRPC protectedProcedure checks this first
    user: mockUser,          // After middleware runs, this is populated
    supabase: mockSupabase as any,
    getUserRecord,           // Function to load user record
    project: mockProject,
    connector: mockConnector,
    mockUser,                // Expose for test assertions
  };
}

describe('Connectors Router - Classification Endpoints', () => {
  describe('getByClassification', () => {
    it('should return connectors with the specified classification', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      const mockConnectors = [
        {
          id: TEST_IDS.connector,
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
        projectId: TEST_IDS.project,
        classification: 'redis',
      });

      expect(result).toHaveLength(1);
      expect(result[0]?.id).toBe(TEST_IDS.connector);
      expect(result[0]?.classifications).toEqual(['redis']);
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
          projectId: TEST_IDS.project,
          classification: 'redis',
        })
      ).rejects.toThrow('Project not found');
    });

    it('should throw FORBIDDEN if user does not own project', async () => {
      const { connectorsRouter } = await import('../connectors');
      const ctx = createMockContext();

      const mockProject = {
        id: TEST_IDS.project,
        name: 'Test Project',
        created_by: TEST_IDS.otherUser,
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
          projectId: TEST_IDS.project,
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
            id: TEST_IDS.connector,
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
        connectorId: TEST_IDS.connector,
        classification: 'redis',
      });

      expect(result.success).toBe(true);
      expect(mockClassificationQuery.insert).toHaveBeenCalledWith({
        connector_id: TEST_IDS.connector,
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
            id: TEST_IDS.connector,
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
        connectorId: TEST_IDS.connector,
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
            id: TEST_IDS.connector,
            project: { id: TEST_IDS.project, created_by: TEST_IDS.otherUser },
          },
          error: null,
        }),
      };

      (ctx.supabase.from as any).mockReturnValue(mockConnectorQuery);

      const caller = connectorsRouter.createCaller(ctx as any);

      await expect(
        caller.addClassification({
          connectorId: TEST_IDS.connector,
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
            id: TEST_IDS.connector,
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
        connectorId: TEST_IDS.connector,
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
          connectorId: TEST_IDS.connector,
          classification: 'redis',
        })
      ).rejects.toThrow('Connector not found');
    });
  });
});

