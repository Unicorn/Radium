/**
 * State Monitoring Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  recordStateAccess,
  getStateMetrics,
  checkStateAlerts,
  getProjectStateAlerts,
} from '../state-metrics';

describe('State Monitoring', () => {
  let mockSupabase: SupabaseClient<Database>;

  beforeEach(() => {
    mockSupabase = {
      from: vi.fn(),
    } as any;
  });

  describe('recordStateAccess', () => {
    it('should record state variable access', async () => {
      const mockExisting = {
        data: {
          id: 'metric-1',
          variable_id: 'var-123',
          scope: 'project',
          size_bytes: 1000,
          access_count: 5,
          last_accessed: new Date().toISOString(),
          created_at: new Date().toISOString(),
        },
        error: null,
      };

      const mockUpdateEq = vi.fn().mockResolvedValue({ error: null });

      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue(mockExisting),
      };

      const mockUpdateQuery = {
        update: vi.fn().mockReturnValue({
          eq: mockUpdateEq,
        }),
      };

      (mockSupabase.from as any)
        .mockReturnValueOnce(mockQuery)
        .mockReturnValueOnce(mockUpdateQuery);

      await recordStateAccess(mockSupabase, 'var-123', 'project', 1500);

      expect(mockUpdateQuery.update).toHaveBeenCalledWith(
        expect.objectContaining({
          size_bytes: 1500,
          access_count: 6,
        })
      );
      expect(mockUpdateEq).toHaveBeenCalledWith('id', 'metric-1');
    });

    it('should create new metrics record when none exists', async () => {
      const mockExisting = {
        data: null,
        error: { code: 'PGRST116', message: 'No rows found' },
      };

      const mockInsert = vi.fn().mockResolvedValue({ error: null });

      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue(mockExisting),
      };

      const mockInsertQuery = {
        insert: vi.fn().mockResolvedValue({ error: null }),
      };

      (mockSupabase.from as any)
        .mockReturnValueOnce(mockQuery)
        .mockReturnValueOnce(mockInsertQuery);

      await recordStateAccess(mockSupabase, 'var-123', 'project', 1000);

      expect(mockInsertQuery.insert).toHaveBeenCalledWith(
        expect.objectContaining({
          variable_id: 'var-123',
          scope: 'project',
          size_bytes: 1000,
          access_count: 1,
        })
      );
    });
  });

  describe('getStateMetrics', () => {
    it('should return metrics with recommendations', async () => {
      const mockMetrics = {
        id: 'metric-1',
        variable_id: 'var-123',
        scope: 'project',
        size_bytes: 5000000, // 5 MB
        access_count: 1000,
        last_accessed: new Date().toISOString(),
        created_at: new Date(Date.now() - 86400000).toISOString(), // 1 day ago
      };

      const mockQuery1 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: mockMetrics,
          error: null,
        }),
      };

      const mockQuery2 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockResolvedValue({
          data: [],
          error: null,
        }),
      };

      (mockSupabase.from as any)
        .mockReturnValueOnce(mockQuery1)
        .mockReturnValueOnce(mockQuery2);

      const result = await getStateMetrics(mockSupabase, 'var-123', 'project');

      expect(result).toBeDefined();
      expect(result?.variableId).toBe('var-123');
      expect(result?.scope).toBe('project');
      expect(result?.sizeBytes).toBe(5000000);
      expect(result?.accessCount).toBe(1000);
      expect(result?.recommendations).toBeDefined();
      expect(Array.isArray(result?.recommendations)).toBe(true);
    });
  });

  describe('getStateMetrics recommendations', () => {
    it('should include recommendations for large data', async () => {
      const mockMetrics = {
        id: 'metric-1',
        variable_id: 'var-123',
        scope: 'project',
        size_bytes: 10 * 1024 * 1024, // 10 MB
        access_count: 10,
        last_accessed: new Date().toISOString(),
        created_at: new Date().toISOString(),
      };

      const mockQuery1 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: mockMetrics,
          error: null,
        }),
      };

      const mockQuery2 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockResolvedValue({
          data: [],
          error: null,
        }),
      };

      (mockSupabase.from as any)
        .mockReturnValueOnce(mockQuery1)
        .mockReturnValueOnce(mockQuery2);

      const result = await getStateMetrics(mockSupabase, 'var-123', 'project');

      expect(result?.recommendations.length).toBeGreaterThan(0);
      expect(result?.recommendations.some((r) => r.toLowerCase().includes('database'))).toBe(true);
    });

    it('should include recommendations for frequently accessed data', async () => {
      const mockMetrics = {
        id: 'metric-1',
        variable_id: 'var-123',
        scope: 'project',
        size_bytes: 1000, // 1 KB
        access_count: 10000, // Very high access count
        last_accessed: new Date().toISOString(),
        created_at: new Date().toISOString(),
      };

      const mockQuery1 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: mockMetrics,
          error: null,
        }),
      };

      const mockQuery2 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockResolvedValue({
          data: [],
          error: null,
        }),
      };

      (mockSupabase.from as any)
        .mockReturnValueOnce(mockQuery1)
        .mockReturnValueOnce(mockQuery2);

      const result = await getStateMetrics(mockSupabase, 'var-123', 'project');

      expect(result?.recommendations.length).toBeGreaterThan(0);
      expect(result?.recommendations.some((r) => r.toLowerCase().includes('redis') || r.toLowerCase().includes('caching'))).toBe(true);
    });
  });

  describe('checkStateAlerts', () => {
    it('should generate error alert for very large variables', async () => {
      const mockMetrics = {
        id: 'metric-1',
        variable_id: 'var-123',
        scope: 'project',
        size_bytes: 60 * 1024 * 1024, // 60 MB
        access_count: 10,
        last_accessed: new Date().toISOString(),
        created_at: new Date().toISOString(),
      };

      const mockQuery1 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: mockMetrics,
          error: null,
        }),
      };

      const mockQuery2 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockResolvedValue({
          data: [],
          error: null,
        }),
      };

      (mockSupabase.from as any)
        .mockReturnValueOnce(mockQuery1)
        .mockReturnValueOnce(mockQuery2);

      const alerts = await checkStateAlerts(mockSupabase, 'var-123', 'project');

      expect(alerts.length).toBeGreaterThan(0);
      expect(alerts.some((a) => a.severity === 'error')).toBe(true);
    });

    it('should generate warning alert for large variables', async () => {
      const mockMetrics = {
        id: 'metric-1',
        variable_id: 'var-123',
        scope: 'project',
        size_bytes: 30 * 1024 * 1024, // 30 MB
        access_count: 10,
        last_accessed: new Date().toISOString(),
        created_at: new Date().toISOString(),
      };

      const mockQuery1 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: mockMetrics,
          error: null,
        }),
      };

      const mockQuery2 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockResolvedValue({
          data: [],
          error: null,
        }),
      };

      (mockSupabase.from as any)
        .mockReturnValueOnce(mockQuery1)
        .mockReturnValueOnce(mockQuery2);

      const alerts = await checkStateAlerts(mockSupabase, 'var-123', 'project');

      expect(alerts.length).toBeGreaterThan(0);
      expect(alerts.some((a) => a.severity === 'warning')).toBe(true);
    });
  });

  describe('getProjectStateAlerts', () => {
    it('should return all alerts for a project', async () => {
      const mockVariables = [
        {
          id: 'var-1',
          project_id: 'project-123',
          name: 'var1',
        },
        {
          id: 'var-2',
          project_id: 'project-123',
          name: 'var2',
        },
      ];

      const mockQuery1 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockResolvedValue({
          data: mockVariables,
          error: null,
        }),
      };

      const mockQuery2 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({
          data: {
            id: 'metric-1',
            variable_id: 'var-1',
            scope: 'project',
            size_bytes: 30 * 1024 * 1024,
            access_count: 10,
            last_accessed: new Date().toISOString(),
            created_at: new Date().toISOString(),
          },
          error: null,
        }),
      };

      const mockQuery3 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockReturnThis(),
        limit: vi.fn().mockResolvedValue({
          data: [],
          error: null,
        }),
      };

      (mockSupabase.from as any)
        .mockReturnValueOnce(mockQuery1)
        .mockReturnValueOnce(mockQuery2)
        .mockReturnValueOnce(mockQuery3)
        .mockReturnValueOnce(mockQuery2)
        .mockReturnValueOnce(mockQuery3);

      const alerts = await getProjectStateAlerts(mockSupabase, 'project-123');

      expect(Array.isArray(alerts)).toBe(true);
      expect(mockSupabase.from).toHaveBeenCalledWith('project_state_variables');
    });
  });
});
