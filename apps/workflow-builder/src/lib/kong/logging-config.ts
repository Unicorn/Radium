/**
 * Kong Logging Configuration Manager
 * Handles configuration and application of Kong logging plugins
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { KongClient } from './client';

export interface LoggingConnectorConfig {
  endpoint?: string;
  url?: string;
  method?: string;
  headers?: Record<string, string>;
  timeout?: number;
  keepalive?: number;
  tls?: boolean;
  tls_verify?: boolean;
}

/**
 * Get project logging configuration
 */
export async function getProjectLoggingConfig(
  projectId: string,
  supabase: SupabaseClient<Database>
) {
  const { data, error } = await supabase
    .from('project_logging_config')
    .select('*, connector:connectors(*)')
    .eq('project_id', projectId)
    .single();

  if (error || !data) {
    return null;
  }

  return data;
}

/**
 * Get connector credentials for logging
 */
export async function getLoggingConnectorCredentials(
  connectorId: string,
  supabase: SupabaseClient<Database>
): Promise<LoggingConnectorConfig | null> {
  const { data: connector, error } = await supabase
    .from('connectors')
    .select('config_data, credentials_encrypted, connector_type, name')
    .eq('id', connectorId)
    .single();

  if (error || !connector) {
    return null;
  }

  // Extract connector configuration
  const config: LoggingConnectorConfig = {};

  // Parse credentials if available
  if (connector.credentials_encrypted) {
    try {
      const credentials = JSON.parse(
        Buffer.from(connector.credentials_encrypted).toString()
      );
      
      // HTTP log connector
      if (connector.connector_type === 'api' || connector.name?.toLowerCase().includes('http')) {
        config.endpoint = credentials.endpoint || credentials.url;
        config.method = credentials.method || 'POST';
        config.headers = credentials.headers || {};
        config.timeout = credentials.timeout || 10000;
        config.keepalive = credentials.keepalive || 60000;
      }
      
      // Syslog connector
      if (connector.name?.toLowerCase().includes('syslog')) {
        config.endpoint = credentials.host || credentials.endpoint;
        config.method = 'UDP'; // Syslog typically uses UDP
        config.port = credentials.port || 514;
      }
    } catch (error) {
      console.warn('Failed to parse connector credentials:', error);
    }
  }

  // Merge with config_data
  if (connector.config_data) {
    Object.assign(config, connector.config_data);
  }

  return config;
}

/**
 * Enable HTTP log plugin on a Kong route
 */
export async function enableHttpLogPlugin(
  routeId: string,
  connectorConfig: LoggingConnectorConfig,
  kong: KongClient
): Promise<void> {
  const pluginConfig: Record<string, any> = {
    http_endpoint: connectorConfig.endpoint || connectorConfig.url,
    method: connectorConfig.method || 'POST',
    timeout: connectorConfig.timeout || 10000,
    keepalive: connectorConfig.keepalive || 60000,
  };

  if (connectorConfig.headers) {
    pluginConfig.headers = connectorConfig.headers;
  }

  if (connectorConfig.tls !== undefined) {
    pluginConfig.https = connectorConfig.tls;
  }

  if (connectorConfig.tls_verify !== undefined) {
    pluginConfig.ssl_verify = connectorConfig.tls_verify;
  }

  await kong.enablePlugin(routeId, 'http-log', pluginConfig);
}

/**
 * Enable syslog plugin on a Kong route
 */
export async function enableSyslogPlugin(
  routeId: string,
  connectorConfig: LoggingConnectorConfig,
  kong: KongClient
): Promise<void> {
  const pluginConfig: Record<string, any> = {
    host: connectorConfig.endpoint || 'localhost',
    port: (connectorConfig as any).port || 514,
    protocol: connectorConfig.method === 'TCP' ? 'tcp' : 'udp',
  };

  await kong.enablePlugin(routeId, 'syslog', pluginConfig);
}

/**
 * Apply logging plugin to routes based on project configuration
 */
export async function applyLoggingToRoutes(
  projectId: string,
  routeIds: string[],
  supabase: SupabaseClient<Database>,
  kong: KongClient
): Promise<void> {
  const loggingConfig = await getProjectLoggingConfig(projectId, supabase);
  
  if (!loggingConfig) {
    return; // No logging configured for this project
  }

  const connectorConfig = await getLoggingConnectorCredentials(
    loggingConfig.connector_id,
    supabase
  );

  if (!connectorConfig) {
    console.warn(`⚠️  Logging connector ${loggingConfig.connector_id} not found or misconfigured`);
    return;
  }

  // Determine plugin type based on connector
  const connector = loggingConfig.connector as any;
  const connectorName = connector?.name?.toLowerCase() || '';
  const connectorType = connector?.connector_type || '';

  // Apply logging plugin to all specified routes
  for (const routeId of routeIds) {
    try {
      if (connectorName.includes('syslog') || connectorType === 'syslog') {
        await enableSyslogPlugin(routeId, connectorConfig, kong);
        console.log(`✅ Enabled syslog logging on route ${routeId}`);
      } else {
        // Default to HTTP log
        await enableHttpLogPlugin(routeId, connectorConfig, kong);
        console.log(`✅ Enabled HTTP log on route ${routeId}`);
      }
    } catch (error) {
      console.error(`❌ Failed to enable logging on route ${routeId}:`, error);
    }
  }
}

