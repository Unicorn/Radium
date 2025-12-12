/**
 * Kong Cache Configuration Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  getCacheConfig,
  getRedisConnectorConfig,
  enableCachePlugin,
  applyCacheToRoute,
  cleanupDeletedCacheKeys,
} from '../cache-config';
import { KongClient } from '../client';

describe('Kong Cache Configuration', () => {
  let mockSupabase: SupabaseClient<Database>;
  let mockKong: KongClient;

  beforeEach(() => {
    mockSupabase = {
      from: vi.fn(),
    } as any;

    mockKong = {
      enablePlugin: vi.fn().mockResolvedValue(undefined),
    } as any;
  });

  describe('getCacheConfig', () => {
    it('should return cache config when it exists', async () => {
      const mockCacheKey = {
        id: 'cache-key-123',
        component_id: 'component-123',
        project_id: 'project-123',
        cache_key: 'abc-123-def-456',
        connector_id: 'connector-123',
        ttl_seconds: 7200,
        cache_key_strategy: 'query-string',
        content_types: ['application/json', 'text/html'],
        response_codes: [200, 201],
        is_saved: true,
        marked_for_deletion: false,
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockCacheKey,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getCacheConfig('component-123', mockSupabase);

      expect(result).toEqual({
        cacheKey: 'abc-123-def-456',
        ttlSeconds: 7200,
        cacheKeyStrategy: 'query-string',
        contentTypes: ['application/json', 'text/html'],
        responseCodes: [200, 201],
        connectorId: 'connector-123',
      });
    });

    it('should return null when cache config does not exist', async () => {
      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: null,
        error: { code: 'PGRST116' },
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getCacheConfig('component-123', mockSupabase);

      expect(result).toBeNull();
    });
  });

  describe('getRedisConnectorConfig', () => {
    it('should parse Upstash Redis connector credentials', async () => {
      const mockConnector = {
        id: 'connector-123',
        name: 'Upstash Redis',
        config_data: {},
        credentials_encrypted: Buffer.from(
          JSON.stringify({
            url: 'https://redis.example.com',
            token: 'secret-token',
          })
        ),
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockConnector,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getRedisConnectorConfig('connector-123', mockSupabase);

      expect(result).toEqual({
        url: 'https://redis.example.com',
        token: 'secret-token',
      });
    });

    it('should parse standard Redis connector credentials', async () => {
      const mockConnector = {
        id: 'connector-456',
        name: 'Redis Server',
        config_data: {},
        credentials_encrypted: Buffer.from(
          JSON.stringify({
            host: 'redis.example.com',
            port: 6379,
            password: 'secret',
            database: 0,
          })
        ),
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockConnector,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getRedisConnectorConfig('connector-456', mockSupabase);

      expect(result).toEqual({
        host: 'redis.example.com',
        port: 6379,
        password: 'secret',
        database: 0,
      });
    });
  });

  describe('enableCachePlugin', () => {
    it('should enable cache plugin with memory strategy', async () => {
      const cacheConfig = {
        cacheKey: 'test-key',
        ttlSeconds: 3600,
        cacheKeyStrategy: 'default',
        contentTypes: ['application/json'],
        responseCodes: [200],
        connectorId: 'connector-123',
      };

      await enableCachePlugin('route-123', cacheConfig, null, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledWith('route-123', 'proxy-cache', {
        response_code: [200],
        content_type: ['application/json'],
        ttl: 3600,
        strategy: 'memory',
      });
    });

    it('should enable cache plugin with Redis strategy', async () => {
      const cacheConfig = {
        cacheKey: 'test-key',
        ttlSeconds: 7200,
        cacheKeyStrategy: 'query-string',
        contentTypes: ['application/json'],
        responseCodes: [200, 201],
        connectorId: 'connector-123',
      };

      const redisConfig = {
        host: 'redis.example.com',
        port: 6379,
        password: 'secret',
        database: 0,
      };

      await enableCachePlugin('route-123', cacheConfig, redisConfig, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledWith('route-123', 'proxy-cache', {
        response_code: [200, 201],
        content_type: ['application/json'],
        ttl: 7200,
        strategy: 'redis',
        redis: {
          host: 'redis.example.com',
          port: 6379,
          password: 'secret',
          database: 0,
        },
        cache_control: true,
      });
    });
  });

  describe('applyCacheToRoute', () => {
    it('should apply cache plugin when cache config exists', async () => {
      const mockCacheKey = {
        id: 'cache-key-123',
        component_id: 'component-123',
        cache_key: 'test-key',
        connector_id: 'connector-123',
        ttl_seconds: 3600,
        cache_key_strategy: 'default',
        content_types: ['application/json'],
        response_codes: [200],
      };

      const mockConnector = {
        id: 'connector-123',
        name: 'Redis',
        config_data: {},
        credentials_encrypted: Buffer.from(
          JSON.stringify({
            host: 'redis.example.com',
            port: 6379,
          })
        ),
      };

      // Mock getCacheConfig
      const mockSelect1 = vi.fn().mockReturnThis();
      const mockEq1 = vi.fn().mockReturnThis();
      const mockSingle1 = vi.fn().mockResolvedValue({
        data: mockCacheKey,
        error: null,
      });

      // Mock getRedisConnectorConfig
      const mockSelect2 = vi.fn().mockReturnThis();
      const mockEq2 = vi.fn().mockReturnThis();
      const mockSingle2 = vi.fn().mockResolvedValue({
        data: mockConnector,
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

      await applyCacheToRoute('component-123', 'route-123', mockSupabase, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledWith(
        'route-123',
        'proxy-cache',
        expect.objectContaining({
          strategy: 'redis',
        })
      );
    });

    it('should not apply cache when config does not exist', async () => {
      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: null,
        error: { code: 'PGRST116' },
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      await applyCacheToRoute('component-123', 'route-123', mockSupabase, mockKong);

      expect(mockKong.enablePlugin).not.toHaveBeenCalled();
    });
  });

  describe('cleanupDeletedCacheKeys', () => {
    it('should cleanup deleted cache keys', async () => {
      const mockDeletedKeys = [
        {
          cache_key: 'key-1',
          connector_id: 'connector-123',
        },
        {
          cache_key: 'key-2',
          connector_id: 'connector-123',
        },
      ];

      const mockConnector = {
        id: 'connector-123',
        name: 'Redis',
        config_data: {},
        credentials_encrypted: Buffer.from(JSON.stringify({ host: 'localhost', port: 6379 })),
      };

      // Mock for getting deleted keys
      const mockSelect1 = vi.fn().mockReturnThis();
      const mockEq1 = vi.fn().mockReturnThis();
      const mockSelectResult = vi.fn().mockResolvedValue({
        data: mockDeletedKeys,
        error: null,
      });

      // Mock for getting connector config (called for each key)
      const mockSelect2 = vi.fn().mockReturnThis();
      const mockEq2 = vi.fn().mockReturnThis();
      const mockSingle2 = vi.fn().mockResolvedValue({
        data: mockConnector,
        error: null,
      });

      // Mock for delete
      const mockDelete = vi.fn().mockReturnThis();
      const mockEq3 = vi.fn().mockReturnThis();
      const mockEq4 = vi.fn().mockResolvedValue({ error: null });

      (mockSupabase.from as any)
        .mockReturnValueOnce({
          select: mockSelect1,
          eq: mockEq1,
        })
        .mockReturnValueOnce({
          select: mockSelect2,
          eq: mockEq2,
          single: mockSingle2,
        })
        .mockReturnValueOnce({
          select: mockSelect2,
          eq: mockEq2,
          single: mockSingle2,
        })
        .mockReturnValueOnce({
          delete: mockDelete,
          eq: mockEq3,
        });

      mockEq1.mockReturnValue({
        eq: mockSelectResult,
      });

      mockEq3.mockReturnValue({
        eq: mockEq4,
      });

      await cleanupDeletedCacheKeys('project-123', mockSupabase);

      expect(mockSupabase.from).toHaveBeenCalledWith('kong_cache_keys');
    });
  });
});

