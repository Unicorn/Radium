/**
 * Tests for Kong Cache Router
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { kongCacheRouter } from '../kongCache';
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

describe('Kong Cache Router', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('get', () => {
    it('should return cache key config if it exists', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const componentId = '00000000-0000-0000-0000-000000000003';
      const mockProject = { id: projectId, created_by: userId };
      const mockCacheKey = {
        id: '00000000-0000-0000-0000-000000000004',
        component_id: componentId,
        project_id: projectId,
        connector_id: '00000000-0000-0000-0000-000000000005',
        cache_key: '550e8400-e29b-41d4-a716-446655440000',
        ttl_seconds: 3600,
        cache_key_strategy: 'path',
        content_types: ['application/json'],
        response_codes: [200, 201, 202],
        is_saved: true,
        marked_for_deletion: false,
      };

      const mockFromProject = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      // Create a chainable mock for Supabase query builder
      const mockQueryBuilder = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({ data: mockCacheKey, error: null }),
      };
      
      const mockFromCache = vi.fn().mockReturnValue(mockQueryBuilder);

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFromProject();
        if (table === 'kong_cache_keys') return mockFromCache();
        return mockFromProject();
      });

      const ctx = createMockContext();
      const caller = kongCacheRouter.createCaller(ctx as any);
      const result = await caller.get({ componentId, projectId });

      expect(result).toEqual(mockCacheKey);
    });

    it('should return null if no cache key config exists', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const componentId = '00000000-0000-0000-0000-000000000003';
      const mockProject = { id: projectId, created_by: userId };

      const mockFromProject = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      const mockFromCache = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnThis(),
          single: vi.fn().mockResolvedValue({ 
            data: null, 
            error: { code: 'PGRST116' }
          }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFromProject();
        if (table === 'kong_cache_keys') return mockFromCache();
        return mockFromProject();
      });

      const ctx = createMockContext();
      const caller = kongCacheRouter.createCaller(ctx as any);
      const result = await caller.get({ componentId, projectId });

      expect(result).toBeNull();
    });
  });

  describe('upsert', () => {
    it('should create cache key configuration', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const componentId = '00000000-0000-0000-0000-000000000003';
      const connectorId = '00000000-0000-0000-0000-000000000005';
      const cacheKey = '550e8400-e29b-41d4-a716-446655440000';
      const mockProject = { id: projectId, created_by: userId };
      const mockConnector = { id: connectorId, project_id: projectId };
      const mockCacheKey = {
        id: '00000000-0000-0000-0000-000000000004',
        component_id: componentId,
        project_id: projectId,
        connector_id: connectorId,
        cache_key: cacheKey,
        ttl_seconds: 3600,
        cache_key_strategy: 'path',
        content_types: ['application/json'],
        response_codes: [200, 201, 202],
        is_saved: true,
        marked_for_deletion: false,
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

      const mockFromCache = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: null, error: { code: 'PGRST116' } }),
          }),
        }),
        upsert: vi.fn().mockReturnValue({
          select: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockCacheKey, error: null }),
          }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFromProject();
        if (table === 'connectors') return mockFromConnector();
        if (table === 'kong_cache_keys') return mockFromCache();
        return mockFromProject();
      });

      const ctx = createMockContext();
      const caller = kongCacheRouter.createCaller(ctx as any);
      const result = await caller.upsert({
        componentId,
        projectId,
        connectorId,
        cacheKey,
        ttlSeconds: 3600,
        cacheKeyStrategy: 'path',
        contentTypes: ['application/json'],
        responseCodes: [200, 201, 202],
        isSaved: true,
      });

      expect(result).toEqual(mockCacheKey);
    });

    it('should throw error if trying to change immutable cache key', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const componentId = '00000000-0000-0000-0000-000000000003';
      const connectorId = '00000000-0000-0000-0000-000000000005';
      const oldCacheKey = '550e8400-e29b-41d4-a716-446655440000';
      const newCacheKey = '660e8400-e29b-41d4-a716-446655440001';
      const mockProject = { id: projectId, created_by: userId };
      const mockConnector = { id: connectorId, project_id: projectId };
      const existingCacheKey = {
        id: '00000000-0000-0000-0000-000000000004',
        component_id: componentId,
        cache_key: oldCacheKey,
        is_saved: true,
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

      const mockFromCache = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnThis(),
          single: vi.fn().mockResolvedValue({ data: existingCacheKey, error: null }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFromProject();
        if (table === 'connectors') return mockFromConnector();
        if (table === 'kong_cache_keys') return mockFromCache();
        return mockFromProject();
      });

      const ctx = createMockContext();
      const caller = kongCacheRouter.createCaller(ctx as any);
      
      await expect(
        caller.upsert({
          componentId,
          projectId,
          connectorId,
          cacheKey: newCacheKey,
          ttlSeconds: 3600,
          cacheKeyStrategy: 'path',
          contentTypes: ['application/json'],
          responseCodes: [200, 201, 202],
          isSaved: true,
        })
      ).rejects.toThrow('Cache key is immutable');
    });
  });

  describe('generateKey', () => {
    it('should generate a new UUID cache key', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const mockProject = { id: projectId, created_by: userId };

      const mockFromProject = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFromProject();
        return mockFromProject();
      });

      const ctx = createMockContext();
      const caller = kongCacheRouter.createCaller(ctx as any);
      const result = await caller.generateKey({ projectId });

      expect(result.cacheKey).toBeDefined();
      expect(result.cacheKey).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i);
    });
  });

  describe('markForDeletion', () => {
    it('should mark cache key for deletion', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const componentId = '00000000-0000-0000-0000-000000000003';
      const mockProject = { id: projectId, created_by: userId };
      const mockCacheKey = {
        id: '00000000-0000-0000-0000-000000000004',
        component_id: componentId,
        marked_for_deletion: true,
      };

      const mockFromProject = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      const mockUpdateBuilder = {
        update: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        select: vi.fn().mockReturnThis(),
        single: vi.fn().mockResolvedValue({ data: mockCacheKey, error: null }),
      };
      
      const mockFromCache = vi.fn().mockReturnValue(mockUpdateBuilder);

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFromProject();
        if (table === 'kong_cache_keys') return mockFromCache();
        return mockFromProject();
      });

      const ctx = createMockContext();
      const caller = kongCacheRouter.createCaller(ctx as any);
      const result = await caller.markForDeletion({ componentId, projectId });

      expect(result.marked_for_deletion).toBe(true);
    });
  });

  describe('getMarkedForDeletion', () => {
    it('should return cache keys marked for deletion', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const mockProject = { id: projectId, created_by: userId };
      const mockCacheKeys = [
        {
          id: '00000000-0000-0000-0000-000000000004',
          component_id: '00000000-0000-0000-0000-000000000003',
          cache_key: '550e8400-e29b-41d4-a716-446655440000',
          marked_for_deletion: true,
        },
      ];

      const mockFromProject = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      // Create a chainable query builder that supports multiple .eq() calls
      const mockQueryBuilder4 = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
      };
      
      // The second .eq() call should resolve with the data
      let eqCallCount = 0;
      (mockQueryBuilder4.eq as any).mockImplementation(() => {
        eqCallCount++;
        if (eqCallCount === 2) {
          // Second .eq() call returns the resolved data
          return Promise.resolve({ data: mockCacheKeys, error: null });
        }
        return mockQueryBuilder4; // First .eq() returns this for chaining
      });
      
      const mockFromCache = vi.fn().mockReturnValue(mockQueryBuilder4);

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFromProject();
        if (table === 'kong_cache_keys') return mockFromCache();
        return mockFromProject();
      });

      const ctx = createMockContext();
      const caller = kongCacheRouter.createCaller(ctx as any);
      const result = await caller.getMarkedForDeletion({ projectId });

      expect(result).toEqual(mockCacheKeys);
      expect(result.length).toBe(1);
    });
  });

  describe('delete', () => {
    it('should delete cache keys marked for deletion', async () => {
      const projectId = '00000000-0000-0000-0000-000000000001';
      const userId = '00000000-0000-0000-0000-000000000002';
      const componentIds = ['00000000-0000-0000-0000-000000000003'];
      const mockProject = { id: projectId, created_by: userId };

      const mockFromProject = vi.fn().mockReturnValue({
        select: vi.fn().mockReturnValue({
          eq: vi.fn().mockReturnValue({
            single: vi.fn().mockResolvedValue({ data: mockProject, error: null }),
          }),
        }),
      });

      // Create a chainable delete builder that supports .delete().in().eq().eq()
      const mockDeleteBuilder = {
        delete: vi.fn().mockReturnThis(),
        in: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
      };
      
      // The second .eq() call should resolve with the result
      let eqCallCount = 0;
      (mockDeleteBuilder.eq as any).mockImplementation(() => {
        eqCallCount++;
        if (eqCallCount === 2) {
          // Second .eq() call returns the resolved result
          return Promise.resolve({ error: null });
        }
        return mockDeleteBuilder; // First .eq() returns this for chaining
      });
      
      const mockFromCache = vi.fn().mockReturnValue(mockDeleteBuilder);

      (mockSupabase.from as any) = vi.fn((table: string) => {
        if (table === 'projects') return mockFromProject();
        if (table === 'kong_cache_keys') return mockFromCache();
        return mockFromProject();
      });

      const ctx = createMockContext();
      const caller = kongCacheRouter.createCaller(ctx as any);
      const result = await caller.delete({ componentIds, projectId });

      expect(result.success).toBe(true);
      expect(result.deletedCount).toBe(1);
    });
  });
});

