/**
 * Tests for Project Logging Router
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { projectLoggingRouter } from '../projectLogging';
import type { Database } from '@/types/database';
import type { SupabaseClient } from '@supabase/supabase-js';

// Mock Supabase client
const mockSupabase = {
  from: vi.fn(),
} as unknown as SupabaseClient<Database>;

const mockUser = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
};

function createMockContext() {
  return {
    supabase: mockSupabase,
    user: mockUser,
    authUser: {
      id: 'auth-123',
      email: 'test@example.com',
    },
    getUserRecord: async () => mockUser,
  };
}

describe('Project Logging Router', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('get', () => {
    it('should return project logging config if it exists', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const mockProject = { id: projectId, created_by: userId };
      const mockConfig = {
        id: '00000000-0000-0000-0000-000000000003',
        project_id: projectId,
        connector_id: '00000000-0000-0000-0000-000000000004',
        logging_component_id: '00000000-0000-0000-0000-000000000005',
        enabled_endpoints: ['00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0000-000000000007'],
        connector: {
          id: '00000000-0000-0000-0000-000000000004',
          name: 'http-log-connector',
          display_name: 'HTTP Log Connector',
          connector_type: 'http-log',
        },
      };

      const mockFrom = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      const mockFromConfig = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockConfig, error: null }),
          }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFrom();
        if (table === 'project_logging_config') return mockFromConfig();
        return mockFrom();
      });

      const ctx = createMockContext();
      const caller = projectLoggingRouter.createCaller(ctx as any);
      const result = await caller.get({ projectId });

      expect(result).toEqual(mockConfig);
    });

    it('should return null if no config exists', async () => {
      const projectId2 = '00000000-0000-0000-0000-000000000001';
      const userId2 = '00000000-0000-0000-0000-000000000002';
      const mockProject = { id: projectId2, created_by: userId2 };

      const mockFrom = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      const mockFromConfig = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ 
              data: null, 
              error: { code: 'PGRST116' } // Not found error
            }),
          }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFrom();
        if (table === 'project_logging_config') return mockFromConfig();
        return mockFrom();
      });

      const projectId = '00000000-0000-0000-0000-000000000001';
      const ctx = createMockContext();
      const caller = projectLoggingRouter.createCaller(ctx as any);
      const result = await caller.get({ projectId });

      expect(result).toBeNull();
    });

    it('should throw error if user does not own project', async () => {
      const projectId3 = '00000000-0000-0000-0000-000000000001';
      const mockProject = { id: projectId3, created_by: 'other-user' };

      const mockFrom = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFrom();
        return mockFrom();
      });

      const ctx = createMockContext();
      const caller = projectLoggingRouter.createCaller(ctx as any);
      await expect(
        caller.get({ projectId: projectId3 })
      ).rejects.toThrow('You do not have access to this project');
    });
  });

  describe('upsert', () => {
    it('should create or update project logging config', async () => {
      const projectId4 = '00000000-0000-0000-0000-000000000001';
      const userId4 = '00000000-0000-0000-0000-000000000002';
      const connectorId4 = '00000000-0000-0000-0000-000000000004';
      const componentId4 = '00000000-0000-0000-0000-000000000005';
      const endpointId4 = '00000000-0000-0000-0000-000000000006';
      const mockProject = { id: projectId4, created_by: userId4 };
      const mockConnector = { id: connectorId4, project_id: projectId4 };
      const mockConfig = {
        id: '00000000-0000-0000-0000-000000000003',
        project_id: projectId4,
        connector_id: connectorId4,
        logging_component_id: componentId4,
        enabled_endpoints: [endpointId4],
      };

      const mockFromProject = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      const mockFromConnector = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockConnector, error: null }),
          }),
        }),
      });

      const mockFromConfig = vi.fn().mockReturnValue({
        upsert: vi.fn().mockReturnValue({
          select: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockConfig, error: null }),
          }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFromProject();
        if (table === 'connectors') return mockFromConnector();
        if (table === 'project_logging_config') return mockFromConfig();
        return mockFromProject();
      });

      const ctx = createMockContext();
      const caller = projectLoggingRouter.createCaller(ctx as any);
      const result = await caller.upsert({
        projectId: projectId4,
        connectorId: connectorId4,
        loggingComponentId: componentId4,
        enabledEndpoints: [endpointId4],
      });

      expect(result).toEqual(mockConfig);
    });
  });

  describe('toggleEndpoint', () => {
    it('should add endpoint to enabled list when enabling', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const endpoint1 = '00000000-0000-0000-0000-000000000006';
      const endpoint2 = '00000000-0000-0000-0000-000000000007';
      const mockProject = { id: projectId, created_by: userId };
      const existingConfig = {
        enabled_endpoints: [endpoint1],
      };
      const updatedConfig = {
        id: '00000000-0000-0000-0000-000000000003',
        enabled_endpoints: [endpoint1, endpoint2],
      };

      const mockFrom = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      const mockFromConfig = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn()
              .mockResolvedValueOnce({ data: existingConfig, error: null })
              .mockResolvedValueOnce({ data: updatedConfig, error: null }),
          }),
        }),
        upsert: vi.fn().mockReturnValue({
          select: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: updatedConfig, error: null }),
          }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFrom();
        if (table === 'project_logging_config') return mockFromConfig();
        return mockFrom();
      });

      const ctx = createMockContext();
      const caller = projectLoggingRouter.createCaller(ctx as any);
      const result = await caller.toggleEndpoint({
        projectId,
        endpointId: endpoint2,
        enabled: true,
      });

      expect(result.enabled_endpoints).toContain(endpoint2);
    });

    it('should remove endpoint from enabled list when disabling', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const endpoint1 = '00000000-0000-0000-0000-000000000006';
      const endpoint2 = '00000000-0000-0000-0000-000000000007';
      const mockProject = { id: projectId, created_by: userId };
      const existingConfig = {
        enabled_endpoints: [endpoint1, endpoint2],
      };
      const updatedConfig = {
        id: '00000000-0000-0000-0000-000000000003',
        enabled_endpoints: [endpoint1],
      };

      const mockFrom = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      const mockFromConfig = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn()
              .mockResolvedValueOnce({ data: existingConfig, error: null })
              .mockResolvedValueOnce({ data: updatedConfig, error: null }),
          }),
        }),
        upsert: vi.fn().mockReturnValue({
          select: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: updatedConfig, error: null }),
          }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFrom();
        if (table === 'project_logging_config') return mockFromConfig();
        return mockFrom();
      });

      const ctx = createMockContext();
      const caller = projectLoggingRouter.createCaller(ctx as any);
      const result = await caller.toggleEndpoint({
        projectId,
        endpointId: endpoint2,
        enabled: false,
      });

      expect(result.enabled_endpoints).not.toContain(endpoint2);
      expect(result.enabled_endpoints).toContain(endpoint1);
    });
  });
});

