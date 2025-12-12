/**
 * Kong CORS Configuration Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  getCorsConfig,
  enableCorsPlugin,
  applyCorsToRoute,
} from '../cors-config';
import { KongClient } from '../client';

describe('Kong CORS Configuration', () => {
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

  describe('getCorsConfig', () => {
    it('should return CORS config when it exists', async () => {
      const mockPublicInterface = {
        id: 'public-123',
        cors_config: {
          allowedOrigins: ['https://example.com', 'https://app.example.com'],
          allowedMethods: ['GET', 'POST', 'PUT'],
          allowedHeaders: ['Content-Type', 'Authorization'],
          exposedHeaders: ['X-Custom-Header'],
          allowCredentials: true,
          maxAge: 86400,
          preflightContinue: false,
        },
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockPublicInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getCorsConfig('public-123', mockSupabase);

      expect(result).toEqual({
        allowedOrigins: ['https://example.com', 'https://app.example.com'],
        allowedMethods: ['GET', 'POST', 'PUT'],
        allowedHeaders: ['Content-Type', 'Authorization'],
        exposedHeaders: ['X-Custom-Header'],
        allowCredentials: true,
        maxAge: 86400,
        preflightContinue: false,
      });
    });

    it('should handle snake_case config format', async () => {
      const mockPublicInterface = {
        id: 'public-123',
        cors_config: {
          allowed_origins: ['https://example.com'],
          allowed_methods: ['GET', 'POST'],
          allowed_headers: ['*'],
          allow_credentials: false,
          max_age: 3600,
        },
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockPublicInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getCorsConfig('public-123', mockSupabase);

      expect(result).toEqual({
        allowedOrigins: ['https://example.com'],
        allowedMethods: ['GET', 'POST'],
        allowedHeaders: ['*'],
        exposedHeaders: [],
        allowCredentials: false,
        maxAge: 3600,
        preflightContinue: false,
      });
    });

    it('should return null when CORS config does not exist', async () => {
      const mockPublicInterface = {
        id: 'public-123',
        cors_config: null,
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockPublicInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getCorsConfig('public-123', mockSupabase);

      expect(result).toBeNull();
    });
  });

  describe('enableCorsPlugin', () => {
    it('should enable CORS plugin with correct config', async () => {
      const corsConfig = {
        allowedOrigins: ['https://example.com'],
        allowedMethods: ['GET', 'POST', 'OPTIONS'],
        allowedHeaders: ['Content-Type', 'Authorization'],
        exposedHeaders: ['X-Custom-Header'],
        allowCredentials: true,
        maxAge: 86400,
        preflightContinue: false,
      };

      await enableCorsPlugin('route-123', corsConfig, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledWith('route-123', 'cors', {
        origins: ['https://example.com'],
        methods: ['GET', 'POST', 'OPTIONS'],
        headers: ['Content-Type', 'Authorization'],
        exposed_headers: ['X-Custom-Header'],
        credentials: true,
        max_age: 86400,
        preflight_continue: false,
      });
    });

    it('should use default values when optional fields are missing', async () => {
      const corsConfig = {
        allowedOrigins: ['*'],
        allowedMethods: ['GET'],
        allowedHeaders: ['*'],
      };

      await enableCorsPlugin('route-123', corsConfig as any, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledWith('route-123', 'cors', {
        origins: ['*'],
        methods: ['GET'],
        headers: ['*'],
        exposed_headers: [],
        credentials: false,
        max_age: 3600,
        preflight_continue: false,
      });
    });
  });

  describe('applyCorsToRoute', () => {
    it('should apply CORS plugin when config exists', async () => {
      const mockPublicInterface = {
        id: 'public-123',
        cors_config: {
          allowedOrigins: ['https://example.com'],
          allowedMethods: ['GET', 'POST'],
          allowedHeaders: ['*'],
        },
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockPublicInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      await applyCorsToRoute('public-123', 'route-123', mockSupabase, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledWith(
        'route-123',
        'cors',
        expect.objectContaining({
          origins: ['https://example.com'],
        })
      );
    });

    it('should not apply CORS when config does not exist', async () => {
      const mockPublicInterface = {
        id: 'public-123',
        cors_config: null,
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockPublicInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      await applyCorsToRoute('public-123', 'route-123', mockSupabase, mockKong);

      expect(mockKong.enablePlugin).not.toHaveBeenCalled();
    });
  });
});

