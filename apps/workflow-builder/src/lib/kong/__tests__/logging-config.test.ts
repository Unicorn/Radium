/**
 * Kong Logging Configuration Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  getProjectLoggingConfig,
  getLoggingConnectorCredentials,
  enableHttpLogPlugin,
  enableSyslogPlugin,
  applyLoggingToRoutes,
} from '../logging-config';
import { KongClient } from '../client';

describe('Kong Logging Configuration', () => {
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

  describe('getProjectLoggingConfig', () => {
    it('should return logging config when it exists', async () => {
      const mockConfig = {
        id: 'config-123',
        project_id: 'project-123',
        connector_id: 'connector-123',
        logging_component_id: 'component-123',
        enabled_endpoints: ['endpoint-1', 'endpoint-2'],
        connector: {
          id: 'connector-123',
          name: 'HTTP Log Connector',
          connector_type: 'api',
        },
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockConfig,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getProjectLoggingConfig('project-123', mockSupabase);

      expect(result).toEqual(mockConfig);
      expect(mockSupabase.from).toHaveBeenCalledWith('project_logging_config');
      expect(mockEq).toHaveBeenCalledWith('project_id', 'project-123');
    });

    it('should return null when config does not exist', async () => {
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

      const result = await getProjectLoggingConfig('project-123', mockSupabase);

      expect(result).toBeNull();
    });
  });

  describe('getLoggingConnectorCredentials', () => {
    it('should parse HTTP log connector credentials', async () => {
      const mockConnector = {
        id: 'connector-123',
        name: 'HTTP Log',
        connector_type: 'api',
        config_data: {},
        credentials_encrypted: Buffer.from(
          JSON.stringify({
            endpoint: 'https://logs.example.com/api/logs',
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            timeout: 5000,
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

      const result = await getLoggingConnectorCredentials('connector-123', mockSupabase);

      expect(result).toEqual({
        endpoint: 'https://logs.example.com/api/logs',
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        timeout: 5000,
        keepalive: 60000,
      });
    });

    it('should parse syslog connector credentials', async () => {
      const mockConnector = {
        id: 'connector-456',
        name: 'Syslog Connector',
        connector_type: 'syslog',
        config_data: {},
        credentials_encrypted: Buffer.from(
          JSON.stringify({
            host: 'syslog.example.com',
            port: 514,
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

      const result = await getLoggingConnectorCredentials('connector-456', mockSupabase);

      expect(result).toEqual({
        endpoint: 'syslog.example.com',
        method: 'UDP',
        port: 514,
      });
    });

    it('should return null when connector does not exist', async () => {
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

      const result = await getLoggingConnectorCredentials('connector-123', mockSupabase);

      expect(result).toBeNull();
    });
  });

  describe('enableHttpLogPlugin', () => {
    it('should enable HTTP log plugin with correct config', async () => {
      const connectorConfig = {
        endpoint: 'https://logs.example.com/api/logs',
        method: 'POST',
        headers: { 'X-Custom-Header': 'value' },
        timeout: 10000,
        keepalive: 60000,
        tls: true,
        tls_verify: true,
      };

      await enableHttpLogPlugin('route-123', connectorConfig, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledWith('route-123', 'http-log', {
        http_endpoint: 'https://logs.example.com/api/logs',
        method: 'POST',
        timeout: 10000,
        keepalive: 60000,
        headers: { 'X-Custom-Header': 'value' },
        https: true,
        ssl_verify: true,
      });
    });
  });

  describe('enableSyslogPlugin', () => {
    it('should enable syslog plugin with correct config', async () => {
      const connectorConfig = {
        endpoint: 'syslog.example.com',
        method: 'UDP',
        port: 514,
      } as any;

      await enableSyslogPlugin('route-123', connectorConfig, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledWith('route-123', 'syslog', {
        host: 'syslog.example.com',
        port: 514,
        protocol: 'udp',
      });
    });

    it('should use TCP protocol when specified', async () => {
      const connectorConfig = {
        endpoint: 'syslog.example.com',
        method: 'TCP',
        port: 514,
      } as any;

      await enableSyslogPlugin('route-123', connectorConfig, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledWith('route-123', 'syslog', {
        host: 'syslog.example.com',
        port: 514,
        protocol: 'tcp',
      });
    });
  });

  describe('applyLoggingToRoutes', () => {
    it('should apply HTTP log plugin when logging config exists', async () => {
      const mockLoggingConfig = {
        id: 'config-123',
        project_id: 'project-123',
        connector_id: 'connector-123',
        connector: {
          id: 'connector-123',
          name: 'HTTP Log Connector',
          connector_type: 'api',
        },
      };

      const mockConnector = {
        id: 'connector-123',
        name: 'HTTP Log',
        connector_type: 'api',
        config_data: {},
        credentials_encrypted: Buffer.from(
          JSON.stringify({
            endpoint: 'https://logs.example.com/api/logs',
            method: 'POST',
          })
        ),
      };

      // Mock getProjectLoggingConfig
      const mockSelect1 = vi.fn().mockReturnThis();
      const mockEq1 = vi.fn().mockReturnThis();
      const mockSingle1 = vi.fn().mockResolvedValue({
        data: mockLoggingConfig,
        error: null,
      });

      // Mock getLoggingConnectorCredentials
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

      await applyLoggingToRoutes('project-123', ['route-1', 'route-2'], mockSupabase, mockKong);

      expect(mockKong.enablePlugin).toHaveBeenCalledTimes(2);
      expect(mockKong.enablePlugin).toHaveBeenCalledWith(
        'route-1',
        'http-log',
        expect.objectContaining({
          http_endpoint: 'https://logs.example.com/api/logs',
        })
      );
    });

    it('should not apply logging when config does not exist', async () => {
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

      await applyLoggingToRoutes('project-123', ['route-1'], mockSupabase, mockKong);

      expect(mockKong.enablePlugin).not.toHaveBeenCalled();
    });
  });
});

