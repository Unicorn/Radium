/**
 * Tests for stateMonitoring router
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { stateMonitoringRouter } from '../stateMonitoring';
import type { Context } from '../../trpc';

describe('stateMonitoringRouter', () => {
  const mockSupabase = {
    from: vi.fn(),
  } as any;

  const mockContext: Context = {
    supabase: mockSupabase,
    user: {
      id: '00000000-0000-0000-0000-000000000001',
      email: 'test@example.com',
    },
    authUser: {
      id: 'auth-123',
      email: 'test@example.com',
    },
    getUserRecord: async () => ({
      id: '00000000-0000-0000-0000-000000000001',
      email: 'test@example.com',
    }),
  } as any;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('getMetrics', () => {
    it('should return metrics for a state variable', async () => {
      const variableId = '00000000-0000-0000-0000-000000000002';
      const mockMetrics = {
        variable_id: variableId,
        scope: 'project',
        size_bytes: 1024,
        access_count: 10,
        last_accessed: new Date().toISOString(),
      };

      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({ data: mockMetrics, error: null }),
      };

      mockSupabase.from.mockReturnValue(mockQuery);

      const caller = stateMonitoringRouter.createCaller(mockContext as any);
      const result = await caller.getMetrics({
        variableId,
        scope: 'project',
      });

      expect(result).toBeTruthy();
      expect(mockSupabase.from).toHaveBeenCalledWith('state_variable_metrics');
    });

    it('should return null if metrics not found', async () => {
      const variableId = '00000000-0000-0000-0000-000000000002';
      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({ data: null, error: { code: 'PGRST116' } }),
      };

      mockSupabase.from.mockReturnValue(mockQuery);

      const caller = stateMonitoringRouter.createCaller(mockContext as any);
      const result = await caller.getMetrics({
        variableId,
        scope: 'project',
      });

      expect(result).toBeNull();
    });
  });

  describe('getAlerts', () => {
    it('should return alerts for a state variable', async () => {
      const variableId = '00000000-0000-0000-0000-000000000002';
      const mockMetrics = {
        variable_id: variableId,
        scope: 'project',
        size_bytes: 51 * 1024 * 1024, // 51 MB - should trigger error alert (> 50 MB)
        access_count: 10,
        last_accessed: new Date().toISOString(),
      };

      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({ data: mockMetrics, error: null }),
      };

      mockSupabase.from.mockReturnValue(mockQuery);

      const caller = stateMonitoringRouter.createCaller(mockContext as any);
      const result = await caller.getAlerts({
        variableId,
        scope: 'project',
      });

      expect(result).toBeTruthy();
      expect(Array.isArray(result)).toBe(true);
      // Should have error alert for size > 50 MB
      expect(result.some(alert => alert.severity === 'error')).toBe(true);
    });
  });

  describe('getProjectAlerts', () => {
    it('should return all alerts for a project', async () => {
      const projectId = '00000000-0000-0000-0000-000000000003';
      const mockProject = { id: projectId };
      const mockProjectVars = [
        { id: '00000000-0000-0000-0000-000000000004' },
        { id: '00000000-0000-0000-0000-000000000005' },
      ];

      const mockProjectQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
      };

      const mockVarsQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
      };

      mockVarsQuery.eq.mockResolvedValue({ data: mockProjectVars, error: null });

      mockSupabase.from.mockImplementation((table: string) => {
        if (table === 'projects') {
          return mockProjectQuery;
        }
        if (table === 'project_state_variables') {
          return mockVarsQuery;
        }
        if (table === 'state_variable_metrics') {
          return {
            select: vi.fn().mockReturnThis(),
            eq: vi.fn().mockReturnThis(),
            order: vi.fn().mockReturnThis(),
            limit: vi.fn().mockReturnThis(),
            single: vi.fn().mockResolvedValue({ data: null, error: { code: 'PGRST116' } }),
          };
        }
        return {};
      });

      const caller = stateMonitoringRouter.createCaller(mockContext as any);
      const result = await caller.getProjectAlerts({
        projectId,
      });

      expect(result).toBeTruthy();
      expect(Array.isArray(result)).toBe(true);
    });

    it('should throw error if project not found', async () => {
      const projectId = '00000000-0000-0000-0000-000000000003';
      const mockProjectQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({ data: null, error: { code: 'PGRST116' } }),
      };

      mockSupabase.from.mockReturnValue(mockProjectQuery);

      const caller = stateMonitoringRouter.createCaller(mockContext as any);
      
      await expect(
        caller.getProjectAlerts({ projectId })
      ).rejects.toThrow('Project not found');
    });
  });

  describe('recordAccess', () => {
    it('should record state variable access', async () => {
      const variableId = '00000000-0000-0000-0000-000000000002';
      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({ data: null, error: { code: 'PGRST116' } }),
      };

      const mockInsert = {
        insert: vi.fn().mockResolvedValue({ data: null, error: null }),
      };

      mockSupabase.from.mockImplementation((table: string) => {
        if (table === 'state_variable_metrics') {
          // For select query
          if (mockQuery.select.mock.calls.length === 0) {
            return mockQuery;
          }
          // For insert
          return mockInsert;
        }
        return {};
      });

      const caller = stateMonitoringRouter.createCaller(mockContext as any);
      const result = await caller.recordAccess({
        variableId,
        scope: 'project',
        sizeBytes: 1024,
      });

      expect(result.success).toBe(true);
    });
  });
});

